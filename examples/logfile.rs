#[macro_use]
extern crate hclog;

use hclog::{FacadeVariant, Level};
use hclog_macros::HCLog;

#[derive(HCLog, Copy, Clone, Debug)]
#[hclog(
    default_level = Level::Info,
    default_facade = FacadeVariant::File("./file.log".into(), false),
)]
enum Keys {
    #[hclog(name = "Foo")]
    KeyA = 0,
}
use Keys::*;

fn main () {
    Keys::init_with_defaults("logfile").unwrap();
    lI!(KeyA, "First Line in Logfile");
    lI!(KeyA, "Log with an  argument: {}", 42);
}
