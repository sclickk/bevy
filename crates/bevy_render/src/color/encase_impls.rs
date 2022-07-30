use crate::Color;

impl encase::ShaderType for Color {
	type ExtraMetadata = ();

	const METADATA: encase::private::Metadata<Self::ExtraMetadata> = {
		let size =
			encase::private::SizeValue::from(<f32 as encase::private::ShaderSize>::SHADER_SIZE).mul(4);
		let alignment = encase::private::AlignmentValue::from_next_power_of_two_size(size);

		encase::private::Metadata {
			alignment,
			has_uniform_min_alignment: false,
			min_size: size,
			extra: (),
		}
	};

	const UNIFORM_COMPAT_ASSERT: fn() = || {};
}

impl encase::private::WriteInto for Color {
	fn write_into<B: encase::private::BufferMut>(&self, writer: &mut encase::private::Writer<B>) {
		let linear = self.as_linear_rgba_f32();
		for el in &linear {
			encase::private::WriteInto::write_into(el, writer);
		}
	}
}

impl encase::private::ReadFrom for Color {
	fn read_from<B: encase::private::BufferRef>(&mut self, reader: &mut encase::private::Reader<B>) {
		let mut buffer = [0.0f32; 4];
		for el in &mut buffer {
			encase::private::ReadFrom::read_from(el, reader);
		}

		*self = Color::RgbaLinear {
			red: buffer[0],
			green: buffer[1],
			blue: buffer[2],
			alpha: buffer[3],
		}
	}
}
impl encase::private::CreateFrom for Color {
	fn create_from<B>(reader: &mut encase::private::Reader<B>) -> Self
	where
		B: encase::private::BufferRef,
	{
		// These are intentionally not inlined in the constructor to make this
		// resilient to internal Color refactors / implicit type changes.
		let red: f32 = encase::private::CreateFrom::create_from(reader);
		let green: f32 = encase::private::CreateFrom::create_from(reader);
		let blue: f32 = encase::private::CreateFrom::create_from(reader);
		let alpha: f32 = encase::private::CreateFrom::create_from(reader);
		Color::RgbaLinear {
			red,
			green,
			blue,
			alpha,
		}
	}
}

impl encase::ShaderSize for Color {}
