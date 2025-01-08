//! Available Options to configure the log output of this crate
//!
//! Each logging output can be configured using constants defined in this module.
//!
//! The Options struct is a bitfield struct that can be used to set the format of the log messages.
//! It is usually passed to the [`hclog_init`](trait@crate::Scope) function but can also
//! modified at runtime via the [`set_module_options`](fn@crate::set_module_options),
//! [`unset_module_options`](fn@crate::unset_module_options) and
//! [`reset_module_options`](fn@crate::reset_module_options) functions.
//!
use crate::{Result, util::read_var_from_env};
use std::ops::{Add, AddAssign, Sub, SubAssign};
/*
 * known environment variables
 *
 * This variables can be used to alter or define the logging behaviour
 *
 * The syntax for HCLOG_FACADE and HCLOG_LEVEL is always:
 *      HCLOG_ENVVAR="<module>:<value>"
 * where <module> is the module to set <value> for. To set a <value> for
 * all known Modules in Context _all can be used.
 * <value> is restricted to known values of the addressed config (see level.rs
 * or facades.rs)
 */
pub (crate) const ENV_OPT_PREFIX: &str = "HCLOG_OPT_";
pub (crate) const ENV_OPT_FACADE: &str = "HCLOG_FACADE";
pub (crate) const ENV_OPT_LEVEL: &str = "HCLOG_LEVEL";

/*
 * Output format options:
 *
 * Formatting directives describe how the log format looks like. For a consistent
 * log format most formatting options may only be set by the application itself
 * and passed to the Module via hclog::init_module().
 *
 * Messages sent to syslog are not affected by 'timestamp', 'datestamp', 'nanosec',
 * 'severity', 'pid' and 'tid'. The lib always excludes these properties from all
 * messages going to syslog. Use your syslog config to switch those properties on and off.
 *
 */
/// No options set at all
///
/// If [`NONE`] is set the log message will be printed without any additional information.
pub const NONE: Options = Options(0x0000);
/// Log messages are written line buffered
pub const LINEBUFFERED: Options = Options(0x0001);
/// Log messages are prefixed with a timestamp
pub const TIMESTAMP: Options = Options(0x0002);
/// Log messages are prefixed with a datestamp
pub const DATESTAMP: Options = Options(0x0004);
/// Log messages timestamp is suffixed with nanoseconds
///
/// If the TIMESTAMP option is not set this option won't have any effect
pub const NANOSEC: Options = Options(0x0008);
/// Log messages include with binary name as set on library initialization
///
/// _NOTE: The binary name is <b>not</b> detected via the [`current_exe`](std::env::current_exe) function
/// but set on library initialization and can not be changed at runtime._
pub const BINNAME: Options = Options(0x0010);
/// Log messages are prefixed with the process id
pub const PID: Options = Options(0x0020);
/// Log messages are prefixed with the thread id
pub const TID: Options = Options(0x0040);
/// Log messages are prefixed with the module name
pub const MODULE: Options = Options(0x0080);
/// Log messages are prefixed with the severity level
pub const SEVERITY: Options = Options(0x0100);
/// Log messages are prefixed with the scope
pub const SCOPE: Options = Options(0x0200);
/// Log messages are prefixed with the function name
pub const FUNC: Options = Options(0x0400);
/// Log messages are prefixed with the file name
pub const FILE: Options = Options(0x0800);
/// Log messages are prefixed with the line number
///
/// If the FILE is not set this option won't have any effect
pub const LINE: Options = Options(0x1000);
/// Enable compatibility with the log crate
pub const LOGCOMPAT: Options = Options(0x2000);
/// Match the exact level when logging a message
///
/// When EXACT_LVL_MATCH is set a log message is only printed if it matches the level exactly
/// as set on library initialization. If the level is set to `INFO` only messages with the
/// level `INFO` are printed.
pub const EXACT_LVL_MATCH: Options = Options(0x4000);

#[allow(clippy::suspicious_arithmetic_impl)]
impl Add for Options {
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        Self(self.0 | rhs.0)
    }
}
impl Sub for Options {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self {
        Self(self.0 & !rhs.0)
    }
}
impl AddAssign for Options {
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs;
    }
}
impl SubAssign for Options {
    fn sub_assign(&mut self, rhs: Self) {
        *self = *self - rhs;
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
/// Configuration options for the log messages
///
/// [Options] is a bitfield struct that can be used to set the format of the log messages.
/// It is usually passed to the [`hclog_init`](trait@crate::Scope) function but can also
/// modified at runtime via the [`set_module_options`](fn@crate::set_module_options),
/// [`unset_module_options`](fn@crate::unset_module_options) and
/// [`reset_module_options`](fn@crate::reset_module_options) functions.
///
/// An Option struct can be created with the [`Options::new`](fn@crate::Options::new) function and
/// manipulated using the arithmetic operators `+` and `-`. Eventhough the Options struct is a
/// bitfield internaly we decided to use the arithmetic operators because its actually _adding_
/// (`+`) or _removing_ (`-`) options. This makes the code more readable and easier to understand.
///
/// All Options can also be set via environment variables. The environment variables are prefixed
/// with `HCLOG_OPT_` and the name of the option in uppercase. For example to set the `LINEBUFFERED`
/// option via environment variable you would set `HCLOG_OPT_LINEBUFFERED=1`. To unset the option
/// you would set `HCLOG_OPT_LINEBUFFERED=0`.
///
/// The default options are:
/// [`LINEBUFFERED`](const@crate::LINEBUFFERED), [`TIMESTAMP`](const@crate::TIMESTAMP),
/// [`DATESTAMP`](const@crate::DATESTAMP), [`NANOSEC`](const@crate::NANOSEC),
/// [`BINNAME`](const@crate::BINNAME), [`PID`](const@crate::PID), [`TID`](const@crate::TID),
/// [`MODULE`](const@crate::MODULE), [`SEVERITY`](const@crate::SEVERITY), [`FUNC`](const@crate::FUNC),
/// [`FILE`](const@crate::FILE), [`LINE`](const@crate::LINE) and [`LOGCOMPAT`](const@crate::LOGCOMPAT).
///
/// If the [`Syslog`](enum@crate::FacadeVariant#variant.Syslog) facade is used the options `TIMESTAMP`,
/// `DATESTAMP`, `NANOSEC`, `BINNAME`, `PID` and `SEVERITY` are ignored and not printed because
/// they are set by the syslog daemon.
///
/// # Available options:
///
/// * [`LINEBUFFERED`](const@crate::LINEBUFFERED): log messages are written line buffered
/// * [`TIMESTAMP`](const@crate::TIMESTAMP): log messages are prefixed with a timestamp
/// * [`DATESTAMP`](const@crate::DATESTAMP): log messages are prefixed with a datestamp
/// * [`NANOSEC`](const@crate::NANOSEC): log messages are prefixed with nanoseconds
/// * [`BINNAME`](const@crate::BINNAME): log messages are prefixed with the binary name
/// * [`PID`](const@crate::PID): log messages are prefixed with the process id
/// * [`TID`](const@crate::TID): log messages are prefixed with the thread id
/// * [`MODULE`](const@crate::MODULE): log messages are prefixed with the module name
/// * [`SEVERITY`](const@crate::SEVERITY): log messages are prefixed with the severity level
/// * [`SCOPE`](const@crate::SCOPE): log messages are prefixed with the scope
/// * [`FUNC`](const@crate::FUNC): log messages are prefixed with the function name
/// * [`FILE`](const@crate::FILE): log messages are prefixed with the file name
/// * [`LINE`](const@crate::LINE): log messages are prefixed with the line number
/// * [`LOGCOMPAT`](const@crate::LOGCOMPAT): enable compatibility with the log crate
/// * [`EXACT_LVL_MATCH`](const@crate::EXACT_LVL_MATCH): log messages are prefixed with the exact level match
///
pub struct Options(u16);
impl Default for Options {
    fn default() -> Self {
        LINEBUFFERED + TIMESTAMP + DATESTAMP + NANOSEC + BINNAME +
        PID + TID + MODULE + SEVERITY + FUNC + FILE + LINE + LOGCOMPAT
    }
}
impl std::fmt::Debug for Options {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("[")?;
        if self.has(LINEBUFFERED) { f.write_str("LINEBUFFERED, ")?; }
        if self.has(TIMESTAMP) { f.write_str("TIMESTAMP, ")?; }
        if self.has(DATESTAMP) { f.write_str("DATESTAMP, ")?; }
        if self.has(NANOSEC) { f.write_str("NANOSEC, ")?; }
        if self.has(BINNAME) { f.write_str("BINNAME, ")?; }
        if self.has(PID) { f.write_str("PID, ")?; }
        if self.has(TID) { f.write_str("TID, ")?; }
        if self.has(MODULE) { f.write_str("MODULE, ")?; }
        if self.has(SEVERITY) { f.write_str("SEVERITY, ")?; }
        if self.has(SCOPE) { f.write_str("SCOPE, ")?; }
        if self.has(FUNC) { f.write_str("FUNC, ")?; }
        if self.has(FILE) { f.write_str("FILE, ")?; }
        if self.has(LINE) { f.write_str("LINE, ")?; }
        if self.has(LOGCOMPAT) { f.write_str("LOGCOMPAT, ")?; }
        if self.has(EXACT_LVL_MATCH) { f.write_str("EXACT_LVL_MATCH, ")?; }
        f.write_str("]")?;
        Ok(())
    }
}
impl Options {
    /// Create a new Options struct with no options set
    ///
    /// This function does not respect the environment variables and will always return an empty
    /// Options struct.
    ///
    /// # Example
    ///
    /// ```
    /// use hclog::options::Options;
    ///
    /// let opts = Options::new();
    /// ```
    pub fn new() -> Self {
        NONE
    }

    /// Check if Options contains a specific flag. Returns `true` if the flag is set
    /// and `false` otherwise.
    ///
    /// # Example
    /// ```
    /// use hclog::options::{Options, LINEBUFFERED};
    ///
    /// let opts = Options::new();
    /// assert_eq!(opts.has(LINEBUFFERED), false);
    /// ```
    #[inline(always)]
    pub fn has(&self, flags: Options) -> bool {
        self.0 & flags.0 == flags.0
    }
    /// Set a specific flag in the [`Options`] struct. This won't disable the flag if it is already
    /// set.
    ///
    /// There is actually no need to call this function because the `+` operator is overloaded
    /// for the Options struct. This function will change the Options struct as referenced by
    /// `&mut self` but not the library internal options.
    /// See [`set_module_options`](fn@crate::set_module_options) instead.
    ///
    /// # Example
    /// ```rust
    /// use hclog::options::{Options, LINEBUFFERED};
    ///
    /// let mut opts = Options::new();
    /// opts.set(LINEBUFFERED);
    /// assert_eq!(opts.has(LINEBUFFERED), true);
    /// ```
    #[inline(always)]
    pub fn set(&mut self, flags: Options) {
        *self += flags;
    }

    /// Unset a specific flag in the Options struct. This won't enable the flag if it is already
    /// unset.
    ///
    /// There is actually no need to call this function because the `-` operator is overloaded
    /// for the Options struct. This function will change the Options struct as referenced by
    /// `&mut self` but not the library internal options.
    /// See [`unset_module_options`](fn@crate::unset_module_options) instead.
    ///
    /// # Example
    /// ```rust
    /// use hclog::options::{Options, LINEBUFFERED};
    ///
    /// let mut opts = Options::new() + LINEBUFFERED;
    /// assert_eq!(opts.has(LINEBUFFERED), true);
    /// opts.unset(LINEBUFFERED);
    /// assert_eq!(opts.has(LINEBUFFERED), false);
    /// ```
    #[inline(always)]
    pub fn unset(&mut self, flags: Options) {
        *self -= flags;
    }
    /// Reset the Options struct to the default values. This functions respects the process
    /// environment variables and will set the options according to the environment variables.
    /// If the environment variables are not set the default values are used.
    ///
    /// This function won't change the library internal options.
    /// See [`reset_module_options`](fn@crate::reset_module_options) instead.
    ///
    /// # Example
    /// ```rust
    /// use hclog::options::{Options, LINEBUFFERED};
    ///
    /// let mut opts = Options::default() - LINEBUFFERED;
    /// assert_eq!(opts.has(LINEBUFFERED), false);
    /// let _ = opts.reset();
    /// assert!(opts.has(LINEBUFFERED));
    /// ```
    pub fn reset(&mut self) -> Result<&Self> {
        *self = Self::default();
        self.parse_from_env()
    }
    #[doc(hidden)]
    pub fn for_syslog(&mut self) {
        *self -= TIMESTAMP + DATESTAMP + NANOSEC + BINNAME + PID + SEVERITY;
    }

    #[doc(hidden)]
    fn opt_from_env(&mut self, key: &str, var: Options) -> Result<()> {
        let envvar = format!("{}{}", ENV_OPT_PREFIX, key);
        match read_var_from_env::<u16>(&envvar) {
            Ok(v) => match v {
                Some(0) => *self -= var,
                Some(1) => *self += var,
                Some(_) | None => (),
            }
            Err(e) => return Err(e),
        }
        Ok(())
    }

    #[doc(hidden)]
    pub fn parse_from_env(&mut self) -> Result<&Self> {
        self.opt_from_env("LINEBUFFERED", LINEBUFFERED)?;
        self.opt_from_env("TIMESTAMP", TIMESTAMP)?;
        self.opt_from_env("DATESTAMP", DATESTAMP)?;
        self.opt_from_env("NANOSEC", NANOSEC)?;
        self.opt_from_env("BINNAME", BINNAME)?;
        self.opt_from_env("PID", PID)?;
        self.opt_from_env("TID", TID)?;
        self.opt_from_env("MODULE", MODULE)?;
        self.opt_from_env("SEVERITY", SEVERITY)?;
        self.opt_from_env("SCOPE", SCOPE)?;
        self.opt_from_env("FUNC", FUNC)?;
        self.opt_from_env("FILE", FILE)?;
        self.opt_from_env("LINE", LINE)?;
        self.opt_from_env("LOG_COMPAT", LOGCOMPAT)?;
        self.opt_from_env("EXACT_LVL_MATCH", EXACT_LVL_MATCH)?;
        Ok(self)
    }
}

/* tests below */
#[cfg(test)]
mod options_tests {
    use super::*;

    #[test]
    fn read_from_env() {
        std::env::set_var("FOO_BAR", "1");
        assert!(read_var_from_env::<i32>("FOO_BAR").is_ok());
        std::env::set_var("FOO_BAR", "BAZ");
        assert!(read_var_from_env::<i32>("FOO_BAR").is_err());
        std::env::set_var("FOO_BAR", "1");
        assert_eq!(read_var_from_env::<i32>("FOO_BAR").unwrap().unwrap(), 1i32);
        std::env::set_var("FOO_BAR", "BAZ");
        assert_eq!(read_var_from_env::<String>("FOO_BAR").unwrap().unwrap(), "BAZ".to_owned())
    }

   #[test]
    fn from_env_default() {
        let mut default = Options::new();
        assert!(default.has(NONE));

        std::env::set_var("HCLOG_OPT_FUNC", "1");
        default.parse_from_env().unwrap();
        assert!(default.has(FUNC));

        std::env::set_var("HCLOG_OPT_DATESTAMP", "1");
        default.parse_from_env().unwrap();
        assert!(default.has(DATESTAMP));

        std::env::set_var("HCLOG_OPT_FUNC", "0");
        default.parse_from_env().unwrap();
        assert!(!default.has(FUNC));

        default.reset().unwrap();
        assert!(default.has(NONE));
    }

    #[test]
    fn from_env_new() {
        let new = Options::default();
        assert!(!new.has(SCOPE));
        assert!(new.has(TID));
        assert!(new.has(LINEBUFFERED));
        assert!(new.has(TIMESTAMP));
        assert!(new.has(DATESTAMP));
    }

    #[test]
    fn syslog_fields() {
        let mut syslog = Options::default() + SCOPE;
        syslog.for_syslog();
        assert!(!syslog.has(TIMESTAMP));
        assert!(!syslog.has(DATESTAMP));
        assert!(!syslog.has(NANOSEC));
        assert!(!syslog.has(BINNAME));
        assert!(!syslog.has(PID));
        assert!(!syslog.has(SEVERITY));
        assert!(syslog.has(TID));
        assert!(syslog.has(MODULE));
        assert!(syslog.has(SCOPE));
        assert!(syslog.has(FILE));
        assert!(syslog.has(FUNC));
        assert!(syslog.has(LINE));
    }
}
