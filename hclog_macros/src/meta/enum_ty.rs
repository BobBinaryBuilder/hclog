use syn::{
    parse::{
        Parse, ParseStream,
    },
    DeriveInput,
    Ident, Path, Expr,
};
use super::*;
use crate::{
    helper::occurrence_error,
};

/**
 * Enum (type level) metadata as decoded from the DeriveInput
 */
#[derive(Debug)]
enum EnumAttrs {
    Scope {
        kw: keywords::scope,
        attr: Path,
    },
    WithLog {
        kw: keywords::with_log,
    },
    DefaultLevel {
        kw: keywords::default_level,
        attr: Path,
    },
    DefaultFacade {
        kw: keywords::default_facade,
        attr: Expr,
    },
    /*
    DefaultOptions {
        kw: keywords::default_options,
        attr: Vec<Ident>,
    },
    */
}
impl Parse for EnumAttrs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let lh = input.lookahead1();
        if lh.peek(keywords::scope) {
            let (kw, attr) = input.parse_keyword::<keywords::scope, Path>()?;
            Ok(Self::Scope { kw, attr })
        } else if lh.peek(keywords::with_log) {
            Ok(Self::WithLog { kw: input.parse::<keywords::with_log>()? })
        } else if lh.peek(keywords::default_level) {
            let (kw, attr) = input.parse_keyword::<keywords::default_level, Path>()?;
            Ok(Self::DefaultLevel { kw, attr })
        } else if lh.peek(keywords::default_facade) {
            let (kw, attr) = input.parse_keyword::<keywords::default_facade, Expr>()?;
            Ok(Self::DefaultFacade { kw, attr })
        } else {
            Err(lh.error())
        }
    }
}


/**
 *
 */
#[derive(Debug, Clone, Default)]
pub struct EnumProperties {
    pub scope: Option<Path>,
    pub logcompat: bool,
    pub default_level: Option<Path>,
    pub default_facade: Option<Expr>,
    pub default_options: Option<Vec<Ident>>,
}
impl DerivePropertiesExt<EnumProperties> for DeriveInput {
    fn parse_properties(&self, ident: &str) -> syn::Result<EnumProperties> {
        let mut out = EnumProperties::default();
        let mut scope_kw = None;
        let mut default_level_kw = None;
        let mut default_facade_kw = None;
        let mut with_log_kw = None;
        for meta in self.decode_meta::<EnumAttrs>(ident)? {
            match meta {
                EnumAttrs::Scope { kw, attr } => {
                    if let Some(lm_kw) = scope_kw {
                        return Err(occurrence_error(lm_kw, kw, "scope", "enum"));
                    }
                    scope_kw = Some(kw);
                    out.scope = Some(attr);
                }
                EnumAttrs::WithLog { kw } => {
                    if let Some(lw_kw) = with_log_kw {
                        return Err(occurrence_error(lw_kw, kw, "with_log", "enum"));
                    }
                    with_log_kw = Some(kw);
                    out.logcompat = true;
                }
                EnumAttrs::DefaultLevel { kw, attr } => {
                    if let Some(dl_kw) = default_level_kw {
                        return Err(occurrence_error(dl_kw, kw, "default_level", "enum"));
                    }
                    default_level_kw = Some(kw);
                    out.default_level = Some(attr);
                }
                EnumAttrs::DefaultFacade { kw, attr } => {
                    if let Some(prev_kw) = default_facade_kw {
                        return Err(occurrence_error(prev_kw, kw, "default_facade", "enum"));
                    }
                    default_facade_kw = Some(kw);
                    out.default_facade = Some(attr);
                }
            }
        }
        Ok(out)
    }
}
