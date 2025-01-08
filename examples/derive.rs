#[macro_use]
extern crate hclog;

use hclog::{Level, FacadeVariant, ScopeKey};
use hclog_macros::HCLog;
use log::{trace, warn};

use std::{
    mem,
};

#[derive(HCLog, Copy, Clone, Debug)]
#[hclog(
    scope = ScopeKey::Application,
    default_level = Level::Info,
    default_facade = FacadeVariant::StdErr,
    with_log,
)]
enum DeriveKeys {
    #[hclog(name = "foo", facade = FacadeVariant::StdOut)]
    KeyA = 0,
    #[hclog(name = "bar", level = Level::Warn)]
    KeyB,
    #[hclog(ignore, name = "ignored", facade = FacadeVariant::Syslog("user".into()))]
    KeyC = 2,
    #[hclog(ignore, name = "KeyD")]
    KeyD = 3,
    #[hclog(name = "KeyE", level = Level::Warn)]
    KeyE = 4,
}
use DeriveKeys::*;

fn main() {
    DeriveKeys::init_with_defaults("derive").unwrap();
    lI!(KeyA, "Key1 => {:?} => {:?}", KeyA, mem::discriminant(&KeyA));
    // won't be printed because level is set to warn
    lI!(KeyB, "Key2 => {} => {:?}", KeyB, mem::discriminant(&KeyB));
    lW!(KeyB, "Key2 => {} => {:?}", KeyB, mem::discriminant(&KeyB));
    lI!(KeyA, "Key3 enabled: {} - but impls display: '{}'", tI!(KeyC), KeyC);
    if tI!(KeyC) {
        lI!(KeyC, "Key3 => {} => {:?}", KeyC, mem::discriminant(&KeyC));
    }
    assert!(hclog::has_module(KeyE).unwrap()); // ensure KeyE is enabled
    hclog::add_submodules(&[KeyC]).unwrap();
    assert!(hclog::has_module(KeyC).unwrap());
    assert!(!hclog::has_module(KeyD).unwrap());
    warn!("will also work with logcompat enabled");
    trace!("won't print trace");
    hclog::dump(&mut std::io::stdout()).unwrap();
}
