use alloc::rc::Rc;
use core::f32::consts::TAU;
use core::mem::swap;
use bevy_app::{App, Plugin};
use bevy_ecs::bundle::Bundle;
use bevy_ecs::component::Component;
use bevy_ecs::entity::Entity;
use bevy_ecs::prelude::Query;
use bevy_ecs::query::QueryData;
use bevy_math::{Dir2, Rot2};
use bevy_reflect::Reflect;
use bevy_ecs::reflect::ReflectComponent;
use bevy_transform::components::GlobalTransform;
use bevy_transform::prelude::Transform;
use glam::{FloatExt, Vec2, Vec3Swizzles};
use pd::graphics::api::Api;
use pd::graphics::bitmap::{Bitmap, Color};
use pd::graphics::{BitmapFlip, Graphics};
use pd::graphics::color::LCDColorConst;
use pd::sys::ffi::LCDColor;
use smallvec::SmallVec;
use bevy_playdate::sprite::Sprite;
use curve::traits::{CurveSegment, CurveType};

pub struct CurvePlugin;

impl Plugin for CurvePlugin {
    fn build(&self, app: &mut App) {
        app
            .register_type::<Segment>()
            .register_type::<Joint>();
    }
}

#[derive(Component, Reflect, Clone, PartialEq, Debug)]
#[reflect(Component)]
pub struct Segment {
    /// The actual curve on the segment
    pub curve: CurveType,
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
                let min = arc.bounds().0;
                let top = arc.center - Vec2::splat(arc.radius);
                let origin = top - min;

                gfx.draw_ellipse(
                    origin.x as i32,
                    origin.y as i32,
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
        let position = self.curve.position(0.0);
        let transform = Transform::from_translation(position.extend(0.0));
        (self, sprite, transform)
    }
}

#[derive(Clone, PartialEq, Debug, Component, Reflect)]
#[reflect(Component)]
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
        q_segment: &Query<CurveQuery>,
    ) -> EnterJointResult {
        let gravity_dir = Vec2::new(gravity_dir.x, -gravity_dir.y);

        if self.connections.len() < 2 {
            return EnterJointResult {
                next: enter_segment_entity,
                t: t_enter,
                v: 0.0,
            };
        }

        let enter_curve = q_segment.get(enter_segment_entity).unwrap();
        let enter_vel = enter_curve.curve().dir(t_enter) * v.signum();

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
            let dir = segment.curve().dir(connection.segments[0].t);
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
            .curve()
            .dir(next.segments[0].t);

        let normalized = Rot2::from_sin_cos(next_dir.y, next_dir.x).inverse() * gravity_dir;
        let next_id = next.eval(Dir2::new_unchecked(normalized));

        // Our joint's directions are all in the same direction,
        // but might be flipped, so let's use the real one
        let next_dir = q_segment.get(next_id.id).unwrap().curve().dir(next_id.t);
        let dot = next_dir.dot(enter_vel);

        EnterJointResult {
            next: next_id.id,
            t: next_id.t,
            v: v.abs() * dot,
        }
    }
}

#[derive(Copy, Clone, PartialEq, Debug, Reflect)]
pub struct SegmentConnection {
    /// The entity for this segment.
    pub id: Entity,
    /// The t-value on the segment the joint exits onto.
    ///
    /// If we were to evaluate the segment in `id` at this
    /// `t`-value, the joint would be at that position
    pub t: f32,
}

#[derive(Clone, PartialEq, Debug, Reflect)]
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

fn lerp_angle(a: f32, b: f32, t: f32) -> f32 {
    let diff = (b - a) % TAU;
    let distance = ((2.0 * diff) % TAU) - diff;
    a + distance * t
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub struct EnterJointResult {
    pub next: Entity,
    pub t: f32,
    pub v: f32,
}

#[derive(Copy, Clone, QueryData)]
pub struct CurveQuery {
    pub segment: &'static Segment,
    pub global_transform: &'static GlobalTransform,
}

impl CurveQueryItem<'_> {
    pub fn curve(&self) -> &CurveType {
        &self.segment.curve
    }
    
    pub fn position(&self, t: f32) -> Vec2 {
        self.curve().position(t) + self.global_transform.translation().xy()
    }
}
