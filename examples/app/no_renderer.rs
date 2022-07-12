//! An application that runs with default plugins and displays an empty
//! window, but without an actual renderer.
//! This can be very useful for integration tests or CI.
//!
//! See also the `headless` example which does not display a window.

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
