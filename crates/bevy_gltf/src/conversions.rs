use anyhow::Result;
use bevy_pbr::AlphaMode;
use bevy_render::{
	camera::{OrthographicProjection, PerspectiveProjection, Projection, ScalingMode},
	render_resource::{AddressMode, PrimitiveTopology},
};

use gltf::{mesh::Mode, texture::WrappingMode, Material};

use crate::GltfError;

/// Maps the texture address mode form glTF to wgpu.
#[inline]
pub(crate) fn texture_address_mode(gltf_address_mode: &gltf::texture::WrappingMode) -> AddressMode {
	match gltf_address_mode {
		WrappingMode::ClampToEdge => AddressMode::ClampToEdge,
		WrappingMode::Repeat => AddressMode::Repeat,
		WrappingMode::MirroredRepeat => AddressMode::MirrorRepeat,
	}
}

/// Maps the `primitive_topology` form glTF to `wgpu`.
#[inline]
pub(crate) fn get_primitive_topology(mode: Mode) -> Result<PrimitiveTopology, GltfError> {
	match mode {
		Mode::Points => Ok(PrimitiveTopology::PointList),
		Mode::Lines => Ok(PrimitiveTopology::LineList),
		Mode::LineStrip => Ok(PrimitiveTopology::LineStrip),
		Mode::Triangles => Ok(PrimitiveTopology::TriangleList),
		Mode::TriangleStrip => Ok(PrimitiveTopology::TriangleStrip),
		mode => Err(GltfError::UnsupportedPrimitive { mode }),
	}
}

#[inline]
pub(crate) fn alpha_mode(material: &Material) -> AlphaMode {
	match material.alpha_mode() {
		gltf::material::AlphaMode::Opaque => AlphaMode::Opaque,
		gltf::material::AlphaMode::Mask => AlphaMode::Mask(material.alpha_cutoff().unwrap_or(0.5)),
		gltf::material::AlphaMode::Blend => AlphaMode::Blend,
	}
}

#[inline]
pub(crate) fn camera_projection(projection: &gltf::camera::Projection) -> Projection {
	match projection {
		gltf::camera::Projection::Orthographic(orthographic) => {
			let orthographic_projection: OrthographicProjection = OrthographicProjection {
				far: orthographic.zfar(),
				near: orthographic.znear(),
				scaling_mode: ScalingMode::FixedHorizontal(1.0),
				scale: orthographic.xmag(),
				..Default::default()
			};

			Projection::Orthographic(orthographic_projection)
		},
		gltf::camera::Projection::Perspective(perspective) => {
			let perspective_projection: PerspectiveProjection = PerspectiveProjection {
				fov: perspective.yfov(),
				near: perspective.znear(),
				far: perspective.zfar().unwrap_or_default(),
				aspect_ratio: perspective.aspect_ratio().unwrap_or_default(),
			};
			Projection::Perspective(perspective_projection)
		},
	}
}
