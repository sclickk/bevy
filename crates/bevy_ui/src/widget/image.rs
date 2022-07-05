use crate::Size;
use crate::{CalculatedSize, UiImage};
use bevy_asset::Assets;
use bevy_ecs::{
	component::Component,
	query::With,
	reflect::ReflectComponent,
	system::{Query, Res},
};
use bevy_reflect::{Reflect, ReflectDeserialize, ReflectSerialize};
use bevy_render::texture::Image;
use serde::{Deserialize, Serialize};

/// Describes how to resize the Image node
#[derive(Component, Debug, Default, Clone, Reflect, Serialize, Deserialize)]
#[reflect_value(Component, Serialize, Deserialize)]
pub enum ImageMode {
	/// Keep the aspect ratio of the image
	#[default]
	KeepAspect,
}

/// Updates calculated size of the node based on the image provided
pub fn image_node_system(
	textures: Res<Assets<Image>>,
	mut query: Query<(&mut CalculatedSize, &UiImage), With<ImageMode>>,
) {
	query.for_each_mut(|(mut calculated_size, image)| {
		if let Some(texture) = textures.get(image) {
			let size = Size::from(texture.texture_descriptor.size);
			// Update only if size has changed to avoid needless layout calculations
			if size != calculated_size.size {
				calculated_size.size = size;
			}
		}
	});
}
