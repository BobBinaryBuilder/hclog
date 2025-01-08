use crate::{level::Level, message::Message, Result, ErrorKind::*};
use libc::{self, c_int};
use strum_macros::Display;
use std::{
    ffi::CString,
    fmt::Debug,
    str::FromStr,
    sync::{Arc, Mutex},
    io::{BufWriter, Write},
    fs::File as StdFile,
    path::{Path, PathBuf},
};

/*
 * base trait describing how a log facade should work
 */
#[doc(hidden)]
pub trait LogFacade: Debug + Send + Sync {
    fn log(&self, level: Level, msg: Message) -> Result<()>;

    // helper
    fn is_syslog(&self) -> bool { false }
}

#[derive(Debug, Default, Display, Clone)]
/// Declaration of the different available log facacdes (log targets).
///
/// # Errors
///
/// When parsing a [`FacadeVariant`] from a string, the following errors can occur:
/// - If the string does not match any of the known variants, a [`String`] is returned with an error
///   message.
/// - If the string matches a known variant but the variant is not implemented, a [`String`] is
///   returned with an error message.
///
/// # Panics
///
/// There are some known panics that can occur when initializing or logging to a facade.
///
/// ## Panic on initialization
///
/// The [`Syslog`](FacadeVariant::Syslog) variant will panic if the given facility is not a valid
/// syslog facility.
///
/// The [`File`](FacadeVariant::File) variant will panic if the given filename is not a valid path
/// or opening the file fails.
///
/// ## Panic on logging
///
pub enum FacadeVariant {
    #[default]
    /// Logging is disabled
    None,
    /// Log to stdout
    ///
    /// _Note: This bypasses the OutputCapture of [`std::io::stdout`]_
    StdOut,
    /// Log to stderr
    ///
    /// _Note: This bypasses the OutputCapture of [`std::io::stderr`]_
    StdErr,
    /// Log to syslog
    ///
    /// The first argument is the syslog facility (e.g. "user")
    // we use owned strings as args to Syslog and File for the moment. Usually this should be
    // some kind of generic value to capture most common types. But making this enum generic
    // would clash with the "no generics in statics" rule because the FacadeVariant is also
    // part of the SubModule and Logmod which is held in static Context
    Syslog(String),
    /// Log to a file
    ///
    /// The first argument is the filename
    /// The second argument is a boolean that indicates whether the file should be truncated
    File(PathBuf, bool), // Filename, truncate-option
}

impl FromStr for FacadeVariant {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s.to_ascii_lowercase().as_str() {
            "none" => Ok(Self::None),
            "stdout" => Ok(Self::StdOut),
            "stderr" => Ok(Self::StdErr),
            "syslog" => Ok(Self::Syslog("user".to_string())),
            "file" => Ok(Self::File("/tmp/hclog.log".into(), false)),
            _ => Err(format!("Facade '{}' not exists or not implemented", s)),
        }
    }
}

/*
 * Both fields are currently the same but may change in the future
 */
#[derive(Clone, Debug)]
#[allow(dead_code)]
pub enum FacadeScope {
    None,
    Global(Arc<(dyn 'static + LogFacade + Send + Sync)>),
    Local(Arc<(dyn LogFacade + Send + Sync)>),
}
impl Default for FacadeScope {
    fn default() -> Self {
        Self::new(&FacadeVariant::None)
    }
}
#[allow(dead_code)]
impl FacadeScope {
    pub fn new(variant: &FacadeVariant) -> Self {
        match variant {
            FacadeVariant::None => Self::None,
            FacadeVariant::StdOut => Self::Global(Arc::new(self::StdOut::init())),
            FacadeVariant::StdErr => Self::Global(Arc::new(self::StdErr::init())),
            FacadeVariant::Syslog(s) => Self::Global(Arc::new(self::Syslog::init(s))),
            FacadeVariant::File(p, t) => Self::Global(Arc::new(self::File::init(p, *t))),
        }
    }
    pub fn to_local(&self) -> Option<Self> {
        match self {
            Self::Global(ref a) => Some(Self::Local(Arc::clone(a))),
            Self::None | Self::Local(_) => None,
        }
    }
    pub fn inner(&self) -> Option<&Arc<dyn LogFacade + Send + Sync>> {
        match *self {
            Self::None => None,
            Self::Global(ref f) | Self::Local(ref f) => Some(f),
        }
    }
}

// Log to stdout
#[derive(Debug)]
#[allow(dead_code)]
pub struct StdOut { handle: std::io::Stdout }
impl StdOut {
    fn init() -> Self { Self { handle: std::io::stdout() } }
}
impl LogFacade for StdOut {
    #[cfg(not(test))]
    fn log(&self, _lvl: Level, msg: Message) -> Result<()> {
        let mut handle = self.handle.lock();
        handle.write_all(msg.to_string().as_bytes())?;
        handle.write_all(b"\n")?;
        Ok(())
    }
    #[cfg(test)]
    fn log(&self, _lvl: Level, msg: Message) -> Result<()> {
        println!("{}", msg);
        Ok(())
    }
}

// Log to stderr
#[derive(Debug)]
pub struct StdErr { handle: std::io::Stderr }
impl StdErr {
    fn init() -> Self { Self { handle: std::io::stderr() }}
}
impl LogFacade for StdErr {
    fn log(&self, _lvl: Level, msg: Message) -> Result<()> {
        let mut handle = self.handle.lock();
        handle.write_all(msg.to_string().as_bytes())?;
        handle.write_all(b"\n")?;
        Ok(())
    }
}

// Log to Syslog
#[derive(Debug, Default, Clone)]
pub struct Syslog {
    facility: c_int,
}
impl Syslog {
    fn init(opt: &str) -> Self {
        let facility = match opt {
            "kern" => libc::LOG_KERN,
            "user" => libc::LOG_USER,
            "mail" => libc::LOG_MAIL,
            "daemon" => libc::LOG_DAEMON,
            "auth" => libc::LOG_AUTH,
            "syslog" => libc::LOG_SYSLOG,
            "lpr" => libc::LOG_LPR,
            "news" => libc::LOG_NEWS,
            "uucp" => libc::LOG_UUCP,
            "local0" => libc::LOG_LOCAL0,
            "local1" => libc::LOG_LOCAL1,
            "local2" => libc::LOG_LOCAL2,
            "local3" => libc::LOG_LOCAL3,
            "local4" => libc::LOG_LOCAL4,
            "local5" => libc::LOG_LOCAL5,
            "local6" => libc::LOG_LOCAL6,
            "local7" => libc::LOG_LOCAL7,
            _ => panic!("unknown syslog facility '{}'", opt),
        };
        Self { facility }
    }
}
impl LogFacade for Syslog {
    fn is_syslog(&self) -> bool {
        true
    }

    fn log(&self, level: Level, msg: Message) -> Result<()> {
        let lvl = match level {
            Level::Off => return Ok(()),
            Level::Emerg => libc::LOG_EMERG,
            Level::Alert => libc::LOG_ALERT,
            Level::Crit => libc::LOG_CRIT,
            Level::Error => libc::LOG_ERR,
            Level::Warn => libc::LOG_WARNING,
            Level::Notice => libc::LOG_NOTICE,
            Level::Info => libc::LOG_INFO,
            d if d >= Level::Debug1 && d <= Level::Debug10 => libc::LOG_DEBUG,
            _ => return Err(UnknownLogLevel),
        };
        let msg_raw = CString::new(msg.to_string())?;
        let fmt = CString::new("%s".to_owned())?;
        unsafe {
            libc::syslog(
                lvl | self.facility,
                fmt.as_ptr(),
                msg_raw.as_ptr(),
            )
        }
        Ok(())
    }
}

// Log to a file
#[derive(Debug)]
pub struct File {
    handle: Arc<Mutex<BufWriter<StdFile>>>,
}
impl File {
    fn init<P: AsRef<Path>>(path: P, truncate: bool) -> Self {
        let handle = std::fs::OpenOptions::new()
            .create(true)
            .append(!truncate)
            .truncate(truncate)
            .write(true)
            .open(path.as_ref())
            .expect("failed to open log file");
        Self { handle: Arc::new(Mutex::new(BufWriter::new(handle))) }
    }
}
impl LogFacade for File {
    fn log(&self, _: Level, msg: Message) -> Result<()> {
        let handle = Arc::clone(&self.handle);
        {
            let mut writer = handle.lock().unwrap();
            writer.write_all(msg.to_string().as_bytes())?;
            writer.write_all(b"\n")?;
            writer.flush()?;
        }
        Ok(())
    }
}
