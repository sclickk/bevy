//! Traits used by label implementations

use std::{
	any::Any,
	hash::{Hash, Hasher},
};

pub trait DynEq: Any {
	fn as_any(&self) -> &dyn Any;

	fn dyn_eq(&self, other: &dyn DynEq) -> bool;
}

impl<T> DynEq for T
where
	T: Any + Eq,
{
	fn as_any(&self) -> &dyn Any {
		self
	}

	fn dyn_eq(&self, other: &dyn DynEq) -> bool {
		if let Some(other) = other.as_any().downcast_ref::<T>() {
			return self == other;
		}
		false
	}
}

pub trait DynHash: DynEq {
	fn as_dyn_eq(&self) -> &dyn DynEq;

	fn dyn_hash(&self, state: &mut dyn Hasher);
}

impl<T> DynHash for T
where
	T: DynEq + Hash,
{
	fn as_dyn_eq(&self) -> &dyn DynEq {
		self
	}

	fn dyn_hash(&self, mut state: &mut dyn Hasher) {
		T::hash(self, &mut state);
		self.type_id().hash(&mut state);
	}
}
