#![no_std]

pub mod debug;
pub mod input;
pub mod jobs;
pub mod sprite;
pub mod time;
pub mod view;

extern crate alloc;

use bevy_app::{App, Plugin};
use bevy_ecs::prelude::*;

pub struct DefaultPlugins;

impl Plugin for DefaultPlugins {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            input::InputPlugin,
            sprite::SpritePlugin,
            time::TimePlugin,
            view::ViewPlugin,
            bevy_transform::TransformPlugin,
        ));
    }
}
