use bevy_math::Dir2;
use glam::Vec2;
// use playdate::graphics::api::Api;
// use playdate::graphics::Graphics;
// use playdate::sys::ffi::{LCDColor, PDRect};
use crate::traits::CurveSegment;

#[derive(Copy, Clone, PartialEq, Debug)]
pub struct LineSegment {
    pub start: Vec2,
    pub end: Vec2,
}

impl CurveSegment for LineSegment {
    fn length(&self) -> f32 {
        (self.end - self.start).length()
    }

    fn position(&self, t: f32) -> Vec2 {
        Vec2::lerp(self.start, self.end, t)
    }

    fn dir(&self, _t: f32) -> Dir2 {
        Dir2::new(self.end - self.start).unwrap()
    }

    fn curvature(&self) -> f32 {
        0.0
    }

    fn bounds(&self) -> (Vec2, Vec2) {
        (
            Vec2::new(
                f32::min(self.start.x, self.end.x),
                f32::min(self.start.y, self.end.y),
            ),
            Vec2::new(
                f32::max(self.start.x, self.end.x),
                f32::max(self.start.y, self.end.y),
            ),
        )
    }
}
