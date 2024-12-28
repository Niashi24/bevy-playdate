#![no_std]

pub mod line;
pub mod traits;
pub mod arc;
pub mod roots;

extern crate alloc;

use alloc::vec::Vec;
use glam::Vec2;
use nalgebra::{matrix, Matrix1x4, Matrix4, Matrix4x1, SMatrix};
use num_traits::Euclid;
// use num_traits::ops::euclid::Euclid;

const ONE_SIXTH: f32 = 0.16666667;
const TWO_THIRDS: f32 = 0.6666667;

const BSPLINE_CHARACTERISTIC: Matrix4<f32> = matrix![
    ONE_SIXTH, TWO_THIRDS, ONE_SIXTH, 0.0;
    -0.5, 0.0, 0.5, 0.0;
    0.5, -1.0, 0.5, 0.0;
    -ONE_SIXTH, 0.5, -0.5, ONE_SIXTH
];

#[derive(Clone, Debug)]
pub struct BSpline {
    points: Vec<Vec2>,
    looped: bool,
}

impl BSpline {
    pub fn new(points: Vec<Vec2>, looped: bool) -> Self {
        Self {
            points,
            looped,
        }
    }

    // note: t is [0, 1]
    fn p_0_idx_at(&self, t: f32) -> usize {
        if self.looped {
            if t == 1.0 {
                self.points.len() - 1
            } else {
                (t * self.points.len() as f32) as usize
            }
        } else {
            if t == 1.0 {
                self.points.len() - 4
            } else {
                (t * (self.points.len() - 3) as f32) as usize
            }
        }
    }

    fn num_segments(&self) -> usize {
        if self.looped {
            self.points.len()
        } else {
            self.points.len() - 3
        }
    }

    // if not looped, t is clamped [0, 1]
    // otherwise t is looped (i.e. t = -0.25 => segment at t = 0.75)
    fn segment_at(&self, mut t: f32) -> CurveSegment {
        t = if self.looped {
            f32::div_rem_euclid(&t, &1.0).1
        } else {
            f32::clamp(t, 0.0, 1.0)
        };

        let p_0_idx = self.p_0_idx_at(t);
        let num_segments = self.num_segments();

        let indices = if self.looped {
            [
                p_0_idx,
                (p_0_idx + 1) % num_segments,
                (p_0_idx + 2) % num_segments,
                (p_0_idx + 3) % num_segments
            ]
        } else {
            [
                p_0_idx,
                p_0_idx + 1,
                p_0_idx + 2,
                p_0_idx + 3,
            ]
        };

        CurveSegment::from_points(indices.map(|i| self.points[i]))
    }
    
    fn local_t(&self, t: f32) -> f32 {
        let t = if self.looped {
            f32::div_rem_euclid(&t, &1.0).1
        } else {
            f32::clamp(t, 0.0, 1.0)
        };
        
        (t * self.num_segments() as f32) % 1.0
    }
    
    pub fn position(&self, t: f32) -> Vec2 {
        self.segment_at(t).position(self.local_t(t))
    }

    pub fn velocity(&self, t: f32) -> Vec2 {
        self.segment_at(t).velocity(self.local_t(t))
    }

    pub fn acceleration(&self, t: f32) -> Vec2 {
        self.segment_at(t).acceleration(self.local_t(t))
    }
    
    // TODO: Replace with proper bounds
    pub fn bounds(&self) -> (Vec2, Vec2) {
        let (mut min_x, mut min_y) = (f32::INFINITY, f32::INFINITY);
        let (mut max_x, mut max_y) = (f32::NEG_INFINITY, f32::NEG_INFINITY);
        
        for point in &self.points {
            min_x = min_x.min(point.x);
            min_y = min_y.min(point.y);
            max_x = max_x.max(point.x);
            max_y = max_y.max(point.y);
        }

        (Vec2::new(min_x, min_y), Vec2::new(max_x, max_y))
    }
    
    pub fn looped(&self) -> bool {
        self.looped
    }
}

#[derive(Copy, Clone, PartialEq, Debug)]
struct CurveSegment {
    // cached matrices for characteristic matrix * points
    x: Matrix4x1<f32>,
    y: Matrix4x1<f32>,
}

impl CurveSegment {
    pub fn from_points(points: [Vec2; 4]) -> Self {
        let x = Matrix4x1::from(points.map(|i| i.x));
        let y = Matrix4x1::from(points.map(|i| i.y));
        Self {
            x: BSPLINE_CHARACTERISTIC * x,
            y: BSPLINE_CHARACTERISTIC * y,
        }
    }
    
    pub fn eval(&self, matrix: Matrix1x4<f32>) -> Vec2 {
        let x = matrix * self.x;
        let y = matrix * self.y;
        Vec2::new(x.x, y.x)
    }
    
    pub fn position(&self, t: f32) -> Vec2 {
        self.eval(Matrix1x4::from([1.0, t, t * t, t * t * t]))
    }
    
    pub fn velocity(&self, t: f32) -> Vec2 {
        self.eval(Matrix1x4::from([0.0, 1.0, 2.0 * t, 3.0 * t * t]))
    }

    pub fn acceleration(&self, t: f32) -> Vec2 {
        self.eval(Matrix1x4::from([0.0, 0.0, 2.0, 6.0 * t]))
    }
}

#[cfg(test)]
mod test {
    use alloc::vec;
    use glam::Vec2;
    use crate::{BSpline, CurveSegment};

    #[test]
    fn p_0_idx_unlooped() {
        let points = vec![
            Vec2::new(-5.0, 5.0),
            Vec2::new(0.0, 0.0),
            Vec2::new(5.0, 5.0),
            Vec2::new(5.0, 0.0),
            Vec2::new(0.0, -5.0),
            Vec2::new(-5.0, 0.0),
        ];
        
        let spline = BSpline::new(points, false);
        assert_eq!(spline.p_0_idx_at(0.0), 0);
        assert_eq!(spline.p_0_idx_at(0.33), 0);
        assert_eq!(spline.p_0_idx_at(0.34), 1);
        assert_eq!(spline.p_0_idx_at(0.66), 1);
        assert_eq!(spline.p_0_idx_at(0.67), 2);
        assert_eq!(spline.p_0_idx_at(0.99), 2);
        assert_eq!(spline.p_0_idx_at(1.0), 2);
    }

    #[test]
    fn p_0_idx_looped() {
        let points = vec![
            Vec2::new(-5.0, 5.0),
            Vec2::new(0.0, 0.0),
            Vec2::new(5.0, 5.0),
            Vec2::new(5.0, 0.0),
            Vec2::new(0.0, -5.0),
            Vec2::new(-5.0, 0.0),
        ];

        let spline = BSpline::new(points, true);
        assert_eq!(spline.p_0_idx_at(0.0), 0);
        assert_eq!(spline.p_0_idx_at(0.16), 0);
        assert_eq!(spline.p_0_idx_at(0.17), 1);
        assert_eq!(spline.p_0_idx_at(0.33), 1);
        assert_eq!(spline.p_0_idx_at(0.34), 2);
        assert_eq!(spline.p_0_idx_at(0.49), 2);
        assert_eq!(spline.p_0_idx_at(0.5), 3);
        assert_eq!(spline.p_0_idx_at(0.51), 3);
        assert_eq!(spline.p_0_idx_at(0.66), 3);
        assert_eq!(spline.p_0_idx_at(0.67), 4);
        assert_eq!(spline.p_0_idx_at(0.83), 4);
        assert_eq!(spline.p_0_idx_at(0.84), 5);
        assert_eq!(spline.p_0_idx_at(0.99), 5);
        assert_eq!(spline.p_0_idx_at(1.0), 5);
    }
    
    #[test]
    fn segment_at_unlooped() {
        let points = vec![
            Vec2::new(-5.0, 5.0),
            Vec2::new(0.0, 0.0),
            Vec2::new(5.0, 5.0),
            Vec2::new(5.0, 0.0),
            Vec2::new(0.0, -5.0),
            Vec2::new(-5.0, 0.0),
        ];

        let spline = BSpline::new(points, false);
        
        let expected_0 = CurveSegment::from_points([
            Vec2::new(-5.0, 5.0),
            Vec2::new(0.0, 0.0),
            Vec2::new(5.0, 5.0),
            Vec2::new(5.0, 0.0)]);
        
        assert_eq!(spline.segment_at(0.0), expected_0);

        let expected_1 = CurveSegment::from_points([
            Vec2::new(5.0, 5.0),
            Vec2::new(5.0, 0.0),
            Vec2::new(0.0, -5.0),
            Vec2::new(-5.0, 0.0),
        ]);

        assert_eq!(spline.segment_at(1.0), expected_1);
        assert_eq!(spline.segment_at(1.1), expected_1);
    }



    #[test]
    fn segment_at_looped() {
        let points = vec![
            Vec2::new(-5.0, 5.0),
            Vec2::new(0.0, 0.0),
            Vec2::new(5.0, 5.0),
            Vec2::new(5.0, 0.0),
            Vec2::new(0.0, -5.0),
            Vec2::new(-5.0, 0.0),
        ];

        let spline = BSpline::new(points, true);

        let expected_0 = CurveSegment::from_points([
            Vec2::new(-5.0, 5.0),
            Vec2::new(0.0, 0.0),
            Vec2::new(5.0, 5.0),
            Vec2::new(5.0, 0.0)]);

        assert_eq!(spline.segment_at(0.0), expected_0);
        assert_eq!(spline.segment_at(1.01), expected_0);

        let expected_1 = CurveSegment::from_points([
            Vec2::new(-5.0, 0.0),
            Vec2::new(-5.0, 5.0),
            Vec2::new(0.0, 0.0),
            Vec2::new(5.0, 5.0),
        ]);

        // assert_eq!(spline.segment_at(0.), expected_1);
    }
    
    #[test]
    fn eval_point() {
        let segment = CurveSegment::from_points([
            Vec2::new(-5.0, 5.0),
            Vec2::new(0.0, 0.0),
            Vec2::new(5.0, 5.0),
            Vec2::new(5.0, 0.0)]);
        // float comparison is bad but it serves the purpose
        assert_eq!(segment.position(0.0), Vec2::new(0.0, 1.6666667));
        assert_eq!(segment.position(1.0), Vec2::new(4.166667, 3.3333335));
    }
    
    #[test]
    fn local_t_looped() {
        let points = vec![
            Vec2::new(-5.0, 5.0),
            Vec2::new(0.0, 0.0),
            Vec2::new(5.0, 5.0),
            Vec2::new(5.0, 0.0),
            Vec2::new(0.0, -5.0),
            Vec2::new(-5.0, 0.0),
        ];

        let spline = BSpline::new(points, true);

        assert_eq!(spline.position(0.5), Vec2::new(0.0, -3.3333335));
        assert_eq!(spline.position(1.0 / 12.0), Vec2::new(2.3958333, 2.5));
        // assert_eq!(spline.position(1.0 / 24.0), Vec2::new(1.23698, 1.92708));
    }
}

