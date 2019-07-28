extern crate proc_macro;
use quote::quote;

#[proc_macro]
pub fn declare_functions(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
	let parsed_input: syn::ItemForeignMod = syn::parse(input).unwrap();

	let mut static_declarations = parsed_input.clone();
	for item in static_declarations.items.iter_mut() {
		if let syn::ForeignItem::Fn(function) = item {
			// discard library() attribute
			function.attrs.retain(|attr| {
				if let Ok(syn::Meta::List(meta)) = attr.parse_meta() {
					meta.ident != "library"
				} else {
					true
				}
			});
		}
	}

	let mut dynamic_declarations = proc_macro2::TokenStream::new();
	let abi = parsed_input.abi;
	for item in parsed_input.items {
		if let syn::ForeignItem::Fn(function) = item {
			// find the value of the library() attribute
			let library = function
				.attrs
				.iter()
				.filter_map(|attr| {
					if let Ok(syn::Meta::List(meta)) = attr.parse_meta() {
						if meta.ident == "library" {
							let nested = meta.nested;
							assert_eq!(nested.len(), 1usize);
							let pair = *nested.first().unwrap().value();
							if let syn::NestedMeta::Meta(meta) = pair {
								if let syn::Meta::Word(word) = meta {
									Some(word.clone())
								} else {
									panic!("expected an identifier in library attribute");
								}
							} else {
								panic!("expected an identifier in library attribute");
							}
						} else {
							None
						}
					} else {
						None
					}
				})
				.next()
				.unwrap();

			// extract call signature
			let ident = function.ident;
			let vis = function.vis;
			let decl = &*function.decl;
			let inputs = &decl.inputs;
			let output = &decl.output;

			// convert identifier to byte literal
			let ident_bytes = syn::LitByteStr::new(ident.to_string().as_bytes(), ident.span());

			// add a new static ref to the lazy_static instance below
			dynamic_declarations.extend(quote!(
				#vis static ref #ident: Option<libloading::Symbol<'static, unsafe #abi fn (#inputs) #output>> = unsafe {
					#library.as_ref().and_then(|lib| lib.get(#ident_bytes).ok())
				};
			));
		}
	}

	quote!(
		#static_declarations
		mod dynamic {
			#![allow(non_upper_case_globals)]
			use super::*;
			lazy_static::lazy_static! {
				static ref USER32: Option<libloading::Library> = { libloading::Library::new("user32.dll").ok() };
				#dynamic_declarations
			}
		}
	)
	.into()
}
