//! Create an application without winit (runs single time, no event loop).

use bevy::prelude::*;
use bevy::winit::WinitPlugin;

fn main() {
	let mut app = App::new();
	app.add_plugins_with(DefaultPlugins, |group| {
		group.disable::<WinitPlugin>();
		group
	});
	app.add_system(setup_system);
	app.run();
}

fn setup_system(mut commands: Commands) {
	commands.init_bundle::<Camera3dBundle>();
}
