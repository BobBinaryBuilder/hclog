#![allow(unused_variables,dead_code)]
#[macro_use]
extern crate hclog;

use hclog::{
    options::{Options, SCOPE},
    LogKey, Scope, FacadeVariant, Level, ContextKey
};

use std::fmt;

#[derive(Clone, Copy, Debug)]
#[repr(u32)]
enum LogKeys {
    ASYNC = 0,
    GLOBAL = 1,
}
use LogKeys::*;

static LOG_KEYS: &'static [LogKeys] = &[ASYNC, GLOBAL];
impl LogKey for LogKeys {
    fn log_key(&self) -> ContextKey { *self as usize }
}
impl Scope for LogKeys {
    fn init<S: fmt::Display>(name: S, level: Level, facade: FacadeVariant,
                 options: Options) -> hclog::Result<()> {
        hclog::init::<Self, S>(name, level, facade, options)?;
        hclog::add_submodules(LOG_KEYS)
    }
}
impl fmt::Display for LogKeys {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Self::ASYNC => write!(f, "async"),
            Self::GLOBAL => write!(f, "global"),
        }
    }
}

mod foo {
    use super::*;
    pub mod bar {
        use super::*;
        pub async fn async_global_log(arg: &str) {
            lEM!(ASYNC, "in async_global_log: caller=\"{}\"", arg);
        }
    }
    pub fn task_local_log(key: LogKeys) -> impl std::future::Future {
        hclog::scope("Task1", key, async move {
            lI!(ASYNC, "in task_local_log within Scope::Task");
            hclog::dump(&mut std::io::stdout()).unwrap();
            foo::bar::async_global_log("task_local_log").await;
            hclog::set_logdest(ASYNC, FacadeVariant::StdErr).unwrap();
            lD10!(ASYNC, "log with debug10 -- not printed");
            hclog::set_level(ASYNC, Level::Debug10).unwrap();
            lD10!(ASYNC, "log with debug10 to stderr");
            // this sets the options for the local context
            hclog::set_module_options(ASYNC, SCOPE).unwrap();
            lE!(ASYNC, "log with scope but without tid to stderr");
            // log with module from global scope which is not in local scope
            lC!(GLOBAL, "log via module from Global Scope to stdout");
        }).unwrap()
    }
}


fn main() {
    LogKeys::init("async", Level::Info, FacadeVariant::StdOut, Options::default()).unwrap();

    let j1 = async { foo::task_local_log(ASYNC).await };
    let j2 = async { foo::bar::async_global_log("main").await };

    // run futures
    futures::executor::block_on(j1);
    futures::executor::block_on(j2);
}
