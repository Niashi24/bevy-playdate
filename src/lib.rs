#![feature(once_cell_get_mut)]
#![no_std]
extern crate alloc;

mod game;
mod ui_test;
mod curve;
mod builder;

#[macro_use]
extern crate playdate as pd;

use alloc::vec;
use core::cell::{LazyCell, OnceCell};
use core::ffi::*;
use core::ptr::NonNull;
use bevy_app::App;
use bevy_ecs::prelude::World;
use pd::sys::EventLoopCtrl;
use pd::sys::ffi::PlaydateAPI;
use pd::system::update::UpdateCtrl;
use pd::display::Display;
use pd::graphics::*;
use pd::graphics::text::*;
use pd::graphics::bitmap::*;
use pd::system::prelude::*;
use pd::sound::prelude::*;
use pd::fs::Path;
use pd::sprite::draw_sprites;

/// Game state
struct State {
	app: App,
}


impl State {
	fn new() -> Self {
		let mut app = App::new();
		app
			.add_plugins(game::register_systems)
			.add_plugins(ui_test::ui_plugin);

		Self {
			app,
		}
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
			},
			// TODO: React to other events
			_ => {},
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
		
		draw_sprites();

		System::Default().draw_fps(0, 0);
		

		UpdateCtrl::Continue
	}
}


/// Entry point
#[no_mangle]
pub fn event_handler(_api: NonNull<PlaydateAPI>, event: SystemEvent, _sim_key_code: u32) -> EventLoopCtrl {
	// Unsafe static storage for our state.
	// Usually it's safe because there's only one thread.
	
	pub static mut STATE: OnceCell<State> = OnceCell::new();

	// Call state.event
	unsafe { STATE.get_mut_or_init(State::new).event(event) }
}


// Needed for debug build, absolutely optional
ll_symbols!();
