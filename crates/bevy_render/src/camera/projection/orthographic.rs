use crate::camera::{CameraProjection, DepthCalculation, ScalingMode, WindowOrigin};

use bevy_ecs::{component::Component, reflect::ReflectComponent};
use bevy_math::Mat4;
use bevy_reflect::{std_traits::ReflectDefault, Reflect};

#[derive(Component, Debug, Clone, Reflect)]
#[reflect(Component, Default)]
pub struct OrthographicProjection {
	pub left: f32,
	pub right: f32,
	pub bottom: f32,
	pub top: f32,
	pub near: f32,
	pub far: f32,
	pub window_origin: WindowOrigin,
	pub scaling_mode: ScalingMode,
	pub scale: f32,
	pub depth_calculation: DepthCalculation,
}

impl CameraProjection for OrthographicProjection {
	fn get_projection_matrix(&self) -> Mat4 {
		Mat4::orthographic_rh(
			self.left * self.scale,
			self.right * self.scale,
			self.bottom * self.scale,
			self.top * self.scale,
			// NOTE: near and far are swapped to invert the depth range from [0,1] to [1,0]
			// This is for interoperability with pipelines using infinite reverse perspective projections.
			self.far,
			self.near,
		)
	}

	fn update(&mut self, width: f32, height: f32) {
		let (viewport_width, viewport_height) = match self.scaling_mode {
			ScalingMode::WindowSize => (width, height),
			ScalingMode::Auto {
				min_width,
				min_height,
			} => {
				if width * min_height > min_width * height {
					(width * min_height / height, min_height)
				} else {
					(min_width, height * min_width / width)
				}
			},
			ScalingMode::FixedVertical(viewport_height) => {
				(width * viewport_height / height, viewport_height)
			},
			ScalingMode::FixedHorizontal(viewport_width) => {
				(viewport_width, height * viewport_width / width)
			},
			ScalingMode::None => return,
		};

		match self.window_origin {
			WindowOrigin::Center => {
				let half_width = viewport_width / 2.0;
				let half_height = viewport_height / 2.0;
				self.left = -half_width;
				self.bottom = -half_height;
				self.right = half_width;
				self.top = half_height;

				if let ScalingMode::WindowSize = self.scaling_mode {
					if self.scale == 1.0 {
						self.left = self.left.floor();
						self.bottom = self.bottom.floor();
						self.right = self.right.floor();
						self.top = self.top.floor();
					}
				}
			},
			WindowOrigin::BottomLeft => {
				self.left = 0.0;
				self.bottom = 0.0;
				self.right = viewport_width;
				self.top = viewport_height;
			},
		}
	}

	fn depth_calculation(&self) -> DepthCalculation {
		self.depth_calculation
	}

	fn far(&self) -> f32 {
		self.far
	}
}

impl Default for OrthographicProjection {
	fn default() -> Self {
		OrthographicProjection {
			left: -1.0,
			right: 1.0,
			bottom: -1.0,
			top: 1.0,
			near: 0.0,
			far: 1000.0,
			window_origin: WindowOrigin::Center,
			scaling_mode: ScalingMode::WindowSize,
			scale: 1.0,
			depth_calculation: DepthCalculation::Distance,
		}
	}
}
