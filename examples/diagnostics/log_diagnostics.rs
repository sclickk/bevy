//! Shows different built-in plugins that logs diagnostics, like frames per second (FPS), to the console.

use bevy::{
	diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin},
	prelude::*,
};

fn main() {
	let mut app = App::new();
	app.add_plugins(DefaultPlugins);
	// Adds frame time diagnostics
	app.init_plugin::<FrameTimeDiagnosticsPlugin>();
	// Adds a system that prints diagnostics to the console
	app.init_plugin::<LogDiagnosticsPlugin>();
	// Any plugin can register diagnostics
	// Uncomment this to add an entity count diagnostics:
	// app.init_plugin::<bevy::diagnostic::EntityCountDiagnosticsPlugin>;
	// Uncomment this to add an asset count diagnostics:
	// app.init_plugin::<bevy::asset::diagnostic::AssetCountDiagnosticsPlugin::<Texture>>;
	app.run();
}
