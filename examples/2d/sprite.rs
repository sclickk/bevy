//! Displays a single [`Sprite`], created from an image.

use bevy::prelude::*;

fn main() {
	App::new()
		.add_plugins(DefaultPlugins)
		.add_startup_system(setup)
		.run();
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
	commands.init_bundle::<Camera2dBundle>();
	commands.spawn_bundle(SpriteBundle {
		texture: asset_server.load("branding/icon.png"),
		..Default::default()
	});
}
