use crate::{
    context::CTX,
    level::Level,
    facades::FacadeVariant,
    options::Options,
    util::read_var_from_env,
    Scope, LogKey, ScopeKey, ContextKey, Result,
};
use std::fmt::{self, Display};

/// Internal keys for the hclog library.
///
/// Those keys are primarily used for internal logging and compatibility with the
/// `log` crate. They can be used to alter the [`Level`] and [`FacadeVariant`] of the
/// internal logger. The keys are always initialized on initialization of the library.
///
/// The `Internal` key is used for the internal logger and the `LogCompat` key is used
/// for the compatibility with the `log` crate.
/// The `Internal` key is initialized without a `Level` or `FacadeVariant` by default.
/// You can set the `CLOG_DEBUG` environment variable to a `1` to enable the logging to
/// `stdout` with a `Level` of `DEBUG`. The initial `Level` and `FacadeVariant` of the
/// `LogCompat` key are the same as set on library initialization.
///
/// Altering the `Level` or `FacadeVariant` of both keys is possible by using the
/// [`set_level`](crate::set_level) and [`set_logdest`](crate::set_logdest) functions
/// provided.
///
/// # Example
///
/// ```rust
/// use hclog::{InternalLogKeys, Level, FacadeVariant};
///
/// let _ = hclog::set_level(InternalLogKeys::Internal, Level::Debug1);
/// let _ = hclog::set_logdest(InternalLogKeys::LogCompat, FacadeVariant::StdErr);
/// ```
///
#[derive(Copy, Clone, Debug, Hash, Eq, PartialEq, Ord, PartialOrd)]
pub enum InternalLogKeys {
    /// LogKey for the internal logger.
    Internal,
    /// LogKey for the compatibility with the `log` crate.
    LogCompat,
}

impl Display for InternalLogKeys {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Self::Internal => f.write_str("hclog"),
            Self::LogCompat => f.write_str("logcompat"),
        }
    }
}
impl Scope for InternalLogKeys {
    fn logscope() -> ScopeKey { ScopeKey::CLog }

    fn init<S: Display>(
        name: S, level: Level, facade: FacadeVariant, options: Options
    ) -> Result<()> {
        let mut level = level;
        let mut facade = facade;
        if let Ok(Some(f)) = read_var_from_env::<u8>("CLOG_DEBUG") {
            facade = FacadeVariant::StdOut;
            level = Level::debug_level(f);
        }
        {
            CTX::get_mut()?.init_mod::<Self, S>(name, level, facade, options)?
                .add_submodule(Self::Internal)?;
        }
        Ok(())
    }
}
impl LogKey for InternalLogKeys {
    fn log_key(&self) -> ContextKey { *self as usize }
}

pub (crate) mod test {
    use crate::{Scope, LogKey, ContextKey, Result, Level, FacadeVariant, options::Options};
    use std::fmt::{self, Display};
    /*
     * All unit tests in this lib should use this libkeys. All LogKeys are initialized
     * in global scope and tests may run in unpredictable order over various files (even
     * with serial).
     * Initializing keys per test module may cause race conditions in tests because the
     * context is _not_ overwritten.
     */
    #[derive(Copy, Clone, Debug)]
    pub enum TestKeys {
        LIBTESTFOO,
        LIBTESTBAR,
    }
    use TestKeys::*;
    impl LogKey for TestKeys {
        fn log_key(&self) -> ContextKey { *self as usize }
    }
    impl Scope for TestKeys {
        fn init<S: Display>(
            name: S, level: Level, facade: FacadeVariant, options: Options
        ) -> Result<()> {
            crate::init::<Self, S>(name, level, facade, options)?;
            crate::add_submodules(&[LIBTESTFOO, LIBTESTBAR])
        }
    }
    impl Display for TestKeys {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            match *self {
                Self::LIBTESTFOO => write!(f, "libtestfoo"),
                Self::LIBTESTBAR => write!(f, "libtestbar"),
            }
        }
    }
}
