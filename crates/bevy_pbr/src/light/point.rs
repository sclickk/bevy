use std::collections::HashSet;

use bevy_ecs::prelude::*;
use bevy_reflect::prelude::*;
use bevy_render::{
	color::Color,
	primitives::Sphere,
};
use bevy_transform::{components::GlobalTransform};

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
