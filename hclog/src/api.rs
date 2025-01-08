use crate::{
    context::CTX,
    options::*,
    facades::FacadeVariant,
    Level,
    ErrorKind::*,
    InternalLogKeys::{self, *},
    Scope, LogKey, Result,
};
use std::{
    future::Future,
    fmt::{Display, Arguments},
    io::Write,
    env,
};

#[cfg(doctest)]
use hclog_macros::HCLog;

/// Initialize the logging system.
///
/// This is the main initialization function used in the [`Scope`] implementation.
/// It initializes the internal logging system with a given name `S`, a [`Level`],
/// a [`FacadeVariant`]. If the [`LOGCOMPAT`] option is set in the [`Options`], the
/// compatibility layer to the `log` crate will be initialized as well.
///
/// # Errors
///
pub fn init<I, S>(
    name: S, level: Level, facade: FacadeVariant, options: Options
) -> Result<()>
    where I: Scope,
          S: Display
{
    InternalLogKeys::init_with_defaults(&name)?;
    CTX::get_mut()?.init_mod::<I, S>(name, level, facade.clone(), options)?;
    if options.has(LOGCOMPAT) {
        crate::compat::init_log_compat(level, facade, Some(options))?;
    }
    Ok(())
}

/// Initialize the compatibility layer to the [`crate-log`] crate
///
/// Some other crates might use the `log` crate for logging. The intention of this compatibility
/// layer is to provide a way to capture the output generated via log macros (like `error!()`,
/// `warn!()`, `info!()`, `debug!()`, `trace!()`) and redirect them to the internal logging system.
/// If the layer is already initialized this is a noop and won't change the current state.
///
/// To alter the settings of the compatibility layer at runtime, you can use the [`set_level()`]
/// or [`set_logdest()`] functions with the
/// [`InteralLogKeys::LogCompat`](enum@InternalLogKeys#variant@LogCompat)
/// module key.
///
/// ## Mapping from `log` to internal log levels
///
/// The `boxed_logger` is always initialized with the highest LevelFilter `Trace`. The filtering
/// of the messages is handled by the internal logging system. Whereas the mapping of the log levels
/// is:
///
/// | Level from crate log         | Level in internal logging system |
/// |------------------------------|----------------------------------|
/// | [`Error`](log::Level::Error) | [`Error`](Level::Error)          |
/// | [`Warn`](log::Level::Warn)   | [`Warn`](Level::Warn)            |
/// | [`Info`](log::Level::Info)   | [`Info`](Level::Info)            |
/// | [`Debug`](log::Level::Debug) | [`Debug1`](Level::Debug1)        |
/// | [`Trace`](log::Level::Trace) | [`Debug10`](Level::Debug10)      |
///
/// There is no distinction of other Debug levels in the compatibility layer. If you need more
/// fine grained control over the log levels, you should use the internal logging system directly.
///
/// # Examples
///
/// ```
/// use hclog::{Level, FacadeVariant};
/// use log::{debug, error, info, trace, warn};
///
/// fn main() {
///    hclog::init_log_compat("log", Level::Debug5, FacadeVariant::StdOut, None).unwrap();
///    // try all log:: macros once
///    error!("error!() print with log::log");
///    warn!("warn!() print with log::log");
///    info!("info!() print with log::log");
///    debug!("debug!() print with log::log");
///    trace!("trace!() print with log::log - not printed because filtered");
/// }
/// ```
///
/// # Errors
///
/// # Panics
pub fn init_log_compat<S: Display>(
    name: S, level: Level, facade: FacadeVariant, options: Option<Options>
) -> Result<()> {
    InternalLogKeys::init_with_defaults(&name)?;
    crate::compat::init_log_compat(level, facade, options)
}

/// Dump the internal state of the logging system to the supplied writer
///
/// Dump the current internal context (state) to a supplied [`Write`]r `W` instance. Usually this
/// will be any type that implements the [`std::io::Write`] trait, like a [`std::fs::File`] or
/// [`std::io::Stdout`].
///
/// The dump will only be performed if the environment variable `HCLOG_DUMP_MODULES` is set to `1`.
/// This is a safety measure to prevent accidental dumps in production code.
///
/// # Examples
///
/// ```rust
///     hclog::dump(&mut std::io::stdout()).unwrap();
/// ```
///
/// # Errors
///
/// Returns an error if:
/// * [`ContextLock`]: the internal context can't be accessed
/// * [`IoError`]: any underlying I/O error while writing to the supplied writer
///
/// # Panics
///
/// This function might panic for any reason the used writer might panic. Please refer
/// to the according documentation of the writer in use.
pub fn dump<W: Write>(w: &mut W) -> Result<()> {
    if let Ok(s) = env::var("HCLOG_DUMP_MODULES") {
        if s == "1" {
            let ctx = CTX::get()?;
            w.write_fmt(format_args!("{:#?}", ctx))?;
        }
    }
    Ok(())
}

/// Print a list of all available modules to the supplied writer `w`
///
/// Print a comma separated list of all available [`LogKey`]s in all [`Scope`]s to the
/// supplied [`Write`]r `w` instance.
///
/// # Examples
///
/// ```rust
///     hclog::list_modules(&mut std::io::stdout()).unwrap();
/// ```
///
/// # Errors
///
/// Returns an error if:
/// * [`ContextLock`]: the internal context can't be accessed
/// * [`IoError`]: any underlying I/O error while writing to the supplied writer
///
pub fn list_modules<W: Write>(w: &mut W) -> Result<()> {
    let ctx = CTX::get()?;
    w.write(b"List of available modules:\n")?;
    let joined = ctx.logmods().flat_map(
        |l| l.submodules().map(|s| s.to_string())
    ).collect::<Vec<_>>();
    w.write_fmt(format_args!("\t{}\n", &joined.join(", ")))?;
    w.flush()?;
    Ok(())
}

/* scope access (async stuff) */
#[doc(hidden)]
pub fn scope<I, K, F>(ident: I, key: K, future: F) -> Result<impl Future>
where
    I: Display + Send,
    K: LogKey,
    F: Future + Send
{
    CTX::new_scoped(ident, key, future)
}

/* submod mgmt functions */
/// Initialize a [`Scope`] with a list of provided [`LogKey`]s.
///
/// The passed [`Level`], [`FacadeVariant`] and [`Options`] will be used for the [`Scope`]
/// and passed down to the each LogKey unless they provide custom values via the [`LogKey`]
/// trait implementation.
///
/// The given modname `S` is the name of the scope used for logging like the binary name in
/// syslog. The modules `I` input is everything which can be converted to a iterator.
///
/// This is a shortcut for calling [`init()`] and [`add_submodules()`] in sequence.
///
/// # Examples
///
/// ```rust
/// use hclog::{Scope, Level, FacadeVariant, options::Options, Result};
/// # use hclog::{LogKey, ContextKey};
///
/// #[derive(Copy, Clone)]
/// enum LogKeys { IA, IB, IC }
///
/// # impl std::fmt::Display for LogKeys {
/// #    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
/// #        match *self {
/// #            Self::IA => fmt.write_str("IA"),
/// #            Self::IB => fmt.write_str("IB"),
/// #            Self::IC => fmt.write_str("IC"),
/// #        }
/// #    }
/// # }
///
/// impl Scope for LogKeys {
///     fn init<S: std::fmt::Display>(
///         name: S, level: Level, facade: FacadeVariant, options: Options
///     ) -> Result<()> {
///         // instead of calling:
///         // hclog::init<Self, S>(name, level, facade, options)?;
///         // hclog::add_submodules(&[Self::IA, Self::IB, Self::IC])
///         // you could call:
///         hclog::init_modules(name, &[Self::IA, Self::IB, Self::IC], level, facade, options)
///     }
/// }
///
/// # impl LogKey for LogKeys {
/// #    fn log_key(&self) -> ContextKey { *self as ContextKey }
/// # }
/// ```
///
/// # Errors
///
/// Returns an error if:
/// * [ContextLock]: the internal context can't be accessed
/// * [LogCompatInitialized]: the compatibility layer is already initialized
///
pub fn init_modules<'a, I, K, S>(
    modname: S, modules: I, l: Level, f: FacadeVariant, o: Options
) -> Result<()>
where
    I: IntoIterator<Item = &'a K>,
    K: LogKey + 'a,
    S: Display,
{
    self::init::<K, S>(modname, l, f, o)?;
    add_submodules(modules)
}

/// Add a list of [`LogKey`]s to the a [`Scope`]
///
/// Add a set of [`LogKey`] implementors to their reserved [`Scope`]. The [`Scope`] must be
/// initialized before calling this function. The passed Iterator `I` must yield a reference
/// to a [`LogKey`] implementor.
///
/// This function is usually called inside the [`init()`] function of a [`Scope`] implementation
/// but is not limited to. It can be called at any time to add more modules to a scope.
///
/// The `LogKey`s receive the [`Level`], [`FacadeVariant`] and [`Options`] from the [`Scope`]
/// or can provide custom values via the [`LogKey`] trait implementation.
///
/// # Examples
///
/// ```rust
/// use hclog::{Scope, Level, FacadeVariant, options::Options, Result};
/// # use hclog::{LogKey, ContextKey};
///
/// #[derive(Copy, Clone)]
/// enum LogKeys { AA, AB, AC }
///
/// # impl std::fmt::Display for LogKeys {
/// #    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
/// #        match *self {
/// #            Self::AA => fmt.write_str("AA"),
/// #            Self::AB => fmt.write_str("AB"),
/// #            Self::AC => fmt.write_str("AC"),
/// #        }
/// #    }
/// # }
///
/// # impl LogKey for LogKeys {
/// #    fn log_key(&self) -> ContextKey { *self as ContextKey }
/// # }
///
/// impl Scope for LogKeys {
///     fn init<S: std::fmt::Display>(
///         name: S, level: Level, facade: FacadeVariant, options: Options
///     ) -> Result<()> {
///         hclog::init::<Self, S>(name, level, facade, options)?;
///         hclog::add_submodules(&[Self::AA, Self::AB, Self::AC])
///     }
/// }
/// ```
///
/// # Errors
///
/// Returns an error if:
/// * [ContextLock]: the internal context can't be accessed
/// * [ScopeNotInitialized]: the scope is not initialized
///
pub fn add_submodules<'a, K, I>(it: I) -> Result<()>
where
    K: LogKey + 'a,
    I: IntoIterator<Item = &'a K>
{
    let mut ctx = CTX::get_mut()?;
    for m in it.into_iter() {
        ctx.get_mod_mut(K::logscope())?.add_submodule(*m)?;
    }
    Ok(())
}

/// Set the log level for a list of modules.
///
/// The input `I` must be an iterator of string slices which are formatted as
/// `key:level,key:level,...`. The `key` is the name of the module and the `level`
/// is a valid [`Level`] as string.
///
/// The `key` can be the name of a known [`LogKey`] or `_all` to set the log level
/// for all available `LogKey`s in the current `Scope`. The `LogKey` and `Level` names
/// are case insensitive.
///
/// This function is primarily used for setting the log level at runtime via the commandline
/// or environment variables.
///
/// # Examples
///
/// ```rust
/// # use hclog_macros::HCLog;
/// // assume a call something like this:
/// // $ myapp _all:warn,ma:info,mb:debug10
///
/// # #[derive(Copy, Clone, HCLog)]
/// enum LogKeys { MA, MB }
///
/// fn main() {
///     // library initialization here - skipped for brevity
///     let args = std::env::args().skip(1).collect::<Vec<_>>();
///     match hclog::set_mod_level(&args) {
///         Ok(_) => (),
///         Err(e) => panic!("Error: {}", e),
///     }
/// }
/// ```
///
/// # Errors
///
/// Returns an error if:
/// * [ContextLock]: the internal context can't be accessed
/// * [ParseArg]: parsing the input fails
/// * [KeyNotInitialized]: the module is not initialized
/// * [UnknownLogLevel]: the log level is unknown
///
pub fn set_mod_level<'a, I, S>(it: I) -> Result<()>
where
    S: AsRef<str> + ?Sized + 'a,
    I: IntoIterator<Item = &'a S>
{
    let mut ctx = CTX::get_mut()?;
    for arg in it.into_iter().flat_map(|a| {
        a.as_ref().split(',').collect::<Vec<_>>()
    }) {
        let Some((module, level)) = arg.split_once(":") else {
            return Err(ParseArg);
        };
        if module.is_empty() || level.is_empty() {
            return Err(ParseArg);
        }
        let level = level.parse::<Level>()?;
        if module.eq_ignore_ascii_case("_all") {
            for logmod in ctx.logmods_mut() {
                for submod in logmod.submodules_mut() {
                    submod.set_logsev(level);
                }
            }
        } else {
            match ctx.get_submod_by_name(module) {
                None => return Err(KeyNotInitialized),
                Some(m) => m.set_logsev(level),
            };
        }
    }
    Ok(())
}

/// Check if a module is initialized in a given [`Scope`]
///
/// # Examples
///
/// ```rust
/// # use hclog_macros::HCLog;
///
/// # #[derive(Copy, Clone, HCLog)]
/// enum LogKeys {
///     A,
///     B,
///     # #[hclog(ignore)]
///     C,
/// }
///
/// # LogKeys::init_with_defaults("test").unwrap();
/// assert_eq!(hclog::has_module(LogKeys::A).unwrap(), true);
/// assert_eq!(hclog::has_module(LogKeys::B).unwrap(), true);
/// // assume C doesn't get initialized
/// assert_eq!(hclog::has_module(LogKeys::C).unwrap(), false);
/// ```
///
/// # Errors
///
/// Returns an Error if:
/// * [ContextLock]: the internal context can't be accessed
/// * [ScopeNotInitialized]: the scope is not initialized
///
pub fn has_module<K: LogKey>(k: K) -> Result<bool> {
    CTX::call(|ctx| { Ok(ctx.get_mod(K::logscope())?.has_submodule(k)) })
}

/// Set the log destination `FacadeVariant` for a given LogKey `K`
///
/// Alter the currently set [`FacadeVariant`] for a given LogKey at runtime. The LogKey `K` is an
/// initialized logkey as passed to the `init_modules` or `add_submodules` function.
/// The `FacadeVariant` can be any valid [`FacadeVariant`].
///
/// # Examples
///
/// ```rust
/// use hclog::FacadeVariant;
/// # use hclog_macros::HCLog;
///
/// # #[derive(Copy, Clone, HCLog)]
/// enum SomeKey { SL }
///
/// # SomeKey::init_with_defaults("test").unwrap();
/// hclog::set_logdest(SomeKey::SL, FacadeVariant::StdErr).unwrap();
/// ```
///
/// # Errors
///
/// Returns an Error if:
/// * the module is not initialized ([`ScopeNotInitialized`])
/// * the submodule is not initialized ([`KeyNotInitialized`])
/// * the context can't be accessed ([`ContextLock`])
pub fn set_logdest<K: LogKey>(k: K, facade: FacadeVariant) -> Result<()> {
    CTX::call_mut(|ctx| {
        ctx.get_mod_mut(K::logscope())?.get_submodule_mut(k).ok_or(KeyNotInitialized)?
            .set_logdest(&facade);
        Ok(())
    })
}

/// Set a `Level` for a single LogKey `K`
///
/// Alters the currently set [`Level`] for a given LogKey at runtime. The LogKey `K` is an
/// initialized logkey as passed to the `init_modules` or `add_submodules` function. The `Level`
/// can be any valid [`Level`].
///
/// # Examples
///
/// ```rust
/// use hclog::Level;
/// # use hclog_macros::HCLog;
///
/// # #[derive(Copy, Clone, HCLog)]
/// enum SomeKey { IM }
///
/// # SomeKey::init_with_defaults("test").unwrap();
/// hclog::set_level(SomeKey::IM, Level::Debug9).unwrap();
/// ```
///
/// # Errors
///
/// Returns an Error if:
/// * the module is not initialized ([`ScopeNotInitialized`])
/// * the submodule is not initialized ([`KeyNotInitialized`])
/// * the context can't be accessed ([`ContextLock`])
///
pub fn set_level<K: LogKey>(k: K, level: Level) -> Result<()> {
    CTX::call_mut(|ctx| {
        ctx.get_mod_mut(K::logscope())?.get_submodule_mut(k).ok_or(KeyNotInitialized)?
            .set_logsev(level);
        Ok(())
    })
}

/// Reset the options of a given LogKey `K`
///
/// This will reset the Options for a given `K` which implements the [`LogKey`] trait.
/// When reseting the options, the default options will be used and the environment variables
/// will be checked. This will not affect the log level or the log destination.
///
/// Resetting the options might be usefull when the options where changed at runtime for some
/// reason and you want restore the original state. Note that this will not restore the state
/// on initialization of the module. Instead the default is used and the environment variables
/// are respected.
///
/// # Examples
///
/// ```rust
/// # use hclog_macros::HCLog;
///
/// # #[derive(Copy, Clone, HCLog)]
/// enum Key { RM }
///
/// # Key::init_with_defaults("test").unwrap();
/// hclog::reset_module_options(Key::RM).unwrap();
/// ```
///
/// # Errors
///
/// Returns an Error if:
/// * the module is not initialized ([`ScopeNotInitialized`])
/// * the submodule is not initialized ([`KeyNotInitialized`])
/// * parsing the environment variables fails ([`ParseArg`])
/// * the context can't be accessed ([`ContextLock`])
///
pub fn reset_module_options<K: LogKey>(k: K) -> Result<()> {
    CTX::call_mut(|ctx| {
        ctx.get_mod_mut(K::logscope())?.get_submodule_mut(k).ok_or(KeyNotInitialized)?
            .reset_options()?;
        Ok(())
    })
}

/// Unset one or multiple [`Options`] for a given LogKey
///
/// The first argument must be a valid `K` which implements the [`LogKey`] trait.
///
/// The second argument is a list of [`Options`] which will be unset for the given `K`.
/// The options can be arithmetically combined using the `+` operator or, if using the
/// default [`Options`], unset via the `-` operator. For a list of valid Options see the
/// [`Options`] documentation.
///
/// # Examples
///
/// ```rust
/// use hclog::{ErrorKind, options::{LOGCOMPAT, TIMESTAMP}};
/// # use hclog_macros::HCLog;
///
/// # #[derive(Copy, Clone, HCLog)]
/// enum SomeKey { UM }
///
/// match hclog::unset_module_options(SomeKey::UM, LOGCOMPAT + TIMESTAMP) {
///    Ok(_) => (),
///    Err(e) if e == ErrorKind::ScopeNotInitialized => println!("Scope not initialized"),
///    Err(e) if e == ErrorKind::KeyNotInitialized => println!("Module not initialized"),
///    Err(_) => panic!("Unexpected error"),
/// }
/// ```
///
/// # Errors
///
/// Returns an Error if:
/// * the module is not initialized ([`ScopeNotInitialized`])
/// * the submodule is not initialized ([`KeyNotInitialized`])
/// * the context can't be accessed ([`ContextLock`])
///
pub fn unset_module_options<K: LogKey>(k: K, options: Options) -> Result<()> {
    CTX::call_mut(|ctx| {
        ctx.get_mod_mut(K::logscope())?.get_submodule_mut(k).ok_or(KeyNotInitialized)?
            .unset_options(options);
        Ok(())
    })
}

/// Set one or multiple [`Options`] for a given LogKey
///
/// The first argument must be a valid `K` which implements the [`LogKey`] trait.
///
/// The second argument is a list of [`Options`] which will be set for the given `K`.
/// The options can be arithmetically combined using the `+` operator or, if using the
/// default [`Options`], unset via the `-` operator. For a list of valid Options see the
/// [`Options`] documentation.
///
/// # Examples
///
/// ```rust
/// use hclog::{ErrorKind, options::{LOGCOMPAT, TIMESTAMP}};
/// # use hclog_macros::HCLog;
///
/// # #[derive(Copy, Clone, HCLog)]
/// enum Key { SM }
///
/// # Key::init_with_defaults("test").unwrap();
/// match hclog::set_module_options(Key::SM, LOGCOMPAT + TIMESTAMP) {
///    Ok(_) => (),
///    Err(e) if e == ErrorKind::ScopeNotInitialized => println!("Scope not initialized"),
///    Err(e) if e == ErrorKind::KeyNotInitialized => println!("Module not initialized"),
///    Err(_) => panic!("Unexpected error"),
/// }
/// ```
///
/// # Errors
///
/// Returns an Error if:
/// * the module is not initialized ([`ScopeNotInitialized`])
/// * the submodule is not initialized ([`KeyNotInitialized`])
/// * the context can't be accessed ([`ContextLock`])
pub fn set_module_options<K: LogKey>(k: K, options: Options) -> Result<()> {
    CTX::call_mut(|ctx| {
        ctx.get_mod_mut(K::logscope())?.get_submodule_mut(k).ok_or(KeyNotInitialized)?
            .set_options(options);
        Ok(())
    })
}

/*
 * Don't document this function. It's only used for internal by the macros
 */
#[doc(hidden)]
pub fn log<K: LogKey>(
    k: K, lvl: Level, file: &str, func: &str, line: u32, fmt: &Arguments
) -> Result<()> {
    CTX::call(|ctx| {
        let lm = ctx.get_mod(K::logscope())?;
        match lm.get_submodule(k) {
            Some(m) if m.will_log(lvl) => {
                let scope = lm.env();
                let ident = lm.env_ident();
                m.do_log(&lm.name(), scope, ident, lvl, file, func, line, fmt)
            }
            Some(_) => Ok(()),
            None => {
                lE!(Internal, "Module {} not initialized", k);
                Err(KeyNotInitialized)
            }
        }
    })
}

#[doc(hidden)]
pub fn test_log<K: LogKey>(k: K, lvl: Level) -> Result<bool> {
    CTX::call(|ctx| {
        match ctx.get_mod(K::logscope())?.get_submodule(k) {
            Some(m) => Ok(m.will_log(lvl)),
            None => Ok(false),
        }
    })
}

#[cfg(test)]
pub (crate) mod libtest {
    use serial_test::serial;
    use crate::{
        log_internal::test::TestKeys::{self, *},
        options::Options,
        Level::{self, *},
        Scope, Result, ErrorKind::*, FacadeVariant,
    };

    pub fn init_libtest_mod() -> Result<()> {
        TestKeys::init("libtest", Level::Debug9, FacadeVariant::StdOut,
                                Options::default())
    }

    #[test]
    #[serial]
    fn set_mod_level_no_module() {
        assert_eq!(crate::api::set_mod_level([":warn"]), Err(ParseArg));
    }

    #[test]
    #[serial]
    fn set_mod_level_no_level() {
        assert_eq!(crate::api::set_mod_level(["_all:"]), Err(ParseArg));
    }

    #[test]
    #[serial]
    fn set_mod_level_inval_mod_fail() {
        assert!(crate::api::set_mod_level(["libtestbaz:warn"]).is_err());
    }

    #[test]
    #[serial]
    fn set_mod_level_inval_level() {
        assert_eq!(crate::api::set_mod_level(["_all:warning"]), Err(UnknownLogLevel));
    }

    #[test]
    #[serial]
    fn set_mod_level_valid_level() {
        init_libtest_mod().unwrap();
        // wont panic - even with no mods initialized
        assert!(crate::api::set_mod_level(["_all:warn"]).is_ok());
        assert!(crate::api::set_mod_level(["_all:INFO"]).is_ok());
        assert!(crate::api::set_mod_level(["libtestfoo:warn,libtestbar:info"]).is_ok());
    }

    #[test]
    #[serial]
    fn set_mod_level_mods_initialized() {
        init_libtest_mod().unwrap();
        assert!(crate::api::set_mod_level(["_all:notice,libtestbar:warn"]).is_ok());
        assert_eq!(crate::api::test_log(LIBTESTFOO, Notice), Ok(true));
        assert_eq!(crate::api::test_log(LIBTESTBAR, Info), Ok(false));

        assert!(crate::api::set_mod_level(["libtestbar:emerg"]).is_ok());
        assert_eq!(crate::api::test_log(LIBTESTFOO, Notice), Ok(true));
        assert_eq!(crate::api::test_log(LIBTESTBAR, Emerg), Ok(true));

        // order is important - _all as last arg will overwrite prev sev
        assert!(crate::api::set_mod_level(["libtestfoo:debug9", "_all:debug10"]).is_ok());
        assert_eq!(crate::api::test_log(LIBTESTFOO, Debug10), Ok(true));
        assert_eq!(crate::api::test_log(LIBTESTBAR, Debug10), Ok(true));
    }
}
