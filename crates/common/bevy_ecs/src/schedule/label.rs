pub use bevy_ecs_macros::{AmbiguitySetLabel, RunCriteriaLabel, StageLabel, SystemLabel};

/// Macro to define a new label trait
///
/// # Example
///
/// ```
/// # use bevy_ecs::define_label;
/// define_label!(
///     /// A class of labels.
///     MyNewLabelTrait,
///     /// Identifies a value that implements `MyNewLabelTrait`.
///     MyNewLabelId,
/// );
/// ```
#[macro_export]
macro_rules! define_label {
	(
		$(#[$label_attr:meta])*
		$label_name:ident,

		$(#[$id_attr:meta])*
		$id_name:ident $(,)?
	) => {
		$(#[$id_attr])*
		#[derive(Clone, Copy, PartialEq, Eq, Hash)]
		pub struct $id_name(::core::any::TypeId, &'static str);

		impl ::core::fmt::Debug for $id_name {
			fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
				write!(f, "{}", self.1)
			}
		}

		$(#[$label_attr])*
		pub trait $label_name: 'static {
			/// Converts this type into an opaque, strongly-typed label.
			fn as_label(&self) -> $id_name {
				let id = self.type_id();
				let label = self.as_str();
				$id_name(id, label)
			}
			/// Returns the [`TypeId`] used to differentiate labels.
			fn type_id(&self) -> ::core::any::TypeId {
				::core::any::TypeId::of::<Self>()
			}
			/// Returns the representation of this label as a string literal.
			///
			/// In cases where you absolutely need a label to be determined at runtime,
			/// you can use [`Box::leak`] to get a `'static` reference.
			fn as_str(&self) -> &'static str;
		}

		impl $label_name for $id_name {
			fn as_label(&self) -> Self {
				*self
			}
			fn type_id(&self) -> ::core::any::TypeId {
				self.0
			}
			fn as_str(&self) -> &'static str {
				self.1
			}
		}

		impl $label_name for &'static str {
			fn as_str(&self) -> Self {
				self
			}
		}
	};
}

define_label!(
	/// A strongly-typed class of labels used to identify [`Stage`](crate::schedule::Stage)s.
	StageLabel,
	/// Strongly-typed identifier for a [`StageLabel`].
	StageLabelId,
);
define_label!(
	/// A strongly-typed class of labels used to identify [`System`](crate::system::System)s.
	SystemLabel,
	/// Strongly-typed identifier for a [`SystemLabel`].
	SystemLabelId,
);
define_label!(
	/// A strongly-typed class of labels used to identify sets of systems with intentionally ambiguous execution order.
	AmbiguitySetLabel,
	/// Strongly-typed identifier for an [`AmbiguitySetLabel`].
	AmbiguitySetLabelId,
);
define_label!(
	/// A strongly-typed class of labels used to identify [run criteria](crate::schedule::RunCriteria).
	RunCriteriaLabel,
	/// Strongly-typed identifier for a [`RunCriteriaLabel`].
	RunCriteriaLabelId,
);
