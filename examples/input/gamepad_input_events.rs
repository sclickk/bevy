//! Iterates and prints gamepad input and connection events.

use bevy::{
	input::gamepad::{GamepadEvent, GamepadEventType},
	prelude::*,
};

fn main() {
	let mut app = App::new();
	app.add_plugins(DefaultPlugins);
	app.add_system(gamepad_events);
	app.run();
}

fn gamepad_events(mut gamepad_event: EventReader<GamepadEvent>) {
	for event in gamepad_event.iter() {
		match event.event_type {
			GamepadEventType::Connected => {
				info!("{:?} Connected", event.gamepad);
			},
			GamepadEventType::Disconnected => {
				info!("{:?} Disconnected", event.gamepad);
			},
			GamepadEventType::ButtonChanged(button_type, value) => {
				info!(
					"{:?} of {:?} is changed to {}",
					button_type, event.gamepad, value
				);
			},
			GamepadEventType::AxisChanged(axis_type, value) => {
				info!(
					"{:?} of {:?} is changed to {}",
					axis_type, event.gamepad, value
				);
			},
		}
	}
}
