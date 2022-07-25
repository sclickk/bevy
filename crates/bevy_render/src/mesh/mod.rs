/// Generation for some primitive shape meshes.
pub mod shape;

mod conversions;

mod indices;
pub use indices::*;

mod plugin;
pub use plugin::*;

pub mod skinning;

pub use wgpu::PrimitiveTopology;

use crate::{
	primitives::Aabb,
	render_asset::{PrepareAssetError, RenderAsset},
	render_resource::{Buffer, VertexBufferLayout},
	renderer::RenderDevice,
};
use bevy_core::cast_slice;
use bevy_derive::EnumVariantMeta;
use bevy_ecs::system::{lifetimeless::SRes, SystemParamItem};
use bevy_math::*;
use bevy_reflect::TypeUuid;
use bevy_utils::{tracing::error, Hashed};
use std::{collections::BTreeMap, hash::Hash};
use thiserror::Error;
use wgpu::{
	util::BufferInitDescriptor, BufferUsages, IndexFormat, VertexAttribute, VertexFormat,
	VertexStepMode,
};

pub const INDEX_BUFFER_ASSET_INDEX: u64 = 0;
pub const VERTEX_ATTRIBUTE_BUFFER_ID: u64 = 10;

// TODO: allow values to be unloaded after been submitting to the GPU to conserve memory
#[derive(Debug, TypeUuid, Clone)]
#[uuid = "8ecbac0f-f545-4473-ad43-e1f4243af51e"]
pub struct Mesh {
	primitive_topology: PrimitiveTopology,
	/// `std::collections::BTreeMap` with all defined vertex attributes (Positions, Normals, ...)
	/// for this mesh. Attribute ids to attribute values.
	/// Uses a BTreeMap because, unlike HashMap, it has a defined iteration order,
	/// which allows easy stable VertexBuffers (i.e. same buffer order)
	attributes: BTreeMap<usize, MeshAttributeData>,
	indices: Option<Indices>,
}

/// Contains geometry in the form of a mesh.
///
/// Often meshes are automatically generated by bevy's asset loaders or primitives, such as
/// [`shape::Cube`](crate::mesh::shape::Cube) or [`shape::Box`](crate::mesh::shape::Box), but you can also construct
/// one yourself.
///
/// Example of constructing a mesh:
/// ```
/// # use bevy_render::mesh::{Mesh, Indices};
/// # use bevy_render::render_resource::PrimitiveTopology;
/// fn create_triangle() -> Mesh {
///     let mut mesh = Mesh::from(PrimitiveTopology::TriangleList);
///     mesh.insert_attribute(MeshVertexAttribute::POSITION, vec![[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [1.0, 1.0, 0.0]]);
///     mesh.set_indices(Some(Indices::U32(vec![0,1,2])));
///     mesh
/// }
/// ```
impl Mesh {
	/// Returns the topology of the mesh.
	pub fn primitive_topology(&self) -> PrimitiveTopology {
		self.primitive_topology
	}

	#[inline]
	pub fn contains_attribute(&self, id: usize) -> bool {
		self.attributes.contains_key(&id)
	}
	/// Sets the data for a vertex attribute (position, normal etc.). The name will
	/// often be one of the associated constants such as [`MeshVertexAttribute::POSITION`].
	///
	/// # Panics
	/// Panics when the format of the values does not match the attribute's format.
	#[inline]
	pub fn insert_attribute(
		&mut self,
		attribute: MeshVertexAttribute,
		values: impl Into<VertexAttributeValues>,
	) {
		let values: VertexAttributeValues = values.into();

		let values_format = VertexFormat::from(&values);
		if values_format != attribute.format {
			error!(
				"Invalid attribute format for {}. Given format is {:?} but expected {:?}",
				attribute.name, values_format, attribute.format
			);
			panic!("Failed to insert attribute");
		}

		self
			.attributes
			.insert(attribute.id, MeshAttributeData { attribute, values });
	}

	/// Retrieves the data currently set to the vertex attribute with the specified `name`.
	#[inline]
	pub fn attribute(&self, id: usize) -> Option<&VertexAttributeValues> {
		self
			.attributes
			.get(&id)
			.map(|data| &data.values)
	}
	/// Removes the data for a vertex attribute
	pub fn remove_attribute(&mut self, attribute: impl Into<usize>) -> Option<VertexAttributeValues> {
		self
			.attributes
			.remove(&attribute.into())
			.map(|data| data.values)
	}

	/// Retrieves the data currently set to the vertex attribute with the specified `name` mutably.
	#[inline]
	pub fn attribute_mut(&mut self, id: impl Into<usize>) -> Option<&mut VertexAttributeValues> {
		self
			.attributes
			.get_mut(&id.into())
			.map(|data| &mut data.values)
	}

	/// Returns an iterator that yields references to the data of each vertex attribute.
	pub fn attributes(&self) -> impl Iterator<Item = (usize, &VertexAttributeValues)> {
		self
			.attributes
			.iter()
			.map(|(id, data)| (*id, &data.values))
	}

	/// Returns an iterator that yields mutable references to the data of each vertex attribute.
	pub fn attributes_mut(&mut self) -> impl Iterator<Item = (usize, &mut VertexAttributeValues)> {
		self
			.attributes
			.iter_mut()
			.map(|(id, data)| (*id, &mut data.values))
	}

	/// Sets the vertex indices of the mesh. They describe how triangles are constructed out of the
	/// vertex attributes and are therefore only useful for the [`PrimitiveTopology`] variants
	/// that use triangles.
	#[inline]
	pub fn set_indices(&mut self, indices: Option<Indices>) {
		self.indices = indices;
	}

	/// Retrieves the vertex `indices` of the mesh.
	#[inline]
	pub fn indices(&self) -> Option<&Indices> {
		self.indices.as_ref()
	}

	/// Retrieves the vertex `indices` of the mesh mutably.
	#[inline]
	pub fn indices_mut(&mut self) -> Option<&mut Indices> {
		self.indices.as_mut()
	}

	/// Computes and returns the index data of the mesh as bytes.
	/// This is used to transform the index data into a GPU friendly format.
	pub fn get_index_buffer_bytes(&self) -> Option<&[u8]> {
		self
			.indices
			.as_ref()
			.map(|indices| match &indices {
				Indices::U16(indices) => cast_slice(&indices[..]),
				Indices::U32(indices) => cast_slice(&indices[..]),
			})
	}

	/// For a given `descriptor` returns a [`VertexBufferLayout`] compatible with this mesh. If this
	/// mesh is not compatible with the given `descriptor` (ex: it is missing vertex attributes), [`None`] will
	/// be returned.
	pub fn get_mesh_vertex_buffer_layout(&self) -> MeshVertexBufferLayout {
		let mut attributes = Vec::with_capacity(self.attributes.len());
		let mut attribute_ids = Vec::with_capacity(self.attributes.len());
		let mut accumulated_offset = 0;
		for (index, data) in self.attributes.values().enumerate() {
			attribute_ids.push(data.attribute.id);
			attributes.push(VertexAttribute {
				offset: accumulated_offset,
				format: data.attribute.format,
				shader_location: index as u32,
			});
			accumulated_offset += data.attribute.format.get_size();
		}

		MeshVertexBufferLayout::new(InnerMeshVertexBufferLayout {
			layout: VertexBufferLayout {
				array_stride: accumulated_offset,
				step_mode: VertexStepMode::Vertex,
				attributes,
			},
			attribute_ids,
		})
	}

	/// Counts all vertices of the mesh.
	///
	/// # Panics
	/// Panics if the attributes have different vertex counts.
	pub fn count_vertices(&self) -> usize {
		let mut vertex_count: Option<usize> = None;
		for (attribute_id, attribute_data) in &self.attributes {
			let attribute_len = attribute_data.values.len();
			if let Some(previous_vertex_count) = vertex_count {
				assert_eq!(
					previous_vertex_count, attribute_len,
					"{:?} has a different vertex count ({}) than other attributes ({}) in this mesh.",
					attribute_id, attribute_len, previous_vertex_count
				);
			}
			vertex_count = Some(attribute_len);
		}

		vertex_count.unwrap_or(0)
	}

	/// Computes and returns the vertex data of the mesh as bytes.
	/// Therefore the attributes are located in alphabetical order.
	/// This is used to transform the vertex data into a GPU friendly format.
	///
	/// # Panics
	/// Panics if the attributes have different vertex counts.
	pub fn get_vertex_buffer_data(&self) -> Vec<u8> {
		let mut vertex_size = 0;
		for attribute_data in self.attributes.values() {
			let vertex_format = attribute_data.attribute.format;
			vertex_size += vertex_format.get_size() as usize;
		}

		let vertex_count = self.count_vertices();
		let mut attributes_interleaved_buffer = vec![0; vertex_count * vertex_size];
		// bundle into interleaved buffers
		let mut attribute_offset = 0;
		for attribute_data in self.attributes.values() {
			let attribute_size = attribute_data.attribute.format.get_size() as usize;
			let attributes_bytes = attribute_data.values.get_bytes();
			for (vertex_index, attribute_bytes) in attributes_bytes
				.chunks_exact(attribute_size)
				.enumerate()
			{
				let offset = vertex_index * vertex_size + attribute_offset;
				attributes_interleaved_buffer[offset..offset + attribute_size]
					.copy_from_slice(attribute_bytes);
			}

			attribute_offset += attribute_size;
		}

		attributes_interleaved_buffer
	}

	/// Duplicates the vertex attributes so that no vertices are shared.
	///
	/// This can dramatically increase the vertex count, so make sure this is what you want.
	/// Does nothing if no [Indices] are set.
	#[allow(clippy::match_same_arms)]
	pub fn duplicate_vertices(&mut self) {
		fn duplicate<T: Copy>(values: &[T], indices: impl Iterator<Item = usize>) -> Vec<T> {
			indices.map(|i| values[i]).collect()
		}

		let indices = match self.indices.take() {
			Some(indices) => indices,
			None => return,
		};

		for attributes in self.attributes.values_mut() {
			let indices = indices.iter();
			match &mut attributes.values {
				VertexAttributeValues::Float32(vec) => *vec = duplicate(vec, indices),
				VertexAttributeValues::Sint32(vec) => *vec = duplicate(vec, indices),
				VertexAttributeValues::Uint32(vec) => *vec = duplicate(vec, indices),
				VertexAttributeValues::Float32x2(vec) => *vec = duplicate(vec, indices),
				VertexAttributeValues::Sint32x2(vec) => *vec = duplicate(vec, indices),
				VertexAttributeValues::Uint32x2(vec) => *vec = duplicate(vec, indices),
				VertexAttributeValues::Float32x3(vec) => *vec = duplicate(vec, indices),
				VertexAttributeValues::Sint32x3(vec) => *vec = duplicate(vec, indices),
				VertexAttributeValues::Uint32x3(vec) => *vec = duplicate(vec, indices),
				VertexAttributeValues::Sint32x4(vec) => *vec = duplicate(vec, indices),
				VertexAttributeValues::Uint32x4(vec) => *vec = duplicate(vec, indices),
				VertexAttributeValues::Float32x4(vec) => *vec = duplicate(vec, indices),
				VertexAttributeValues::Sint16x2(vec) => *vec = duplicate(vec, indices),
				VertexAttributeValues::Snorm16x2(vec) => *vec = duplicate(vec, indices),
				VertexAttributeValues::Uint16x2(vec) => *vec = duplicate(vec, indices),
				VertexAttributeValues::Unorm16x2(vec) => *vec = duplicate(vec, indices),
				VertexAttributeValues::Sint16x4(vec) => *vec = duplicate(vec, indices),
				VertexAttributeValues::Snorm16x4(vec) => *vec = duplicate(vec, indices),
				VertexAttributeValues::Uint16x4(vec) => *vec = duplicate(vec, indices),
				VertexAttributeValues::Unorm16x4(vec) => *vec = duplicate(vec, indices),
				VertexAttributeValues::Sint8x2(vec) => *vec = duplicate(vec, indices),
				VertexAttributeValues::Snorm8x2(vec) => *vec = duplicate(vec, indices),
				VertexAttributeValues::Uint8x2(vec) => *vec = duplicate(vec, indices),
				VertexAttributeValues::Unorm8x2(vec) => *vec = duplicate(vec, indices),
				VertexAttributeValues::Sint8x4(vec) => *vec = duplicate(vec, indices),
				VertexAttributeValues::Snorm8x4(vec) => *vec = duplicate(vec, indices),
				VertexAttributeValues::Uint8x4(vec) => *vec = duplicate(vec, indices),
				VertexAttributeValues::Unorm8x4(vec) => *vec = duplicate(vec, indices),
			}
		}
	}

	/// Calculates the [`MeshVertexAttribute::NORMAL`] of a mesh.
	///
	/// # Panics
	/// Panics if [`Indices`] are set or [`MeshVertexAttribute::POSITION`] is not of type `float3` or
	/// if the mesh has any other topology than [`PrimitiveTopology::TriangleList`].
	/// Consider calling [`Mesh::duplicate_vertices`] or export your mesh with normal attributes.
	pub fn compute_flat_normals(&mut self) {
		assert!(
			self.indices().is_none(),
			"`compute_flat_normals` can't work on indexed geometry. Consider calling `Mesh::duplicate_vertices`."
		);

		assert!(
			matches!(self.primitive_topology, PrimitiveTopology::TriangleList),
			"`compute_flat_normals` can only work on `TriangleList`s"
		);

		let positions = self
			.attribute(MeshVertexAttribute::POSITION.id)
			.unwrap()
			.as_float3()
			.expect("`MeshVertexAttribute::POSITION` vertex attributes should be of type `float3`");

		let normals: Vec<_> = positions
			.chunks_exact(3)
			.map(|p| face_normal(p[0], p[1], p[2]))
			.flat_map(|normal| [normal; 3])
			.collect();

		self.insert_attribute(MeshVertexAttribute::NORMAL, normals);
	}

	/// Generate tangents for the mesh using the `mikktspace` algorithm.
	///
	/// Sets the [`MeshVertexAttribute::TANGENT`] attribute if successful.
	/// Requires a [`PrimitiveTopology::TriangleList`] topology and the [`MeshVertexAttribute::POSITION`], [`MeshVertexAttribute::NORMAL`] and [`MeshVertexAttribute::UV_0`] attributes set.
	pub fn generate_tangents(&mut self) -> Result<(), GenerateTangentsError> {
		let tangents = generate_tangents_for_mesh(self)?;
		self.insert_attribute(MeshVertexAttribute::TANGENT, tangents);
		Ok(())
	}

	/// Compute the Axis-Aligned Bounding Box of the mesh vertices in model space
	pub fn compute_aabb(&self) -> Option<Aabb> {
		if let Some(VertexAttributeValues::Float32x3(values)) =
			self.attribute(MeshVertexAttribute::POSITION.id)
		{
			let mut minimum = VEC3_MAX;
			let mut maximum = VEC3_MIN;
			for p in values {
				minimum = minimum.min(Vec3::from_slice(p));
				maximum = maximum.max(Vec3::from_slice(p));
			}
			if minimum.x != std::f32::MAX
				&& minimum.y != std::f32::MAX
				&& minimum.z != std::f32::MAX
				&& maximum.x != std::f32::MIN
				&& maximum.y != std::f32::MIN
				&& maximum.z != std::f32::MIN
			{
				return Some(Aabb::from_min_max(minimum, maximum));
			}
		}

		None
	}
}

impl Into<PrimitiveTopology> for Mesh {
	fn into(self) -> PrimitiveTopology {
		self.primitive_topology
	}
}

impl From<PrimitiveTopology> for Mesh {
	/// Construct a new mesh. You need to provide a [`PrimitiveTopology`] so that the
	/// renderer knows how to treat the vertex data. Most of the time this will be
	/// [`PrimitiveTopology::TriangleList`].
	fn from(primitive_topology: PrimitiveTopology) -> Self {
		Mesh {
			primitive_topology,
			attributes: Default::default(),
			indices: None,
		}
	}
}

#[derive(Debug, Clone)]
pub struct MeshVertexAttribute {
	/// The friendly name of the vertex attribute
	pub name: &'static str,

	/// The _unique_ id of the vertex attribute. This will also determine sort ordering
	/// when generating vertex buffers. Built-in / standard attributes will use "close to zero"
	/// indices. When in doubt, use a random / very large usize to avoid conflicts.
	pub id: usize,

	/// The format of the vertex attribute.
	pub format: VertexFormat,
}

impl MeshVertexAttribute {
	/// Where the vertex is located in space. Use in conjunction with [`Mesh::insert_attribute`]
	pub const POSITION: Self = Self::new("Vertex_Position", 0, VertexFormat::Float32x3);

	/// The direction the vertex normal is facing in.
	/// Use in conjunction with [`Mesh::insert_attribute`]
	pub const NORMAL: Self = Self::new("Vertex_Normal", 1, VertexFormat::Float32x3);

	/// Texture coordinates for the vertex. Use in conjunction with [`Mesh::insert_attribute`]
	pub const UV_0: Self = Self::new("Vertex_Uv", 2, VertexFormat::Float32x2);

	/// The direction of the vertex tangent. Used for normal mapping
	pub const TANGENT: Self = Self::new("Vertex_Tangent", 3, VertexFormat::Float32x4);

	/// Per vertex coloring. Use in conjunction with [`Mesh::insert_attribute`]
	pub const COLOR: Self = Self::new("Vertex_Color", 4, VertexFormat::Float32x4);

	/// Per vertex joint transform matrix weight. Use in conjunction with [`Mesh::insert_attribute`]
	pub const JOINT_WEIGHT: Self = Self::new("Vertex_JointWeight", 5, VertexFormat::Float32x4);
	/// Per vertex joint transform matrix index. Use in conjunction with [`Mesh::insert_attribute`]
	pub const JOINT_INDEX: Self = Self::new("Vertex_JointIndex", 6, VertexFormat::Uint16x4);

	pub const fn new(name: &'static str, id: usize, format: VertexFormat) -> Self {
		Self { name, id, format }
	}

	pub const fn at_shader_location(&self, shader_location: u32) -> VertexAttributeDescriptor {
		VertexAttributeDescriptor {
			shader_location,
			id: self.id,
			name: self.name,
		}
	}
}

pub type MeshVertexBufferLayout = Hashed<InnerMeshVertexBufferLayout>;

#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub struct InnerMeshVertexBufferLayout {
	attribute_ids: Vec<usize>,
	layout: VertexBufferLayout,
}

impl InnerMeshVertexBufferLayout {
	#[inline]
	pub fn contains(&self, attribute_id: usize) -> bool {
		self.attribute_ids.contains(&attribute_id)
	}

	#[inline]
	pub fn attribute_ids(&self) -> &[usize] {
		&self.attribute_ids
	}

	#[inline]
	pub fn layout(&self) -> &VertexBufferLayout {
		&self.layout
	}

	pub fn get_layout(
		&self,
		attribute_descriptors: &[VertexAttributeDescriptor],
	) -> Result<VertexBufferLayout, MissingVertexAttributeError> {
		let mut attributes = Vec::with_capacity(attribute_descriptors.len());
		for attribute_descriptor in attribute_descriptors.into_iter() {
			if let Some(index) = self
				.attribute_ids
				.iter()
				.position(|id| *id == attribute_descriptor.id)
			{
				let layout_attribute = &self.layout.attributes[index];
				attributes.push(VertexAttribute {
					format: layout_attribute.format,
					offset: layout_attribute.offset,
					shader_location: attribute_descriptor.shader_location,
				});
			} else {
				return Err(MissingVertexAttributeError {
					id: attribute_descriptor.id,
					name: attribute_descriptor.name,
					pipeline_type: None,
				});
			}
		}

		Ok(VertexBufferLayout {
			array_stride: self.layout.array_stride,
			step_mode: self.layout.step_mode,
			attributes,
		})
	}
}

#[derive(Error, Debug)]
#[error("Mesh is missing requested attribute: {name} ({id:?}, pipeline type: {pipeline_type:?})")]
pub struct MissingVertexAttributeError {
	pub(crate) pipeline_type: Option<&'static str>,
	id: usize,
	name: &'static str,
}

pub struct VertexAttributeDescriptor {
	pub shader_location: u32,
	pub id: usize,
	name: &'static str,
}

#[derive(Debug, Clone)]
struct MeshAttributeData {
	attribute: MeshVertexAttribute,
	values: VertexAttributeValues,
}

const VEC3_MIN: Vec3 = Vec3::splat(std::f32::MIN);
const VEC3_MAX: Vec3 = Vec3::splat(std::f32::MAX);

fn face_normal(a: [f32; 3], b: [f32; 3], c: [f32; 3]) -> [f32; 3] {
	let (a, b, c) = (Vec3::from(a), Vec3::from(b), Vec3::from(c));
	(b - a).cross(c - a).normalize().into()
}

pub trait VertexFormatSize {
	fn get_size(self) -> u64;
}

impl VertexFormatSize for wgpu::VertexFormat {
	#[allow(clippy::match_same_arms)]
	fn get_size(self) -> u64 {
		match self {
			VertexFormat::Uint8x2 => 2,
			VertexFormat::Uint8x4 => 4,
			VertexFormat::Sint8x2 => 2,
			VertexFormat::Sint8x4 => 4,
			VertexFormat::Unorm8x2 => 2,
			VertexFormat::Unorm8x4 => 4,
			VertexFormat::Snorm8x2 => 2,
			VertexFormat::Snorm8x4 => 4,
			VertexFormat::Uint16x2 => 2 * 2,
			VertexFormat::Uint16x4 => 2 * 4,
			VertexFormat::Sint16x2 => 2 * 2,
			VertexFormat::Sint16x4 => 2 * 4,
			VertexFormat::Unorm16x2 => 2 * 2,
			VertexFormat::Unorm16x4 => 2 * 4,
			VertexFormat::Snorm16x2 => 2 * 2,
			VertexFormat::Snorm16x4 => 2 * 4,
			VertexFormat::Float16x2 => 2 * 2,
			VertexFormat::Float16x4 => 2 * 4,
			VertexFormat::Float32 => 4,
			VertexFormat::Float32x2 => 4 * 2,
			VertexFormat::Float32x3 => 4 * 3,
			VertexFormat::Float32x4 => 4 * 4,
			VertexFormat::Uint32 => 4,
			VertexFormat::Uint32x2 => 4 * 2,
			VertexFormat::Uint32x3 => 4 * 3,
			VertexFormat::Uint32x4 => 4 * 4,
			VertexFormat::Sint32 => 4,
			VertexFormat::Sint32x2 => 4 * 2,
			VertexFormat::Sint32x3 => 4 * 3,
			VertexFormat::Sint32x4 => 4 * 4,
			VertexFormat::Float64 => 8,
			VertexFormat::Float64x2 => 8 * 2,
			VertexFormat::Float64x3 => 8 * 3,
			VertexFormat::Float64x4 => 8 * 4,
		}
	}
}

/// Contains an array where each entry describes a property of a single vertex.
/// Matches the [`VertexFormats`](VertexFormat).
#[derive(Clone, Debug, EnumVariantMeta)]
pub enum VertexAttributeValues {
	Float32(Vec<f32>),
	Sint32(Vec<i32>),
	Uint32(Vec<u32>),
	Float32x2(Vec<[f32; 2]>),
	Sint32x2(Vec<[i32; 2]>),
	Uint32x2(Vec<[u32; 2]>),
	Float32x3(Vec<[f32; 3]>),
	Sint32x3(Vec<[i32; 3]>),
	Uint32x3(Vec<[u32; 3]>),
	Float32x4(Vec<[f32; 4]>),
	Sint32x4(Vec<[i32; 4]>),
	Uint32x4(Vec<[u32; 4]>),
	Sint16x2(Vec<[i16; 2]>),
	Snorm16x2(Vec<[i16; 2]>),
	Uint16x2(Vec<[u16; 2]>),
	Unorm16x2(Vec<[u16; 2]>),
	Sint16x4(Vec<[i16; 4]>),
	Snorm16x4(Vec<[i16; 4]>),
	Uint16x4(Vec<[u16; 4]>),
	Unorm16x4(Vec<[u16; 4]>),
	Sint8x2(Vec<[i8; 2]>),
	Snorm8x2(Vec<[i8; 2]>),
	Uint8x2(Vec<[u8; 2]>),
	Unorm8x2(Vec<[u8; 2]>),
	Sint8x4(Vec<[i8; 4]>),
	Snorm8x4(Vec<[i8; 4]>),
	Uint8x4(Vec<[u8; 4]>),
	Unorm8x4(Vec<[u8; 4]>),
}

impl VertexAttributeValues {
	/// Returns the number of vertices in this [`VertexAttributeValues`]. For a single
	/// mesh, all of the [`VertexAttributeValues`] must have the same length.
	#[allow(clippy::match_same_arms)]
	pub fn len(&self) -> usize {
		match *self {
			VertexAttributeValues::Float32(ref values) => values.len(),
			VertexAttributeValues::Sint32(ref values) => values.len(),
			VertexAttributeValues::Uint32(ref values) => values.len(),
			VertexAttributeValues::Float32x2(ref values) => values.len(),
			VertexAttributeValues::Sint32x2(ref values) => values.len(),
			VertexAttributeValues::Uint32x2(ref values) => values.len(),
			VertexAttributeValues::Float32x3(ref values) => values.len(),
			VertexAttributeValues::Sint32x3(ref values) => values.len(),
			VertexAttributeValues::Uint32x3(ref values) => values.len(),
			VertexAttributeValues::Float32x4(ref values) => values.len(),
			VertexAttributeValues::Sint32x4(ref values) => values.len(),
			VertexAttributeValues::Uint32x4(ref values) => values.len(),
			VertexAttributeValues::Sint16x2(ref values) => values.len(),
			VertexAttributeValues::Snorm16x2(ref values) => values.len(),
			VertexAttributeValues::Uint16x2(ref values) => values.len(),
			VertexAttributeValues::Unorm16x2(ref values) => values.len(),
			VertexAttributeValues::Sint16x4(ref values) => values.len(),
			VertexAttributeValues::Snorm16x4(ref values) => values.len(),
			VertexAttributeValues::Uint16x4(ref values) => values.len(),
			VertexAttributeValues::Unorm16x4(ref values) => values.len(),
			VertexAttributeValues::Sint8x2(ref values) => values.len(),
			VertexAttributeValues::Snorm8x2(ref values) => values.len(),
			VertexAttributeValues::Uint8x2(ref values) => values.len(),
			VertexAttributeValues::Unorm8x2(ref values) => values.len(),
			VertexAttributeValues::Sint8x4(ref values) => values.len(),
			VertexAttributeValues::Snorm8x4(ref values) => values.len(),
			VertexAttributeValues::Uint8x4(ref values) => values.len(),
			VertexAttributeValues::Unorm8x4(ref values) => values.len(),
		}
	}

	/// Returns `true` if there are no vertices in this [`VertexAttributeValues`].
	pub fn is_empty(&self) -> bool {
		match *self {
			VertexAttributeValues::Float32(ref values) => values.is_empty(),
			VertexAttributeValues::Sint32(ref values) => values.is_empty(),
			VertexAttributeValues::Uint32(ref values) => values.is_empty(),
			VertexAttributeValues::Float32x2(ref values) => values.is_empty(),
			VertexAttributeValues::Sint32x2(ref values) => values.is_empty(),
			VertexAttributeValues::Uint32x2(ref values) => values.is_empty(),
			VertexAttributeValues::Float32x3(ref values) => values.is_empty(),
			VertexAttributeValues::Sint32x3(ref values) => values.is_empty(),
			VertexAttributeValues::Uint32x3(ref values) => values.is_empty(),
			VertexAttributeValues::Float32x4(ref values) => values.is_empty(),
			VertexAttributeValues::Sint32x4(ref values) => values.is_empty(),
			VertexAttributeValues::Uint32x4(ref values) => values.is_empty(),
			VertexAttributeValues::Sint16x2(ref values) => values.is_empty(),
			VertexAttributeValues::Snorm16x2(ref values) => values.is_empty(),
			VertexAttributeValues::Uint16x2(ref values) => values.is_empty(),
			VertexAttributeValues::Unorm16x2(ref values) => values.is_empty(),
			VertexAttributeValues::Sint16x4(ref values) => values.is_empty(),
			VertexAttributeValues::Snorm16x4(ref values) => values.is_empty(),
			VertexAttributeValues::Uint16x4(ref values) => values.is_empty(),
			VertexAttributeValues::Unorm16x4(ref values) => values.is_empty(),
			VertexAttributeValues::Sint8x2(ref values) => values.is_empty(),
			VertexAttributeValues::Snorm8x2(ref values) => values.is_empty(),
			VertexAttributeValues::Uint8x2(ref values) => values.is_empty(),
			VertexAttributeValues::Unorm8x2(ref values) => values.is_empty(),
			VertexAttributeValues::Sint8x4(ref values) => values.is_empty(),
			VertexAttributeValues::Snorm8x4(ref values) => values.is_empty(),
			VertexAttributeValues::Uint8x4(ref values) => values.is_empty(),
			VertexAttributeValues::Unorm8x4(ref values) => values.is_empty(),
		}
	}

	/// Returns the values as float triples if possible.
	pub fn as_float3(&self) -> Option<&[[f32; 3]]> {
		match self {
			VertexAttributeValues::Float32x3(values) => Some(values),
			_ => None,
		}
	}

	// TODO: add vertex format as parameter here and perform type conversions
	/// Flattens the [`VertexAttributeValues`] into a sequence of bytes. This is
	/// useful for serialization and sending to the GPU.
	#[allow(clippy::match_same_arms)]
	pub fn get_bytes(&self) -> &[u8] {
		match self {
			VertexAttributeValues::Float32(values) => cast_slice(&values[..]),
			VertexAttributeValues::Sint32(values) => cast_slice(&values[..]),
			VertexAttributeValues::Uint32(values) => cast_slice(&values[..]),
			VertexAttributeValues::Float32x2(values) => cast_slice(&values[..]),
			VertexAttributeValues::Sint32x2(values) => cast_slice(&values[..]),
			VertexAttributeValues::Uint32x2(values) => cast_slice(&values[..]),
			VertexAttributeValues::Float32x3(values) => cast_slice(&values[..]),
			VertexAttributeValues::Sint32x3(values) => cast_slice(&values[..]),
			VertexAttributeValues::Uint32x3(values) => cast_slice(&values[..]),
			VertexAttributeValues::Float32x4(values) => cast_slice(&values[..]),
			VertexAttributeValues::Sint32x4(values) => cast_slice(&values[..]),
			VertexAttributeValues::Uint32x4(values) => cast_slice(&values[..]),
			VertexAttributeValues::Sint16x2(values) => cast_slice(&values[..]),
			VertexAttributeValues::Snorm16x2(values) => cast_slice(&values[..]),
			VertexAttributeValues::Uint16x2(values) => cast_slice(&values[..]),
			VertexAttributeValues::Unorm16x2(values) => cast_slice(&values[..]),
			VertexAttributeValues::Sint16x4(values) => cast_slice(&values[..]),
			VertexAttributeValues::Snorm16x4(values) => cast_slice(&values[..]),
			VertexAttributeValues::Uint16x4(values) => cast_slice(&values[..]),
			VertexAttributeValues::Unorm16x4(values) => cast_slice(&values[..]),
			VertexAttributeValues::Sint8x2(values) => cast_slice(&values[..]),
			VertexAttributeValues::Snorm8x2(values) => cast_slice(&values[..]),
			VertexAttributeValues::Uint8x2(values) => cast_slice(&values[..]),
			VertexAttributeValues::Unorm8x2(values) => cast_slice(&values[..]),
			VertexAttributeValues::Sint8x4(values) => cast_slice(&values[..]),
			VertexAttributeValues::Snorm8x4(values) => cast_slice(&values[..]),
			VertexAttributeValues::Uint8x4(values) => cast_slice(&values[..]),
			VertexAttributeValues::Unorm8x4(values) => cast_slice(&values[..]),
		}
	}
}

impl From<&VertexAttributeValues> for VertexFormat {
	fn from(values: &VertexAttributeValues) -> Self {
		match values {
			VertexAttributeValues::Float32(_) => VertexFormat::Float32,
			VertexAttributeValues::Sint32(_) => VertexFormat::Sint32,
			VertexAttributeValues::Uint32(_) => VertexFormat::Uint32,
			VertexAttributeValues::Float32x2(_) => VertexFormat::Float32x2,
			VertexAttributeValues::Sint32x2(_) => VertexFormat::Sint32x2,
			VertexAttributeValues::Uint32x2(_) => VertexFormat::Uint32x2,
			VertexAttributeValues::Float32x3(_) => VertexFormat::Float32x3,
			VertexAttributeValues::Sint32x3(_) => VertexFormat::Sint32x3,
			VertexAttributeValues::Uint32x3(_) => VertexFormat::Uint32x3,
			VertexAttributeValues::Float32x4(_) => VertexFormat::Float32x4,
			VertexAttributeValues::Sint32x4(_) => VertexFormat::Sint32x4,
			VertexAttributeValues::Uint32x4(_) => VertexFormat::Uint32x4,
			VertexAttributeValues::Sint16x2(_) => VertexFormat::Sint16x2,
			VertexAttributeValues::Snorm16x2(_) => VertexFormat::Snorm16x2,
			VertexAttributeValues::Uint16x2(_) => VertexFormat::Uint16x2,
			VertexAttributeValues::Unorm16x2(_) => VertexFormat::Unorm16x2,
			VertexAttributeValues::Sint16x4(_) => VertexFormat::Sint16x4,
			VertexAttributeValues::Snorm16x4(_) => VertexFormat::Snorm16x4,
			VertexAttributeValues::Uint16x4(_) => VertexFormat::Uint16x4,
			VertexAttributeValues::Unorm16x4(_) => VertexFormat::Unorm16x4,
			VertexAttributeValues::Sint8x2(_) => VertexFormat::Sint8x2,
			VertexAttributeValues::Snorm8x2(_) => VertexFormat::Snorm8x2,
			VertexAttributeValues::Uint8x2(_) => VertexFormat::Uint8x2,
			VertexAttributeValues::Unorm8x2(_) => VertexFormat::Unorm8x2,
			VertexAttributeValues::Sint8x4(_) => VertexFormat::Sint8x4,
			VertexAttributeValues::Snorm8x4(_) => VertexFormat::Snorm8x4,
			VertexAttributeValues::Uint8x4(_) => VertexFormat::Uint8x4,
			VertexAttributeValues::Unorm8x4(_) => VertexFormat::Unorm8x4,
		}
	}
}

/// The GPU-representation of a [`Mesh`].
/// Consists of a vertex data buffer and an optional index data buffer.
#[derive(Debug, Clone)]
pub struct GpuMesh {
	/// Contains all attribute data for each vertex.
	pub vertex_buffer: Buffer,
	pub buffer_info: GpuBufferInfo,
	pub primitive_topology: PrimitiveTopology,
	pub layout: MeshVertexBufferLayout,
}

/// The index/vertex buffer info of a [`GpuMesh`].
#[derive(Debug, Clone)]
pub enum GpuBufferInfo {
	Indexed {
		/// Contains all index data of a mesh.
		buffer: Buffer,
		count: u32,
		index_format: IndexFormat,
	},
	NonIndexed {
		vertex_count: u32,
	},
}

impl RenderAsset for Mesh {
	type ExtractedAsset = Mesh;
	type PreparedAsset = GpuMesh;
	type Param = SRes<RenderDevice>;

	/// Clones the mesh.
	fn extract_asset(&self) -> Self::ExtractedAsset {
		self.clone()
	}

	/// Converts the extracted mesh a into [`GpuMesh`].
	fn prepare_asset(
		mesh: Self::ExtractedAsset,
		render_device: &mut SystemParamItem<Self::Param>,
	) -> Result<Self::PreparedAsset, PrepareAssetError<Self::ExtractedAsset>> {
		Ok(GpuMesh {
			vertex_buffer: render_device.create_buffer_with_data(&BufferInitDescriptor {
				usage: BufferUsages::VERTEX,
				label: Some("Mesh Vertex Buffer"),
				contents: &mesh.get_vertex_buffer_data(),
			}),
			buffer_info: mesh.get_index_buffer_bytes().map_or(
				GpuBufferInfo::NonIndexed {
					vertex_count: mesh.count_vertices() as u32,
				},
				|data| {
					let i = mesh.indices().unwrap();
					GpuBufferInfo::Indexed {
						buffer: render_device.create_buffer_with_data(&BufferInitDescriptor {
							usage: BufferUsages::INDEX,
							contents: data,
							label: Some("Mesh Index Buffer"),
						}),
						count: i.len() as u32,
						index_format: i.into(),
					}
				},
			),
			layout: mesh.get_mesh_vertex_buffer_layout(),
			primitive_topology: mesh.into(),
		})
	}
}

struct MikktspaceGeometryHelper<'a> {
	indices: &'a Indices,
	positions: &'a Vec<[f32; 3]>,
	normals: &'a Vec<[f32; 3]>,
	uvs: &'a Vec<[f32; 2]>,
	tangents: Vec<[f32; 4]>,
}

impl MikktspaceGeometryHelper<'_> {
	fn index(&self, face: usize, vert: usize) -> usize {
		let index_index = face * 3 + vert;

		match self.indices {
			Indices::U16(indices) => indices[index_index] as usize,
			Indices::U32(indices) => indices[index_index] as usize,
		}
	}
}

impl bevy_mikktspace::Geometry for MikktspaceGeometryHelper<'_> {
	fn num_faces(&self) -> usize {
		self.indices.len() / 3
	}

	fn num_vertices_of_face(&self, _: usize) -> usize {
		3
	}

	fn position(&self, face: usize, vert: usize) -> [f32; 3] {
		self.positions[self.index(face, vert)]
	}

	fn normal(&self, face: usize, vert: usize) -> [f32; 3] {
		self.normals[self.index(face, vert)]
	}

	fn tex_coord(&self, face: usize, vert: usize) -> [f32; 2] {
		self.uvs[self.index(face, vert)]
	}

	fn set_tangent_encoded(&mut self, tangent: [f32; 4], face: usize, vert: usize) {
		let idx = self.index(face, vert);
		self.tangents[idx] = tangent;
	}
}

#[derive(thiserror::Error, Debug)]
/// Failed to generate tangents for the mesh.
pub enum GenerateTangentsError {
	#[error("cannot generate tangents for {0:?}")]
	UnsupportedTopology(PrimitiveTopology),
	#[error("missing indices")]
	MissingIndices,
	#[error("missing vertex attributes '{0}'")]
	MissingVertexAttribute(&'static str),
	#[error("the '{0}' vertex attribute should have {1:?} format")]
	InvalidVertexAttributeFormat(&'static str, VertexFormat),
	#[error("mesh not suitable for tangent generation")]
	MikktspaceError,
}

fn generate_tangents_for_mesh(mesh: &Mesh) -> Result<Vec<[f32; 4]>, GenerateTangentsError> {
	match mesh.primitive_topology() {
		PrimitiveTopology::TriangleList => {},
		other => return Err(GenerateTangentsError::UnsupportedTopology(other)),
	};

	let positions = match mesh
		.attribute(MeshVertexAttribute::POSITION.id)
		.ok_or(GenerateTangentsError::MissingVertexAttribute(
			MeshVertexAttribute::POSITION.name,
		))? {
		VertexAttributeValues::Float32x3(vertices) => vertices,
		_ => {
			return Err(GenerateTangentsError::InvalidVertexAttributeFormat(
				MeshVertexAttribute::POSITION.name,
				VertexFormat::Float32x3,
			))
		},
	};

	let normals = match mesh
		.attribute(MeshVertexAttribute::NORMAL.id)
		.ok_or(GenerateTangentsError::MissingVertexAttribute(
			MeshVertexAttribute::NORMAL.name,
		))? {
		VertexAttributeValues::Float32x3(vertices) => vertices,
		_ => {
			return Err(GenerateTangentsError::InvalidVertexAttributeFormat(
				MeshVertexAttribute::NORMAL.name,
				VertexFormat::Float32x3,
			))
		},
	};

	let uvs = match mesh
		.attribute(MeshVertexAttribute::UV_0.id)
		.ok_or(GenerateTangentsError::MissingVertexAttribute(
			MeshVertexAttribute::UV_0.name,
		))? {
		VertexAttributeValues::Float32x2(vertices) => vertices,
		_ => {
			return Err(GenerateTangentsError::InvalidVertexAttributeFormat(
				MeshVertexAttribute::UV_0.name,
				VertexFormat::Float32x2,
			))
		},
	};

	let mut mikktspace_mesh = MikktspaceGeometryHelper {
		indices: mesh
			.indices()
			.ok_or(GenerateTangentsError::MissingIndices)?,
		positions,
		normals,
		uvs,
		tangents: vec![[0., 0., 0., 0.]; positions.len()],
	};

	bevy_mikktspace::generate_tangents(&mut mikktspace_mesh)
		.then(|| {
			// mikktspace seems to assume left-handedness so we can flip the sign to correct for this
			for tangent in &mut mikktspace_mesh.tangents {
				tangent[3] = -tangent[3];
			}
			mikktspace_mesh.tangents
		})
		.ok_or(GenerateTangentsError::MikktspaceError)
}

#[cfg(test)]
mod tests {
	use super::{Mesh, MeshVertexAttribute};
	use wgpu::PrimitiveTopology;

	#[test]
	#[should_panic]
	fn panic_invalid_format() {
		let mut mesh = Mesh::from(PrimitiveTopology::TriangleList);
		mesh.insert_attribute(MeshVertexAttribute::UV_0, vec![[0.0, 0.0, 0.0]]);
	}
}
