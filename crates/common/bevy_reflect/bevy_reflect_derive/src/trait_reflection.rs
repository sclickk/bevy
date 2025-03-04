use bevy_manifest::BevyManifest;
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse::Parse, parse_macro_input, Attribute, ItemTrait, Token};

pub(crate) struct TraitInfo {
	item_trait: ItemTrait,
}

impl Parse for TraitInfo {
	fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
		let attrs = input.call(Attribute::parse_outer)?;
		let lookahead = input.lookahead1();
		if lookahead.peek(Token![pub]) || lookahead.peek(Token![trait]) {
			let mut item_trait: ItemTrait = input.parse()?;
			item_trait.attrs = attrs;
			Ok(TraitInfo { item_trait })
		} else {
			Err(lookahead.error())
		}
	}
}

/// A trait attribute macro that allows a reflected type to be downcast to a trait object.
///
/// This generates a struct that takes the form `ReflectMyTrait`. An instance of this struct can then be
/// used to perform the conversion.
pub(crate) fn reflect_trait(_args: &TokenStream, input: TokenStream) -> TokenStream {
	let trait_info = parse_macro_input!(input as TraitInfo);
	let item_trait = &trait_info.item_trait;
	let trait_ident = &item_trait.ident;
	let trait_vis = &item_trait.vis;
	let reflect_trait_ident = crate::utility::get_reflect_ident(&item_trait.ident.to_string());
	let bevy_reflect_path = BevyManifest::default().get_path("bevy_reflect");

	let struct_doc = format!(
        " A type generated by the #[reflect_trait] macro for the `{}` trait.\n\n This allows casting from `dyn Reflect` to `dyn {}`.",
        trait_ident,
        trait_ident
    );
	let get_doc = format!(
        " Downcast a `&dyn Reflect` type to `&dyn {}`.\n\n If the type cannot be downcast, `None` is returned.",
        trait_ident,
    );
	let get_mut_doc = format!(
        " Downcast a `&mut dyn Reflect` type to `&mut dyn {}`.\n\n If the type cannot be downcast, `None` is returned.",
        trait_ident,
    );
	let get_box_doc = format!(
        " Downcast a `Box<dyn Reflect>` type to `Box<dyn {}>`.\n\n If the type cannot be downcast, this will return `Err(Box<dyn Reflect>)`.",
        trait_ident,
    );

	TokenStream::from(quote! {
		#item_trait

		#[doc = #struct_doc]
		#[derive(Clone)]
		#trait_vis struct #reflect_trait_ident {
			get_func: fn(&dyn #bevy_reflect_path::Reflect) -> Option<&dyn #trait_ident>,
			get_mut_func: fn(&mut dyn #bevy_reflect_path::Reflect) -> Option<&mut dyn #trait_ident>,
			get_boxed_func: fn(Box<dyn #bevy_reflect_path::Reflect>) -> Result<Box<dyn #trait_ident>, Box<dyn #bevy_reflect_path::Reflect>>,
		}

		impl #reflect_trait_ident {
			#[doc = #get_doc]
			pub fn get<'a>(&self, reflect_value: &'a dyn #bevy_reflect_path::Reflect) -> Option<&'a dyn #trait_ident> {
				(self.get_func)(reflect_value)
			}

			#[doc = #get_mut_doc]
			pub fn get_mut<'a>(&self, reflect_value: &'a mut dyn #bevy_reflect_path::Reflect) -> Option<&'a mut dyn #trait_ident> {
				(self.get_mut_func)(reflect_value)
			}

			#[doc = #get_box_doc]
			pub fn get_boxed(&self, reflect_value: Box<dyn #bevy_reflect_path::Reflect>) -> Result<Box<dyn #trait_ident>, Box<dyn #bevy_reflect_path::Reflect>> {
				(self.get_boxed_func)(reflect_value)
			}
		}

		impl<T: #trait_ident + #bevy_reflect_path::Reflect> #bevy_reflect_path::FromType<T> for #reflect_trait_ident {
			fn from_type() -> Self {
				Self {
					get_func: |reflect_value| {
						reflect_value.downcast_ref::<T>().map(|value| value as &dyn #trait_ident)
					},
					get_mut_func: |reflect_value| {
						reflect_value.downcast_mut::<T>().map(|value| value as &mut dyn #trait_ident)
					},
					get_boxed_func: |reflect_value| {
						reflect_value.downcast::<T>().map(|value| value as Box<dyn #trait_ident>)
					}
				}
			}
		}
	})
}
