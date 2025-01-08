use strum_macros::Display;
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

#[cfg(doctest)]
use hclog_macros::HCLog;

use crate::ErrorKind;

/// Declaration of available Log Levels which describe the importance of a log message.
///
/// The basic levels are based on the syslog severity levels. Instead of a single DEBUG level,
/// as described in syslog, the crate provides 10 different debug levels from
/// [`Debug1`](crate::Level::Debug1) to [`Debug10`](crate::Level::Debug10) to allow a more
/// fine-grained control over the verbosity of the log messages.
///
/// # Examples
///
/// ```
/// use hclog::Level;
/// # use hclog_macros::HCLog;
///
/// // the default level is 'off' - if no level is set, logging is disabled
///
/// # #[derive(HCLog, Copy, Clone)]
/// enum LogKeys {
///    Foo,
///    Bar,
/// }
///
/// fn main() {
///    # LogKeys::init_with_defaults("test").unwrap();
///    // set loglevel of LogKeys::Foo to Debug1
///     hclog::set_level(LogKeys::Foo, Level::Debug1).unwrap();
/// }
/// ```
///
/// for more Informations see [`set_level`](fn@crate::set_level)
#[derive(Copy, Clone, Debug, Default, Display, EnumIter, Hash, Eq, PartialEq, Ord, PartialOrd)]
#[strum(serialize_all = "lowercase")]
pub enum Level {
    #[default]  /* logging is disabled by default */
    /// Logging is disabled
    Off = 0, /* no logging */
    /// The "Emergency" level
    ///
    /// System is unstable
    Emerg,
    /// The Alert level
    ///
    /// Action must be taken immediately
    Alert,
    /// The Critical level
    ///
    /// Critical conditions
    Crit,
    /// The Error level
    ///
    /// Error conditions
    Error,
    /// The Warning level
    ///
    /// Warning conditions
    Warn,
    /// The Notice level
    ///
    /// Normal but significant condition
    Notice,
    /// The Information level
    ///
    /// Informational messages
    Info,
    /// The Debug1 level
    ///
    /// Debug messages with the highest significance/lowest verbosity
    Debug1,
    /// The Debug2 level
    Debug2,
    /// The Debug3 level
    Debug3,
    /// The Debug4 level
    Debug4,
    /// The Debug5 level
    Debug5,
    /// The Debug6 level
    Debug6,
    /// The Debug7 level
    Debug7,
    /// The Debug8 level
    Debug8,
    /// The Debug9 level
    Debug9,
    /// The Debug10 level
    ///
    /// Debug messages with the lowest significance/most verbosity
    Debug10,
}

/// Parses a string to a [`Level`]
///
/// The string is compared case-insensitive to the available [`Level`]s.
///
/// # Examples
/// ```
/// use hclog::Level;
///
/// fn main() {
///     let level = "info".parse::<Level>().unwrap();
///     assert_eq!(level, Level::Info);
/// }
/// ```
///
/// # Errors
///
/// If the given string does not match any of the available [`Level`]s, an
/// [`ErrorKind::UnknownLogLevel`] is returned.
impl std::str::FromStr for Level {
    type Err = ErrorKind;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match Level::iter().find(|r| r.to_string().eq_ignore_ascii_case(s)) {
            Some(l) => Ok(l),
            None => Err(ErrorKind::UnknownLogLevel),
        }
    }
}

#[doc(hidden)]
// don't document this public function
// they are only used for internal purposes and should not be used by the user
impl Level {
    #[inline(always)]
    pub fn max() -> Self {
        Self::Debug10
    }

    #[inline(always)]
    pub fn min() -> Self {
        Self::Off
    }

    #[inline(always)]
    pub fn is_enabled(&self, other: Self) -> bool {
        other <= *self
    }

    pub fn debug_level(id: u8) -> Self {
        if id == 0 {
            Self::Off
        } else {
            Level::iter().nth(Self::Info as usize + id as usize).unwrap_or(Self::Debug10)
        }
    }
}

#[cfg(test)]
mod level_tests {
    use crate::level::Level;
    use crate::ErrorKind::UnknownLogLevel;
    use strum::IntoEnumIterator;

    #[test]
    fn test_level_from_str() {
        assert_eq!("".parse::<Level>(), Err(UnknownLogLevel));
        assert_eq!("none".parse::<Level>(), Err(UnknownLogLevel));
        assert_eq!("Emerg".parse::<Level>(), Ok(Level::Emerg));
        assert_eq!("aLeRT".parse::<Level>(), Ok(Level::Alert));
        assert_eq!("CRIT".parse::<Level>(), Ok(Level::Crit));
        assert_eq!("error".parse::<Level>(), Ok(Level::Error));
        assert_eq!("warn".parse::<Level>(), Ok(Level::Warn));
        assert_eq!("notice".parse::<Level>(), Ok(Level::Notice));
        assert_eq!("info".parse::<Level>(), Ok(Level::Info));
        assert_eq!("debug1".parse::<Level>(), Ok(Level::Debug1));
        assert_eq!("debug2".parse::<Level>(), Ok(Level::Debug2));
        assert_eq!("debug3".parse::<Level>(), Ok(Level::Debug3));
        assert_eq!("debug4".parse::<Level>(), Ok(Level::Debug4));
        assert_eq!("debug5".parse::<Level>(), Ok(Level::Debug5));
        assert_eq!("debug6".parse::<Level>(), Ok(Level::Debug6));
        assert_eq!("debug7".parse::<Level>(), Ok(Level::Debug7));
        assert_eq!("debug8".parse::<Level>(), Ok(Level::Debug8));
        assert_eq!("debug9".parse::<Level>(), Ok(Level::Debug9));
        assert_eq!("debug10".parse::<Level>(), Ok(Level::Debug10));
        assert_eq!("debug11".parse::<Level>(), Err(UnknownLogLevel));
    }
    #[test]
    fn test_level_to_str() {
        assert_eq!(Level::Off.to_string(), "off");
        assert_eq!(Level::Emerg.to_string(), "emerg");
        assert_eq!(Level::Alert.to_string(), "alert");
        assert_eq!(Level::Crit.to_string(), "crit");
        assert_eq!(Level::Error.to_string(), "error");
        assert_eq!(Level::Warn.to_string(), "warn");
        assert_eq!(Level::Notice.to_string(), "notice");
        assert_eq!(Level::Info.to_string(), "info");
        assert_eq!(Level::Debug1.to_string(), "debug1");
        assert_eq!(Level::Debug2.to_string(), "debug2");
        assert_eq!(Level::Debug3.to_string(), "debug3");
        assert_eq!(Level::Debug4.to_string(), "debug4");
        assert_eq!(Level::Debug5.to_string(), "debug5");
        assert_eq!(Level::Debug6.to_string(), "debug6");
        assert_eq!(Level::Debug7.to_string(), "debug7");
        assert_eq!(Level::Debug8.to_string(), "debug8");
        assert_eq!(Level::Debug9.to_string(), "debug9");
        assert_eq!(Level::Debug10.to_string(), "debug10");
    }
    #[test]
    fn test_level_from_usize() {
        assert_eq!(Level::iter().nth(1).unwrap_or(Level::Off), Level::Emerg);
        assert_eq!(Level::iter().nth(6).unwrap_or(Level::Off), Level::Notice);
        assert_eq!(Level::iter().nth(42).unwrap_or(Level::Off), Level::Off);
    }
}
