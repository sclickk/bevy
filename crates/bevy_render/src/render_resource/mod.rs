mod bind_group;
mod buffer;
mod pipeline;
mod shader;
mod texture;

pub use bind_group::*;
pub use buffer::*;
pub use pipeline::*;
pub use shader::*;
pub use texture::*;

// TODO: decide where re-exports should go
pub use wgpu::{
	util::BufferInitDescriptor, AdapterInfo as WgpuAdapterInfo, AddressMode, BindGroupDescriptor,
	BindGroupEntry, BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingResource, BindingType,
	BlendComponent, BlendFactor, BlendOperation, BlendState, BufferAddress, BufferBinding,
	BufferBindingType, BufferDescriptor, BufferSize, BufferUsages, ColorTargetState, ColorWrites,
	CommandEncoder, CommandEncoderDescriptor, CompareFunction, ComputePass, ComputePassDescriptor,
	ComputePipelineDescriptor as RawComputePipelineDescriptor, DepthBiasState, DepthStencilState,
	Extent3d, Face, Features as WgpuFeatures, FilterMode, FragmentState as RawFragmentState,
	FrontFace, ImageCopyBuffer, ImageCopyBufferBase, ImageCopyTexture, ImageCopyTextureBase,
	ImageDataLayout, ImageSubresourceRange, IndexFormat, Limits as WgpuLimits, LoadOp, MapMode,
	MultisampleState, Operations, Origin3d, PipelineLayout, PipelineLayoutDescriptor, PolygonMode,
	PrimitiveState, PrimitiveTopology, RenderPassColorAttachment, RenderPassDepthStencilAttachment,
	RenderPassDescriptor, RenderPipelineDescriptor as RawRenderPipelineDescriptor,
	SamplerBindingType, SamplerDescriptor, ShaderModule, ShaderModuleDescriptor, ShaderSource,
	ShaderStages, StencilFaceState, StencilOperation, StencilState, StorageTextureAccess,
	TextureAspect, TextureDescriptor, TextureDimension, TextureFormat, TextureSampleType,
	TextureUsages, TextureViewDescriptor, TextureViewDimension, VertexAttribute,
	VertexBufferLayout as RawVertexBufferLayout, VertexFormat, VertexState as RawVertexState,
	VertexStepMode,
};

pub mod encase {
	pub use bevy_encase_derive::ShaderType;
	pub use encase::*;
}

pub use self::encase::{ShaderSize, ShaderType};

pub use naga::ShaderStage;
