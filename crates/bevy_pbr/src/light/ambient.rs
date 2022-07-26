use bevy_ecs::prelude::*;
use bevy_reflect::prelude::*;
use bevy_render::{color::Color, extract_resource::ExtractResource};

/// An ambient light, which lights the entire scene equally.
#[derive(Clone, Debug, ExtractResource, Reflect)]
#[reflect(Resource)]
pub struct AmbientLight {
	pub color: Color,
	/// A direct scale factor multiplied with `color` before being passed to the shader.
	pub brightness: f32,
}

impl Default for AmbientLight {
	fn default() -> Self {
		Self {
			color: Color::rgb(1.0, 1.0, 1.0),
			brightness: 0.05,
		}
	}
}
