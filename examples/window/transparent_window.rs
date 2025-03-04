//! Shows how to display a window in transparent mode.
//!
//! This feature works as expected depending on the platform. Please check the
//! [documentation](https://docs.rs/bevy/latest/bevy/prelude/struct.WindowDescriptor.html#structfield.transparent)
//! for more details.

use bevy::{prelude::*, window::WindowDescriptor};

fn main() {
	let mut app = App::new();
	// ClearColor must have 0 alpha, otherwise some color will bleed through
	app.insert_resource(ClearColor(Color::NONE));
	app.insert_resource(WindowDescriptor {
		// Setting `transparent` allows the `ClearColor`'s alpha value to take effect
		transparent: true,
		// Disabling window decorations to make it feel more like a widget than a window
		decorations: false,
		..Default::default()
	});
	app.add_startup_system(setup);
	app.add_plugins(DefaultPlugins);
	app.run();
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
	commands.init_bundle::<Camera2dBundle>();
	commands.spawn_bundle(SpriteBundle {
		texture: asset_server.load("branding/icon.png"),
		..Default::default()
	});
}
