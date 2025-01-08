#![allow(
    clippy::needless_doctest_main,
)]
#![warn(
    missing_debug_implementations,
    missing_docs,
)]
#![deny(
    unconditional_recursion,
    rustdoc::broken_intra_doc_links,
)]
#![doc(test(
    no_crate_inject,
    attr(deny(warnings, rust_2018_idioms), allow(dead_code, unused_variables))
))]
#![cfg_attr(docsrs, allow(unused_attributes))]

//! A procedural macro crate to be used with the `hclog` crate
//!
//! For more informations on `hclog` please refer to the [hclog](https://crates.io/crates/hclog) crate
//! documentation.
//!
//! This crate is a stripped down and modified version of the [strum_macros](https://crates.io/crates/strum_macros) crate
//! by Peter Glotfelty. It is used to derive the necessary traits for the `hclog` crate via the
//! `HCLog` derive macro. It also provides a few additional attributes to customize the LogKey
//! variants.
//!
//! # Usage
//!
//! Currently only enums with unit variants are supported. The `HCLog` derive macro will generate
//! the necessary code to implement the `hclog` traits for the given enum.
//!
//! The macro allows to define attributes on the enum and its variants to customize the behavior of
//! the generated code. The following attributes are supported by the `#[hclog()]` attribute:
//!
//! * enum attributes:
//!     * `scope`: the `Scope` the `LogKey`s belong to. It expects a value of type `ScopeKey`.
//!     * `default_level`: the default `Level` for all `LogKey`s. It expects a value of type `Level`.
//!     * `default_facade`: the default `FacadeVariant` for all `LogKey`s. It expects a value of type `FacadeVariant`.
//!     * `with_log`: initialize the `hclog` compatibility mode with crate `log`. It expects a boolean value.
//!
//! * variant attributes:
//!     * `name`: the `Display` name of the `LogKey`. It expects a [`str`] value.
//!     * `level`: the `Level` of the `LogKey`. It expects a value of type `Level`.
//!     * `facade`: the `FacadeVariant` of the `LogKey`. It expects a value of type `FacadeVariant`.
//!
//! The enum attributes are used as a default for all variants if they don't define their own
//! attributes. All attributes are optional and can be omitted if the default behavior is sufficient.
//! If no attributes are given the defaults from the `hclog` crate are used.
//!
//! # Example
//!
//! ### Derive the `HCLog` trait
//!
//! ```rust
//! use hclog_macros::HCLog;
//!
//! #[derive(HCLog, Copy, Clone, Debug, PartialEq)]
//! enum MyLog {
//!    AA,
//!    AB,
//!    AC,
//!    AD,
//! }
//! ```
//!
//! ### Extend the `HCLog` trait with additional attributes
//!
//! It's possible to set additional attributes via the `hclog` attribute. For detailed informations
//! of the attributes consult the [`hclog`](https://crates.io/crates/hclog) crate documentation.
//!
//! ```rust
//! use hclog_macros::HCLog;
//! use hclog::{Level, FacadeVariant, ScopeKey};
//!
//! #[derive(HCLog, Copy, Clone, Debug, PartialEq)]
//! #[hclog(scope = ScopeKey::Application, default_level = Level::Info, default_facade = FacadeVariant::None)]
//! enum MyLog {
//!     #[hclog(name = "AA", level = Level::Debug1, facade = FacadeVariant::StdOut)]
//!     AA,
//!     #[hclog(name = "AB", level = Level::Info, facade = FacadeVariant::StdErr)]
//!     AB,
//!     #[hclog(name = "AC", level = Level::Warn, facade = FacadeVariant::Syslog("user".to_string()))]
//!     AC,
//!     #[hclog(name = "AD", level = Level::Error, facade = FacadeVariant::File("log.txt".into(), false))]
//!     AD,
//! }
//!
use proc_macro;
use proc_macro2::{
    TokenStream,
};
use quote::quote;
use syn::{
    DeriveInput, Data, Fields, Path,
    parse_macro_input,
};

mod helper;
mod meta;

use crate::meta::DerivePropertiesExt;
use crate::helper::{assert_variant, assert_discriminant_value};

const CLOG_ATTR_IDENT: &str = "hclog";

/* public exported macro(s) to other crates */
#[doc(hidden)]
#[proc_macro_derive(HCLog, attributes(hclog))]
pub fn hclog_derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    parse_derive_macro(&input).unwrap_or_else(|err| err.into_compile_error()).into()
}

fn parse_derive_macro(ast: &DeriveInput) -> syn::Result<proc_macro2::TokenStream> {
    let ident = &ast.ident;
    let generics = &ast.generics;
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
    let attrs = ast.parse_properties(CLOG_ATTR_IDENT)?;

    let data = match &ast.data {
        Data::Enum(v) => v,
        _ => return Err(helper::assert_enum()),
    };

    let lmk_ident = syn::parse_str::<Path>("::hclog::ScopeKey")?;
    let lvl_ident = syn::parse_str::<Path>("::hclog::Level")?;
    let fav_ident = syn::parse_str::<Path>("::hclog::FacadeVariant")?;
    let opt_ident = syn::parse_str::<Path>("::hclog::options::Options")?;
    let res_ident = syn::parse_str::<Path>("::hclog::Result")?;

    let mut init_trait_fns = vec![];

    if let Some(ref logmod) = attrs.scope {
        init_trait_fns.push(quote! { fn logscope() -> #lmk_ident { #logmod } });
    }
    if let Some(ref level) = attrs.default_level {
        init_trait_fns.push(quote! { fn default_level() -> #lvl_ident { #level } });
    }
    if let Some(ref facade) = attrs.default_facade {
        init_trait_fns.push(quote! { fn default_facade() -> #fav_ident { #facade } });
    }
    if let Some(ref options) = attrs.default_options {
        init_trait_fns.push(quote! { fn default_options() -> #opt_ident { #(#options)* } });
    }

    let with_log = if attrs.logcompat {
        quote! { options + hclog::options::LOGCOMPAT }
    } else {
        quote! { options }
    };

    let variants = &data.variants;
    let mut v_idents = vec![];
    let mut fmt_arms = vec![];
    let mut lvl_arms = vec![];
    let mut fav_arms = vec![];
    let mut dsc_arms = vec![];

    let mut idx = 0usize;
    for variant in variants {
        let v_ident = &variant.ident;
        let v_attrs = variant.parse_properties(CLOG_ATTR_IDENT)?;
        let v_discriminant = &variant.discriminant;

        // allow unit variants only - at least for now
        // there is currently no real use case to use Named or Unnamed fields
        match &variant.fields {
            Fields::Named(_) => return Err(assert_variant(variant.ident.span(), "Struct")),
            Fields::Unnamed(_) => return Err(assert_variant(variant.ident.span(), "Tuple")),
            Fields::Unit => (),
        }

        if let Some((_, syn::Expr::Lit(ref d))) = v_discriminant {
            let syn::Lit::Int(ref a) = d.lit else {
                // should never happen since enum discriminants can't be other than int
                return Err(syn::Error::new(v_ident.span(), "invalid discriminant"));
            };
            let value = a.base10_parse::<usize>()?;
            // if discriminants are given ensure the hclog Index doesn't get out of bounds on
            // access - otherwise the submodule index would raise a panic in hclog itself
            if value != idx {
                return Err(assert_discriminant_value(&v_ident, value, idx));
            }
            dsc_arms.push(quote! { (&#ident::#v_ident,) => #value, });
        } else {
            dsc_arms.push(quote! { (&#ident::#v_ident,) => #idx, });
        }
        idx+=1;

        let v_display_name = match v_attrs.name {
            Some(n) => quote! { #n },
            None => quote! { stringify!(#v_ident) },
        };
        if let Some(level) = v_attrs.level {
            if v_attrs.ignore {
                lvl_arms.push(quote! {(&#ident::#v_ident,) => Some(Level::Off), });
            } else {
                lvl_arms.push(quote! {(&#ident::#v_ident,) => Some(#level), });
            }
        }
        if let Some(facade) = v_attrs.facade {
            if v_attrs.ignore {
                fav_arms.push(quote! {(&#ident::#v_ident,) => Some(FacadeVariant::None), });
            } else {
                fav_arms.push(quote! {(&#ident::#v_ident,) => Some(#facade), });
            }
        }

        if !v_attrs.ignore {
            v_idents.push(quote! { #ident::#v_ident, });
        }
        fmt_arms.push(quote! {(&#ident::#v_ident,) => f.write_str(#v_display_name), });
    }

    let init_fav_fn = if !fav_arms.is_empty() {
        quote! {
            fn init_facade(&self) -> ::core::option::Option<#fav_ident> {
                match (&*self,) {
                    #(#fav_arms)*
                    _ => None,
                }
            }
        }
    } else {
        quote! {}
    };

    let init_lvl_fn = if !lvl_arms.is_empty() {
        quote! {
            fn init_level(&self) -> ::core::option::Option<#lvl_ident> {
                match (&*self,) {
                    #(#lvl_arms)*
                    _ => None,
                }
            }
        }
    } else {
        quote! {}
    };

    // generate the output and all necessary impls
    let output = TokenStream::from(quote! {
        // bring traits into scope
        use hclog::{Scope, LogKey};

        #[automatically_derived]
        impl #impl_generics std::fmt::Display for #ident #ty_generics #where_clause {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                match (&*self,) {
                    #(#fmt_arms)*
                }
            }
        }

        #[automatically_derived]
        impl #impl_generics hclog::Scope for #ident #ty_generics #where_clause {
            #(#init_trait_fns)*
            /* the init call(s) itself */
            fn init<S: std::fmt::Display>(name: S, level: #lvl_ident, facade: #fav_ident,
                         options: #opt_ident) -> #res_ident<()> {
                hclog::init::<Self, S>(name, level, facade, #with_log)?;
                hclog::add_submodules(&[#(#v_idents)*])?;
                Ok(())
            }
        }

        #[automatically_derived]
        impl #impl_generics hclog::LogKey for #ident #ty_generics #where_clause {
            fn log_key(&self) -> hclog::ContextKey {
                match (&*self,) {
                    #(#dsc_arms)*
                }
            }
            #init_lvl_fn
            #init_fav_fn
        }
    });
    helper::debug_print_generated(ast, &output);
    Ok(output)
}
