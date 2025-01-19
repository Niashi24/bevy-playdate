use alloc::rc::Rc;
use alloc::vec;
use alloc::vec::Vec;
use arrayvec::ArrayVec;
use bevy_ecs::component::Component;
use bevy_ecs::entity::Entity;
use bevy_ecs::prelude::{Bundle, Commands, Query};
use bevy_ecs::query::{QueryData, QueryFilter};
use bevy_math::{Dir2, Rot2};
use bevy_playdate::dbg;
use bevy_playdate::sprite::Sprite;
use core::cmp::Ordering;
use core::f32::consts::TAU;
use core::fmt::{Debug, Formatter};
use core::mem::swap;
use core::ops::Range;
use curve::arc::ArcSegment;
use curve::line::LineSegment;
use curve::traits::{CurveSegment, CurveType};
use glam::{FloatExt, Vec2};
use num_traits::Euclid;
use pd::graphics::api::Api;
use pd::graphics::bitmap::{Bitmap, Color, LCDColorConst};
use pd::graphics::{BitmapFlip, Graphics};
use pd::sys::ffi::LCDColor;
use smallvec::{smallvec, SmallVec};

#[derive(Component, Debug, PartialEq, Copy, Clone)]
pub struct MovingSplineDot {
    pub t: f32,
    pub v: f32,
    pub spline_entity: Entity,
}

pub enum DotAttach {
    Segment(Entity),
    None,
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

    pub fn to_bundle(self, line_width: i32) -> impl Bundle {
        let sprite = self.to_sprite(Graphics::Cached(), line_width, LCDColor::BLACK);
        (self, sprite)
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum JointEnter {
    /// previous segment was t = 1
    /// going from t = 1 of previous to t = 0 of current
    Start,
    /// going from t = 0 of previous to t = 1 of current
    End,
}

impl JointEnter {
    /// returns the t-value of the segment we're entering the joint from
    pub fn t(&self) -> f32 {
        match self {
            JointEnter::Start => 1.0,
            JointEnter::End => 0.0,
        }
    }
}

#[derive(Clone, PartialEq, Debug, Component)]
pub struct Joint {
    pub connections: SmallVec<[JointConnection; 2]>,
}

impl Joint {
    pub fn new(connections: SmallVec<[JointConnection; 2]>) -> Self {
        Joint {
            connections,
            // space,
        }
    }

    /// Picks the exit that most closely matches with the gravity direction.
    /// Prioritizes exits that are in the same direction or directly perpendicular
    /// (angle between <= 90deg)
    pub fn enter(
        &self,
        v: f32,
        mut gravity_dir: Dir2,
        enter_segment_entity: Entity,
        t_enter: f32,
        q_segment: &Query<&Segment>,
    ) -> EnterJointResult {
        let gravity_dir = Vec2::new(gravity_dir.x, -gravity_dir.y);

        if self.connections.len() < 2 {
            return EnterJointResult {
                next: enter_segment_entity,
                t: t_enter,
                v: 0.0,
            };
        }

        let enter_segment = q_segment.get(enter_segment_entity).unwrap();
        let enter_vel = enter_segment.curve.dir(t_enter) * v.signum();

        let mut best_in_front = None;
        let mut best_any = None;

        for connection in self.connections.iter() {
            // ignore exits from those in the same direction
            if connection
                .segments
                .iter()
                .any(|i| i.id == enter_segment_entity && i.t == t_enter)
            {
                continue;
            }
            let segment = q_segment.get(connection.segments[0].id).unwrap();
            let dir = segment.curve.dir(connection.segments[0].t);
            let target_dot = gravity_dir.dot(dir.into());
            // if new direction in same direction
            // => angle between <= 90 degrees
            // => dot product >= 0 (then add floating point precision)
            if enter_vel.dot(dir.into()) > -1e5 {
                if let Some((dot, cxn)) = best_in_front.as_mut() {
                    if target_dot > *dot {
                        *dot = target_dot;
                        *cxn = connection;
                    }
                } else {
                    best_in_front = Some((target_dot, connection));
                }
            }

            if let Some((dot, cxn)) = best_any.as_mut() {
                if target_dot > *dot {
                    *dot = target_dot;
                    *cxn = connection;
                }
            } else {
                best_any = Some((target_dot, connection));
            }
        }

        let (_, next) = best_in_front
            .or(best_any)
            // safe because length > 1
            .unwrap();

        let next_dir = q_segment
            .get(next.segments[0].id)
            .unwrap()
            .curve
            .dir(next.segments[0].t);

        let normalized = Rot2::from_sin_cos(next_dir.y, next_dir.x).inverse() * gravity_dir;
        let next_id = next.eval(Dir2::new_unchecked(normalized));

        // Our joint's directions are all in the same direction,
        // but might be flipped, so let's use the real one
        let next_dir = q_segment.get(next_id.id).unwrap().curve.dir(next_id.t);
        let dot = next_dir.dot(enter_vel);

        EnterJointResult {
            next: next_id.id,
            t: next_id.t,
            v: v.abs() * dot,
        }
    }
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub struct SegmentConnection {
    /// The entity for this segment.
    pub id: Entity,
    /// The t-value on the segment the joint exits onto.
    ///
    /// If we were to evaluate the segment in `id` at this
    /// `t`-value, the joint would be at that position
    pub t: f32,
}

#[derive(Clone, PartialEq, Debug)]
/// Contains all the segments pointing in the same direction,
/// sorted by curvature
pub struct JointConnection {
    pub segments: SmallVec<[SegmentConnection; 1]>,
}

impl JointConnection {
    /// angle should be from 0 (either negative or positive)
    pub fn eval(&self, dir: Dir2) -> &SegmentConnection {
        const MIN_ANGLE: f32 = 0.174533; // 10 degrees

        let len = self.segments.len() as i32;

        if len == 1 {
            &self.segments[0]
        } else {
            // avoid doing somewhat expensive atan2 call
            // unless necessary
            let angle = dir.to_angle();

            for i in 0..len - 1 {
                let end = 2 * (i + 1) - len;
                let end = (end as f32) * MIN_ANGLE;

                if angle < end {
                    return &self.segments[i as usize];
                }
            }
            self.segments.last().unwrap()
        }
    }
}

#[derive(Clone, PartialEq)]
struct JointSpace {
    ranges: Vec<(Range<f32>, usize)>,
}

impl Debug for JointSpace {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        let mut list = f.debug_list();

        for (r, i) in self.ranges.iter() {
            list.entry_with(|f| write!(f, r#"r=1\left\{{{}\le\theta<{}\right\}}"#, r.start, r.end));
        }

        list.finish()
    }
}

fn lerp_angle(a: f32, b: f32, t: f32) -> f32 {
    let diff = (b - a) % TAU;
    let distance = ((2.0 * diff) % TAU) - diff;
    a + distance * t
}

// #[derive(Copy, Clone, PartialEq, Debug)]
// enum JointType {
// }

#[derive(Copy, Clone, PartialEq, Debug)]
pub struct EnterJointResult {
    pub next: Entity,
    pub t: f32,
    pub v: f32,
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

        println!(
            "start: {:.2?} w/ {:.2?}, end: {:.2?} w/ {:.2?}",
            segment.position(0.0),
            segment.dir(0.0),
            segment.position(1.0),
            segment.dir(1.0)
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

            LineSegment { start, end }.into()
        } else {
            let arc = ArcSegment::from_pos_dir_curvature_length(
                self.cur_pos,
                self.cur_dir,
                curvature,
                length,
            );

            self.cur_pos = arc.position(1.0);
            self.cur_dir = arc.dir(1.0).normalize_or_zero();

            arc.into()
        };

        println!("{:?}", segment);

        println!(
            "start: {:.2?} w/ {:.2?}, end: {:.2?} w/ {:.2?}",
            segment.position(0.0),
            segment.dir(0.0).normalize_or_zero(),
            segment.position(1.0),
            segment.dir(1.0).normalize_or_zero()
        );

        self.segments.push(SplineSegment {
            running_distance: self.total_length,
            segment,
        });

        self.total_length += length;

        self
    }

    pub fn build(
        self,
        commands: &mut Commands,
        line_width: i32,
        join_ends: bool,
    ) -> (Vec<Entity>, Vec<Entity>) {
        if self.segments.is_empty() {
            return Default::default();
        }
        // TODO when bevy_hierarchy ports, make children
        let parent = commands.spawn_empty().id();

        let seg_entities: Vec<Entity> = (0..self.segments.len())
            .map(|_| commands.spawn_empty().id())
            .collect();
        let joint_entities: Vec<Entity> = (0..self.segments.len() + if join_ends { 0 } else { 1 })
            .map(|_| commands.spawn_empty().id())
            .collect();

        // Spawn joints

        for i in 1..self.segments.len() {
            commands.entity(joint_entities[i]).insert(Joint {
                connections: smallvec![
                    JointConnection {
                        segments: smallvec![SegmentConnection {
                            id: seg_entities[i - 1],
                            t: 1.0,
                        }],
                    },
                    JointConnection {
                        segments: smallvec![SegmentConnection {
                            id: seg_entities[i],
                            t: 0.0,
                        }],
                    },
                ],
            });
        }

        if join_ends {
            commands.entity(joint_entities[0]).insert(Joint {
                connections: smallvec![
                    JointConnection {
                        segments: smallvec![SegmentConnection {
                            id: seg_entities[0],
                            t: 0.0,
                        },],
                    },
                    JointConnection {
                        segments: smallvec![SegmentConnection {
                            id: *seg_entities.last().unwrap(),
                            t: 1.0,
                        },],
                    },
                ],
            });
        } else {
            commands.entity(joint_entities[0]).insert(Joint {
                connections: smallvec![JointConnection {
                    segments: smallvec![SegmentConnection {
                        id: seg_entities[0],
                        t: 0.0,
                    }],
                }],
            });

            commands
                .entity(*joint_entities.last().unwrap())
                .insert(Joint {
                    connections: smallvec![JointConnection {
                        segments: smallvec![SegmentConnection {
                            id: *seg_entities.last().unwrap(),
                            t: 1.0,
                        },],
                    }],
                });
        }

        // Spawn segments
        let mut segments = self.segments.into_iter();

        for i in 0..seg_entities.len() {
            let segment = Segment {
                curve: segments.next().unwrap().segment,
                parent,
                start_joint: joint_entities[i],
                end_joint: joint_entities[(i + 1) % joint_entities.len()],
            };

            commands
                .entity(seg_entities[i])
                .insert(segment.to_bundle(line_width));
        }

        (seg_entities, joint_entities)
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
    use crate::builder::SectionBuilder;
    use curve::arc::ArcSegment;
    use curve::line::LineSegment;
    use curve::traits::{CurveSegment, CurveType};
    use glam::Vec2;
    use num_traits::FloatConst;

    pub fn line(length: f32) -> impl SectionBuilder {
        move |pos: &mut Vec2, dir: &mut Vec2| {
            let start = *pos;
            let end = *pos + *dir * length;
            *pos = end;

            LineSegment { start, end }.into()
        }
    }

    pub fn arc_curvature_length(length: f32, curvature: f32) -> impl SectionBuilder {
        move |pos: &mut Vec2, dir: &mut Vec2| {
            let arc = ArcSegment::from_pos_dir_curvature_length(*pos, *dir, curvature, length);

            *pos = arc.position(1.0);
            *dir = arc.dir(1.0).normalize_or_zero();

            arc.into()
        }
    }

    /// Appends an arc with the given radius and number of revolutions (1 revolution = 1 full circle).
    /// Use positive revolutions to go counterclockwise, negative for clockwise.
    /// Panics if you put in a negative radius.
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
