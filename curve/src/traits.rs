use bevy_math::Dir2;
use crate::arc::ArcSegment;
use crate::line::LineSegment;
use derive_more::From;
use glam::Vec2;
use ode_solvers::{SVector, System};

pub trait CurveSegment: Send + Sync + 'static {
    /// Length of this curve segment.
    fn length(&self) -> f32;
    
    /// Position at the given `t`-value along the curve segment.
    /// `t` = 0 is the start, `t` = 1 is the end.
    fn position(&self, t: f32) -> Vec2;
    
    /// Direction of the curve at the given `t` value along the curve segment.
    fn dir(&self, t: f32) -> Dir2;
    
    /// Curvature of the curve segment (constant along the curve)
    fn curvature(&self) -> f32;
    
    /// Bounds of the curve segment
    fn bounds(&self) -> (Vec2, Vec2);
}

pub enum CurveEnd {
    Start,
    End,
}

pub struct CurveSegmentSystem<'a, Curve: CurveSegment> {
    pub curve: &'a Curve,
    pub gravity: Vec2,
}

impl<Curve: CurveSegment> System<f32, SVector<f32, 2>> for CurveSegmentSystem<'_, Curve> {
    fn system(&self, _x: f32, y: &SVector<f32, 2>, dy: &mut SVector<f32, 2>) {
        let [[t, v]] = y.data.0;
        let deriv = self.curve.dir(t);

        dy[0] = v / self.curve.length();
        dy[1] = self.gravity.dot(deriv.normalize_or_zero());
    }

    fn solout(&mut self, _x: f32, y: &SVector<f32, 2>, _dy: &SVector<f32, 2>) -> bool {
        y[0] > 1.0 || y[0] < 0.0
    }
}

#[derive(Debug, PartialEq, Clone)]
#[derive(From)]
pub enum CurveType {
    Line(LineSegment),
    Arc(ArcSegment),
}

impl CurveSegment for CurveType {
    fn length(&self) -> f32 {
        match self {
            CurveType::Line(l) => l.length(),
            CurveType::Arc(a) => a.length(),
        }
    }

    fn position(&self, t: f32) -> Vec2 {
        match self {
            CurveType::Line(l) => l.position(t),
            CurveType::Arc(a) => a.position(t),
        }
    }

    fn dir(&self, t: f32) -> Dir2 {
        match self {
            CurveType::Line(l) => l.dir(t),
            CurveType::Arc(a) => a.dir(t),
        }
    }

    fn curvature(&self) -> f32 {
        match self {
            CurveType::Line(l) => l.curvature(),
            CurveType::Arc(a) => a.curvature(),
        }
    }

    fn bounds(&self) -> (Vec2, Vec2) {
        match self {
            CurveType::Line(l) => l.bounds(),
            CurveType::Arc(a) => a.bounds(),
        }
    }
}




