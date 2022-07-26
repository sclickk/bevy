//! Prints out all touch inputs.

use bevy::{input::touch::*, prelude::*};

fn main() {
	let mut app = App::new();
	app.add_plugins(DefaultPlugins);
	app.add_system(touch_event_system);
	app.run();
}

fn touch_event_system(mut touch_events: EventReader<TouchInput>) {
	for event in touch_events.iter() {
		info!("{:?}", event);
	}
}
