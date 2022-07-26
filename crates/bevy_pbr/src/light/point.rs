use std::{collections::HashSet, num::NonZeroU64};

use bevy_ecs::prelude::*;
use bevy_math::Vec4;
use bevy_reflect::prelude::*;
use bevy_render::{
	color::Color,
	primitives::Sphere,
	render_resource::{BindingResource, BufferBindingType, ShaderType, StorageBuffer, UniformBuffer},
	renderer::{RenderDevice, RenderQueue},
};
use bevy_transform::components::GlobalTransform;

use crate::MAX_UNIFORM_BUFFER_POINT_LIGHTS;

/// A light that emits light in all directions from a central point.
///
/// Real-world values for `intensity` (luminous power in lumens) based on the electrical power
/// consumption of the type of real-world light are:
///
/// | Luminous Power (lumen) (i.e. the intensity member) | Incandescent non-halogen (Watts) | Incandescent halogen (Watts) | Compact fluorescent (Watts) | LED (Watts |
/// |------|-----|----|--------|-------|
/// | 200  | 25  |    | 3-5    | 3     |
/// | 450  | 40  | 29 | 9-11   | 5-8   |
/// | 800  | 60  |    | 13-15  | 8-12  |
/// | 1100 | 75  | 53 | 18-20  | 10-16 |
/// | 1600 | 100 | 72 | 24-28  | 14-17 |
/// | 2400 | 150 |    | 30-52  | 24-30 |
/// | 3100 | 200 |    | 49-75  | 32    |
/// | 4000 | 300 |    | 75-100 | 40.5  |
///
/// Source: [Wikipedia](https://en.wikipedia.org/wiki/Lumen_(unit)#Lighting)
#[derive(Component, Debug, Clone, Copy, Reflect)]
#[reflect(Component, Default)]
pub struct PointLight {
	pub color: Color,
	pub intensity: f32,
	pub range: f32,
	pub radius: f32,
	pub shadows_enabled: bool,
	pub shadow_depth_bias: f32,
	/// A bias applied along the direction of the fragment's surface normal. It is scaled to the
	/// shadow map's texel size so that it can be small close to the camera and gets larger further
	/// away.
	pub shadow_normal_bias: f32,
}

impl Default for PointLight {
	fn default() -> Self {
		PointLight {
			color: Color::rgb(1.0, 1.0, 1.0),
			/// Luminous power in lumens
			intensity: 800.0, // Roughly a 60W non-halogen incandescent bulb
			range: 20.0,
			radius: 0.0,
			shadows_enabled: false,
			shadow_depth_bias: Self::DEFAULT_SHADOW_DEPTH_BIAS,
			shadow_normal_bias: Self::DEFAULT_SHADOW_NORMAL_BIAS,
		}
	}
}

impl PointLight {
	pub const DEFAULT_SHADOW_DEPTH_BIAS: f32 = 0.02;
	pub const DEFAULT_SHADOW_NORMAL_BIAS: f32 = 0.6;
}

#[derive(Clone, Debug, Reflect)]
#[reflect(Resource)]
pub struct PointLightShadowMap {
	pub size: usize,
}

impl Default for PointLightShadowMap {
	fn default() -> Self {
		Self { size: 1024 }
	}
}

#[derive(Clone, Component, Debug, Default)]
pub struct VisiblePointLights {
	pub(crate) entities: Vec<Entity>,
	pub point_light_count: usize,
	pub spot_light_count: usize,
}

impl VisiblePointLights {
	#[inline]
	pub fn iter(&self) -> impl DoubleEndedIterator<Item = &Entity> {
		self.entities.iter()
	}

	#[inline]
	pub fn len(&self) -> usize {
		self.entities.len()
	}

	#[inline]
	pub fn is_empty(&self) -> bool {
		self.entities.is_empty()
	}
}

#[derive(Clone, Copy)]
// data required for assigning lights to clusters
pub(crate) struct PointLightAssignmentData {
	pub(crate) entity: Entity,
	pub(crate) transform: GlobalTransform,
	pub(crate) range: f32,
	pub(crate) shadows_enabled: bool,
	pub(crate) spot_light_angle: Option<f32>,
}

impl PointLightAssignmentData {
	pub fn sphere(&self) -> Sphere {
		Sphere {
			center: self.transform.translation_vec3a(),
			radius: self.range,
		}
	}
}

#[derive(Default)]
pub struct GlobalVisiblePointLights {
	pub(crate) entities: HashSet<Entity>,
}

impl GlobalVisiblePointLights {
	#[inline]
	pub fn iter(&self) -> impl Iterator<Item = &Entity> {
		self.entities.iter()
	}

	#[inline]
	pub fn contains(&self, entity: Entity) -> bool {
		self.entities.contains(&entity)
	}
}

#[derive(Component)]
pub struct ExtractedPointLight {
	pub(crate) color: Color,
	/// luminous intensity in lumens per steradian
	pub(crate) intensity: f32,
	pub(crate) range: f32,
	pub(crate) radius: f32,
	pub(crate) transform: GlobalTransform,
	pub(crate) shadows_enabled: bool,
	pub(crate) shadow_depth_bias: f32,
	pub(crate) shadow_normal_bias: f32,
	pub(crate) spot_light_angles: Option<(f32, f32)>,
}

#[derive(Copy, Clone, ShaderType, Default, Debug)]
pub struct GpuPointLight {
	// For point lights: the lower-right 2x2 values of the projection matrix [2][2] [2][3] [3][2] [3][3]
	// For spot lights: 2 components of the direction (x,z), spot_scale and spot_offset
	pub(crate) light_custom_data: Vec4,
	pub(crate) color_inverse_square_range: Vec4,
	pub(crate) position_radius: Vec4,
	pub(crate) flags: u32,
	pub(crate) shadow_depth_bias: f32,
	pub(crate) shadow_normal_bias: f32,
	pub(crate) spot_light_tan_angle: f32,
}

#[derive(ShaderType)]
pub struct GpuPointLightsUniform {
	data: Box<[GpuPointLight; MAX_UNIFORM_BUFFER_POINT_LIGHTS]>,
}

impl Default for GpuPointLightsUniform {
	fn default() -> Self {
		Self {
			data: Box::new([GpuPointLight::default(); MAX_UNIFORM_BUFFER_POINT_LIGHTS]),
		}
	}
}

#[derive(ShaderType, Default)]
pub struct GpuPointLightsStorage {
	#[size(runtime)]
	data: Vec<GpuPointLight>,
}

pub enum GpuPointLights {
	Uniform(UniformBuffer<GpuPointLightsUniform>),
	Storage(StorageBuffer<GpuPointLightsStorage>),
}

impl GpuPointLights {
	fn uniform() -> Self {
		Self::Uniform(UniformBuffer::default())
	}

	fn storage() -> Self {
		Self::Storage(StorageBuffer::default())
	}

	pub(crate) fn set(&mut self, mut lights: Vec<GpuPointLight>) {
		match self {
			GpuPointLights::Uniform(buffer) => {
				let len = lights
					.len()
					.min(MAX_UNIFORM_BUFFER_POINT_LIGHTS);
				let src = &lights[..len];
				let dst = &mut buffer.get_mut().data[..len];
				dst.copy_from_slice(src);
			},
			GpuPointLights::Storage(buffer) => {
				buffer.get_mut().data.clear();
				buffer.get_mut().data.append(&mut lights);
			},
		}
	}

	pub(crate) fn write_buffer(&mut self, render_device: &RenderDevice, render_queue: &RenderQueue) {
		match self {
			GpuPointLights::Uniform(buffer) => buffer.write_buffer(render_device, render_queue),
			GpuPointLights::Storage(buffer) => buffer.write_buffer(render_device, render_queue),
		}
	}

	pub fn binding(&self) -> Option<BindingResource> {
		match self {
			GpuPointLights::Uniform(buffer) => buffer.binding(),
			GpuPointLights::Storage(buffer) => buffer.binding(),
		}
	}

	pub fn min_size(buffer_binding_type: BufferBindingType) -> NonZeroU64 {
		match buffer_binding_type {
			BufferBindingType::Storage { .. } => GpuPointLightsStorage::min_size(),
			BufferBindingType::Uniform => GpuPointLightsUniform::min_size(),
		}
	}
}

impl From<BufferBindingType> for GpuPointLights {
	fn from(buffer_binding_type: BufferBindingType) -> Self {
		match buffer_binding_type {
			BufferBindingType::Storage { .. } => Self::storage(),
			BufferBindingType::Uniform => Self::uniform(),
		}
	}
}

// NOTE: These must match the bit flags in bevy_pbr2/src/render/pbr.frag!
bitflags::bitflags! {
	#[repr(transparent)]
	pub(crate) struct PointLightFlags: u32 {
		const SHADOWS_ENABLED       = (1 << 0);
		const SPOT_LIGHT_Y_NEGATIVE = (1 << 1);
		const NONE                  = 0;
		const UNINITIALIZED         = 0xFFFF;
	}
}
