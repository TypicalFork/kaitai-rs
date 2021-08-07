#![feature(proc_macro_span, register_tool)]
#![register_tool(tarpaulin)]

mod keys;
mod utils;

use keys::*;
use utils::{get_attribute, Result};

use std::path::Path;

use syn::parse_macro_input;
use yaml_rust::Yaml;

// Since it gets re-exported in kaitai, crate-level refers to kaitai not kaitai-macros.
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
            panic!("attribute not on struct");
        }
    };

    match &struct_item.fields {
        syn::Fields::Unit => {}
        _ => {
            // TODO
            panic!("struct has fields");
        }
    }

    // // Span::call_site() is a nightly feature.
    let mut source_file_path = proc_macro::Span::call_site().source_file().path();
    source_file_path.pop();
    let file_path = source_file_path.join(Path::new(&ks_source_path.value()));

    let file_contents = std::fs::read_to_string(file_path).expect("error reading file: ");
    let structure =
        &yaml_rust::YamlLoader::load_from_str(&file_contents).expect("error parsing file: ")[0];

    let result = match structure {
        Yaml::Hash(hm) => types::create_type(
            hm,
            types::TypeOptions {
                ident: Some(struct_item.ident),
                attrs: struct_item.attrs,
                visibility: struct_item.vis,
            },
        ),
        _ => panic!("file does not have the correct structure"),
    };
    result.unwrap().into()
}
