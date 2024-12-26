use glam::{FloatExt, Vec2};
// use playdate::graphics::api::Api;
// use playdate::graphics::Graphics;
// use playdate::sys::ffi::{LCDColor, PDRect};
use crate::traits::CurveSegment;
use num_traits::{Float, FloatConst, Euclid};

#[derive(Copy, Clone, PartialEq, Debug)]
pub struct ArcSegment {
    pub center: Vec2,
    pub start: f32,
    pub end: f32,
    pub radius: f32,
}

impl ArcSegment {
    pub fn from_pos_dir_curvature_length(pos: Vec2, mut dir: Vec2, curvature: f32, length: f32) -> Self {
        let circle = 1.0 / curvature;
        let radius = circle.abs();
        let total_angle = length * curvature;
        // dir.y = -dir.y;
        let perp = Vec2::new(-dir.y, dir.x) * circle;
        let start = f32::atan2(-perp.y, perp.x).rem_euclid(&f32::TAU());
        let center = pos - perp;

        ArcSegment {
            center,
            start,
            end: start + total_angle,
            radius,
        }
    }
}

impl CurveSegment for ArcSegment {    
    fn length(&self) -> f32 {
        (self.end - self.start).abs() * self.radius
    }

    fn position(&self, t: f32) -> Vec2 {
        let angle = f32::lerp(self.start, self.end, t);
        let (sin, cos) = f32::sin_cos(angle);
        Vec2::new(cos, -sin) * self.radius + self.center
    }

    fn velocity(&self, t: f32) -> Vec2 {
        let angle = f32::lerp(self.start, self.end, t);
        let (sin, cos) = f32::sin_cos(angle);
        Vec2::new(-sin, -cos) * self.radius * (self.end - self.start).signum()
    }

    fn curvature(&self) -> f32 {
        1.0 / self.radius
    }

    // fn draw<A: Api>(&self, gfx: Graphics<A>, line_width: i32, c: LCDColor) {
    //     gfx.draw_ellipse(
    //         self.center.x as i32,
    //         self.center.y as i32,
    //         self.radius as i32,
    //         self.radius as i32,
    //         line_width,
    //         self.start,
    //         self.end,
    //         c
    //     );
    // }

    fn bounds(&self) -> (Vec2, Vec2) {
        // TODO: Replace with more optimized bounds
        (self.center - Vec2::splat(self.radius), self.center + Vec2::splat(self.radius))
    }
}

#[cfg(test)]
mod test {
    use glam::Vec2;
    use crate::arc::ArcSegment;
    use crate::CurveSegment;

    #[test]
    fn test_create_arc() {
        let pos = Vec2::new(135.40, 77.27);
        let dir = Vec2::new(0.91, 0.42);
        let curvature = 1.0 / 25.0;
        let length = 50.0;
        
        ArcSegment::from_pos_dir_curvature_length(pos, dir, curvature, length);

        ()
    }
}