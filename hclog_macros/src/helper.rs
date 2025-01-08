use quote::ToTokens;
use proc_macro2::{TokenStream, Span};
use syn::{
    DeriveInput,
    Error,
    Ident,
};

pub (crate) fn debug_print_generated(ast: &DeriveInput, toks: &TokenStream) {
    if let Some(s) = option_env!("HCLOG_MACRO_DEBUG") {
        if s == "1" {
            println!("{}", toks);
        } else if ast.ident == s {
            println!("{}", toks);
        }
    }
}

pub (crate) fn assert_enum() -> syn::Error {
    Error::new(Span::call_site(), "This macro supports enums only")
}
pub (crate) fn assert_variant(span: Span, ty: &str) -> syn::Error {
    Error::new(span, format!("Invalid enum variant: expect Unit but got {} variant", ty))
}
pub (crate) fn assert_discriminant_value(i: &Ident, got: usize, expect: usize) -> syn::Error {
    Error::new(i.span(), format!("Discriminant value for {} out of bounds: expected={} got={}",
            i, expect, got,
        ))
}

pub (crate) fn occurrence_error<T: ToTokens>(fst: T, snd: T, attr: &str, ty: &str) -> syn::Error {
    let mut e = Error::new_spanned(
        snd,
        format!("{} attribute {} can't occur more than once", ty, attr),
    );
    e.combine(Error::new_spanned(fst, "first occurrence was here"));
    e
}
