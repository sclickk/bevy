use crate::{
	schedule::{
		AmbiguitySetLabel, IntoRunCriteria, IntoSystemDescriptor, RunCriteriaDescriptorOrLabel, State,
		StateData, SystemContainerMeta, SystemDescriptor, SystemLabel,
	},
	system::AsSystemLabel,
};

/// A builder for describing several systems at the same time.
#[derive(Default)]
pub struct SystemSet {
	pub(crate) systems: Vec<SystemDescriptor>,
	pub(crate) run_criteria: Option<RunCriteriaDescriptorOrLabel>,
	pub(crate) meta: SystemContainerMeta,
}

impl SystemSet {
	pub fn new() -> Self {
		Default::default()
	}

	pub fn on_update<T>(s: T) -> SystemSet
	where
		T: StateData,
	{
		Self::new().with_run_criteria(State::<T>::on_update(s))
	}

	pub fn on_inactive_update<T>(s: T) -> SystemSet
	where
		T: StateData,
	{
		Self::new().with_run_criteria(State::<T>::on_inactive_update(s))
	}

	pub fn on_in_stack_update<T>(s: T) -> SystemSet
	where
		T: StateData,
	{
		Self::new().with_run_criteria(State::<T>::on_in_stack_update(s))
	}

	pub fn on_enter<T>(s: T) -> SystemSet
	where
		T: StateData,
	{
		Self::new().with_run_criteria(State::<T>::on_enter(s))
	}

	pub fn on_exit<T>(s: T) -> SystemSet
	where
		T: StateData,
	{
		Self::new().with_run_criteria(State::<T>::on_exit(s))
	}

	pub fn on_pause<T>(s: T) -> SystemSet
	where
		T: StateData,
	{
		Self::new().with_run_criteria(State::<T>::on_pause(s))
	}

	pub fn on_resume<T>(s: T) -> SystemSet
	where
		T: StateData,
	{
		Self::new().with_run_criteria(State::<T>::on_resume(s))
	}

	/// Short for `SystemSet::on_update(state).with_system(system)`
	pub fn when_update<T, Params>(state: T, system: impl IntoSystemDescriptor<Params>) -> SystemSet
	where
		T: StateData,
	{
		Self::on_update(state).with_system(system)
	}

	/// Short for `SystemSet::on_inactive_update(state).with_system(system)`
	pub fn when_inactive_update<T, Params>(
		state: T,
		system: impl IntoSystemDescriptor<Params>,
	) -> SystemSet
	where
		T: StateData,
	{
		Self::on_inactive_update(state).with_system(system)
	}

	/// Short for `SystemSet::on_in_stack_update(state).with_system(system)`
	pub fn when_in_stack_update<T, Params>(
		state: T,
		system: impl IntoSystemDescriptor<Params>,
	) -> SystemSet
	where
		T: StateData,
	{
		Self::on_in_stack_update(state).with_system(system)
	}

	/// Short for `SystemSet::on_enter(state).with_system(system)`
	pub fn when_enter<T, Params>(state: T, system: impl IntoSystemDescriptor<Params>) -> SystemSet
	where
		T: StateData,
	{
		Self::on_enter(state).with_system(system)
	}

	/// Short for `SystemSet::on_exit(state).with_system(system)`
	pub fn when_exit<T, Params>(state: T, system: impl IntoSystemDescriptor<Params>) -> SystemSet
	where
		T: StateData,
	{
		Self::on_exit(state).with_system(system)
	}

	/// Short for `SystemSet::on_pause(state).with_system(system)`
	pub fn when_pause<T, Params>(state: T, system: impl IntoSystemDescriptor<Params>) -> SystemSet
	where
		T: StateData,
	{
		Self::on_pause(state).with_system(system)
	}

	/// Short for `SystemSet::on_resume(state).with_system(system)`
	pub fn when_resume<T, Params>(state: T, system: impl IntoSystemDescriptor<Params>) -> SystemSet
	where
		T: StateData,
	{
		Self::on_resume(state).with_system(system)
	}

	#[must_use]
	pub fn in_ambiguity_set(mut self, set: impl AmbiguitySetLabel) -> Self {
		self.meta.ambiguity_sets.push(set.as_label());
		self
	}

	#[must_use]
	pub fn with_system<Params>(mut self, system: impl IntoSystemDescriptor<Params>) -> Self {
		self.systems.push(system.into_descriptor());
		self
	}

	#[must_use]
	pub fn with_run_criteria<Marker>(mut self, run_criteria: impl IntoRunCriteria<Marker>) -> Self {
		self.run_criteria = Some(run_criteria.into());
		self
	}

	#[must_use]
	pub fn label(mut self, label: impl SystemLabel) -> Self {
		self.meta.labels.push(label.as_label());
		self
	}

	#[must_use]
	pub fn before<Marker>(mut self, label: impl AsSystemLabel<Marker>) -> Self {
		self
			.meta
			.before
			.push(label.as_system_label().as_label());
		self
	}

	#[must_use]
	pub fn after<Marker>(mut self, label: impl AsSystemLabel<Marker>) -> Self {
		self
			.meta
			.after
			.push(label.as_system_label().as_label());
		self
	}

	pub(crate) fn bake(self) -> (Option<RunCriteriaDescriptorOrLabel>, Vec<SystemDescriptor>) {
		let SystemSet {
			mut systems,
			run_criteria,
			meta,
		} = self;
		for descriptor in &mut systems {
			match descriptor {
				SystemDescriptor::Parallel(descriptor) => {
					descriptor
						.meta
						.labels
						.extend(meta.labels.iter().cloned());
					descriptor
						.meta
						.before
						.extend(meta.before.iter().cloned());
					descriptor
						.meta
						.after
						.extend(meta.after.iter().cloned());
					descriptor
						.meta
						.ambiguity_sets
						.extend(meta.ambiguity_sets.iter().cloned());
				},
				SystemDescriptor::Exclusive(descriptor) => {
					descriptor
						.meta
						.labels
						.extend(meta.labels.iter().cloned());
					descriptor
						.meta
						.before
						.extend(meta.before.iter().cloned());
					descriptor
						.meta
						.after
						.extend(meta.after.iter().cloned());
					descriptor
						.meta
						.ambiguity_sets
						.extend(meta.ambiguity_sets.iter().cloned());
				},
			}
		}
		(run_criteria, systems)
	}
}
