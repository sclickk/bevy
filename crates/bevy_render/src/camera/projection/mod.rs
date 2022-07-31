mod orthographic;
pub use orthographic::*;

mod perspective;
pub use perspective::*;

use std::marker::PhantomData;

use super::DepthCalculation;
use bevy_app::{App, CoreStage, Plugin, StartupStage};
use bevy_ecs::{prelude::*, reflect::ReflectComponent};
use bevy_math::Mat4;
use bevy_reflect::{
	std_traits::ReflectDefault, GetTypeRegistration, Reflect, ReflectDeserialize, ReflectSerialize,
};
use bevy_window::ModifiesWindows;
use serde::{Deserialize, Serialize};

/// Adds [`Camera`](crate::camera::Camera) driver systems for a given projection type.
pub struct CameraProjectionPlugin<T: CameraProjection>(PhantomData<T>);

impl<T: CameraProjection> Default for CameraProjectionPlugin<T> {
	fn default() -> Self {
		Self(Default::default())
	}
}

#[derive(SystemLabel, Clone, Eq, PartialEq, Hash, Debug)]
pub struct CameraUpdateSystem;

impl<T: CameraProjection + Component + GetTypeRegistration> Plugin for CameraProjectionPlugin<T> {
	fn build(&self, app: &mut App) {
		app.register_type::<T>();
		app.add_startup_system_to_stage(StartupStage::PostStartup, crate::camera::camera_system::<T>);
		app.add_system_to_stage(
			CoreStage::PostUpdate,
			crate::camera::camera_system::<T>
				.label(CameraUpdateSystem)
				.after(ModifiesWindows),
		);
	}
}

pub trait CameraProjection {
	fn get_projection_matrix(&self) -> Mat4;
	fn update(&mut self, width: f32, height: f32);
	fn depth_calculation(&self) -> DepthCalculation;
	fn far(&self) -> f32;
}

/// A configurable [`CameraProjection`] that can select its projection type at runtime.
#[derive(Component, Debug, Clone, Reflect)]
#[reflect(Component, Default)]
pub enum Projection {
	Perspective(PerspectiveProjection),
	Orthographic(OrthographicProjection),
}

impl From<PerspectiveProjection> for Projection {
	fn from(p: PerspectiveProjection) -> Self {
		Self::Perspective(p)
	}
}

impl From<OrthographicProjection> for Projection {
	fn from(p: OrthographicProjection) -> Self {
		Self::Orthographic(p)
	}
}

impl CameraProjection for Projection {
	fn get_projection_matrix(&self) -> Mat4 {
		match self {
			Projection::Perspective(projection) => projection.get_projection_matrix(),
			Projection::Orthographic(projection) => projection.get_projection_matrix(),
		}
	}

	fn update(&mut self, width: f32, height: f32) {
		match self {
			Projection::Perspective(projection) => projection.update(width, height),
			Projection::Orthographic(projection) => projection.update(width, height),
		}
	}

	fn depth_calculation(&self) -> DepthCalculation {
		match self {
			Projection::Perspective(projection) => projection.depth_calculation(),
			Projection::Orthographic(projection) => projection.depth_calculation(),
		}
	}

	fn far(&self) -> f32 {
		match self {
			Projection::Perspective(projection) => projection.far(),
			Projection::Orthographic(projection) => projection.far(),
		}
	}
}

impl Default for Projection {
	fn default() -> Self {
		Projection::Perspective(Default::default())
	}
}

// TODO: make this a component instead of a property
#[derive(Debug, Clone, Reflect, Serialize, Deserialize)]
#[reflect_value(Serialize, Deserialize)]
pub enum WindowOrigin {
	Center,
	BottomLeft,
}

#[derive(Debug, Clone, Reflect, Serialize, Deserialize)]
#[reflect_value(Serialize, Deserialize)]
pub enum ScalingMode {
	/// Manually specify left/right/top/bottom values.
	/// Ignore window resizing; the image will stretch.
	None,
	/// Match the window size. 1 world unit = 1 pixel.
	WindowSize,
	/// Use minimal possible viewport size while keeping the aspect ratio.
	/// Arguments are in world units.
	Auto { min_width: f32, min_height: f32 },
	/// Keep vertical axis constant; resize horizontal with aspect ratio.
	/// The argument is the desired height of the viewport in world units.
	FixedVertical(f32),
	/// Keep horizontal axis constant; resize vertical with aspect ratio.
	/// The argument is the desired width of the viewport in world units.
	FixedHorizontal(f32),
}
