use crate::{
    facades::{FacadeScope, FacadeVariant},
    logmod::ScopeEnv,
    level::Level,
    message::Message,
    log_internal::InternalLogKeys::Internal,
    options::*,
    Result, ContextKey, LogKey,
};
use std::fmt::{self, Debug, Display, Arguments};

#[derive(Debug, Clone)]
pub (crate) struct Submodule {
    key: ContextKey,
    name: String,
    options: Options,
    initialized: bool,
    logsev: Level,
    logdest: FacadeScope,
}
impl Display for Submodule {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("{}", self.name))
    }
}
impl Default for Submodule {
    fn default() -> Self {
        Self {
            key: Internal.log_key().to_owned(),
            name: Internal.to_string(),
            options: Options::default(),
            initialized: false,
            logsev: Level::default(),
            logdest: FacadeScope::None,
        }
    }
}
impl Submodule {
    pub fn new(key: impl LogKey, logsev: Level, f: &FacadeVariant, options: Options) -> Self {
        let mut global = Self {
            key: key.log_key().to_owned(),
            name: key.to_string(),
            options,
            initialized: true,
            logsev,
            logdest: FacadeScope::new(f),
        };
        global.set_options_internal();
        global
    }
    fn set_options_internal(&mut self) -> &mut Self {
        if let Some(d) = self.logdest.inner() {
            if d.is_syslog() {
                // disable all fields already contained in syslog msgs
                self.options.for_syslog();
            }
        }
        self
    }
    #[inline]
    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn initialized(&self) -> bool {
        self.initialized
    }
    pub fn key(&self) -> ContextKey {
        self.key
    }
    pub fn set_logsev(&mut self, logsev: Level) -> &mut Self {
        self.logsev = logsev;
        self
    }
    pub fn set_logdest(&mut self, variant: &FacadeVariant) -> &mut Self {
        self.logdest = FacadeScope::new(variant);
        self
    }
    pub fn reset_options(&mut self) -> Result<&mut Self> {
        self.options.reset()?;
        Ok(self.set_options_internal())
    }
    pub fn set_options(&mut self, flags: Options) -> &mut Self {
        self.options.set(flags);
        self.set_options_internal()
    }
    pub fn unset_options(&mut self, flags: Options) -> &mut Self {
        self.options.unset(flags);
        self.set_options_internal()
    }
    pub fn will_log(&self, logsev: Level) -> bool {
        if self.options.has(EXACT_LVL_MATCH) {
            self.logsev == logsev
        } else {
            self.logsev.is_enabled(logsev)
        }
    }
    pub fn do_log(
        &self, cratename: &str, scope: ScopeEnv, scope_ident: Option<&str>,
        lvl: Level, file: &str, func: &str, line: u32, fmt: &Arguments,
    ) -> Result<()> {
        let logdest = match self.logdest.inner() {
            None => return Ok(()),
            Some(f) => f,
        };
        let opts = &self.options;
        let mut msg = Message::new(opts, cratename, file, func, line, fmt);
        msg.set_severity(&lvl);
        msg.set_modname(&self.name);
        msg.set_scope(&scope, scope_ident);
        logdest.log(self.logsev, msg)
    }
}

#[cfg(test)]
mod submodule_test {
    use crate::{
        level::Level,
        submodule::Submodule,
        options::EXACT_LVL_MATCH,
    };

    #[test]
    fn test_will_log() {
        let mut logmod = Submodule::default();
        logmod.set_logsev(Level::Info);
        assert_eq!(logmod.will_log(Level::Emerg), true);
        assert_eq!(logmod.will_log(Level::Error), true);
        assert_eq!(logmod.will_log(Level::Info), true);
        assert_eq!(logmod.will_log(Level::Debug1), false);
        logmod.set_logsev(Level::Off);
        assert_eq!(logmod.will_log(Level::Alert), false);
        assert_eq!(logmod.will_log(Level::Debug5), false);
    }

    #[test]
    fn exact_level_match() {
        let mut submod = Submodule::default();
        submod.set_logsev(Level::Info);
        submod.set_options(EXACT_LVL_MATCH);

        assert!(submod.options.has(EXACT_LVL_MATCH));
        assert_eq!(submod.will_log(Level::Emerg), false);
        assert_eq!(submod.will_log(Level::Error), false);
        assert_eq!(submod.will_log(Level::Info), true);
        assert_eq!(submod.will_log(Level::Debug1), false);
        submod.unset_options(EXACT_LVL_MATCH);
        assert_eq!(submod.will_log(Level::Crit), true);
        assert_eq!(submod.will_log(Level::Debug1), false);
    }
}
