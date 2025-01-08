#![allow(unused_variables)] // TMP

use crate::{
    options::{self, Options},
    submodule::Submodule,
    facades::FacadeVariant,
    level::Level,
    ErrorKind::ScopeNotInitialized,
    Scope, LogKey, Result,
    util::read_var_from_env,
};
use std::{
    vec::Vec,
    ops::{Index, IndexMut},
    fmt::{self, Display},
};

#[derive(Copy, Clone, Debug, Default, Hash, Eq, PartialEq, Ord, PartialOrd)]
#[allow(clippy::upper_case_acronyms)]
/// Identifier of the `Scope`
///
/// This identifier is used to distinguish between different scopes in the
/// Context. Its value is unique and used to index the `Context`'s `Vec<Scope>`.
///
/// _Note: This enum is subjected to change in the future because it does not
/// allow for dynamic scope creation. This is a limitation that should be
/// addressed in the future._
pub enum ScopeKey {
    /// Application scope (default)
    ///
    /// This scope is used for the main application and is the default scope.
    /// It should be used for the main application and its submodules. Any other
    /// scope should be used for libraries or other parts of the application.
    #[default]
    Application = 0,
    /// Internal Logging scope
    ///
    /// This is a reserved scope for `hclog` internal logging which can be enabled
    /// with HCLOG_DEBUG=1. It is not recommended to use this scope for any other
    /// purpose.
    CLog,
    /// Scope for libraries
    Lib,
    #[doc(hidden)]
    MAX,    // must remain last (for slice initialization)
}
impl Display for ScopeKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Self::Application => f.write_str("application"),
            Self::CLog => f.write_str("hclog"),
            Self::Lib => f.write_str("library"),
            Self::MAX => f.write_str("max"),
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub (crate) enum ScopeEnv {
    #[default]
    Global,
    Task,
}
impl fmt::Display for ScopeEnv {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Self::Global => f.write_str("global"),
            Self::Task => f.write_str("task"),
        }
    }
}

/*
 * just a container with some metadata for the submodule Vec<>
 *
 * default_options are global options derived from env and passed down to the
 * Submodule on initialization if the caller does not provide any. The Submodule
 * options itself can be changed any time
 */
#[derive(Debug, Default)]
pub (crate) struct LogScope {
    name: String,
    lm: ScopeKey,
    env: ScopeEnv,
    env_ident: Option<String>,
    initialized: bool,
    submodules: Vec<Submodule>,
    // defaults passed down to submodules on init (if not given)
    default_options: Options,
    default_facade: FacadeVariant,
    default_level: Level,
}
impl<K> Index<K> for LogScope where K: LogKey {
    type Output = Submodule;
    fn index(&self, index: K) -> &Self::Output {
        &self.submodules[index.log_key()]
    }
}
impl<K> IndexMut<K> for LogScope where K: LogKey {
    fn index_mut(&mut self, index: K) -> &mut Self::Output {
        &mut self.submodules[index.log_key()]
    }
}

impl LogScope {
    pub (crate) fn init<I: Scope, S: Display>(
        name: S, level: Level, facade: FacadeVariant, options: Options
    ) -> Result<Self> {
        let mut default_level = level;
        if let Ok(Some(l)) = read_var_from_env::<Level>(options::ENV_OPT_LEVEL) {
            default_level = l;
        };
        let mut default_facade = facade;
        if let Ok(Some(f)) = read_var_from_env::<FacadeVariant>(options::ENV_OPT_FACADE) {
            default_facade = f;
        }
        let mut default_options = options;
        default_options.parse_from_env()?;

        Ok(Self {
            name: name.to_string(),
            lm: I::logscope(),
            env: ScopeEnv::Global,
            initialized: true,
            default_options,
            default_facade,
            default_level,
            ..Default::default()
        })
    }
    pub (crate) fn to_scoped(&self, ident: impl fmt::Display) -> Self {
        Self {
            env: ScopeEnv::Task,
            env_ident: Some(ident.to_string()),
            name: self.name.clone(),
            lm: self.lm,
            initialized: self.initialized,
            default_options: self.default_options,
            default_facade: self.default_facade.clone(),
            default_level: self.default_level,
            submodules: self.submodules.clone(),
        }
    }
    pub (crate) fn initialized(&self) -> bool {
        self.initialized
    }
    pub (crate) fn name(&self) -> &str {
        &self.name
    }
    pub (crate) fn key(&self) -> ScopeKey {
        self.lm
    }
    pub (crate) fn env(&self) -> ScopeEnv {
        self.env
    }
    pub (crate) fn env_ident(&self) -> Option<&str> {
        self.env_ident.as_deref()
    }
    pub (crate) fn has_submodule<K: LogKey>(&self, key: K) -> bool {
        let ckey = key.log_key();
        if !self.initialized || self.submodules.len() <= ckey {
            return false;
        }
        let submod = &self.submodules[ckey];
        if !submod.initialized() || submod.key() != ckey {
            return false;
        }
        true
    }
    pub (crate) fn add_submodule<K: LogKey>(&mut self, submod: K)
        -> Result<&mut Submodule>
    {
        if !self.initialized {
            return Err(ScopeNotInitialized);
        }
        let level = submod.init_level().unwrap_or(self.default_level);
        let facade = submod.init_facade().unwrap_or(self.default_facade.clone());
        let opts = submod.init_options().unwrap_or(self.default_options);
        match self.submodules.get_mut(submod.log_key()) {
            Some(sub) => {
                // in case the added module was not initialized and is later
                // added - ensure we don't overwrite the existing module
                //
                // silently ignore if the module is already initialized
                if !sub.initialized() {
                    *sub = Submodule::new(submod, level, &facade, opts);
                }
            }
            None => {
                if self.submodules.len() != submod.log_key() {
                    // insert a dummies here - this is necessary to not mess up the internal
                    // index. Otherwise further add's would panic because oob access
                    for _ in self.submodules.len()..submod.log_key() {
                        self.submodules.push(Submodule::default());
                    }
                }
                self.submodules.push(Submodule::new(submod, level, &facade, opts));
            }
        }
        Ok(&mut self.submodules[submod.log_key()])
    }
    pub (crate) fn submodules(&self) -> impl Iterator<Item = &Submodule> {
        self.submodules.iter()
    }
    pub (crate) fn submodules_mut(&mut self) -> impl Iterator<Item = &mut Submodule> {
        self.submodules.iter_mut()
    }
    pub (crate) fn get_submodule<K: LogKey>(&self, key: K) -> Option<&Submodule> {
        self.submodules.get(key.log_key())
    }
    pub (crate) fn get_submodule_mut<K: LogKey>(&mut self, key: K) -> Option<&mut Submodule> {
        self.submodules.get_mut(key.log_key())
    }
    pub (crate) fn get_submod_by_name(&mut self, key: &str) -> Option<&mut Submodule> {
        if self.submodules.is_empty() {
            return None;
        }
        self.submodules.iter_mut().find(|submod| submod.name() == key)
    }
}
