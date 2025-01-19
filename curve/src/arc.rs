use bevy_math::Dir2;
use glam::{FloatExt, Vec2};
// use playdate::graphics::api::Api;
// use playdate::graphics::Graphics;
// use playdate::sys::ffi::{LCDColor, PDRect};
use crate::traits::CurveSegment;
use num_traits::{Euclid, Float, FloatConst};

#[derive(Copy, Clone, PartialEq, Debug)]
pub struct ArcSegment {
    pub center: Vec2,
    pub start: f32,
    pub end: f32,
    pub radius: f32,
}

impl ArcSegment {
    pub fn from_pos_dir_curvature_length(
        pos: Vec2,
        dir: Vec2,
        curvature: f32,
        length: f32,
    ) -> Self {
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
    
    pub fn eval_angle(&self, angle: f32) -> Vec2 {
        let (sin, cos) = f32::sin_cos(angle);
        Vec2::new(cos, -sin) * self.radius + self.center
    }
}

impl CurveSegment for ArcSegment {
    fn length(&self) -> f32 {
        (self.end - self.start).abs() * self.radius
    }

    fn position(&self, t: f32) -> Vec2 {
        let angle = f32::lerp(self.start, self.end, t);
        self.eval_angle(angle)
    }

    fn dir(&self, t: f32) -> Dir2 {
        let angle = f32::lerp(self.start, self.end, t);
        let dir = (self.end - self.start).signum();
        let (sin, cos) = f32::sin_cos(angle);
        Dir2::from_xy_unchecked(-sin * dir, -cos * dir)
    }

    fn curvature(&self) -> f32 {
        1.0 / self.radius * (self.start - self.end).signum()
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
        // start
        let mut min = self.eval_angle(self.start);
        let mut max = min;

        // end
        {
            let end = self.eval_angle(self.end);
            min.x = min.x.min(end.x);
            min.y = min.y.min(end.y);
            max.x = max.x.max(end.x);
            max.y = max.y.max(end.y);
        }
        
        const CRITICAL_POINTS: [f32; 4] = [
            0.0,
            core::f32::consts::FRAC_PI_2,
            core::f32::consts::PI,
            3.0 * core::f32::consts::FRAC_PI_2,
        ];
        
        for angle in CRITICAL_POINTS {
            if !angle_in_range(angle, self.start, self.end) {
                continue;
            }
            
            let pos = self.eval_angle(angle);

            min.x = min.x.min(pos.x);
            min.y = min.y.min(pos.y);
            max.x = max.x.max(pos.x);
            max.y = max.y.max(pos.y);
        }

        (min, max)
        
        // (
        //     self.center - Vec2::splat(self.radius),
        //     self.center + Vec2::splat(self.radius),
        // )
    }
}

fn angle_in_range(a: f32, start: f32, end: f32) -> bool {
    let (lower, upper) = (start.min(end), start.max(end));
    use core::f32::consts::TAU as TAU;
    // if end == TAU, we don't want it to be rounded
    let tau = TAU + 1e-6;
    (a - lower) % tau <= (upper - lower) % tau
}

#[cfg(test)]
mod test {
    use crate::arc::ArcSegment;
    use crate::CurveSegment;
    use glam::Vec2;

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
