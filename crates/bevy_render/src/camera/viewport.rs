use bevy_math::UVec2;
use bevy_reflect::prelude::*;
use serde::{Deserialize, Serialize};
use std::ops::Range;

/// Render viewport configuration for the [`Camera`] component.
///
/// The viewport defines the area on the render target to which the camera renders its image.
/// You can overlay multiple cameras in a single window using viewports to create effects like
/// split screen, minimaps, and character viewers.
// TODO: remove reflect_value when possible
#[derive(Reflect, Debug, Clone, Serialize, Deserialize)]
#[reflect_value(Default, Serialize, Deserialize)]
pub struct Viewport {
	/// The physical position to render this viewport to within the [`RenderTarget`] of this [`Camera`].
	/// (0,0) corresponds to the top-left corner
	pub physical_position: UVec2,
	/// The physical size of the viewport rectangle to render to within the [`RenderTarget`] of this [`Camera`].
	/// The origin of the rectangle is in the top-left corner.
	pub physical_size: UVec2,
	/// The minimum and maximum depth to render (on a scale from 0.0 to 1.0).
	pub depth: Range<f32>,
}

impl Default for Viewport {
	fn default() -> Self {
		Self {
			physical_position: Default::default(),
			physical_size: Default::default(),
			depth: 0.0..1.0,
		}
	}
}
