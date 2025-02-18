//! Shows how to create a 3D orthographic view (for isometric-look games or CAD applications).

use bevy::{prelude::*, render::camera::ScalingMode};

fn main() {
	let mut app = App::new();
	app.add_plugins(DefaultPlugins);
	app.add_startup_system(setup);
	app.run();
}

/// set up a simple 3D scene
fn setup(
	mut commands: Commands,
	mut meshes: ResMut<Assets<Mesh>>,
	mut materials: ResMut<Assets<StandardMaterial>>,
) {
	// camera
	commands.spawn_bundle(Camera3dBundle {
		projection: OrthographicProjection {
			scale: 3.0,
			scaling_mode: ScalingMode::FixedVertical(2.0),
			..Default::default()
		}
		.into(),
		transform: Transform::from_xyz(5.0, 5.0, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
		..Default::default()
	});

	// plane
	commands.spawn_bundle(PbrBundle {
		mesh: meshes.add(Mesh::from(shape::Plane { size: 5.0 })),
		material: materials.add(Color::rgb(0.3, 0.5, 0.3).into()),
		..Default::default()
	});
	// cubes
	commands.spawn_bundle(PbrBundle {
		mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
		material: materials.add(Color::rgb(0.8, 0.7, 0.6).into()),
		transform: Transform::from_xyz(1.5, 0.5, 1.5),
		..Default::default()
	});
	commands.spawn_bundle(PbrBundle {
		mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
		material: materials.add(Color::rgb(0.8, 0.7, 0.6).into()),
		transform: Transform::from_xyz(1.5, 0.5, -1.5),
		..Default::default()
	});
	commands.spawn_bundle(PbrBundle {
		mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
		material: materials.add(Color::rgb(0.8, 0.7, 0.6).into()),
		transform: Transform::from_xyz(-1.5, 0.5, 1.5),
		..Default::default()
	});
	commands.spawn_bundle(PbrBundle {
		mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
		material: materials.add(Color::rgb(0.8, 0.7, 0.6).into()),
		transform: Transform::from_xyz(-1.5, 0.5, -1.5),
		..Default::default()
	});
	// light
	commands.spawn_bundle(PointLightBundle {
		transform: Transform::from_xyz(3.0, 8.0, 5.0),
		..Default::default()
	});
}
