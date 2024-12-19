use bevy_ecs::prelude::Component;
use derive_more::{Deref, DerefMut, From};
use glam::Vec2;
use ode_solvers::{Dopri5, SVector, System, Vector2};
use pd::graphics::bitmap::LCDColorConst;
use pd::graphics::{draw_line, Graphics};
use pd::graphics::bitmap::api::Cache;
use pd::sys::ffi::LCDColor;
use curve::BSpline;

#[derive(Deref, DerefMut, From, Component)]
pub struct BCurve {
    #[deref]
    #[deref_mut]
    #[from]
    pub spline: BSpline,
}

impl BCurve {
    pub fn draw(&self, gfx: Graphics<Cache>) {
        let num_segments = 80;
        // let mut out = alloc::string::String::from("L = [");
        for i in 0..(num_segments - 2) {
            let t_0 = i as f32 / (num_segments - 1) as f32;
            let t_1 = (i + 1) as f32 / (num_segments - 1) as f32;
            
            
            let start = self.position(t_0);
            let end = self.position(t_1);
            // out += &alloc::format!("({}, {}), ", start.x, -start.y);
            
            gfx.draw_line(start.x as i32, start.y as i32, end.x as i32, end.y as i32, 3, LCDColor::BLACK);
        }
        
        if self.spline.looped() {
            let start = self.position((num_segments - 3) as f32 / (num_segments - 1) as f32);
            let end = self.position(0.0);
            
            println!("{:?} {:?}", start, end);

            gfx.draw_line(start.x as i32, start.y as i32, end.x as i32, end.y as i32, 3, LCDColor::BLACK);
        }
        
        // out += "]";
        
        // println!("{out}");
    }
}

pub struct BCurveFallSystem<'a> {
    spline: &'a BSpline,
    gravity: Vec2,
}

impl System<f32, SVector<f32, 2>> for BCurveFallSystem<'_> {
    fn system(&self, _x: f32, y: &SVector<f32, 2>, dy: &mut SVector<f32, 2>) {
        let [[t, v]] = y.data.0;
        let deriv = self.spline.velocity(t);
        
        dy[0] = v / deriv.length();
        dy[1] = self.gravity.dot(deriv.normalize_or_zero());
    }

    fn solout(&mut self, _x: f32, y: &SVector<f32, 2>, _dy: &SVector<f32, 2>) -> bool {
        y[0] > 1.0 || y[0] < 0.0
    }
}

impl<'a> BCurveFallSystem<'a> {
    pub fn new(spline: &'a BSpline, gravity: Vec2) -> Self {
        Self {
            spline,
            gravity
        }
    }
    
    pub fn integrate(self, t: f32, v: f32, dt: f32) -> (f32, f32) {
        let y0 = SVector::<f32, 2>::new(t, v);
        // Set up the solver
        let mut solver = Dopri5::new(
            self,       // The ODE system
            0.0,        // Start time
            dt,          // End time
            dt / 4.0,
            y0,             // Initial state
            1e-9,           // Tolerance
            1e-6,           // Absolute tolerance
        );
        
        solver.integrate().unwrap();
        
        
        let out = *solver.y_out().last().unwrap();
        
        // for (x, y) in solver.x_out().iter().zip(solver.y_out().iter()) {
        //     println!("t: {x}, {:?}", y);
        // }
        // println!();
        
        
        

        (out.x, out.y)
    }
}
