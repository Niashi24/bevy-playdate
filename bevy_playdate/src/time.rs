use core::time::Duration;
use bevy_app::{App, First, Plugin};
use bevy_ecs::prelude::{ResMut, Resource};
use playdate::println;
use playdate::system::System;

pub struct TimePlugin;

impl Plugin for TimePlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<Time>()
            .add_systems(First, advance_time);
    }
}

#[derive(Resource)]
pub struct Time {
    now: Duration,
    delta: Duration,
}

impl Time {
    pub fn delta_seconds(&self) -> f32 {
        self.delta.as_secs_f32()
    }
}

impl Default for Time {
    fn default() -> Self {
        let sys = System::Default();
        
        sys.reset_elapsed_time();
        
        Self {
            now: Duration::ZERO,
            delta: Duration::ZERO,
        }
    }
}

pub fn advance_time(
    mut time: ResMut<Time>,
) {
    let sys = System::Default();
    let dur = sys.elapsed_time();
    time.now += dur;
    time.delta = dur;
    
    sys.reset_elapsed_time();
}