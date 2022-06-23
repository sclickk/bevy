//! Shows how to render simple primitive shapes with a single color.

use bevy::{prelude::*, sprite::MaterialMesh2dBundle};

fn main() {
	let mut app = App::new();
	app.add_plugins(DefaultPlugins);
	app.add_startup_system(setup);
	app.run();
}

fn setup(
	mut commands: Commands,
	mut meshes: ResMut<Assets<Mesh>>,
	mut materials: ResMut<Assets<ColorMaterial>>,
) {
	commands.init_bundle::<Camera2dBundle>();

	// Rectangle
	commands.spawn_bundle(SpriteBundle {
		sprite: Sprite {
			color: Color::rgb(0.25, 0.25, 0.75),
			custom_size: Some(Vec2::new(50.0, 100.0)),
			..Default::default()
		},
		..Default::default()
	});

	// Circle
	commands.spawn_bundle(MaterialMesh2dBundle {
		mesh: meshes
			.add(shape::Circle::new(50.).into())
			.into(),
		material: materials.add(ColorMaterial::from(Color::PURPLE)),
		transform: Transform::from_translation(Vec3::new(-100., 0., 0.)),
		..Default::default()
	});

	// Hexagon
	commands.spawn_bundle(MaterialMesh2dBundle {
		mesh: meshes
			.add(shape::RegularPolygon::new(50., 6).into())
			.into(),
		material: materials.add(ColorMaterial::from(Color::TURQUOISE)),
		transform: Transform::from_translation(Vec3::new(100., 0., 0.)),
		..Default::default()
	});
}
