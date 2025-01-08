use syn::{
    parse::{
        Parse, ParseStream,
    },
    Variant,
    LitStr, Path, Expr,
};
use super::*;
use crate::helper::occurrence_error;

/**
 * Enum Variant metadata
 */
pub enum VariantMeta {
    Level {
        kw: keywords::level,
        attr: Path,
    },
    Facade {
        kw: keywords::facade,
        attr: Expr,
    },
    Name {
        kw: keywords::name,
        attr: LitStr,
    },
    Ignore {
        kw: keywords::ignore,
    }
}
impl Parse for VariantMeta {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let lh = input.lookahead1();
        if lh.peek(keywords::level) {
            let (kw, attr) = input.parse_keyword::<keywords::level, Path>()?;
            Ok(Self::Level { kw, attr })
        } else if lh.peek(keywords::facade) {
            let (kw, attr) = input.parse_keyword::<keywords::facade, Expr>()?;
            Ok(Self::Facade { kw, attr })
        } else if lh.peek(keywords::name) {
            let (kw, attr) = input.parse_keyword::<keywords::name, LitStr>()?;
            Ok(Self::Name { kw, attr })
        } else if lh.peek(keywords::ignore) {
            Ok(Self::Ignore { kw: input.parse::<keywords::ignore>()? })
        } else {
            Err(lh.error())
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct VariantProperties {
    pub level: Option<Path>,
    pub facade: Option<Expr>,
    pub name: Option<LitStr>,
    pub ignore: bool,
}
impl DerivePropertiesExt<VariantProperties> for Variant {
    fn parse_properties(&self, ident: &str) -> syn::Result<VariantProperties> {
        let mut out = VariantProperties::default();
        let mut level_kw = None;
        let mut facade_kw = None;
        let mut name_kw = None;
        let mut ignore_kw = None;

        for meta in self.decode_meta::<VariantMeta>(ident)? {
            match meta {
                VariantMeta::Ignore { kw } => {
                    if let Some(ign_kw) = ignore_kw {
                        return Err(occurrence_error(ign_kw, kw, "ignore", "variant"));
                    }
                    ignore_kw = Some(kw);
                    out.ignore = true;
                }
                VariantMeta::Level { kw, attr } => {
                    if let Some(lvl_kw) = level_kw {
                        return Err(occurrence_error(lvl_kw, kw, "level", "variant"));
                    }
                    level_kw = Some(kw);
                    out.level = Some(attr);
                }
                VariantMeta::Facade { kw, attr } => {
                    if let Some(fa_kw) = facade_kw {
                        return Err(occurrence_error(fa_kw, kw, "facade", "variant"));
                    }
                    facade_kw = Some(kw);
                    out.facade = Some(attr);
                }
                VariantMeta::Name { kw, attr } => {
                    if let Some(n_kw) = name_kw {
                        return Err(occurrence_error(n_kw, kw, "name", "variant"));
                    }
                    name_kw = Some(kw);
                    out.name = Some(attr);
                }
            }
        }

        Ok(out)
    }
}
