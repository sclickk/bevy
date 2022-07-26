use bevy_ecs::prelude::*;
use bevy_reflect::prelude::*;
use bevy_render::color::Color;

/// A light that emits light in a given direction from a central point.
/// Behaves like a point light in a perfectly absorbant housing that
/// shines light only in a given direction. The direction is taken from
/// the transform, and can be specified with [`Transform::looking_at`](bevy_transform::components::Transform::looking_at).
#[derive(Component, Debug, Clone, Copy, Reflect)]
#[reflect(Component, Default)]
pub struct SpotLight {
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
	/// Angle defining the distance from the spot light direction to the outer limit
	/// of the light's cone of effect.
	/// `outer_angle` should be < `PI / 2.0`.
	/// `PI / 2.0` defines a hemispherical spot light, but shadows become very blocky as the angle
	/// approaches this limit.
	pub outer_angle: f32,
	/// Angle defining the distance from the spot light direction to the inner limit
	/// of the light's cone of effect.
	/// Light is attenuated from `inner_angle` to `outer_angle` to give a smooth falloff.
	/// `inner_angle` should be <= `outer_angle`
	pub inner_angle: f32,
}

impl SpotLight {
	pub const DEFAULT_SHADOW_DEPTH_BIAS: f32 = 0.02;
	pub const DEFAULT_SHADOW_NORMAL_BIAS: f32 = 0.6;
}

impl Default for SpotLight {
	fn default() -> Self {
		// a quarter arc attenuating from the centre
		Self {
			color: Color::rgb(1.0, 1.0, 1.0),
			/// Luminous power in lumens
			intensity: 800.0, // Roughly a 60W non-halogen incandescent bulb
			range: 20.0,
			radius: 0.0,
			shadows_enabled: false,
			shadow_depth_bias: Self::DEFAULT_SHADOW_DEPTH_BIAS,
			shadow_normal_bias: Self::DEFAULT_SHADOW_NORMAL_BIAS,
			inner_angle: 0.0,
			outer_angle: std::f32::consts::FRAC_PI_4,
		}
	}
}
