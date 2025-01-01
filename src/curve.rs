use bevy_ecs::prelude::Component;
use derive_more::{Deref, DerefMut, From};
use glam::Vec2;
use ode_solvers::{Dopri5, SVector, System, Vector2};
use pd::graphics::bitmap::LCDColorConst;
use pd::graphics::{draw_line, Graphics};
use pd::graphics::bitmap::api::Cache;
use pd::sys::ffi::LCDColor;
use curve::BSpline;

// pub 

