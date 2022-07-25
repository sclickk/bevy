use crate::render_resource::IndexFormat;
use std::iter::FusedIterator;

/// An array of indices into the [`VertexAttributeValues`] for a mesh.
///
/// It describes the order in which the vertex attributes should be joined into faces.
#[derive(Debug, Clone)]
pub enum Indices {
	U16(Vec<u16>),
	U32(Vec<u32>),
}

impl Indices {
	/// Returns an iterator over the indices.
	pub fn iter(&self) -> impl Iterator<Item = usize> + '_ {
		match self {
			Indices::U16(vec) => IndicesIter::U16(vec.into_iter()),
			Indices::U32(vec) => IndicesIter::U32(vec.into_iter()),
		}
	}

	/// Returns the number of indices.
	pub fn len(&self) -> usize {
		match self {
			Indices::U16(vec) => vec.len(),
			Indices::U32(vec) => vec.len(),
		}
	}

	/// Returns `true` if there are no indices.
	pub fn is_empty(&self) -> bool {
		match self {
			Indices::U16(vec) => vec.is_empty(),
			Indices::U32(vec) => vec.is_empty(),
		}
	}
}

/// An Iterator for the [`Indices`].
enum IndicesIter<'a> {
	U16(std::slice::Iter<'a, u16>),
	U32(std::slice::Iter<'a, u32>),
}

impl Iterator for IndicesIter<'_> {
	type Item = usize;

	fn next(&mut self) -> Option<Self::Item> {
		match self {
			IndicesIter::U16(iter) => iter.next().map(|val| *val as usize),
			IndicesIter::U32(iter) => iter.next().map(|val| *val as usize),
		}
	}

	fn size_hint(&self) -> (usize, Option<usize>) {
		match self {
			IndicesIter::U16(iter) => iter.size_hint(),
			IndicesIter::U32(iter) => iter.size_hint(),
		}
	}
}

impl<'a> ExactSizeIterator for IndicesIter<'a> {}
impl<'a> FusedIterator for IndicesIter<'a> {}

impl From<&Indices> for IndexFormat {
	fn from(indices: &Indices) -> Self {
		match indices {
			Indices::U16(_) => IndexFormat::Uint16,
			Indices::U32(_) => IndexFormat::Uint32,
		}
	}
}
