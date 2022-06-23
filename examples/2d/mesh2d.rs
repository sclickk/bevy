//! Shows how to render a polygonal [`Mesh`], generated from a [`Quad`] primitive, in a 2D scene.

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
	commands.spawn_bundle(MaterialMesh2dBundle {
		mesh: meshes
			.add(Mesh::from(shape::Quad::default()))
			.into(),
		transform: Transform::from_scale(Vec3::splat(128.)),
		material: materials.add(ColorMaterial::from(Color::PURPLE)),
		..Default::default()
	});
}
