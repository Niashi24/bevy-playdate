use bevy_app::{App, Plugin, PreUpdate};
use bevy_ecs::schedule::{IntoSystemConfigs, SystemSet};
use bevy_ecs::system::{NonSend, ResMut, Resource};
use playdate::controls::api::Cache;
use playdate::controls::peripherals::{Accelerometer, Crank};

/// Adds crank input (and once `bevy_input` is `no_std`, d-pad and buttons)
pub struct InputPlugin;

impl Plugin for InputPlugin {
    fn build(&self, app: &mut App) {
        app.insert_non_send_resource(Crank::Cached())
            .init_resource::<CrankInput>()
            .insert_non_send_resource(Accelerometer::Cached())
            .init_resource::<AccelerometerInput>()
            .add_systems(
                PreUpdate,
                (crank_input_system, accelerometer_input_system).in_set(PdInputSystem),
            );
    }
}

/// Label for systems that update the input data.
#[derive(Debug, PartialEq, Eq, Clone, Hash, SystemSet)]
pub struct PdInputSystem;

/// A resource reporting the current input or state of the crank.
#[derive(Resource, Copy, Clone, Debug, PartialEq, Default)]
pub struct CrankInput {
    /// The angle change of the crank this frame.
    /// Negative values are anti-clockwise.
    pub change: f32,
    /// The current position of the crank, in the range 0-360.
    /// Zero is pointing up, and the value increases as the crank moves clockwise,
    /// as viewed from the right side of the device.
    pub angle: f32,
    /// Whether the crank is folded into the unit.
    pub docked: bool,
}

/// Updates the [`CrankInput`] resource with the latest [`Crank`] inputs.
pub fn crank_input_system(mut input: ResMut<CrankInput>, crank: NonSend<Crank<Cache>>) {
    input.change = crank.change();
    input.angle = crank.angle();
    input.docked = crank.docked();
}

/// A resource reporting the current input or state of the accelerometer.
#[derive(Resource, Copy, Clone, Debug, PartialEq, Default)]
pub struct AccelerometerInput {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

/// Updates the [`AccelerometerInput`] resource with the latest [`Accelerometer`] inputs.
pub fn accelerometer_input_system(
    mut input: ResMut<AccelerometerInput>,
    accelerometer: NonSend<Accelerometer<Cache>>,
) {
    (input.x, input.y, input.z) = accelerometer.get();
}
