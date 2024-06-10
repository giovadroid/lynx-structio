extern crate proc_macro;
use proc_macro::TokenStream;

use quote::quote;
use syn::{parse_macro_input, DeriveInput};


///
/// Derive macro for the FileWatch trait
/// Implement the FileStructTrait trait for the annotated struct
/// The trait implementation will call the save, load and reload functions from the core module
///
#[proc_macro_derive(FileStruct)]
pub fn derive_save(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    let name = &ast.ident;

    let gen = quote! {
        impl FileStructTrait for #name {
            fn save(&self) -> MonitorResult<()> {
                lynx_structio::derive_functions::save(self)
            }
            fn load() -> MonitorResult<Self> {
                lynx_structio::derive_functions::load()
            }
            fn reload(&self) -> MonitorResult<()> {
                lynx_structio::derive_functions::reload(self)
            }
        }
    };
    gen.into()
}

///
/// Derive macro for the FileWatch trait
/// Implement the FileStructTrait trait for the annotated struct
/// The trait implementation will call the save, load and reload functions from the core module
/// The load function will also call the monitor and listen functions from the core module
///
#[proc_macro_derive(FileWatch)]
pub fn derive_watch(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    let name = &ast.ident;

    let gen = quote! {
        impl FileStructTrait for #name {
            fn save(&self) -> MonitorResult<()> {
                lynx_structio::derive_functions::save(self)
            }
            fn load() -> MonitorResult<Self> {
                let instance = lynx_structio::derive_functions::load()?;
                lynx_structio::derive_functions::monitor(&instance)?;
                lynx_structio::monitor::listen();
                Ok(instance)
            }
            fn reload(&self) -> MonitorResult<()> {
                lynx_structio::derive_functions::reload(self)
            }
        }
    };
    gen.into()}