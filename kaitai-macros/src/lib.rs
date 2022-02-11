//! Please see the main [kaitai](https://www.crates.io/crates/kaitai) crate.
#![feature(proc_macro_span, register_tool)]
#![allow(dead_code)]
#![deny(
    non_ascii_idents,
    missing_docs,
    rust_2018_idioms,
    rust_2021_compatibility,
    future_incompatible,
    missing_debug_implementations,
    missing_copy_implementations,
    rustdoc::broken_intra_doc_links
)]
#![register_tool(tarpaulin)]

mod de;

mod error;
mod keys;
mod util;

use keys::*;

use std::path::Path;

use syn::parse_macro_input;
use yaml_rust::Yaml;

// Since this macro gets re-exported in kaitai, crate-level refers to kaitai not kaitai-macros.
// TODO is there a way to link "crate-level documentation" to the main kaitai crate?
/// See crate-level documentation for information on how to use this macro.
#[tarpaulin::skip]
#[proc_macro_attribute]
pub fn kaitai_source(
    args: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let ks_source_path = parse_macro_input!(args as syn::LitStr);
    let item_ast = parse_macro_input!(item as syn::Item);

    let struct_item = match item_ast {
        syn::Item::Struct(s) => s,
        _ => {
            // TODO
            panic!("kaitai_source macro can only be applied to a struct");
        }
    };

    match &struct_item.fields {
        syn::Fields::Unit => {}
        _ => {
            // TODO
            panic!("kaitai_source macro can only be applied to a fieldless struct");
        }
    }

    // // Span::call_site() is a nightly feature.
    let mut source_file_path = proc_macro::Span::call_site().source_file().path();
    source_file_path.pop();
    let file_path = source_file_path.join(Path::new(&ks_source_path.value()));

    let file_contents = std::fs::read_to_string(file_path).expect("error reading ksy file: ");
    // TODO do we need to check that length == 1?
    let structure =
        &yaml_rust::YamlLoader::load_from_str(&file_contents).expect("error parsing ksy file: ")[0];

    let result = match structure {
        Yaml::Hash(map) => types::ty(types::TypeData {
            map,
            ident: struct_item.ident,
            attrs: struct_item.attrs,
            visibility: struct_item.vis,
            inherited_meta: None,
        }),
        _ => panic!("ksy file does not have the correct structure"),
    };

    match result {
        Ok(t) => quote::quote! { #t }.into(),
        Err(e) => panic!("{:?}", e),
    }
}
