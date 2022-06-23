//! Displays a single [`Sprite`], created from an image, but flipped on one axis.

use bevy::prelude::*;

fn main() {
	let mut app = App::new();
	app.add_plugins(DefaultPlugins);
	app.add_startup_system(setup);
	app.run();
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
	commands.init_bundle::<Camera2dBundle>();
	commands.spawn_bundle(SpriteBundle {
		texture: asset_server.load("branding/icon.png"),
		sprite: Sprite {
			// Flip the logo to the left
			flip_x: true,
			// And don't flip it upside-down ( the default )
			flip_y: false,
			..Default::default()
		},
		..Default::default()
	});
}
