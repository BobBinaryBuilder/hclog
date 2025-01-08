#![allow(unused_variables)] // TMP

use once_cell::sync::Lazy;
use crate::{
    submodule::Submodule,
    logmod::{LogScope, ScopeKey},
    task::TaskLocalErr,
    InternalLogKeys::Internal,
    Scope, LogKey, ErrorKind, Result, Level,
    FacadeVariant, Options,
};
use std::{
    ops::{Index, IndexMut},
    sync::{RwLock, RwLockReadGuard, RwLockWriteGuard},
    fmt::Display,
    future::Future,
};

/*
 * with msrv > 1.63 we can remove the outer lazy because RwLock will also work
 * in const/static context.
 */
pub static GLOBAL_CONTEXT: Lazy<RwLock<Context>> = Lazy::new(||RwLock::new(Context::default()));
crate::task_local! {
    pub static TASK_CONTEXT: Context;
}
// dummy struct to encapsulate access to the static context - at least for now (Maybe tmp)
#[allow(clippy::upper_case_acronyms)]
pub (crate) struct CTX;
impl CTX {
    pub (crate) fn get() -> Result<RwLockReadGuard<'static, Context>> {
        Ok(GLOBAL_CONTEXT.read()?)
    }
    pub (crate) fn get_mut() -> Result<RwLockWriteGuard<'static, Context>> {
        Ok(GLOBAL_CONTEXT.write()?)
    }

    /*
     * scoped access
     * NOTE: This is currently just a temp impl and will be replaced */
    pub (crate) fn new_scoped<I, K, F>(ident: I, key: K, future: F) -> Result<impl Future>
    where
        I: Display + Send, K: LogKey, F: Future + Send
    {
        let global = GLOBAL_CONTEXT.read()?;
        let logmod = global.get_mod(K::logscope())?;
        let modname = logmod.name();
        // initialize task local struct
        let mut local = Context::default();
        // just init with the requested module(s)
        local[logmod.key()] = logmod.to_scoped(ident);
        local[logmod.key()].add_submodule(key)?; //, None, None, None)?;

        lD1!(Internal, "new_scoped: {} with key: {}", modname, key);

        Ok(TASK_CONTEXT.scope(local, future))
    }
    pub (crate) fn call<F, R>(f: F) -> Result<R>
    where
        F: FnOnce(&Context) -> Result<R> + Copy,
    {
        match TASK_CONTEXT.try_with(|ctx| { f(ctx) }) {
            Err(TaskLocalErr::AccessError) | Ok(Err(ErrorKind::KeyNotInitialized)) => {
                let ctx = GLOBAL_CONTEXT.read()?;
                f(&ctx)
            }
            Err(e) => Err(e.into()),
            Ok(Err(e)) => Err(e),
            Ok(o) => o,
        }
    }
    pub (crate) fn call_mut<F>(f: F) -> Result<()>
    where
        F: FnOnce(&mut Context) -> Result<()> + Copy,
    {
        match TASK_CONTEXT.try_with_mut(|v| { f(v) }) {
            Err(TaskLocalErr::AccessError) | Ok(Err(ErrorKind::KeyNotInitialized)) => {
                let mut ctx = GLOBAL_CONTEXT.write()?;
                f(&mut ctx)
            }
            Err(e) => Err(e.into()),
            Ok(Err(e)) => Err(e),
            Ok(o) => o,
        }
    }
}

#[derive(Debug, Default)]
pub (crate) struct Context {
    log_modules: [LogScope; ScopeKey::MAX as usize],
}
// impl index for easier access - not really necessary but avoids some bloat
impl Index<ScopeKey> for Context {
    type Output = LogScope;
    fn index(&self, index: ScopeKey) -> &Self::Output {
        &self.log_modules[index as usize]
    }
}
impl IndexMut<ScopeKey> for Context {
    fn index_mut(&mut self, index: ScopeKey) -> &mut Self::Output {
        &mut self.log_modules[index as usize]
    }
}
impl Context {
    /* *** Module access below ***/
    pub fn logmods(&self) -> impl Iterator<Item = &LogScope> {
        self.log_modules.iter()
    }
    pub fn logmods_mut(&mut self) -> impl Iterator<Item = &mut LogScope> {
        self.log_modules.iter_mut()
    }
    #[inline(always)]
    pub fn has(&self, key: ScopeKey) -> bool {
        self.log_modules.len() < key as usize || self[key].initialized()
    }
    pub fn init_mod<I: Scope, S: Display>(
        &mut self, name: S, level: Level, facade: FacadeVariant, options: Options,
    ) -> Result<&mut LogScope> {
        let lm = I::logscope();
        if !self.has(lm) {
            self[lm] = LogScope::init::<I, S>(name, level, facade, options)?;
        }
        Ok(&mut self[lm])
    }
    pub fn get_mod(&self, lm: ScopeKey) -> Result<&LogScope> {
        if !self.has(lm) {
            return Err(ErrorKind::ScopeNotInitialized);
        }
        Ok(&self[lm])
    }
    pub fn get_mod_mut(&mut self, lm: ScopeKey) -> Result<&mut LogScope> {
        if !self.has(lm) {
            return Err(ErrorKind::ScopeNotInitialized);
        }
        Ok(&mut self[lm])
    }

    /* search a submodule by it's name in every existing lockmod */
    pub fn get_submod_by_name(&mut self, key: &str) -> Option<&mut Submodule> {
        if self.log_modules.is_empty() {
            return None;
        }
        for cmod in self.logmods_mut() {
            if !cmod.initialized() { continue; }
            if let Some(m) = cmod.get_submod_by_name(key) {
                return Some(m);
            }
        }
        None
    }
}
