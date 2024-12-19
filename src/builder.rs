use alloc::rc::Rc;
use alloc::vec::Vec;
use core::mem::swap;
use bevy_ecs::component::Component;
use bevy_ecs::entity::Entity;
use bevy_ecs::prelude::Commands;
use glam::{FloatExt, Vec2};
use pd::graphics::api::Api;
use pd::graphics::bitmap::{Bitmap, Color, LCDColorConst};
use pd::graphics::{BitmapFlip, Graphics};
use pd::sys::ffi::LCDColor;
use bevy_playdate::dbg;
use bevy_playdate::sprite::Sprite;
use curve::arc::ArcSegment;
use curve::line::LineSegment;
use curve::traits::{CurveSegment, CurveType};

#[derive(Component)]
pub struct MovingSplineDot {
    pub t: f32,
    pub v: f32,
    pub spline_entity: Entity,
}

pub struct MovingSplineDot2 {
    pub t: f32,
    pub v: f32,
    pub current_segment: Option<Entity>,
}

#[derive(Component, Clone, PartialEq, Debug)]
pub struct Segment {
    /// The actual curve on the segment
    pub curve: CurveType,
    /// parent curve
    pub parent: Entity,
    /// joint at t = 0
    pub start_joint: Entity,
    /// joint at t = 1
    pub end_joint: Entity,
}

impl Segment {
    pub fn to_sprite<T: Api>(&self, gfx: Graphics<T>, line_width: i32, color: LCDColor) -> Sprite {
        let (min, mut max) = self.curve.bounds();
        let start = self.curve.position(0.0);
        let s_t = if min.x != max.x {
            f32::inverse_lerp(
                min.x - line_width as f32 / 2.0,
                max.x + line_width as f32 / 2.0,
                start.x,
            )
        } else {
            0.5
        };
        let e_t = if min.y != max.y {
            f32::inverse_lerp(
                min.y - line_width as f32 / 2.0,
                max.y + line_width as f32 / 2.0,
                start.y,
            )
        } else {
            0.5
        };

        // println!("{:?}", self.curve);
        // println!("{s_t} {e_t}");

        // dbg!(&self.curve);

        max -= min;
        max += Vec2::splat(line_width as f32);

        let out = Bitmap::new(max.x as i32, max.y as i32, Color::CLEAR).unwrap();

        gfx.push_context(&out);


        match self.curve {
            CurveType::Line(line) => {
                let start = line.start - min;
                let end = line.end - min;
                gfx.draw_line(
                    start.x as i32 + line_width / 2,
                    start.y as i32 + line_width / 2,
                    end.x as i32 + line_width / 2,
                    end.y as i32 + line_width / 2,
                    line_width,
                    color,
                );
            }
            CurveType::Arc(arc) => {
                let mut end = arc.end;
                let mut start = arc.start;
                end = (90.0 - end.to_degrees());
                start = (90.0 - start.to_degrees());
                if arc.start < arc.end {
                    swap(&mut end, &mut start);
                }
                // println!("{:?}", center);
                gfx.draw_ellipse(
                    0,
                    0,
                    (arc.radius * 2.0) as i32 + line_width,
                    (arc.radius * 2.0) as i32 + line_width,
                    line_width,
                    start,
                    end,
                    color,
                );
            }
        }
        // gfx.draw_rect(0, 0, max.x as i32, max.y as i32, LCDColor::XOR);


        // gfx.fill_rect((start.x - min.x) as i32 - 2, (start.y - min.y) as i32 - 2, 8, 8, LCDColor::BLACK);
        let end = self.curve.position(1.0);
        // gfx.fill_rect((end.x - min.x) as i32 - 2, (end.y - min.y) as i32 - 2, 8, 8, LCDColor::BLACK);

        gfx.pop_context();

        // dbg!(s_t, e_t);


        let mut spr = Sprite::new_from_bitmap(Rc::new(out), BitmapFlip::kBitmapUnflipped);
        spr.set_center(s_t, e_t);
        let pos = self.curve.position(0.0);
        spr.move_to(pos.x, pos.y);

        spr
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
enum JointExit {
    /// previous segment was t = 1
    Start,
    /// next segment is t = 0
    End,
}

#[derive(Copy, Clone, PartialEq, Debug, Component)]
pub enum Joint {
    /// Continues to next segment
    Continue {
        /// t = 0
        start: Entity,
        /// t = 1
        end: Entity,
        /// fraction of speed kept when transitioning
        sustained_speed: f32,
    },
    /// Stops at this segment, velocity canceled
    Stop {
        from: Entity,
        side: JointExit,
    },
    // /// Falls off the edge
    // Fall {
    //     from: Entity,
    //     side: JointExit,
    // },
}

impl Joint {
    pub fn new_stop(start: Entity, side: JointExit) -> Self {
        Self::Stop {
            from: start,
            side,
        }
    }

    pub fn new_continue(
        start_seg: &CurveType, start_entity: Entity,
        end_seg: &CurveType, end_entity: Entity,
    ) -> Self {
        let v_s = start_seg.velocity(1.0).normalize_or_zero();
        let v_e = end_seg.velocity(0.0).normalize_or_zero();
        let mut sustained_speed = f32::max(0.0, v_s.dot(v_e));
        // for floating point precision i guess
        if sustained_speed > 0.995 {
            sustained_speed = 1.0;
        }

        Self::Continue {
            start: start_entity,
            end: end_entity,
            sustained_speed,
        }
    }

    pub fn enter_joint(
        &self,
        v: f32,
        enter: JointExit,
    ) -> EnterJointResult {
        match *self {
            Joint::Continue {
                start,
                end,
                sustained_speed,
            } => {
                let (entity, t) = match enter {
                    JointExit::Start => (end, 0.0),
                    JointExit::End => (start, 1.0),
                };

                let new_v = v * sustained_speed;

                EnterJointResult {
                    next: Some(entity),
                    t,
                    v: new_v,
                }
            }
            Joint::Stop { from, side } => {
                assert_eq!(side, enter);

                EnterJointResult {
                    next: Some(from),
                    t: match side {
                        JointExit::Start => 1.0,
                        JointExit::End => 0.0,
                    },
                    v: 0.0,
                }
            }
        }
    }
}

// #[derive(Copy, Clone, PartialEq, Debug)]
// enum JointType {
// }

pub struct EnterJointResult {
    next: Option<Entity>,
    t: f32,
    v: f32,
}
// 
// enum EnterJointResult {
//     Continue(Entity),
//     Stop,
//     Fall,
// }

struct ContinueJoint {
    pub next_segment: Entity,
    pub is_vel_discontinuity: bool,
}

struct Curve {
    pub total_length: f32,
    pub segments: Vec<Entity>,
}

impl MovingSplineDot2 {
    pub fn advance(
        &mut self,

    ) {

    }
}

pub struct CurveBuilder {
    segments: Vec<SplineSegment>,
    cur_pos: Vec2,
    cur_dir: Vec2,
    total_length: f32,
    // TODO: add start cap
}

impl CurveBuilder {
    pub fn new(pos: Vec2, dir: Vec2) -> Self {
        CurveBuilder {
            segments: Vec::new(),
            cur_pos: pos,
            cur_dir: dir,
            total_length: 0.0,
        }
    }
    
    pub fn push(mut self, builder: impl SectionBuilder) -> Self {
        let segment = builder.add_segment(&mut self.cur_pos, &mut self.cur_dir);


        println!("{:?}", segment);

        println!("start: {:.2?} w/ {:.2?}, end: {:.2?} w/ {:.2?}",
                 segment.position(0.0),
                 segment.velocity(0.0).normalize_or_zero(),
                 segment.position(1.0),
                 segment.velocity(1.0).normalize_or_zero()
        );
        
        let len = segment.length();
        
        self.segments.push(SplineSegment {
            running_distance: self.total_length,
            segment,
        });

        self.total_length += len;

        self
    }

    /// Adds a new segment to the curve with a `length` and `curvature`.
    ///
    /// `curvature == 0.0` is a straight line.
    /// `curvature > 0.0` is a curved arc to the left.
    /// `curvature < 0.0` is a curved arc to the right.
    /// Higher `abs(curvature)` the sharper it turns.
    pub fn segment(mut self, length: f32, curvature: f32) -> Self {
        if length == 0.0 {
            return self;
        }

        let segment: CurveType = if curvature == 0.0 {
            let start = self.cur_pos;
            let end = self.cur_pos + self.cur_dir * length;
            self.cur_pos = end;

            LineSegment {
                start,
                end,
            }.into()
        } else {
            let arc = ArcSegment::from_pos_dir_curvature_length(self.cur_pos, self.cur_dir, curvature, length);

            self.cur_pos = arc.position(1.0);
            self.cur_dir = arc.velocity(1.0).normalize_or_zero();

            arc.into()
        };

        println!("{:?}", segment);

        println!("start: {:.2?} w/ {:.2?}, end: {:.2?} w/ {:.2?}",
                 segment.position(0.0),
                 segment.velocity(0.0).normalize_or_zero(),
                 segment.position(1.0),
                 segment.velocity(1.0).normalize_or_zero()
        );

        self.segments.push(SplineSegment {
            running_distance: self.total_length,
            segment,
        });

        self.total_length += length;

        self
    }

    pub fn build(self, commands: &mut Commands, line_width: i32) {
        if self.segments.is_empty() {
            return;
        }
        // TODO when bevy_hierarchy ports, make children
        let parent = commands.spawn_empty().id();

        let seg_entities: Vec<Entity> = (0..self.segments.len())
            .map(|_| commands.spawn_empty().id())
            .collect();
        let joint_entities: Vec<Entity> = (0..self.segments.len() + 1)
            .map(|_| commands.spawn_empty().id())
            .collect();

        // Spawn joints

        commands.entity(joint_entities[0])
            .insert(Joint::new_stop(joint_entities[0], JointExit::End));

        for i in 1..(self.segments.len() - 1) {
            commands.entity(joint_entities[i])
                .insert(Joint::new_continue(
                    &self.segments[i].segment,
                    seg_entities[i],
                    &self.segments[i + 1].segment,
                    seg_entities[i + 1],
                ));
        }

        commands.entity(*joint_entities.last().unwrap())
            .insert(Joint::new_stop(*joint_entities.last().unwrap(), JointExit::Start));

        // TODO: Spawn segments
        let mut segments = self.segments.into_iter();

        for i in 0..seg_entities.len() {
            let segment = Segment {
                curve: segments.next().unwrap().segment,
                parent,
                start_joint: joint_entities[i],
                end_joint: joint_entities[i + 1],
            };

            let spr = segment.to_sprite(Graphics::Cached(), line_width, LCDColor::BLACK);

            // let bitmap = ;
            commands.entity(seg_entities[i])
                .insert((
                    spr,
                    segment,
                ));
        }
    }
}

pub trait SectionBuilder {
    /// takes in previous 
    fn add_segment(self, pos: &mut Vec2, dir: &mut Vec2) -> CurveType;
}

impl<F: FnOnce(&mut Vec2, &mut Vec2) -> CurveType> SectionBuilder for F {
    fn add_segment(self, pos: &mut Vec2, dir: &mut Vec2) -> CurveType {
        self(pos, dir)
    }
}

pub mod builders {
    use glam::Vec2;
    use num_traits::FloatConst;
    use curve::arc::ArcSegment;
    use curve::line::LineSegment;
    use curve::traits::{CurveSegment, CurveType};
    use crate::builder::SectionBuilder;

    pub fn line(length: f32) -> impl SectionBuilder {
        move |pos: &mut Vec2, dir: &mut Vec2| {
            let start = *pos;
            let end = *pos + *dir * length;
            *pos = end;

            LineSegment {
                start,
                end,
            }.into()
        }
    }
    
    pub fn arc_curvature_length(length: f32, curvature: f32) -> impl SectionBuilder {
        move |pos: &mut Vec2, dir: &mut Vec2| {
            let arc = ArcSegment::from_pos_dir_curvature_length(*pos, *dir, curvature, length);

            *pos = arc.position(1.0);
            *dir = arc.velocity(1.0).normalize_or_zero();

            arc.into()
        }
    }
    
    pub fn arc(radius: f32, revolutions: f32) -> impl SectionBuilder {
        let curvature = revolutions.signum() / radius;
        let length = radius * revolutions.abs() * f32::TAU();
        arc_curvature_length(length, curvature)
    }
}


struct SplineSegment {
    pub running_distance: f32,
    pub segment: CurveType,
}