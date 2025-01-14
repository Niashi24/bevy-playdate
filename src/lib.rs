#![feature(once_cell_get_mut)]
#![feature(debug_closure_helpers)]
#![no_std]
extern crate alloc;
#[macro_use]
extern crate playdate as pd;

mod builder;
mod curve;
mod game;
mod ui_test;

use bevy_app::{App, PostUpdate};
use bevy_playdate::DefaultPlugins;
use core::cell::OnceCell;
use core::ptr::NonNull;
use pd::display::Display;
use pd::graphics::bitmap::*;
use pd::graphics::text::*;
use pd::graphics::*;
use pd::sprite::draw_sprites;
use pd::sys::ffi::PlaydateAPI;
use pd::sys::EventLoopCtrl;
use pd::system::prelude::*;
use pd::system::update::UpdateCtrl;

/// Game state
struct State {
    app: App,
}

impl State {
    fn new() -> Self {
        let mut app = App::new();
        app.add_plugins(DefaultPlugins)
            .add_plugins(game::register_systems)
            .add_plugins(ui_test::ui_plugin)
            // todo: add .after(propagate_transforms) and .after(sync_simple_transforms)
            .add_systems(PostUpdate, draw_sprites_system);

        Self { app }
    }

    /// System event handler
    fn event(&'static mut self, event: SystemEvent) -> EventLoopCtrl {
        match event {
            // Initial setup
            SystemEvent::Init => {
                // Set FPS to 30
                Display::Default().set_refresh_rate(50.0);

                // Register our update handler that defined below
                self.set_update_handler();

                println!("Game init complete");
            }
            // TODO: React to other events
            _ => {}
        }
        EventLoopCtrl::Continue
    }
}

impl Update for State {
    /// Updates the state
    fn update(&mut self) -> UpdateCtrl {
        // clear(Color::WHITE);

        // self.app.update();
        // self.app.run_system
        self.app.update();

        System::Default().draw_fps(0, 0);

        UpdateCtrl::Continue
    }
}

pub fn draw_sprites_system() {
    draw_sprites();
}

/// Entry point
#[no_mangle]
pub fn event_handler(
    _api: NonNull<PlaydateAPI>,
    event: SystemEvent,
    _sim_key_code: u32,
) -> EventLoopCtrl {
    // Unsafe static storage for our state.
    // Usually it's safe because there's only one thread.

    pub static mut STATE: OnceCell<State> = OnceCell::new();

    // Call state.event
    unsafe { STATE.get_mut_or_init(State::new).event(event) }
}

// Needed for debug build, absolutely optional
ll_symbols!();
