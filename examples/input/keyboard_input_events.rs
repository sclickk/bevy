//! Prints out all keyboard events.

use bevy::{input::keyboard::KeyboardInput, prelude::*};

fn main() {
	let mut app = App::new();
	app.add_plugins(DefaultPlugins);
	app.add_system(print_keyboard_event_system);
	app.run();
}

/// This system prints out all keyboard events as they come in
fn print_keyboard_event_system(mut keyboard_input_events: EventReader<KeyboardInput>) {
	for event in keyboard_input_events.iter() {
		info!("{:?}", event);
	}
}
