/*
 * Compatibility to log crate. crate log is the de facto base standard to use
 * logging macros in rust.
 * Define an implicit "module" as LogCompat to redirect each call to info! and so
 * on to this module.
 */
use log::{Level as LogLevel, Metadata, Record};
use crate::{
    level::Level,
    facades::FacadeVariant,
    log_internal::InternalLogKeys::{self, Internal, LogCompat},
    options::Options,
    context::CTX,
    Scope, lD1, Result,
};
use std::sync::atomic::{AtomicBool, Ordering};

impl From<LogLevel> for Level {
    fn from(level: LogLevel) -> Self {
        match level {
            LogLevel::Error => Level::Error,
            LogLevel::Warn => Level::Warn,
            LogLevel::Info => Level::Info,
            LogLevel::Debug => Level::Debug1,
            LogLevel::Trace => Level::Debug10,
        }
    }
}

static INITIALIZED: AtomicBool = AtomicBool::new(false);

struct CLogLogger;
impl log::Log for CLogLogger {
    fn enabled(&self, metadata: &Metadata<'_>) -> bool {
        if !crate::api::has_module(LogCompat).unwrap_or(false) {
            return false;
        }
        if !crate::api::test_log(LogCompat, metadata.level().into()).unwrap() {
            return false;
        }
        true
    }
    fn log(&self, record: &Record<'_>) {
        if self.enabled(record.metadata()) {
            crate::api::log(
                LogCompat,
                record.level().into(),
                record.file().unwrap_or(""),
                "", // record has no function name metadata
                record.line().unwrap_or(0u32),
                record.args(),
            )
            .unwrap();
        }
    }
    fn flush(&self) {}
}

pub(crate) fn init_log_compat(
    level: Level, facade: FacadeVariant, _options: Option<Options>
)-> Result<()> {
    if INITIALIZED.load(Ordering::Relaxed) {
        return Ok(());
    }
    {
        CTX::get_mut()?.get_mod_mut(InternalLogKeys::logscope())?
            .add_submodule(LogCompat)?
            .set_logsev(level)
            .set_logdest(&facade);
    }
    lD1!(Internal, "initializing crate log compatibility support");

    log::set_boxed_logger(Box::new(CLogLogger))?;
    // always set the highest level Filter. Filtering is handled by hclog
    log::set_max_level(log::LevelFilter::Trace);
    INITIALIZED.store(true, Ordering::Release);
    Ok(())
}
