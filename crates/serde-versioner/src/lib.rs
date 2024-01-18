use proc_macro::{Ident, Span, TokenStream};
use proc_macro_error::{abort_call_site, proc_macro_error};
use quote::{format_ident, quote};
use syn::{
    self, parse_macro_input, punctuated::Punctuated, token::Comma, Item, Meta, Path, PathSegment,
};

#[proc_macro_attribute]
#[proc_macro_error]
pub fn version(args: TokenStream, input: TokenStream) -> TokenStream {
    let original = input.clone();

    let args = parse_macro_input!(args with Punctuated::<Meta, syn::Token![,]>::parse_terminated);
    eprintln!("ARGS: {:#?}", args);

    let version_content_paths = get_version_content_paths(&args);

    // Construct a representation of Rust code as a syntax tree
    // that we can manipulate
    let ast = parse_macro_input!(input as syn::Item);
    eprintln!("TOKENS: {:#?}", original);
    eprintln!("INPUT: {:#?}", ast);

    if let Item::Struct(struct_ast) = ast {
        let struct_name = struct_ast.ident.to_string();
        let versioned_name = format_ident!("{struct_name}Versioned");
        // let enum_versions = Vec::new();
        // for path in version_content_paths {
        //     enum_versions.push(quote! {
        //         V1
        //     })
        // }

        let versioned_struct = quote! {
            enum #versioned_name {
                V1
            }
        };

        let quoted_versioned_name = versioned_name.to_string();
        let serde_attr = quote! {
            #[serde(into = #quoted_versioned_name, from = #quoted_versioned_name)]
        };

        let mut out = TokenStream::from(serde_attr);
        out.extend(original);
        out.extend(TokenStream::from(versioned_struct));

        eprintln!("RESULT: {}", out);
        out
    } else {
        abort_call_site!("Only works on structs");
    }

    // Build the trait implementation
    // impl_hello_macro(&ast)
}

fn get_version_content_paths(args: &Punctuated<Meta, Comma>) -> Vec<Path> {
    let mut paths = Vec::new();
    for meta in args.iter() {
        if let Meta::Path(path) = meta {
            paths.push(path.clone());
        } else {
            abort_call_site!("Invalid argument type");
        }
    }

    paths
}
