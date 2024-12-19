use alloc::boxed::Box;
use alloc::vec::Vec;
use glam::Vec2;
use num_traits::real::Real;
use derive_more::From;
use ode_solvers::{SVector, System};
// use playdate::graphics::api::Api;
// use playdate::graphics::Graphics;
// use playdate::sys::ffi::LCDColor;
use crate::arc::ArcSegment;
use crate::line::LineSegment;

pub trait CurveSegment: Send + Sync + 'static {
    fn length(&self) -> f32;
    
    fn position(&self, t: f32) -> Vec2;
    
    fn velocity(&self, t: f32) -> Vec2;

    // fn draw<A: Api>(&self, gfx: Graphics<A>, line_width: i32, c: LCDColor);
    
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
        let deriv = self.curve.velocity(t);

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

    fn velocity(&self, t: f32) -> Vec2 {
        match self {
            CurveType::Line(l) => l.velocity(t),
            CurveType::Arc(a) => a.velocity(t),
        }
    }

    fn bounds(&self) -> (Vec2, Vec2) {
        match self {
            CurveType::Line(l) => l.bounds(),
            CurveType::Arc(a) => a.bounds(),
        }
    }
}




