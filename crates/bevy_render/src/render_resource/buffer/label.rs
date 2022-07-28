pub struct BufferLabel {
	pub(crate) label: Option<String>,
	pub(crate) changed: bool,
}

impl Default for BufferLabel {
	fn default() -> Self {
		Self {
			label: None,
			changed: false,
		}
	}
}

impl BufferLabel {
	pub fn set(&mut self, label: Option<&str>) {
		let label = label.map(str::to_string);

		if label != self.label {
			self.changed = true;
		}

		self.label = label;
	}

	pub fn get(&self) -> Option<&str> {
		self.label.as_deref()
	}
}
