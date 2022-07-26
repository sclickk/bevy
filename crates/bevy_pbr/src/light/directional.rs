use bevy_ecs::prelude::*;
use bevy_reflect::prelude::*;
use bevy_render::{
	camera::OrthographicProjection,
	color::Color,
};

/// A Directional light.
///
/// Directional lights don't exist in reality but they are a good
/// approximation for light sources VERY far away, like the sun or
/// the moon.
///
/// Valid values for `illuminance` are:
///
/// | Illuminance (lux) | Surfaces illuminated by                        |
/// |-------------------|------------------------------------------------|
/// | 0.0001            | Moonless, overcast night sky (starlight)       |
/// | 0.002             | Moonless clear night sky with airglow          |
/// | 0.05–0.3          | Full moon on a clear night                     |
/// | 3.4               | Dark limit of civil twilight under a clear sky |
/// | 20–50             | Public areas with dark surroundings            |
/// | 50                | Family living room lights                      |
/// | 80                | Office building hallway/toilet lighting        |
/// | 100               | Very dark overcast day                         |
/// | 150               | Train station platforms                        |
/// | 320–500           | Office lighting                                |
/// | 400               | Sunrise or sunset on a clear day.              |
/// | 1000              | Overcast day; typical TV studio lighting       |
/// | 10,000–25,000     | Full daylight (not direct sun)                 |
/// | 32,000–100,000    | Direct sunlight                                |
///
/// Source: [Wikipedia](https://en.wikipedia.org/wiki/Lux)
#[derive(Component, Debug, Clone, Reflect)]
#[reflect(Component, Default)]
pub struct DirectionalLight {
	pub color: Color,
	/// Illuminance in lux
	pub illuminance: f32,
	pub shadows_enabled: bool,
	pub shadow_projection: OrthographicProjection,
	pub shadow_depth_bias: f32,
	/// A bias applied along the direction of the fragment's surface normal. It is scaled to the
	/// shadow map's texel size so that it is automatically adjusted to the orthographic projection.
	pub shadow_normal_bias: f32,
}

impl Default for DirectionalLight {
	fn default() -> Self {
		let size = 100.0;
		DirectionalLight {
			color: Color::rgb(1.0, 1.0, 1.0),
			illuminance: 100000.0,
			shadows_enabled: false,
			shadow_projection: OrthographicProjection {
				left: -size,
				right: size,
				bottom: -size,
				top: size,
				near: -size,
				far: size,
				..Default::default()
			},
			shadow_depth_bias: Self::DEFAULT_SHADOW_DEPTH_BIAS,
			shadow_normal_bias: Self::DEFAULT_SHADOW_NORMAL_BIAS,
		}
	}
}

impl DirectionalLight {
	pub const DEFAULT_SHADOW_DEPTH_BIAS: f32 = 0.02;
	pub const DEFAULT_SHADOW_NORMAL_BIAS: f32 = 0.6;
}

#[derive(Clone, Debug, Reflect)]
#[reflect(Resource)]
pub struct DirectionalLightShadowMap {
	pub size: usize,
}

impl Default for DirectionalLightShadowMap {
	fn default() -> Self {
		#[cfg(feature = "webgl")]
		return Self { size: 2048 };
		#[cfg(not(feature = "webgl"))]
		return Self { size: 4096 };
	}
}
