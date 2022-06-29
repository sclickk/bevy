//! An application that runs with default plugins, but without an actual renderer.
//! This can be very useful for integration tests or CI.

use bevy::{prelude::*, render::settings::WgpuSettings};

fn main() {
	let mut app = App::new();
	app.insert_resource(WgpuSettings {
			backends: None,
			..Default::default()
		});
	app.add_plugins(DefaultPlugins);
	app.run();
}
