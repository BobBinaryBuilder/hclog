use syn::{
    parse::{
        Parse,
        ParseStream
    },
    punctuated::Punctuated,
    custom_keyword,
    DeriveInput,
    Attribute,
    Token,
};

pub (crate) mod enum_ty;
pub (crate) mod variant;

mod keywords {
    use super::custom_keyword;
    /*
     * private mod because custom_keyword creates keywords pub which clashes with proc_macro
     * visibility rules. proc_macro is currently not able to export anything other than tagged
     * #[proc_macro...] functions.
     *
     * we could use the same keywords for ty and variant level to reduce complexity. But still use
     * different ones to keep the scope of those keywords separated and clear for ty and v level.
     */
    // enum metadata
    custom_keyword!(scope);
    custom_keyword!(with_log);
    custom_keyword!(default_level);
    custom_keyword!(default_facade);
    custom_keyword!(default_options);

    // variant metadata
    custom_keyword!(ignore);
    custom_keyword!(level);
    custom_keyword!(facade);
    custom_keyword!(name);
}

use std::fmt::Debug;
// Some extension traits for ParseStream and DeriveInput to parse the Attributes
trait ParseStreamExt {
    fn parse_keyword<K, F>(&self) -> syn::Result<(K, F)> where K: Parse + Debug, F: Parse + Debug;
}
impl ParseStreamExt for ParseStream<'_> {
    #[inline]
    fn parse_keyword<K,F>(&self) -> syn::Result<(K, F)> where K: Parse + Debug, F: Parse + Debug {
        let kw = self.parse::<K>()?;
        self.parse::<Token![=]>()?; // just skip the separator token
        let attr = self.parse::<F>()?;
        Ok((kw, attr))
    }
}

pub trait DeriveInputExt {
    fn decode_meta<T: Parse>(&self, ident: &str) -> syn::Result<Vec<T>>;
}
impl DeriveInputExt for DeriveInput {
    fn decode_meta<T: Parse>(&self, ident: &str) -> syn::Result<Vec<T>> {
        decode_meta_inner(ident, &self.attrs)
    }
}
impl DeriveInputExt for syn::Variant {
    fn decode_meta<T: Parse>(&self, ident: &str) -> syn::Result<Vec<T>> {
        decode_meta_inner(ident, &self.attrs)
    }
}

pub trait DerivePropertiesExt<T> {
    fn parse_properties(&self, ident: &str) -> syn::Result<T>;
}

#[inline(always)]
fn decode_meta_inner<'a, T: Parse>(ident: &str, it: impl IntoIterator<Item = &'a Attribute>)
-> syn::Result<Vec<T>> {
    it.into_iter().filter(|a| a.path().is_ident(ident)).try_fold(
        Vec::new(), |mut vec, attr| {
            vec.extend(attr.parse_args_with(Punctuated::<T, Token![,]>::parse_terminated)?);
            Ok(vec)
        }
    )
}
