use proc_macro::TokenStream as TS;
use proc_macro2::TokenStream;
use quote::quote;

mod host;
mod plugin;

#[proc_macro_attribute]
pub fn plugin(_attr: TS, item: TS) -> TS {
    let res = TokenStream::from(plugin::get_export_functions(item.clone()));
    let calls = TokenStream::from(plugin::get_plugin_calls(item.clone()));
    let item = TokenStream::from(item);
    let res = quote! {
        #item
        #res
        #calls
    };

    res.into()
}

#[proc_macro_attribute]
pub fn host(_attr: TS, item: TS) -> TS {
    let res = TokenStream::from(host::get_export_functions(item.clone()));
    let calls = TokenStream::from(host::get_host_calls(item.clone()));
    let item = TokenStream::from(item);
    let res = quote! {
        #item
        #res
        #calls
    };

    res.into()
}
