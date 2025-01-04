use proc_macro::TokenStream as TS;
use proc_macro2::Span;
use quote::quote;
use syn::{
    FnArg, Ident, ItemTrait, Pat, PatIdent, ReturnType, TraitItem, parse_macro_input,
    punctuated::Punctuated, token::Comma,
};

pub fn get_export_functions(item: TS) -> TS {
    quote! {}.into()
}

pub fn get_host_calls(item: TS) -> TS {
    let tree = syn::parse::<ItemTrait>(item).unwrap();
    let mut exports = quote! {};
    let mut pubs = quote! {};
    for func in tree.items {
        let TraitItem::Fn(func) = func else {
            panic!("only functions are supported")
        };
        let sig = func.sig;
        let func_name = sig.ident.clone();
        let args = sig
            .inputs
            .iter()
            .map(|f| {
                let FnArg::Typed(pt) = f else {
                    panic!("self arguments are not allowed");
                };
                let Pat::Ident(id) = &*pt.pat else {
                    panic!("only idents are allowed")
                };
                id.ident.clone()
            })
            .collect::<Punctuated<_, Comma>>();
        exports = quote! {
            #exports
            pub unsafe #sig;
        };
        let panic_handler = if func_name != "debug_log" {
            quote! {
                inner::debug_log(format!("plugin crashed calling host function: {e}")).unwrap();
            }
        } else {
            quote! {}
        };
        pubs = quote! {
            #pubs
            pub #sig {
                unsafe {
                    match inner::#func_name(#args) {
                        Ok(o) => o,
                        Err(e) => {
                            #panic_handler
                            panic!()
                        },
                    }
                }
            }
        }
    }
    // let d = format!("{:?}", format!("{tree:#?}"));
    quote! {
        #[macro_export]
        macro_rules! host_calls {
            () => {
                mod host {
                    mod inner {
                        #[extism_pdk::host_fn]
                        unsafe extern "ExtismHost" {
                            #exports
                        }
                    }
                    #pubs
                }
            }
        }
    }
    .into()
}
