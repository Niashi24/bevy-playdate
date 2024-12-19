
#![no_std]

pub mod input;
pub mod sprite;
mod bitmap;
pub mod jobs;
pub mod time;
pub mod debug;

extern crate alloc;

use bevy_ecs::prelude::*;
use playdate::graphics::api::Cache;
use playdate::graphics::Graphics as PlaydateGraphics;

