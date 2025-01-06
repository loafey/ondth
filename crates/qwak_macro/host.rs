use proc_macro::TokenStream as TS;
use quote::quote;
use syn::{FnArg, ItemTrait, Pat, TraitItem, Type, punctuated::Punctuated, token::Comma};

fn stringify(ty: &Type) -> String {
    match ty {
        Type::Path(type_path) => type_path
            .path
            .segments
            .iter()
            .map(|ps| ps.ident.to_string())
            .collect::<Vec<_>>()
            .join("::"),
        Type::Array(_) => todo!("type_array"),
        Type::BareFn(_) => todo!("type_bare_fn"),
        Type::Group(_) => todo!("type_group"),
        Type::ImplTrait(_) => todo!("type_impl_trait"),
        Type::Infer(_) => todo!("type_infer"),
        Type::Macro(_) => todo!("type_macro"),
        Type::Never(_) => todo!("type_never"),
        Type::Paren(_) => todo!("type_paren"),
        Type::Ptr(_) => todo!(")"),
        Type::Reference(_) => todo!("type_reference"),
        Type::Slice(_) => todo!("type_slice"),
        Type::TraitObject(_) => todo!("type_trait_object"),
        Type::Tuple(_) => todo!("type_tuple"),
        Type::Verbatim(_) => todo!("token_stream"),
        _ => panic!(),
    }
}
fn get_type(ty: &Type) -> Type {
    let ty = stringify(ty);
    let id = match &*ty {
        "String" => quote!(qwak::PTR),
        "u32" => quote!(qwak::ValType::I64),
        "u64" => quote!(qwak::ValType::I64),
        "f32" => quote!(qwak::ValType::I64), // this is most likely a bug in extism?
        _ => panic!("undefined type conversion for \"{ty}\""),
    };
    syn::parse(id.into()).unwrap()
}

pub fn get_export_functions(item: TS) -> TS {
    let tree = syn::parse::<ItemTrait>(item).unwrap();

    let mut funcs = quote! {};
    let mut defs = quote! {};
    for func in tree.items {
        let TraitItem::Fn(func) = func else {
            panic!("only functions are supported")
        };
        let sig = func.sig;
        let func_name = sig.ident.clone();
        let string_name = format!("{func_name}");
        let inputs = &sig.inputs;
        let mut args: Punctuated<_, Comma> = Punctuated::new();
        let mut ins: Punctuated<_, Comma> = Punctuated::new();
        let mut out: Punctuated<_, Comma> = Punctuated::new();
        sig.inputs.iter().for_each(|f| {
            let FnArg::Typed(pt) = f else {
                panic!("self arguments are not allowed");
            };
            let Pat::Ident(id) = &*pt.pat else {
                panic!("only idents are allowed")
            };
            ins.insert(0, get_type(&pt.ty));
            args.push(id.ident.clone());
        });
        match sig.output {
            syn::ReturnType::Default => {}
            syn::ReturnType::Type(_, ty) => out.push(get_type(&ty)),
        }
        funcs = quote! {
            #funcs
            qwak::Function::new(
                #string_name,
                [#ins],
                [#out],
                qwak::UserData::Rust(std::sync::Mutex::new(()).into()),
                #func_name
            ),
        };
        defs = quote! {
            #defs
            qwak::host_fn!(pub #func_name(#inputs) -> () {
                use qwak_shared::QwakHostFunctions;
                Ok(super::$name:: #func_name (#args))
            });
        };
    }
    quote! {
        /// Generates the interface for calling host functions.
        #[macro_export]
        macro_rules! host_gen {
            ($name:ident) => {
                pub mod inner {
                    pub fn functions() -> impl IntoIterator<Item = qwak::Function> {
                        [#funcs]
                    }
                    #defs
                }
            }
        }
    }
    .into()
}

pub fn get_host_calls(item: TS) -> TS {
    let tree = syn::parse::<ItemTrait>(item).unwrap();
    let mut exports = quote! {};
    let mut pubs = quote! {};
    for func in tree.items {
        let TraitItem::Fn(func) = func else {
            panic!("only functions are supported")
        };
        let mut attrs = quote! {};
        for attr in func.attrs {
            attrs = quote! {
                #attrs
                #attr
            };
        }
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
            #[allow(missing_docs)]
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
            #attrs
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
        /// Generates the boilerplate for calling host functions.
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
