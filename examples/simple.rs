#[macro_use]
extern crate hclog;
use hclog::{
    options::{Options, FUNC, FILE, PID, DATESTAMP},
    Scope, Level, FacadeVariant, LogKey, ContextKey,
};
use std::fmt;

// note that derive won't work here but it's still possible to impl all
// traits
#[derive(Clone, Copy, Debug)]
struct MAIN;

impl Scope for MAIN {
    fn init<S: fmt::Display>(name: S, level: Level, facade: FacadeVariant,
                 options: Options) -> hclog::Result<()> {
        hclog::init::<Self, S>(name, level, facade.clone(), options)?;
        hclog::add_submodules(&[MAIN])?;
        //init_log_compat(name, level, facade, Some(options))?;
        Ok(())
    }
}
impl LogKey for MAIN {
    fn log_key(&self) -> ContextKey { 0usize }
}
impl fmt::Display for MAIN {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "MAIN")
    }
}

fn test_macros() {
    hclog::set_level(MAIN, Level::Debug9).unwrap();
    if tD9!(MAIN) {
        lD9!(MAIN, "Debug9 is enabled!");
    }
    if tD10!(MAIN) {
        lD10!(MAIN, "debug10 is not enabled");
    }
}

fn print_with_options() -> hclog::Result<()> {
    hclog::unset_module_options(MAIN, PID + DATESTAMP)?;
    lI!(MAIN, "This will be printed without pid and datestamp");
    hclog::set_module_options(MAIN, PID + DATESTAMP)?;
    hclog::unset_module_options(MAIN, FUNC + FILE)?;
    lI!(MAIN, "This will be printed without func and file (and no line)");
    hclog::reset_module_options(MAIN)?;
    Ok(())
}

fn main() {
    MAIN::init("simple", Level::Debug10, FacadeVariant::StdOut, Options::default()).unwrap();
    hclog::dump(&mut std::io::stdout()).unwrap();

    lE!(MAIN, "Some Test: {:?}", MAIN);
    print_with_options().unwrap();
    lEM!(MAIN, "This wild be printed with old options");
    test_macros();
}
