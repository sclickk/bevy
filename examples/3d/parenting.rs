//! Illustrates how to create parent-child relationships between entities and how parent transforms
//! are propagated to their descendants.

use bevy::prelude::*;

fn main() {
	let mut app = App::new();
	app.add_plugins(DefaultPlugins);
	app.add_startup_system(setup);
	app.add_system(rotator_system);
	app.run();
}

/// this component indicates what entities should rotate
#[derive(Component)]
struct Rotator;

/// rotates the parent, which will result in the child also rotating
fn rotator_system(time: Res<Time>, mut query: Query<&mut Transform, With<Rotator>>) {
	for mut transform in &mut query {
		transform.rotate_x(3.0 * time.delta_seconds());
	}
}

/// set up a simple scene with a "parent" cube and a "child" cube
fn setup(
	mut commands: Commands,
	mut meshes: ResMut<Assets<Mesh>>,
	mut materials: ResMut<Assets<StandardMaterial>>,
) {
	let cube_handle = meshes.add(Mesh::from(shape::Cube { size: 2.0 }));
	let cube_material_handle = materials.add(StandardMaterial {
		base_color: Color::rgb(0.8, 0.7, 0.6),
		..Default::default()
	});

	// parent cube
	commands
		.spawn_bundle(PbrBundle {
			mesh: cube_handle.clone(),
			material: cube_material_handle.clone(),
			transform: Transform::from_xyz(0.0, 0.0, 1.0),
			..Default::default()
		})
		.insert(Rotator)
		.with_children(|parent| {
			// child cube
			parent.spawn_bundle(PbrBundle {
				mesh: cube_handle,
				material: cube_material_handle,
				transform: Transform::from_xyz(0.0, 0.0, 3.0),
				..Default::default()
			});
		});
	// light
	commands.spawn_bundle(PointLightBundle {
		transform: Transform::from_xyz(4.0, 5.0, -4.0),
		..Default::default()
	});
	// camera
	commands.spawn_bundle(Camera3dBundle {
		transform: Transform::from_xyz(5.0, 10.0, 10.0).looking_at(Vec3::ZERO, Vec3::Y),
		..Default::default()
	});
}
