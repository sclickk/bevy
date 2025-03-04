//! This example shows how to manually render 2d items using "mid level render apis" with a custom
//! pipeline for 2d meshes.
//! It doesn't use the [`Material2d`] abstraction, but changes the vertex buffer to include vertex color.
//! Check out the "mesh2d" example for simpler / higher level 2d meshes.

use bevy::{
	core_pipeline::core_2d::Transparent2d,
	float_ord::FloatOrd,
	prelude::*,
	reflect::TypeUuid,
	render::{
		mesh::{Indices, MeshVertexAttribute},
		render_asset::RenderAssets,
		render_phase::{AddRenderCommand, DrawFunctions, RenderPhase, SetItemPipeline},
		render_resource::{
			BlendState, ColorTargetState, ColorWrites, Face, FragmentState, FrontFace, MultisampleState,
			PipelineCache, PipelineDescriptorMeta, PolygonMode, PrimitiveState, PrimitiveTopology,
			RenderPipelineDescriptor, ShaderMeta, SpecializedRenderPipeline, SpecializedRenderPipelines,
			TextureFormat, VertexBufferLayout, VertexFormat, VertexState, VertexStepMode,
		},
		texture::BevyDefault,
		view::VisibleEntities,
		Extract, RenderApp, RenderStage,
	},
	sprite::{
		DrawMesh2d, Mesh2dHandle, Mesh2dPipeline, Mesh2dPipelineKey, Mesh2dUniform, SetMesh2dBindGroup,
		SetMesh2dViewBindGroup,
	},
};

fn main() {
	let mut app = App::new();
	app.add_plugins(DefaultPlugins);
	app.add_plugin(ColoredMesh2dPlugin);
	app.add_startup_system(star);
	app.run();
}

fn star(
	mut commands: Commands,
	// We will add a new Mesh for the star being created
	mut meshes: ResMut<Assets<Mesh>>,
) {
	// Let's define the mesh for the object we want to draw: a nice star.
	// We will specify here what kind of topology is used to define the mesh,
	// that is, how triangles are built from the vertices. We will use a
	// triangle list, meaning that each vertex of the triangle has to be
	// specified.
	let mut star = Mesh::from(PrimitiveTopology::TriangleList);

	// Vertices need to have a position attribute. We will use the following
	// vertices (I hope you can spot the star in the schema).
	//
	//        1
	//
	//     10   2
	// 9      0      3
	//     8     4
	//        6
	//   7        5
	//
	// These vertices are specificed in 3D space.
	let mut v_pos = vec![[0.0, 0.0, 0.0]];
	for i in 0..10 {
		// Angle of each vertex is 1/10 of TAU, plus PI/2 for positioning vertex 0
		let a = std::f32::consts::FRAC_PI_2 - i as f32 * std::f32::consts::TAU / 10.0;
		// Radius of internal vertices (2, 4, 6, 8, 10) is 100, it's 200 for external
		let r = (1 - i % 2) as f32 * 100.0 + 100.0;
		// Add the vertex coordinates
		v_pos.push([r * a.cos(), r * a.sin(), 0.0]);
	}
	// Set the position attribute
	star.insert_attribute(MeshVertexAttribute::POSITION, v_pos);
	// And a RGB color attribute as well
	let mut v_color: Vec<u32> = vec![Color::BLACK.as_linear_rgba_u32()];
	v_color.extend_from_slice(&[Color::YELLOW.as_linear_rgba_u32(); 10]);
	star.insert_attribute(
		MeshVertexAttribute::new("Vertex_Color", 1, VertexFormat::Uint32),
		v_color,
	);

	// Now, we specify the indices of the vertex that are going to compose the
	// triangles in our star. Vertices in triangles have to be specified in CCW
	// winding (that will be the front face, colored). Since we are using
	// triangle list, we will specify each triangle as 3 vertices
	//   First triangle: 0, 2, 1
	//   Second triangle: 0, 3, 2
	//   Third triangle: 0, 4, 3
	//   etc
	//   Last triangle: 0, 1, 10
	let mut indices = vec![0, 1, 10];
	for i in 2..=10 {
		indices.extend_from_slice(&[0, i, i - 1]);
	}
	star.set_indices(Some(Indices::U32(indices)));

	// We can now spawn the entities for the star and the camera
	commands.spawn_bundle((
		// We use a marker component to identify the custom colored meshes
		ColoredMesh2d::default(),
		// The `Handle<Mesh>` needs to be wrapped in a `Mesh2dHandle` to use 2d rendering instead of 3d
		Mesh2dHandle(meshes.add(star)),
		// These other components are needed for 2d meshes to be rendered
		Transform::default(),
		GlobalTransform::default(),
		Visibility::default(),
		ComputedVisibility::default(),
	));
	commands
		// And use an orthographic projection
		.init_bundle::<Camera2dBundle>();
}

/// A marker component for colored 2d meshes
#[derive(Component, Default)]
pub struct ColoredMesh2d;

/// Custom pipeline for 2d meshes with vertex colors
pub struct ColoredMesh2dPipeline {
	/// this pipeline wraps the standard [`Mesh2dPipeline`]
	mesh2d_pipeline: Mesh2dPipeline,
}

impl FromWorld for ColoredMesh2dPipeline {
	fn from_world(world: &mut World) -> Self {
		Self {
			mesh2d_pipeline: Mesh2dPipeline::from_world(world),
		}
	}
}

// We implement `SpecializedPipeline` to customize the default rendering from `Mesh2dPipeline`
impl SpecializedRenderPipeline for ColoredMesh2dPipeline {
	type Key = Mesh2dPipelineKey;

	fn specialize(&self, key: Self::Key) -> RenderPipelineDescriptor {
		// Customize how to store the meshes' vertex attributes in the vertex buffer
		// Our meshes only have position and color
		let formats = vec![
			// Position
			VertexFormat::Float32x3,
			// Color
			VertexFormat::Uint32,
		];

		let vertex_layout = VertexBufferLayout::from_vertex_formats(VertexStepMode::Vertex, formats);

		RenderPipelineDescriptor {
			meta: PipelineDescriptorMeta {
				label: Some("colored_mesh2d_pipeline".into()),
				// Use the two standard uniforms for 2d meshes
				layout: Some(vec![
					// Bind group 0 is the view uniform
					self.mesh2d_pipeline.view_layout.clone(),
					// Bind group 1 is the mesh uniform
					self.mesh2d_pipeline.mesh_layout.clone(),
				]),
			},
			vertex: VertexState {
				// Use our custom shader
				meta: ShaderMeta {
					shader: COLORED_MESH2D_SHADER_HANDLE.typed::<Shader>(),
					entry_point: "vertex".into(),
					shader_defs: Vec::new(),
				},
				// Use our custom vertex buffer
				buffers: vec![vertex_layout],
			},
			fragment: Some(FragmentState {
				// Use our custom shader
				meta: ShaderMeta {
					shader: COLORED_MESH2D_SHADER_HANDLE.typed::<Shader>(),
					shader_defs: Vec::new(),
					entry_point: "fragment".into(),
				},
				targets: vec![Some(ColorTargetState {
					format: TextureFormat::bevy_default(),
					blend: Some(BlendState::ALPHA_BLENDING),
					write_mask: ColorWrites::ALL,
				})],
			}),
			multisample: MultisampleState {
				count: key.msaa_samples(),
				mask: !0,
				alpha_to_coverage_enabled: false,
			},
			primitive: PrimitiveState {
				front_face: FrontFace::Ccw,
				cull_mode: Some(Face::Back),
				unclipped_depth: false,
				polygon_mode: PolygonMode::Fill,
				conservative: false,
				topology: key.into(),
				strip_index_format: None,
			},
			depth_stencil: None,
		}
	}
}

// This specifies how to render a colored 2d mesh
type DrawColoredMesh2d = (
	// Set the pipeline
	SetItemPipeline,
	// Set the view uniform as bind group 0
	SetMesh2dViewBindGroup<0>,
	// Set the mesh uniform as bind group 1
	SetMesh2dBindGroup<1>,
	// Draw the mesh
	DrawMesh2d,
);

// The custom shader can be inline like here, included from another file at build time
// using `include_str!()`, or loaded like any other asset with `asset_server.load()`.
const COLORED_MESH2D_SHADER: &str = r"
// Import the standard 2d mesh uniforms and set their bind groups
#import bevy_sprite::mesh2d_types
#import bevy_sprite::mesh2d_view_bindings

@group(1) @binding(0)
var<uniform> mesh: Mesh2d;

// NOTE: Bindings must come before functions that use them!
#import bevy_sprite::mesh2d_functions

// The structure of the vertex buffer is as specified in `specialize()`
struct Vertex {
    @location(0) position: vec3<f32>,
    @location(1) color: u32,
};

struct VertexOutput {
    // The vertex shader must set the on-screen position of the vertex
    @builtin(position) clip_position: vec4<f32>,
    // We pass the vertex color to the fragment shader in location 0
    @location(0) color: vec4<f32>,
};

/// Entry point for the vertex shader
@vertex
fn vertex(vertex: Vertex) -> VertexOutput {
    var out: VertexOutput;
    // Project the world position of the mesh into screen position
    out.clip_position = mesh2d_position_local_to_clip(mesh.model, vec4<f32>(vertex.position, 1.0));
    // Unpack the `u32` from the vertex buffer into the `vec4<f32>` used by the fragment shader
    out.color = vec4<f32>((vec4<u32>(vertex.color) >> vec4<u32>(0u, 8u, 16u, 24u)) & vec4<u32>(255u)) / 255.0;
    return out;
}

// The input of the fragment shader must correspond to the output of the vertex shader for all `location`s
struct FragmentInput {
    // The color is interpolated between vertices by default
    @location(0) color: vec4<f32>,
};

/// Entry point for the fragment shader
@fragment
fn fragment(in: FragmentInput) -> @location(0) vec4<f32> {
    return in.color;
}
";

/// Plugin that renders [`ColoredMesh2d`]s
pub struct ColoredMesh2dPlugin;

/// Handle to the custom shader with a unique random ID
pub const COLORED_MESH2D_SHADER_HANDLE: HandleUntyped =
	HandleUntyped::weak_from_u64(Shader::TYPE_UUID, 13828845428412094821);

impl Plugin for ColoredMesh2dPlugin {
	fn build(&self, app: &mut App) {
		// Load our custom shader
		let mut shaders = app.world.resource_mut::<Assets<Shader>>();
		shaders.set_untracked(
			COLORED_MESH2D_SHADER_HANDLE,
			Shader::from_wgsl(COLORED_MESH2D_SHADER),
		);

		// Register our custom draw function and pipeline, and add our render systems
		if let Ok(render_app) = app.get_sub_app_mut(RenderApp) {
			render_app.add_render_command::<Transparent2d, DrawColoredMesh2d>();
			render_app.init_resource::<ColoredMesh2dPipeline>();
			render_app.init_resource::<SpecializedRenderPipelines<ColoredMesh2dPipeline>>();
			render_app.add_system_to_stage(RenderStage::Extract, extract_colored_mesh2d);
			render_app.add_system_to_stage(RenderStage::Queue, queue_colored_mesh2d);
		}
	}
}

/// Extract the [`ColoredMesh2d`] marker component into the render app
pub fn extract_colored_mesh2d(
	mut commands: Commands,
	mut previous_len: Local<usize>,
	// When extracting, you must use `Extract` to mark the `SystemParam`s
	// which should be taken from the main world.
	query: Extract<Query<(Entity, &ComputedVisibility), With<ColoredMesh2d>>>,
) {
	let mut values = Vec::with_capacity(*previous_len);
	query.for_each(|(entity, computed_visibility)| {
		if computed_visibility.is_visible() {
			values.push((entity, (ColoredMesh2d,)));
		}
	});
	*previous_len = values.len();
	commands.insert_or_spawn_batch(values);
}

/// Queue the 2d meshes marked with [`ColoredMesh2d`] using our custom pipeline and draw function
#[allow(clippy::too_many_arguments)]
pub fn queue_colored_mesh2d(
	transparent_draw_functions: Res<DrawFunctions<Transparent2d>>,
	colored_mesh2d_pipeline: Res<ColoredMesh2dPipeline>,
	mut pipelines: ResMut<SpecializedRenderPipelines<ColoredMesh2dPipeline>>,
	mut pipeline_cache: ResMut<PipelineCache>,
	msaa: Res<Msaa>,
	render_meshes: Res<RenderAssets<Mesh>>,
	colored_mesh2d: Query<(&Mesh2dHandle, &Mesh2dUniform), With<ColoredMesh2d>>,
	mut views: Query<(&VisibleEntities, &mut RenderPhase<Transparent2d>)>,
) {
	if colored_mesh2d.is_empty() {
		return;
	}
	// Iterate each view (a camera is a view)
	for (visible_entities, mut transparent_phase) in &mut views {
		let draw_colored_mesh2d = transparent_draw_functions
			.read()
			.get_id::<DrawColoredMesh2d>()
			.unwrap();

		let mesh_key = Mesh2dPipelineKey::from_msaa_samples(msaa.samples);

		// Queue all entities visible to that view
		for visible_entity in &visible_entities.entities {
			if let Ok((mesh2d_handle, mesh2d_uniform)) = colored_mesh2d.get(*visible_entity) {
				// Get our specialized pipeline
				let mut mesh2d_key = mesh_key;
				if let Some(mesh) = render_meshes.get(&mesh2d_handle.0) {
					mesh2d_key |= Mesh2dPipelineKey::from(mesh.primitive_topology);
				}

				let pipeline_id =
					pipelines.specialize(&mut pipeline_cache, &colored_mesh2d_pipeline, mesh2d_key);

				let mesh_z = mesh2d_uniform.transform.w_axis.z;
				transparent_phase.add(Transparent2d {
					entity: *visible_entity,
					draw_function: draw_colored_mesh2d,
					pipeline: pipeline_id,
					// The 2d render items are sorted according to their z value before rendering,
					// in order to get correct transparency
					sort_key: FloatOrd(mesh_z),
					// This material is not batched
					batch_range: None,
				});
			}
		}
	}
}
