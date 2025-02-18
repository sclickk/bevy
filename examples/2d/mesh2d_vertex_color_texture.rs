//! Shows how to render a polygonal [`Mesh`], generated from a [`Quad`] primitive, in a 2D scene.
//! Adds a texture and colored vertices, giving per-vertex tinting.

use bevy::{prelude::*, render::mesh::MeshVertexAttribute, sprite::MaterialMesh2dBundle};

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
	asset_server: Res<AssetServer>,
) {
	// Load the Bevy logo as a texture
	let texture_handle = asset_server.load("branding/banner.png");
	// Build a default quad mesh
	let mut mesh = Mesh::from(shape::Quad::default());
	// Build vertex colors for the quad. One entry per vertex (the corners of the quad)
	let vertex_colors: Vec<[f32; 4]> = vec![
		Color::RED.as_rgba_f32(),
		Color::GREEN.as_rgba_f32(),
		Color::BLUE.as_rgba_f32(),
		Color::WHITE.as_rgba_f32(),
	];
	// Insert the vertex colors as an attribute
	mesh.insert_attribute(MeshVertexAttribute::COLOR, vertex_colors);
	// Spawn
	commands.init_bundle::<Camera2dBundle>();
	commands.spawn_bundle(MaterialMesh2dBundle {
		mesh: meshes.add(mesh).into(),
		transform: Transform::from_scale(Vec3::splat(128.)),
		material: materials.add(ColorMaterial::from(texture_handle)),
		..Default::default()
	});
}
