use crate::{level::Level, logmod::ScopeEnv, options::*};
use chrono::{DateTime, Utc};
use std::{
    fmt::{self, Display, Debug, Arguments},
    borrow::Cow,
    process,
    thread,
};

#[derive(Debug)]
pub struct Message<'a> {
    options: &'a Options,
    time: DateTime<Utc>,
    binname: &'a str,
    severity: Option<&'a Level>,
    modname: Option<Cow<'a, str>>,
    scope: Option<&'a ScopeEnv>,
    scope_ident: Option<&'a str>,
    file: &'a str,
    func: &'a str,
    line: u32,
    fmt: Cow<'a, str>,
}
/*
 * NOTE: fmt::Display is always buffered and options.line_buffered = false is not implemented.
 * To implement we would need to use std::io::Write and a custom (or different) formatter."
 */
impl<'a> Display for Message<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.options.has(DATESTAMP) {
            write!(f, "{} ", self.time.format("%F"))?;
        }
        if self.options.has(TIMESTAMP) {
            match self.options.has(NANOSEC) {
                true => write!(f, "{} ", self.time.format("%X.%f"))?,
                false => write!(f, "{} ", self.time.format("%X"))?,
            };
        }
        if self.options.has(BINNAME) {
            write!(f, "{}", self.binname)?;
        }
        if self.options.has(PID) {
            if self.options.has(TID) {
                write!(f, "[{}/{}] ", process::id(), Self::get_current_thread_id())?;
            } else {
                write!(f, "[{}] ", process::id())?;
            }
        } else if self.options.has(TID) {
            write!(f, "[{}] ", Self::get_current_thread_id())?;
        } else if self.options.has(BINNAME) {
                write!(f, " ")?;
        }

        if let Some(s) = self.severity {
            write!(f, "{} ", s)?;
        }
        if let Some(ref m) = self.modname {
            write!(f, "{} ", m)?;
        }
        if let Some(s) = self.scope {
            if let Some(ref i) = self.scope_ident {
                write!(f, "{}[{}] ", s, i)?;
            } else {
                write!(f, "{} ", s)?;
            }
        }
        // file=false implicitly also disables line
        if self.options.has(FILE) {
            if self.options.has(LINE) {
                write!(f, "{}:{} ", self.file, self.line)?;
            } else {
                write!(f, "{} ", self.file)?;
            }
        }
        if self.options.has(FUNC) {
            if !self.func.is_empty() {
                write!(f, "{} ", self.func)?;
            }
        }

        write!(f, "{}", self.fmt)
    }
}
impl<'a> Message<'a> {
    pub (crate) fn new(
        options: &'a Options, binname: &'a str, file: &'a str, func: &'a str,
        line: u32, fmt: &'a Arguments,
    ) -> Self {
        let fmt = match fmt.as_str() {
            Some(s) => Cow::Borrowed(s),
            None => Cow::Owned(fmt.to_string()),
        };
        Self {
            options,
            time: Utc::now(),
            binname,
            severity: None,
            modname: None,
            scope: None,
            scope_ident: None,
            file,
            func,
            line,
            fmt,
        }
    }
    // those fields are set by the module (if set in config)
    pub (crate) fn set_severity(&mut self, lvl: &'a Level) -> &mut Self {
        if self.options.has(SEVERITY) {
            self.severity = Some(lvl);
        }
        self
    }
    pub (crate) fn set_modname(&mut self, name: &'a str) -> &mut Self {
        if self.options.has(MODULE) {
            self.modname = Some(Cow::Borrowed(name));
        }
        self
    }
    pub (crate) fn set_scope(&mut self, scope: &'a ScopeEnv, ident: Option<&'a str>) -> &mut Self {
        if self.options.has(SCOPE) {
            self.scope = Some(scope);
            self.scope_ident = ident;
        }
        self
    }

    /*
     * ThreadId has no display, to_str or to_u64 method. Since str::parse would
     * fail if we not strip the ThreadId() part from the debug output we have to
     * remove this beforehand
     * Will be obsolete once https://github.com/rust-lang/rust/issues/67939 is
     * stable and available in our rust version
     */
    fn get_current_thread_id() -> u64 {
        let tid = format!("{:?}", thread::current().id());
        let tid = tid.strip_prefix("ThreadId(").unwrap_or(&tid);
        let tid = tid.strip_suffix(")").unwrap_or(tid);
        tid.parse::<u64>().unwrap()
    }
}

