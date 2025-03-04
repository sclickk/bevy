//! Demonstrates the creation and registration of a custom plugin.
//!
//! Plugins are the foundation of Bevy. They are scoped sets of components, resources, and systems
//! that provide a specific piece of functionality (generally the smaller the scope, the better).
//! This example illustrates how to create a simple plugin that prints out a message.

use bevy::{prelude::*, utils::Duration};

fn main() {
	let mut app = App::new();
	app.add_plugins(DefaultPlugins);
	// plugins are registered as part of the "app building" process
	app.add_plugin(PrintMessagePlugin {
		wait_duration: Duration::from_secs(1),
		message: "This is an example plugin".to_string(),
	});
	app.run();
}

// This "print message plugin" prints a `message` every `wait_duration`
pub struct PrintMessagePlugin {
	// Put your plugin configuration here
	wait_duration: Duration,
	message: String,
}

impl Plugin for PrintMessagePlugin {
	// this is where we set up our plugin
	fn build(&self, app: &mut App) {
		let state = PrintMessageState {
			message: self.message.clone(),
			timer: Timer::new(self.wait_duration, true),
		};
		app.insert_resource(state);
		app.add_system(print_message_system);
	}
}

struct PrintMessageState {
	message: String,
	timer: Timer,
}

fn print_message_system(mut state: ResMut<PrintMessageState>, time: Res<Time>) {
	if state.timer.tick(time.delta()).finished() {
		info!("{}", state.message);
	}
}
