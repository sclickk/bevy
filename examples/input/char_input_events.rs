//! Prints out all chars as they are inputted.

use bevy::{prelude::*, window::ReceivedCharacter};

fn main() {
	let mut app = App::new();
	app.add_plugins(DefaultPlugins);
	app.add_system(print_char_event_system);
	app.run();
}

/// This system prints out all char events as they come in
fn print_char_event_system(mut char_input_events: EventReader<ReceivedCharacter>) {
	for event in char_input_events.iter() {
		info!("{}", event);
	}
}
