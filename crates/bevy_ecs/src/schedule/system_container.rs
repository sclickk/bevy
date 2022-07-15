use crate::{
	component::ComponentId,
	query::Access,
	schedule::{
		AmbiguitySetLabelId, ExclusiveSystemDescriptor, GraphNode, ParallelSystemDescriptor,
		RunCriteriaLabelId, SystemLabelId,
	},
	system::{ExclusiveSystem, System},
};
use std::borrow::Cow;

// TODO: FIX CODE DUPLICATION IN HERE!
#[derive(Default)]
pub(crate) struct SystemContainerMeta {
	pub(crate) labels: Vec<SystemLabelId>,
	pub(crate) before: Vec<SystemLabelId>,
	pub(crate) after: Vec<SystemLabelId>,
	pub(crate) ambiguity_sets: Vec<AmbiguitySetLabelId>,
}

pub(crate) struct RunCriteriaMeta {
	pub(super) index: Option<usize>,
	pub(super) label: Option<RunCriteriaLabelId>,
}

impl RunCriteriaMeta {
	fn index(&self) -> Option<usize> {
		self.index
	}

	fn set_index(&mut self, index: usize) {
		self.index = Some(index);
	}

	fn label(&self) -> Option<&RunCriteriaLabelId> {
		self.label.as_ref()
	}
}

/// System metadata like its name, labels, order requirements and component access.
pub trait SystemContainer: GraphNode<Label = SystemLabelId> {
	#[doc(hidden)]
	fn dependencies(&self) -> &[usize];
	#[doc(hidden)]
	fn set_dependencies(&mut self, dependencies: impl IntoIterator<Item = usize>);
	#[doc(hidden)]
	fn run_criteria(&self) -> Option<usize>;
	#[doc(hidden)]
	fn set_run_criteria(&mut self, index: usize);
	fn run_criteria_label(&self) -> Option<&RunCriteriaLabelId>;
	fn ambiguity_sets(&self) -> &[AmbiguitySetLabelId];
	fn component_access(&self) -> Option<&Access<ComponentId>>;
}

pub(super) struct ExclusiveSystemContainer {
	system: Box<dyn ExclusiveSystem>,
	pub(super) run_criteria_meta: RunCriteriaMeta,
	dependencies: Vec<usize>,
	meta: SystemContainerMeta,
}

impl ExclusiveSystemContainer {
	pub(super) fn system_mut(&mut self) -> &mut Box<dyn ExclusiveSystem> {
		&mut self.system
	}
}

impl From<ExclusiveSystemDescriptor> for ExclusiveSystemContainer {
	fn from(descriptor: ExclusiveSystemDescriptor) -> Self {
		Self {
			system: descriptor.system,
			run_criteria_meta: RunCriteriaMeta {
				label: None,
				index: None,
			},
			dependencies: Vec::new(),
			meta: descriptor.meta,
		}
	}
}

impl GraphNode for ExclusiveSystemContainer {
	type Label = SystemLabelId;

	fn name(&self) -> Cow<'static, str> {
		self.system.name()
	}

	fn labels(&self) -> &[SystemLabelId] {
		&self.meta.labels
	}

	fn before(&self) -> &[SystemLabelId] {
		&self.meta.before
	}

	fn after(&self) -> &[SystemLabelId] {
		&self.meta.after
	}
}

impl SystemContainer for ExclusiveSystemContainer {
	fn dependencies(&self) -> &[usize] {
		&self.dependencies
	}

	fn set_dependencies(&mut self, dependencies: impl IntoIterator<Item = usize>) {
		self.dependencies.clear();
		self.dependencies.extend(dependencies);
	}

	fn run_criteria(&self) -> Option<usize> {
		self.run_criteria_meta.index()
	}

	fn set_run_criteria(&mut self, index: usize) {
		self.run_criteria_meta.set_index(index);
	}

	fn run_criteria_label(&self) -> Option<&RunCriteriaLabelId> {
		self.run_criteria_meta.label.as_ref()
	}

	fn ambiguity_sets(&self) -> &[AmbiguitySetLabelId] {
		&self.meta.ambiguity_sets
	}

	fn component_access(&self) -> Option<&Access<ComponentId>> {
		None
	}
}

pub struct ParallelSystemContainer {
	system: Box<dyn System<In = (), Out = ()>>,
	pub(crate) run_criteria_meta: RunCriteriaMeta,
	pub(crate) should_run: bool,
	dependencies: Vec<usize>,
	meta: SystemContainerMeta,
}

impl ParallelSystemContainer {
	pub fn name(&self) -> Cow<'static, str> {
		GraphNode::name(self)
	}

	pub fn system(&self) -> &dyn System<In = (), Out = ()> {
		&*self.system
	}

	pub fn system_mut(&mut self) -> &mut dyn System<In = (), Out = ()> {
		&mut *self.system
	}

	pub fn should_run(&self) -> bool {
		self.should_run
	}

	pub fn dependencies(&self) -> &[usize] {
		&self.dependencies
	}
}

impl From<ParallelSystemDescriptor> for ParallelSystemContainer {
	fn from(descriptor: ParallelSystemDescriptor) -> Self {
		Self {
			system: descriptor.system,
			should_run: false,
			run_criteria_meta: RunCriteriaMeta {
				index: None,
				label: None,
			},
			dependencies: Vec::new(),
			meta: descriptor.meta,
		}
	}
}

impl GraphNode for ParallelSystemContainer {
	type Label = SystemLabelId;

	fn name(&self) -> Cow<'static, str> {
		self.system().name()
	}

	fn labels(&self) -> &[SystemLabelId] {
		&self.meta.labels
	}

	fn before(&self) -> &[SystemLabelId] {
		&self.meta.before
	}

	fn after(&self) -> &[SystemLabelId] {
		&self.meta.after
	}
}

impl SystemContainer for ParallelSystemContainer {
	fn dependencies(&self) -> &[usize] {
		&self.dependencies
	}

	fn set_dependencies(&mut self, dependencies: impl IntoIterator<Item = usize>) {
		self.dependencies.clear();
		self.dependencies.extend(dependencies);
	}

	fn run_criteria(&self) -> Option<usize> {
		self.run_criteria_meta.index()
	}

	fn set_run_criteria(&mut self, index: usize) {
		self.run_criteria_meta.set_index(index);
	}

	fn run_criteria_label(&self) -> Option<&RunCriteriaLabelId> {
		self.run_criteria_meta.label.as_ref()
	}

	fn ambiguity_sets(&self) -> &[AmbiguitySetLabelId] {
		&self.meta.ambiguity_sets
	}

	fn component_access(&self) -> Option<&Access<ComponentId>> {
		Some(self.system().component_access())
	}
}
