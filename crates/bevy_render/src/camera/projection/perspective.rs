use crate::camera::{CameraProjection, DepthCalculation};

use bevy_ecs::{component::Component, reflect::ReflectComponent};
use bevy_math::Mat4;
use bevy_reflect::{std_traits::ReflectDefault, Reflect};

#[derive(Component, Debug, Clone, Reflect)]
#[reflect(Component, Default)]
pub struct PerspectiveProjection {
	pub fov: f32,
	pub aspect_ratio: f32,
	pub near: f32,
	pub far: f32,
}

impl CameraProjection for PerspectiveProjection {
	fn get_projection_matrix(&self) -> Mat4 {
		Mat4::perspective_infinite_reverse_rh(self.fov, self.aspect_ratio, self.near)
	}

	fn update(&mut self, width: f32, height: f32) {
		self.aspect_ratio = width / height;
	}

	fn depth_calculation(&self) -> DepthCalculation {
		DepthCalculation::Distance
	}

	fn far(&self) -> f32 {
		self.far
	}
}

impl Default for PerspectiveProjection {
	fn default() -> Self {
		PerspectiveProjection {
			fov: std::f32::consts::FRAC_PI_4,
			near: 0.1,
			far: 1000.0,
			aspect_ratio: 1.0,
		}
	}
}
