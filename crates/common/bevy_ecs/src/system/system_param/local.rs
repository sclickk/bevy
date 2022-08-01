use crate::{
	system::{
		ReadOnlySystemParamFetch, Resource, SystemMeta, SystemParam, SystemParamFetch, SystemParamState,
	},
	world::{FromWorld, World},
};

use std::{
	fmt::Debug,
	ops::{Deref, DerefMut},
};

/// A system local [`SystemParam`].
///
/// A local may only be accessed by the system itself and is therefore not visible to other systems.
/// If two or more systems specify the same local type each will have their own unique local.
///
/// # Examples
///
/// ```
/// # use bevy_ecs::prelude::*;
/// # let world = &mut World::default();
/// fn write_to_local(mut local: Local<usize>) {
///     *local = 42;
/// }
/// fn read_from_local(local: Local<usize>) -> usize {
///     *local
/// }
/// let mut write_system = IntoSystem::into_system(write_to_local);
/// let mut read_system = IntoSystem::into_system(read_from_local);
/// write_system.initialize(world);
/// read_system.initialize(world);
///
/// assert_eq!(read_system.run((), world), 0);
/// write_system.run((), world);
/// // Note how the read local is still 0 due to the locals not being shared.
/// assert_eq!(read_system.run((), world), 0);
/// ```
///
/// N.B. A [`Local`]s value cannot be read or written to outside of the containing system.
/// To add configuration to a system, convert a capturing closure into the system instead:
///
/// ```
/// # use bevy_ecs::prelude::*;
/// # use bevy_ecs::system::assert_is_system;
/// struct Config(u32);
/// struct Myu32Wrapper(u32);
/// fn reset_to_system(value: Config) -> impl FnMut(ResMut<Myu32Wrapper>) {
///     move |mut val| val.0 = value.0
/// }
///
/// // .add_system(reset_to_system(my_config))
/// # assert_is_system(reset_to_system(Config(10)));
/// ```
pub struct Local<'a, T: Resource + FromWorld>(&'a mut T);

impl<'a, T: Resource + FromWorld> Debug for Local<'a, T>
where
	T: Debug,
{
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.debug_tuple("Local").field(&self.0).finish()
	}
}

impl<'a, T: Resource + FromWorld> Deref for Local<'a, T> {
	type Target = T;

	#[inline]
	fn deref(&self) -> &Self::Target {
		self.0
	}
}

impl<'a, T: Resource + FromWorld> DerefMut for Local<'a, T> {
	#[inline]
	fn deref_mut(&mut self) -> &mut Self::Target {
		self.0
	}
}

impl<'a, T: Resource + FromWorld> SystemParam for Local<'a, T> {
	type Fetch = LocalState<T>;
}

/// The [`SystemParamState`] of [`Local<T>`].
#[doc(hidden)]
pub struct LocalState<T: Resource>(T);

// SAFETY: Local only accesses internal state
unsafe impl<T: Resource> ReadOnlySystemParamFetch for LocalState<T> {}

// SAFETY: only local state is accessed
unsafe impl<T: Resource + FromWorld> SystemParamState for LocalState<T> {
	fn init(world: &mut World, _system_meta: &mut SystemMeta) -> Self {
		Self(T::from_world(world))
	}
}

impl<'w, 's, T: Resource + FromWorld> SystemParamFetch<'w, 's> for LocalState<T> {
	type Item = Local<'s, T>;

	#[inline]
	unsafe fn get_param(
		state: &'s mut Self,
		_system_meta: &SystemMeta,
		_world: &'w World,
		_change_tick: u32,
	) -> Self::Item {
		Local(&mut state.0)
	}
}
