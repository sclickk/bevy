//! Displays a single [`Sprite`], created from an image.

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
		..Default::default()
	});
}
