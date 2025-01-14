use proc_macro::TokenStream as TS;
use proc_macro2::Span;
use quote::quote;
use syn::{
    FnArg, Ident, ItemTrait, Pat, PatIdent, ReturnType, parse_macro_input, punctuated::Punctuated,
    token::Comma,
};

pub fn get_export_functions(item: TS) -> TS {
    let input = parse_macro_input!(item as ItemTrait);
    let mut res = quote!();

    for item in input.items.iter().filter_map(|f| match f {
        syn::TraitItem::Fn(f) => Some(f),
        _ => None,
    }) {
        let sig = &item.sig;
        let mut args: Punctuated<_, Comma> = Punctuated::new();
        let macro_sig = {
            let mut sig = sig.clone();
            sig.ident = Ident::new(&format!("plugin_{}", sig.ident), Span::call_site());
            let ReturnType::Type(_, ty) = &mut sig.output else {
                panic!("expected return type!")
            };
            let og = *ty.clone();
            *ty =
                syn::parse(quote!(extism_pdk::FnResult<extism_pdk::Msgpack<#og>>).into()).unwrap();

            let params = sig
                .inputs
                .into_iter()
                .enumerate()
                .map(|(i, a)| {
                    let FnArg::Typed(mut pat) = a else {
                        panic!("no self types in argument")
                    };
                    let name = Ident::new(&format!("arg{i}"), Span::call_site());
                    pat.pat = Box::new(Pat::Ident(PatIdent {
                        attrs: Vec::new(),
                        by_ref: None,
                        mutability: None,
                        ident: name.clone(),
                        subpat: None,
                    }));
                    args.push(name);
                    FnArg::Typed(pat)
                })
                .collect::<Punctuated<_, Comma>>();
            sig.inputs = params;
            sig
        };
        let call = &sig.ident;
        res = quote! {
            #res

            #[extism_pdk::plugin_fn]
            pub #macro_sig {
                Ok(extism_pdk::Msgpack($name::#call(#args)))
            }
        };
    }

    quote! {
        /// Generates the interface for calling plugin functions.
        #[macro_export]
        macro_rules! plugin_gen {
            ($name:ident) => {
                #res
            }
        }
    }
    .into()
}

pub fn get_plugin_calls(item: TS) -> TS {
    let input = parse_macro_input!(item as ItemTrait);
    let mut res = quote!();

    for item in input.items.iter().filter_map(|f| match f {
        syn::TraitItem::Fn(f) => Some(f),
        _ => None,
    }) {
        let sig = &item.sig;

        let mut args: Punctuated<_, Comma> = Punctuated::new();
        let (attrs, macro_sig, rt) = {
            let mut sig = sig.clone();
            sig.ident = Ident::new(&format!("{}", sig.ident), Span::call_site());

            let ReturnType::Type(rl, mut ty) = sig.output else {
                panic!("expected return type!")
            };
            let copy = *ty.clone();
            *ty = syn::parse(quote!(Result<#copy, extism::Error>).into()).unwrap();
            sig.output = ReturnType::Type(rl, ty);

            let mut attrs = quote! {};
            for attr in &item.attrs {
                attrs = quote! {
                    #attrs
                    #attr
                };
            }

            let params = sig
                .inputs
                .into_iter()
                .enumerate()
                .map(|(i, a)| {
                    let FnArg::Typed(mut pat) = a else {
                        panic!("no self types in argument")
                    };
                    let name = Ident::new(&format!("arg{i}"), Span::call_site());
                    pat.pat = Box::new(Pat::Ident(PatIdent {
                        attrs: Vec::new(),
                        by_ref: None,
                        mutability: None,
                        ident: name.clone(),
                        subpat: None,
                    }));
                    args.push(name);
                    FnArg::Typed(pat)
                })
                .collect::<Punctuated<_, Comma>>();
            sig.inputs = params;
            sig.inputs
                .insert(0, syn::parse(quote! {&self}.into()).unwrap());
            (attrs, sig, copy)
        };
        let args = if args.len() == 1 {
            quote! {#args}
        } else {
            quote! { () }
        };
        let call = format!("plugin_{}", sig.ident);
        res = quote! {
            #res

            #attrs
            pub #macro_sig {
                let res: extism_pdk::Msgpack<#rt> = match self.inner.lock() {
                    Ok(mut o) => o.call(#call, #args)?,
                    Err(e) => e.into_inner().call(#call, #args)?
                };
                Ok(res.into_inner())
            }
        };
    }

    quote! {
        /// Generates the struct for calling plugin functions.
        #[macro_export]
        macro_rules! plugin_calls {
            () => {
                mod calls {
                    use extism::{*, convert::*};
                    use std::sync::{Arc, Mutex};
                    /// A WASM qwak plugin.
                    #[derive(Debug)]
                    pub struct QwakPlugin {
                        inner: std::sync::Arc<std::sync::Mutex<extism::Plugin>>
                    }
                    impl QwakPlugin {
                        /// Returns a new [QwakPlugin].
                        /// 
                        /// # Errors
                        /// Fails if the path does not exist or if the binary files is invalid.
                        pub fn new(path: impl AsRef<std::path::Path>, functions: impl IntoIterator<Item = extism::Function>) -> Result<Self, String> {
                            let wasm = extism::Wasm::file(path);
                            let mut manifest = extism::Manifest::new([wasm]);
                            manifest.memory.max_pages = Some(u32::MAX);
                            manifest.memory.max_var_bytes = Some(u32::MAX as u64);
                            let plug = Arc::new(Mutex::new(
                                extism::Plugin::new(&manifest, functions, true)
                                    .map_err(|e| format!("{e}"))?,
                            ));
                            Ok(QwakPlugin { inner: plug })
                        }

                        #res
                    }
                }
            }
        }
    }
    .into()
}
