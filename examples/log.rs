extern crate hclog;
extern crate log;

use hclog::{FacadeVariant, Level};
use log::{debug, error, info, trace, warn};

fn main() {
    hclog::init_log_compat("log", Level::Debug5, FacadeVariant::StdOut, None).unwrap();
    // try all log:: macros once
    error!("error!() print with log::log");
    warn!("warn!() print with log::log");
    info!("info!() print with log::log");
    debug!("debug!() print with log::log");
    trace!("trace!() print with log::log - not printed because filtered");
}
