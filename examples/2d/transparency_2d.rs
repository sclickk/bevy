//! Demonstrates how to use transparency in 2D.
//! Shows 3 bevy logos on top of each other, each with a different amount of transparency.

use bevy::prelude::*;

fn main() {
	let mut app = App::new();
	app.add_plugins(DefaultPlugins);
	app.add_startup_system(setup);
	app.run();
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
	commands.init_bundle::<Camera2dBundle>();

	let sprite_handle = asset_server.load("branding/icon.png");

	commands.spawn_bundle(SpriteBundle {
		texture: sprite_handle.clone(),
		..Default::default()
	});
	commands.spawn_bundle(SpriteBundle {
		sprite: Sprite {
			// Alpha channel of the color controls transparency.
			color: Color::rgba(0.0, 0.0, 1.0, 0.7),
			..Default::default()
		},
		texture: sprite_handle.clone(),
		transform: Transform::from_xyz(100.0, 0.0, 0.0),
		..Default::default()
	});
	commands.spawn_bundle(SpriteBundle {
		sprite: Sprite {
			color: Color::rgba(0.0, 1.0, 0.0, 0.3),
			..Default::default()
		},
		texture: sprite_handle,
		transform: Transform::from_xyz(200.0, 0.0, 0.0),
		..Default::default()
	});
}
