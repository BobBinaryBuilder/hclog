// TODO
use syn::{
    parse::{
        Lookahead1,
        Parse, ParseStream,
    },
    Ident, LitStr, LitBool, Path, Token,
};
use quote::{quote, ToTokens};
use proc_macro2::Span;
use std::{
    convert::TryFrom,
};
use super::*;
use crate::{
    helper::occurrence_error
};

pub struct StructProperties {
    pub slicename: i32,
}
