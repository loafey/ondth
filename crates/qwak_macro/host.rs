use proc_macro::TokenStream as TS;
use proc_macro2::{Span, TokenStream};
use quote::quote;
use std::collections::BTreeMap;
use syn::{
    FnArg, Ident, ItemTrait, Pat, Signature, TraitItem, Type, punctuated::Punctuated, token::Comma,
};

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
        "qwak_helper_types::MapInteraction" => quote!(qwak::PTR),
        "qwak_helper_types::MsgVec3" => quote!(qwak::PTR),
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

#[derive(Default)]
struct Namespace {
    nest: BTreeMap<String, Namespace>,
    functions: Vec<(String, String, Signature, TokenStream)>,
}
impl Namespace {
    fn pretty(&self, depth: usize) -> String {
        const S: &str = "    ";
        let mut s = String::new();
        s += "{\n";
        for (new_name, og_name, _, _) in &self.functions {
            s += &format!("{}{new_name} -> {og_name}\n", S.repeat(depth + 1));
        }
        for (module, nest) in &self.nest {
            s += &format!(
                "{}{module} :: {}",
                S.repeat(depth + 1),
                nest.pretty(depth + 1)
            );
        }
        s += &format!("{}}}\n", S.repeat(depth));
        s
    }
}
impl std::fmt::Debug for Namespace {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.pretty(0))
    }
}
fn get_tree(
    names: &[&str],
    og_name: &str,
    signature: &Signature,
    attrs: &TokenStream,
    ns: &mut Namespace,
) {
    if names.len() == 1 {
        ns.functions.push((
            names[0].to_string(),
            og_name.to_string(),
            signature.clone(),
            attrs.clone(),
        ));
    } else {
        let entry = ns.nest.entry(names[0].to_string()).or_default();
        get_tree(&names[1..], og_name, signature, attrs, entry);
    }
}
fn build_structure(ns: &Namespace, res: &mut TokenStream, depth: usize) {
    let mut inner = quote! {};

    for (name, og_name, sig, attrs) in &ns.functions {
        let og_name = Ident::new(og_name, Span::call_site());
        let function = Ident::new(name, Span::call_site());
        let mut new_sig = sig.clone();
        new_sig.ident = function;

        let mut namespace_prefix = quote! {};
        for _ in 0..depth {
            namespace_prefix = quote! {#namespace_prefix super::};
        }
        let panic_handler = match name != "log__debug" {
            true => {
                quote! {
                    #namespace_prefix inner::log__debug(format!("plugin crashed calling host function: {e}")).unwrap();
                }
            }
            false => quote! {},
        };

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

        inner = quote! {
            #inner
            #attrs
            pub #new_sig {
                unsafe {
                    match #namespace_prefix inner::#og_name(#args) {
                        Ok(o) => o,
                        Err(e) => {
                            #panic_handler
                            panic!()
                        },
                    }
                }
            }
        };
    }

    for (name, nest) in &ns.nest {
        let name = Ident::new(name, Span::call_site());
        let mut temp = quote! {};
        build_structure(nest, &mut temp, depth + 1);
        inner = quote! {
            #inner
            pub mod #name { #temp }
        };
    }

    *res = quote! {
        #res
        #inner
    }
}
pub fn get_host_calls(item: TS) -> TS {
    let tree = syn::parse::<ItemTrait>(item).unwrap();
    let mut exports = quote! {};

    // Build up the module tree
    let mut top = Namespace::default();
    for func in &tree.items {
        let TraitItem::Fn(func) = func else {
            panic!("only functions are supported")
        };
        let func_name = func.sig.ident.clone();
        let f = func_name.to_string();
        let mut attrs = quote! {};
        for attr in &func.attrs {
            attrs = quote! {
                #attrs
                #attr
            };
        }
        get_tree(
            &f.split("__").collect::<Vec<_>>(),
            &f,
            &func.sig,
            &attrs,
            &mut top,
        );
    }
    let mut test = quote! {};
    build_structure(&top, &mut test, 0);

    for func in tree.items {
        let TraitItem::Fn(func) = func else {
            panic!("only functions are supported")
        };
        let og_sig = &func.sig;

        exports = quote! {
            #exports
            pub unsafe #og_sig;
        };
    }

    // let d = format!("{:?}", format!("{tree:#?}"));
    quote! {
        /// Generates the boilerplate for calling host functions.
        #[macro_export]
        macro_rules! host_calls {
            () => {
                mod host {
                    #test
                    #[allow(missing_docs, non_snake_case)]
                    mod inner {
                        #[extism_pdk::host_fn]
                        unsafe extern "ExtismHost" {
                            #exports
                        }
                    }
                }
            }
        }
    }
    .into()
}
