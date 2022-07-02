//! Renders a 2D scene containing a single, moving sprite.

use bevy::prelude::*;

fn main() {
	let mut app = App::new();
	app.add_plugins(DefaultPlugins);
	app.add_startup_system(setup);
	app.add_system(sprite_movement);
	app.run();
}

#[derive(Component)]
enum Direction {
	Up,
	Down,
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
	commands.init_bundle::<Camera2dBundle>();
	commands
		.spawn_bundle(SpriteBundle {
			texture: asset_server.load("branding/icon.png"),
			transform: Transform::from_xyz(100., 0., 0.),
			..Default::default()
		})
		.insert(Direction::Up);
}

/// The sprite is animated by changing its translation depending on the time that has passed since
/// the last frame.
fn sprite_movement(time: Res<Time>, mut sprite_position: Query<(&mut Direction, &mut Transform)>) {
	sprite_position.for_each_mut(|(mut logo, mut transform)| {
		match *logo {
			Direction::Up => transform.translation.y += 150. * time.delta_seconds(),
			Direction::Down => transform.translation.y -= 150. * time.delta_seconds(),
		}

		if transform.translation.y > 200. {
			*logo = Direction::Down;
		} else if transform.translation.y < -200. {
			*logo = Direction::Up;
		}
	});
}
