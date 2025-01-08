#[macro_use]
extern crate hclog;

use hclog::{
    options::Options,
    Level, Scope, LogKey, ContextKey, FacadeVariant
};

use std::fmt;

#[derive(Copy, Clone, Debug)]
enum LogKeys {
    FOO,
    BAR,
    BAZ,
}
impl fmt::Display for LogKeys {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Self::FOO => fmt.write_str("FOO"),
            Self::BAR => fmt.write_str("BAR"),
            Self::BAZ => fmt.write_str("BAZ"),
        }
    }
}
impl Scope for LogKeys {
    fn init<S: fmt::Display>(name: S, level: Level, facade: FacadeVariant,
                 options: Options) -> hclog::Result<()> {
        hclog::init::<Self, S>(name, level, facade.clone(), options)?;
        hclog::add_submodules(&[Self::FOO, Self::BAR])
    }
}
impl LogKey for LogKeys {
    fn log_key(&self) -> ContextKey { *self as ContextKey }
    fn init_level(&self) -> Option<Level> {
        match *self {
            Self::FOO => Some(Level::Info),
            _ => None,
        }
    }
    fn init_facade(&self) -> Option<FacadeVariant> {
        match *self {
            Self::BAR => Some(FacadeVariant::StdOut),
            _ => None,
        }
    }
}

use LogKeys::*;
fn main() {
    LogKeys::init("enum", Level::Debug10, FacadeVariant::StdErr, Options::default()).unwrap();
    assert_eq!(hclog::has_module(FOO), Ok(true));
    assert_eq!(hclog::has_module(BAR), Ok(true));
    assert_eq!(hclog::has_module(BAZ), Ok(false));

    assert_eq!(tE!(FOO), true);

    hclog::list_modules(&mut std::io::stdout()).unwrap();

    lI!(FOO, "Hello from enum example");
    lI!(FOO, "hello from enum example with {}, {} and {}", FOO, BAR, BAZ);
}
