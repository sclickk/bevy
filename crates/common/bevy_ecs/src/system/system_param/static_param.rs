use crate::{
	archetype::Archetype,
	system::{
		ReadOnlySystemParamFetch, SystemMeta, SystemParam, SystemParamFetch, SystemParamItem,
		SystemParamState,
	},
	world::World,
};

use std::{
	marker::PhantomData,
	ops::{Deref, DerefMut},
};

/// A helper for using system parameters in generic contexts
///
/// This type is a [`SystemParam`] adapter which always has
/// `Self::Fetch::Item == Self` (ignoring lifetimes for brevity),
/// no matter the argument [`SystemParam`] (`P`) (other than
/// that `P` must be `'static`)
///
/// This makes it useful for having arbitrary [`SystemParam`] type arguments
/// to function systems, or for generic types using the [`derive@SystemParam`]
/// derive:
///
/// ```
/// # use bevy_ecs::prelude::*;
/// use bevy_ecs::system::{SystemParam, StaticSystemParam};
/// #[derive(SystemParam)]
/// struct GenericParam<'w,'s, T: SystemParam + 'static> {
///     field: StaticSystemParam<'w, 's, T>,
/// }
/// fn do_thing_generically<T: SystemParam + 'static>(t: StaticSystemParam<T>) {}
///
/// fn check_always_is_system<T: SystemParam + 'static>(){
///     bevy_ecs::system::assert_is_system(do_thing_generically::<T>);
/// }
/// ```
/// Note that in a real case you'd generally want
/// additional bounds on `P`, for your use of the parameter
/// to have a reason to be generic.
///
/// For example, using this would allow a type to be generic over
/// whether a resource is accessed mutably or not, with
/// impls being bounded on [`P: Deref<Target=MyType>`](Deref), and
/// [`P: DerefMut<Target=MyType>`](DerefMut) depending on whether the
/// method requires mutable access or not.
///
/// The method which doesn't use this type will not compile:
/// ```compile_fail
/// # use bevy_ecs::prelude::*;
/// # use bevy_ecs::system::{SystemParam, StaticSystemParam};
///
/// fn do_thing_generically<T: SystemParam + 'static>(t: T) {}
///
/// #[derive(SystemParam)]
/// struct GenericParam<'w,'s, T: SystemParam> {
///     field: T,
///     #[system_param(ignore)]
///     // Use the lifetimes, as the `SystemParam` derive requires them
///     phantom: core::marker::PhantomData<&'w &'s ()>
/// }
/// # fn check_always_is_system<T: SystemParam + 'static>(){
/// #    bevy_ecs::system::assert_is_system(do_thing_generically::<T>);
/// # }
/// ```
///
pub struct StaticSystemParam<'w, 's, P: SystemParam>(SystemParamItem<'w, 's, P>);

impl<'w, 's, P: SystemParam> Deref for StaticSystemParam<'w, 's, P> {
	type Target = SystemParamItem<'w, 's, P>;

	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

impl<'w, 's, P: SystemParam> DerefMut for StaticSystemParam<'w, 's, P> {
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.0
	}
}

impl<'w, 's, P: SystemParam> StaticSystemParam<'w, 's, P> {
	/// Get the value of the parameter
	pub fn into_inner(self) -> SystemParamItem<'w, 's, P> {
		self.0
	}
}

/// The [`SystemParamState`] of [`StaticSystemParam`].
#[doc(hidden)]
pub struct StaticSystemParamState<S, P>(S, PhantomData<fn() -> P>);

// SAFETY: This doesn't add any more reads, and the delegated fetch confirms it
unsafe impl<S: ReadOnlySystemParamFetch, P> ReadOnlySystemParamFetch
	for StaticSystemParamState<S, P>
{
}

impl<'world, 'state, P: SystemParam + 'static> SystemParam
	for StaticSystemParam<'world, 'state, P>
{
	type Fetch = StaticSystemParamState<P::Fetch, P>;
}

impl<'world, 'state, S: SystemParamFetch<'world, 'state>, P: SystemParam + 'static>
	SystemParamFetch<'world, 'state> for StaticSystemParamState<S, P>
where
	P: SystemParam<Fetch = S>,
{
	type Item = StaticSystemParam<'world, 'state, P>;

	unsafe fn get_param(
		state: &'state mut Self,
		system_meta: &SystemMeta,
		world: &'world World,
		change_tick: u32,
	) -> Self::Item {
		// SAFETY: We properly delegate SystemParamState
		StaticSystemParam(S::get_param(&mut state.0, system_meta, world, change_tick))
	}
}

// SAFETY: all methods are just delegated to `S`'s `SystemParamState` implementation
unsafe impl<S: SystemParamState, P: SystemParam + 'static> SystemParamState
	for StaticSystemParamState<S, P>
{
	fn init(world: &mut World, system_meta: &mut SystemMeta) -> Self {
		Self(S::init(world, system_meta), PhantomData)
	}

	fn new_archetype(&mut self, archetype: &Archetype, system_meta: &mut SystemMeta) {
		self.0.new_archetype(archetype, system_meta);
	}

	fn apply(&mut self, world: &mut World) {
		self.0.apply(world);
	}
}
