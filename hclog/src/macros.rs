// all log related macro definitions
/*
 * helper macros
 *
 * This code is actually taken from crate stdext. There is currently no other
 * way in stable rust to resolve the current function name. Another alternative
 * would be crate function_name which defines a #[named] attribute macro but would
 * require each function to be prefixed
 */
#[macro_export]
#[doc(hidden)]
macro_rules! fn_name {
    () => {{
        fn hclog_fn() {}
        fn type_name_of<T>(_: T) -> &'static str {
            std::any::type_name::<T>()
        }
        type_name_of(hclog_fn)
            .rsplit("::")
            .find(|&part| part != "hclog_fn" && part != "{{closure}}")
            .unwrap_or("")
    }};
}
#[macro_export]
#[doc(hidden)]
macro_rules! fn_path {
    () => {{&format!("{}::{}", std::module_path!(), $crate::fn_name!())}}
}

#[cfg(doctest)]
use hclog_macros::HCLog;

/*
 * base log macro, needs to be exported to be found by other macros
 *
 * unwrap here to cause a panic in the calling frame with a msg thrown by hclog::api::log
 * It's also possible to panic! in log() itself but would add this as a frame in the resulting trace
 *
 */
#[macro_export]
/// Log a message with severity [`$lvl`](crate::Level) via [`LogKey`](crate::LogKey)
///
/// This is this main log macro where all other log macros are based on. It takes a [`Level`](crate::Level)
/// and a `LogKey` as well as a format string and arguments to log. Please refer to the
/// [rust documentation](https://doc.rust-lang.org/stable/std/fmt/index.html) for more information
/// on how to format strings.
/// Additionally it logs the file, line and function name where the log was called from.
/// File and line are resolved using the rust compiler built-in macros `std::file!()` and
/// `std::line!()`. The function name is resolved via a macro taken from the [`crate-stdext`] crate.
///
/// It is not intended to call this macro directly because its implementation might change. Use the
/// shortcut macros provided instead:
///
/// [`lEM`](macro@crate::lEM), [`lA`](macro@crate::lA), [`lC`](macro@crate::lC),
/// [`lE`](macro@crate::lE), [`lW`](macro@crate::lW), [`lN`](macro@crate::lN),
/// [`lI`](macro@crate::lI), [`lD1`](macro@crate::lD1), [`lD2`](macro@crate::lD2),
/// [`lD3`](macro@crate::lD3), [`lD4`](macro@crate::lD4), [`lD5`](macro@crate::lD5),
/// [`lD6`](macro@crate::lD6), [`lD7`](macro@crate::lD7), [`lD8`](macro@crate::lD8),
/// [`lD9`](macro@crate::lD9), [`lD10`](macro@crate::lD10)
///
/// # Example
///
/// ```rust
/// # use hclog_macros::HCLog;
/// use hclog::{Level, FacadeVariant, options::Options};
///
/// # #[derive(HCLog, Copy, Clone)]
/// enum HclogKeys { Foo }
///
/// use HclogKeys::*;
///
/// fn main() {
///    # HclogKeys::init("foo", Level::Info, FacadeVariant::StdOut, Options::default()).unwrap();
///    hclog::hclog!(Level::Info, Foo, "Hello World");
///    // the lI macro is actually a shortcut for the above line
///    hclog::lI!(Foo, "Hello World");
/// }
/// ```
///
/// # Panics
///
/// This macro panics if the given `$key` is not initialized or `$lvl` is not a valid
/// [`Level`](crate::Level).
/// See [`init_modules`](crate::init_modules) for more information.
///
macro_rules! hclog {
    ($lvl:path, $key:ident, $($arg:tt)*) => {{
        if $crate::tX!($key, $lvl) {
            $crate::log(
                $key, $lvl, std::file!(), $crate::fn_path!(), std::line!(), &format_args!($($arg)*)
            ).unwrap();
        }
    }};
}

/*
 * Just a few shortcuts for the hclog! macro.
 * simply pass the whole tt, everything else is handled by hclog!().
 */
/// Log a message with severity [`Emerg`](crate::Level::Emerg) via `LogKey`
///
/// For more details see [`hclog`](macro@crate::hclog)
#[macro_export]
macro_rules! lEM {($key:ident, $($args:tt)+) => {$crate::hclog!($crate::Level::Emerg, $key, $($args)+)}}

/// Log a message with severity [`Alert`](crate::Level::Alert) via `LogKey`
///
/// For more details see [`hclog`](macro@crate::hclog)
#[macro_export]
macro_rules! lA {($key:ident, $($args:tt)+) => {$crate::hclog!($crate::Level::Alert, $key, $($args)+)}}

/// Log a message with severity [`Crit`](crate::Level::Crit) via `LogKey`
///
/// For more details see [`hclog`](macro@crate::hclog)
#[macro_export]
macro_rules! lC {($key:ident, $($args:tt)+) => {$crate::hclog!($crate::Level::Crit, $key, $($args)+)}}

/// Log a message with severity [`Error`](crate::Level::Error) via `LogKey`
///
/// For more details see [`hclog`](macro@crate::hclog)
#[macro_export]
macro_rules! lE {($key:ident, $($args:tt)+) => {$crate::hclog!($crate::Level::Error, $key, $($args)+)}}

/// Log a message with severity [`Warn`](crate::Level::Warn) via `LogKey`
///
/// For more details see [`hclog`](macro@crate::hclog)
#[macro_export]
macro_rules! lW {($key:ident, $($args:tt)+) => {$crate::hclog!($crate::Level::Warn, $key, $($args)+)}}

/// Log a message with severity [`Notice`](crate::Level::Notice) via `LogKey`
///
/// For more details see [`hclog`](macro@crate::hclog)
#[macro_export]
macro_rules! lN {($key:ident, $($args:tt)+) => {$crate::hclog!($crate::Level::Notice, $key, $($args)+)}}

/// Log a message with severity [`Info`](crate::Level::Info) via `LogKey`
///
/// For more details see [`hclog`](macro@crate::hclog)
#[macro_export]
macro_rules! lI {($key:ident, $($args:tt)+) => {$crate::hclog!($crate::Level::Info, $key, $($args)+)}}

/// Log a message with severity [`Debug1`](crate::Level::Debug1) via `LogKey`
///
/// For more details see [`hclog`](macro@crate::hclog)
#[macro_export]
macro_rules! lD1 {($key:ident, $($args:tt)+) => {$crate::hclog!($crate::Level::Debug1, $key, $($args)+)}}

/// Log a message with severity [`Debug2`](crate::Level::Debug2) via `LogKey`
///
/// For more details see [`hclog`](macro@crate::hclog)
#[macro_export]
macro_rules! lD2 {($key:ident, $($args:tt)+) => {$crate::hclog!($crate::Level::Debug2, $key, $($args)+)}}

/// Log a message with severity [`Debug3`](crate::Level::Debug3) via `LogKey`
///
/// For more details see [`hclog`](macro@crate::hclog)
#[macro_export]
macro_rules! lD3 {($key:ident, $($args:tt)+) => {$crate::hclog!($crate::Level::Debug3, $key, $($args)+)}}

/// Log a message with severity [`Debug4`](crate::Level::Debug4) via `LogKey`
///
/// For more details see [`hclog`](macro@crate::hclog)
#[macro_export]
macro_rules! lD4 {($key:ident, $($args:tt)+) => {$crate::hclog!($crate::Level::Debug4, $key, $($args)+)}}

/// Log a message with severity [`Debug5`](crate::Level::Debug5) via `LogKey`
///
/// For more details see [`hclog`](macro@crate::hclog)
#[macro_export]
macro_rules! lD5 {($key:ident, $($args:tt)+) => {$crate::hclog!($crate::Level::Debug5, $key, $($args)+)}}

/// Log a message with severity [`Debug6`](crate::Level::Debug6) via `LogKey`
///
/// For more details see [`hclog`](macro@crate::hclog)
#[macro_export]
macro_rules! lD6 {($key:ident, $($args:tt)+) => {$crate::hclog!($crate::Level::Debug6, $key, $($args)+)}}

/// Log a message with severity [`Debug7`](crate::Level::Debug7) via `LogKey`
///
/// For more details see [`hclog`](macro@crate::hclog)
#[macro_export]
macro_rules! lD7 {($key:ident, $($args:tt)+) => {$crate::hclog!($crate::Level::Debug7, $key, $($args)+)}}

/// Log a message with severity [`Debug8`](crate::Level::Debug8) via `LogKey`
///
/// For more details see [`hclog`](macro@crate::hclog)
#[macro_export]
macro_rules! lD8 {($key:ident, $($args:tt)+) => {$crate::hclog!($crate::Level::Debug8, $key, $($args)+)}}

/// Log a message with severity [`Debug9`](crate::Level::Debug9) via `LogKey`
///
/// For more details see [`hclog`](macro@crate::hclog)
#[macro_export]
macro_rules! lD9 {($key:ident, $($args:tt)+) => {$crate::hclog!($crate::Level::Debug9, $key, $($args)+)}}

/// Log a message with severity [`Debug10`](crate::Level::Debug10) via `LogKey`
///
/// For more details see [`hclog`](macro@crate::hclog)
#[macro_export]
macro_rules! lD10 {($key:ident, $($args:tt)+) => {$crate::hclog!($crate::Level::Debug10, $key, $($args)+)}}

// exported test macros
/// Test if a given [`Level`](enum@crate::Level) is enabled for a given [`LogKey`](trait@crate::LogKey)
///
/// If given the `$lvl` is enabled for `$key` the function returns `true`, otherwise `false`.
///
/// It is not intended to use this Macro directly. Use the shortcut macros provided instead:
///
/// [`tEM`](macro@crate::tEM), [`tA`](macro@crate::tA), [`tC`](macro@crate::tC),
/// [`tE`](macro@crate::tE), [`tW`](macro@crate::tW), [`tN`](macro@crate::tN),
/// [`tI`](macro@crate::tI), [`tD1`](macro@crate::tD1), [`tD2`](macro@crate::tD2),
/// [`tD3`](macro@crate::tD3), [`tD4`](macro@crate::tD4), [`tD5`](macro@crate::tD5),
/// [`tD6`](macro@crate::tD6), [`tD7`](macro@crate::tD7), [`tD8`](macro@crate::tD8),
/// [`tD9`](macro@crate::tD9), [`tD10`](macro@crate::tD10)
///
/// # Example
///
/// ```rust
/// use hclog::{Level, FacadeVariant, options::Options};
/// # use hclog_macros::HCLog;
///
/// # #[derive(HCLog, Copy, Clone)]
/// enum TxKey { AA, }
///
/// use TxKey::AA;
///
/// fn main() {
///     # TxKey::init("foo", Level::Debug9, FacadeVariant::StdOut, Options::default()).unwrap();
///
///     assert!(hclog::tX!(AA, Level::Debug9));            // Debug9 is enabled
///     assert_eq!(hclog::tX!(AA, Level::Debug10), false); // Debug10 is not enabled
///     // the lines below are actually shortcuts for the above lines
///     assert!(hclog::tD9!(AA));
///     assert_eq!(hclog::tD10!(AA), false);               // Debug10 is not enabled
/// }
/// ```
///
/// # Panics
///
/// Panics if the given [`LogKey`](trait@crate::LogKey) is not initialized.
/// See [`add_submodules`](fn@crate::add_submodules) for more information.
///
#[macro_export]
macro_rules! tX {
    ($key:ident, $level:path) => {
        $crate::test_log($key, $level).unwrap()
    };
}

/// Test if severity [`Emerg`](crate::Level::Emerg) is enabled for `LogKey`
///
/// For more details see [`tX`](macro@crate::tX)
#[macro_export]
macro_rules! tEM {($key:ident) => {$crate::tX!($key, $crate::Level::Emerg)};}

/// Test if severity [`Alert`](crate::Level::Alert) is enabled for `LogKey`
///
/// For more details see [`tX`](macro@crate::tX)
#[macro_export]
macro_rules! tA {($key:ident) => {$crate::tX!($key, $crate::Level::Alert)};}

/// Test if severity [`Crit`](crate::Level::Crit) is enabled for `LogKey`
///
/// For more details see [`tX`](macro@crate::tX)
#[macro_export]
macro_rules! tC {($key:ident) => {$crate::tX!($key, $crate::Level::Crit)};}

/// Test if severity [`Error`](crate::Level::Error) is enabled for `LogKey`
///
/// For more details see [`tX`](macro@crate::tX)
#[macro_export]
macro_rules! tE {($key:ident) => {$crate::tX!($key, $crate::Level::Error)};}

/// Test if severity [`Warn`](crate::Level::Warn) is enabled for `LogKey`
///
/// For more details see [`tX`](macro@crate::tX)
#[macro_export]
macro_rules! tW {($key:ident) => {$crate::tX!($key, $crate::Level::Warn)};}

/// Test if severity [`Notice`](crate::Level::Notice) is enabled for `LogKey`
///
/// For more details see [`tX`](macro@crate::tX)
#[macro_export]
macro_rules! tN {($key:ident) => {$crate::tX!($key, $crate::Level::Notice)};}

/// Test if severity [`Info`](crate::Level::Info) is enabled for `LogKey`
///
/// For more details see [`tX`](macro@crate::tX)
#[macro_export]
macro_rules! tI {($key:ident) => {$crate::tX!($key, $crate::Level::Info)};}

/// Test if severity [`Debug1`](crate::Level::Debug1) is enabled for `LogKey`
///
/// For more details see [`tX`](macro@crate::tX)
#[macro_export]
macro_rules! tD1 {($key:ident) => {$crate::tX!($key, $crate::Level::Debug1)};}

/// Test if severity [`Debug2`](crate::Level::Debug2) is enabled for `LogKey`
///
/// For more details see [`tX`](macro@crate::tX)
#[macro_export]
macro_rules! tD2 {($key:ident) => {$crate::tX!($key, $crate::Level::Debug2)};}

/// Test if severity [`Debug3`](crate::Level::Debug3) is enabled for `LogKey`
///
/// For more details see [`tX`](macro@crate::tX)
#[macro_export]
macro_rules! tD3 {($key:ident) => {$crate::tX!($key, $crate::Level::Debug3)};}

/// Test if severity [`Debug4`](crate::Level::Debug4) is enabled for `LogKey`
///
/// For more details see [`tX`](macro@crate::tX)
#[macro_export]
macro_rules! tD4 {($key:ident) => {$crate::tX!($key, $crate::Level::Debug4)};}

/// Test if severity [`Debug5`](crate::Level::Debug5) is enabled for `LogKey`
///
/// For more details see [`tX`](macro@crate::tX)
#[macro_export]
macro_rules! tD5 {($key:ident) => {$crate::tX!($key, $crate::Level::Debug5)};}

/// Test if severity [`Debug6`](crate::Level::Debug6) is enabled for `LogKey`
///
/// For more details see [`tX`](macro@crate::tX)
#[macro_export]
macro_rules! tD6 {($key:ident) => {$crate::tX!($key, $crate::Level::Debug6)};}

/// Test if severity [`Debug7`](crate::Level::Debug7) is enabled for `LogKey`
///
/// For more details see [`tX`](macro@crate::tX)
#[macro_export]
macro_rules! tD7 {($key:ident) => {$crate::tX!($key, $crate::Level::Debug7)};}

/// Test if severity [`Debug8`](crate::Level::Debug8) is enabled for `LogKey`
///
/// For more details see [`tX`](macro@crate::tX)
#[macro_export]
macro_rules! tD8 {($key:ident) => {$crate::tX!($key, $crate::Level::Debug8)};}

/// Test if severity [`Debug9`](crate::Level::Debug9) is enabled for `LogKey`
///
/// For more details see [`tX`](macro@crate::tX)
#[macro_export]
macro_rules! tD9 {($key:ident) => {$crate::tX!($key, $crate::Level::Debug9)};}

/// Test if severity [`Debug10`](crate::Level::Debug10) is enabled for `LogKey`
///
/// For more details see [`tX`](macro@crate::tX)
#[macro_export]
macro_rules! tD10 {($key:ident) => {$crate::tX!($key, $crate::Level::Debug10)};}

// Test below just ensure that macros are expanded as expected
#[cfg(test)]
mod macro_test {
    #![allow(unused_must_use)]
    use serial_test::serial;
    use crate::{
        libtest::init_libtest_mod,
        log_internal::test::TestKeys::*,
        Level::*,
    };

    #[test]
    fn hclog_macro() {
        init_libtest_mod().unwrap();
        hclog!(Emerg, LIBTESTFOO, "foo");
        hclog!(Emerg, LIBTESTFOO, "Foo {} {:?}", 42, Debug10);
        hclog!(Debug1, LIBTESTFOO, "foo_{} baz {:?}", "bar", (42u32, -1));
        hclog!(Debug3, LIBTESTFOO, "ensure $path works");
    }

    #[test]
    fn log_macros() {
        init_libtest_mod().unwrap();
        lEM!(LIBTESTFOO, "Foo");
        lA!(LIBTESTFOO, "Foo");
        lC!(LIBTESTFOO, "Foo");
        lE!(LIBTESTFOO, "Foo");
        lW!(LIBTESTFOO, "Foo");
        lN!(LIBTESTFOO, "Foo");
        lI!(LIBTESTFOO, "Foo");
        lD1!(LIBTESTFOO, "Foo");
        lD2!(LIBTESTFOO, "Foo");
        lD3!(LIBTESTFOO, "Foo");
        lD4!(LIBTESTFOO, "Foo");
        lD5!(LIBTESTFOO, "Foo");
        lD6!(LIBTESTFOO, "Foo");
        lD7!(LIBTESTFOO, "Foo");
        lD8!(LIBTESTFOO, "Foo");
        lD9!(LIBTESTFOO, "Foo");
        lD10!(LIBTESTFOO, "Foo");
    }

    #[test]
    #[serial]
    fn test_macros() {
        init_libtest_mod().unwrap();
        crate::api::set_level(LIBTESTFOO, Debug5);
        assert!(tEM!(LIBTESTFOO));
        assert!(tA!(LIBTESTFOO));
        assert!(tC!(LIBTESTFOO));
        assert!(tE!(LIBTESTFOO));
        assert!(tW!(LIBTESTFOO));
        assert!(tN!(LIBTESTFOO));
        assert!(tI!(LIBTESTFOO));
        assert!(tD1!(LIBTESTFOO));
        assert!(tD2!(LIBTESTFOO));
        assert!(tD3!(LIBTESTFOO));
        assert!(tD4!(LIBTESTFOO));
        assert!(tD5!(LIBTESTFOO));
        assert_eq!(tD6!(LIBTESTFOO), false, "Debug6 is enabled");
        assert_eq!(tD7!(LIBTESTFOO), false);
        assert_eq!(tD8!(LIBTESTFOO), false);
        assert_eq!(tD9!(LIBTESTFOO), false);
        assert_eq!(tD10!(LIBTESTFOO), false);
    }
}
