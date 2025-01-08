#![allow(
    clippy::needless_doctest_main,
)]
#![warn(
    missing_debug_implementations,
    missing_docs,
)]
#![deny(
    unconditional_recursion,
    rustdoc::broken_intra_doc_links,
)]
#![doc(test(
    //no_crate_inject,
    attr(deny(warnings, rust_2018_idioms), allow(dead_code, unused_variables))
))]
#![cfg_attr(docsrs, allow(unused_attributes))]

//! A configurable logging library for Rust
//!
//! `hclog` is a logging library for Rust which is designed to be highly configurable and
//! flexible. It is designed to be as flexible as possible and to be used in a wide range of
//! underlying log backends. Customer of this this library can choose which log backend they
//! want to use and how the log messages should be formatted.
//!
//! # Design of the crate
//!
//! The crate uses a modular approach. Most of the implementation decisions are based on actual
//! needs we had durring the implementation.
//! The Logging is based on [`LogKey`] identifiers. Each `LogKey` is a unique identifier for a
//! which exists within a [`Scope`]. The `Scope` is a container for a set of `LogKey`s and
//!
//! # Key based logging
//!
//! The main idea of this library is to provide a key based logging system. A `LogKey` is a
//! unique identifier for a logger or, to be precise, a lock backend. Each LogKey can be configured
//! on his own with a dedicated [`Level`] and [`FacadeVariant`].
//! To prevent identifier collisions in different crates or modules the `LogKey` are in a scope,
//! which is a namespace or container.
//!
//! An Application, which links to different libraries, defines its own set of `LogKey` set. The
//! linked libraries aswell. All of them are stored in a seperated scope. This may look like this:
//!
//! ```none
//! +---------------+
//! |  Application  |
//! +-------+-------+
//!         |
//!         |     +---+---+ +---+---+ +---+---+       +---+---+
//!         +-----+ Lib A +-+ Lib B +-+ Lib C +-[...]-+  Lib  |
//!         |     +---+---+ +---+---+ +---+---+       +---+---+
//!         |         |         |         |               |
//! +-------+---------+---------+---------+---------------+-------+
//! | AppScopeKey | LibScopeA | LibScopeB | LibScopeC | LibScopeN |
//! +-------------+-----------+-----------+-----------+-----------+
//! | LogKeyA     | LogKeyA   | LogKeyA   | LogKeyA   | LogKeyA   |
//! | LogKeyB     | LogKeyB   | LogKeyB   | LogKeyB   | LogKeyB   |
//! | LogKeyC     | LogKeyC   | LogKeyC   | LogKeyC   | LogKeyC   |
//! | LogKeyD     | LogKeyD   | LogKeyD   | LogKeyD   | LogKeyD   |
//! +---------------------------+---------------------------------+
//! ```
//!
//! Whereas every LogKey can implement its own logging backend and [`Options`] how and where should
//! be logged to.
//!
//! # Usage
//!
//! The basic usage of this library is to define a set of `LogKey`s which are used to log messages
//! in the application or library via defined macros like [`lI!`], [`lD1!`], [`lE!`] and so on. A
//! logging  macro expects a `LogKey` and a format string similar to common `print!` macros like
//! this:
//! ```rust
//! # use hclog_macros::HCLog;
//! # #[derive(HCLog, Copy, Clone)]
//! # enum Keys { Key }
//! # Keys::init_with_defaults("Keys").unwrap();
//! # use Keys::Key;
//!
//! hclog::lI!(Key, "This is an info message");
//! hclog::lD5!(Key, "This is a debug message with level {}", 5);
//! ```
//! Avoid writing expressions with side-effects in log statements. They may not be evaluated.
//!
//! ## In Libraries
//!
//! Libraries define their own set of [`LogKey`]s which are used to log messages within this Library.
//! The initialization of the library is done by the application which uses the library via a common
//! `init` function which calls the hclog initialization function.
//!
//! ```
//! use hclog::{Scope, LogKey, Level, FacadeVariant, options::Options, Result};
//!
//! #[derive(Copy, Clone, Debug)]
//! enum LibraryLogKeys {
//!  LA,
//!  LB,
//!  LC,
//!  // ... more keys
//! }
//! use LibraryLogKeys::*;
//!
//! impl std::fmt::Display for LibraryLogKeys {
//!     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//!         write!(f, "{}", match self {
//!             Self::LA => "LA",
//!             Self::LB => "LB",
//!             Self::LC => "LC",
//!         })
//!     }
//! }
//!
//! impl Scope for LibraryLogKeys {
//!     fn default_level() -> Level { Level::Info }
//!     fn init<S: std::fmt::Display>(
//!         name: S, level: Level, facade: FacadeVariant, options: Options
//!     ) -> Result<()> {
//!         hclog::init::<Self, S>(name, level, facade, options)?;
//!         hclog::add_submodules(&[Self::LA, Self::LB, Self::LC])
//!     }
//! }
//!
//! impl LogKey for LibraryLogKeys {
//!     fn log_key(&self) -> usize {
//!         match self {
//!             Self::LA => 0,
//!             Self::LB => 1,
//!             Self::LC => 2,
//!         }
//!     }
//! }
//!
//! fn do_log() {
//!     hclog::lI!(LA, "this is a info message in Library Scope");
//! }
//!
//! fn init_mylibrary(level: Level, facade: FacadeVariant, options: Options) -> Result<()> {
//!     /* main initialization part of the library */
//!     LibraryLogKeys::init("MyLibrary", level, facade, options)?;
//!     Ok(())
//! }
//! ```
//!
//! The `LibraryLogKeys` are a defined [`LogKey`] set of the library and does not need to be
//! visible to the application nor is intended to be used by the application. Keys defined in the
//! library should be exclusive to the library and not be used in the application.
//!
//! ## In Applications/executables
//!
//! Applications or executables also define their own set of [`LogKey`]s which are used to log
//! messages within the scope of the application. The initialization of the application is done
//! by calling the `init` function of the [`Scope`] implementation of the [`LogKey`] type.
//! The application can also initialize the loggers of the libraries it uses.
//!
//! _Note: The example below uses the same [`Level`] and [`FacadeVariant`] for the application
//! and the libraries. This is not necessary and can be configured individually._
//!
//! Taking the example above a sample application could look like this:
//!
//! ```
//! use hclog::{Scope, LogKey, Level, FacadeVariant, options::Options, Result};
//! // use mylibrary;  // import the 'library' above
//!
//! #[derive(Copy, Clone, Debug)]
//! enum AppLogKeys {
//!    AA,
//!    AB,
//!    AC,
//! }
//! use AppLogKeys::*;
//!
//! impl std::fmt::Display for AppLogKeys {
//!    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//!         write!(f, "{}", match self {
//!             Self::AA => "AA",
//!             Self::AB => "AB",
//!             Self::AC => "AC",
//!         })
//!     }
//! }
//!
//! impl Scope for AppLogKeys {
//!     fn init<S: std::fmt::Display>(
//!         name: S, level: Level, facade: FacadeVariant, options: Options
//!     ) -> Result<()> {
//!         hclog::init::<Self, S>(name, level, facade, options)?;
//!         hclog::add_submodules(&[Self::AA, Self::AB, Self::AC])?;
//!         // call initialization routine in the library
//!         // mylibrary::init_mylibrary(level, facade, options)?;
//!         Ok(())
//!     }
//! }
//!
//! impl LogKey for AppLogKeys {
//!     fn log_key(&self) -> usize {
//!         match self {
//!             Self::AA => 0,
//!             Self::AB => 1,
//!             Self::AC => 2,
//!         }
//!     }
//! }
//!
//! fn main() {
//!    AppLogKeys::init("MyApp", Level::Debug1, FacadeVariant::StdOut, Options::default()).unwrap();
//!
//!    // mylibrary::do_log();     // logs in the scope of the library
//!    hclog::lI!(AA, "this is a info message in App Scope");
//! }
//! ```
//!
//! ## Usage with crate `hclog_macros`
//!
//! The `hclog_macros` crate provides a attributable derive macro for the usage with this crate.
//! It allows you to derive all traits necessary to initialize the library and configure default
//! values at compiletime.
//! For more informations consult the documentation of the `hclog_macros` crate.
//!
//! ```rust
//! use hclog::{Level, FacadeVariant};
//! use hclog_macros::HCLog;
//!
//! #[derive(HCLog, Copy, Clone)]
//! #[hclog(default_level = Level::Info, default_facade = FacadeVariant::StdOut)]
//! enum MyLogKeys {
//!   #[hclog(name = "A", level = Level::Debug1)]
//!   DA,
//!   #[hclog(name = "B", facade = FacadeVariant::StdErr)]
//!   DB,
//!   #[hclog(name = "C")]
//!   DC,
//! }
//!
//! use MyLogKeys::*;
//!
//! fn main() {
//!    MyLogKeys::init_with_defaults("MyLogKeys").unwrap();
//!    hclog::lD1!(DB, "this won't get logged because of lvl Info for Key DB");
//!    hclog::lD1!(DA, "this will get logged because of lvl Debug1 for Key DA");
//!    hclog::lE!(DB, "this will be logged to stderr");
//! }
//!
//! ```
//!
//! # Warning
//!
//! The library internal context may be init only once.
//!
//! # Crate Feature Flags
//!
//! The following feature flags are available for this crate. They are configured in your Cargo.toml:
//!
//! * `std`: Enabled by default. This flag does not enable any additional features.
//! * `derive`: This flag enables the derive macro for the `HCLog` trait.
//!
//! ```toml
//! [dependencies]
//! hclog = { version = "0.1", features = ["derive"] }
//! ```
//!
//! # Version compatibility
//!
//! This crate is currently compatible with Rust 1.74.1 and later. We will try to keep this
//! compatibility as long as possible. If you need a specific version of Rust please open an
//! issue on the GitHub repository. Hence this is still a version 0.1.0 crate the API is not
//! stable and might change in the future. We will try to keep the changes as minimal as possible
//! and provide a migration guide or compability layer if necessary.
//!
//! [crate-log]: https://docs.rs/log/latest/log/
//! [crate-stdext]: https://docs.rs/stdext/latest/stdext/
extern crate log;

// required to derive trait in doc tests which would fail to compile otherwise
// NOTE: doc tests / examples hide the derive by intention. This is a workaround to make it work.
#[cfg(doctest)]
extern crate hclog_macros;

use std::fmt::Display;

mod error;
pub use crate::error::{Result, ErrorKind};

#[macro_use]
#[doc(hidden)]
pub mod macros;

mod log_internal;
pub use crate::log_internal::InternalLogKeys;

#[doc(hidden)]
mod context;

mod logmod;
#[doc(inline)]
pub use crate::logmod::ScopeKey;

#[doc(hidden)]
mod submodule;

mod facades;
#[doc(inline)]
pub use crate::facades::FacadeVariant;

mod level;
#[doc(inline)]
pub use crate::level::Level;

#[doc(hidden)]
mod message;

pub mod options;

#[doc(hidden)]
mod compat;

#[doc(hidden)]
mod task;

mod api;
#[doc(inline)]
pub use crate::api::*;

#[doc(hidden)]
mod util;

// library internal imports
use crate::options::*;

/// Alias for the Index of the Scope in the context
///
/// This demands that passed [`LogKey`] implementors are `#[repr(usize)]` which is
/// currently the default.
pub type ContextKey = usize;

/// Initialization trait for the log scope
///
/// Trait to initialize to library which is implemented on a set of [`LogKey`]s. It can
/// either be implemented manually or derived from the `HCLog` derive macro from the
/// `hclog_macros` crate.
/// If the [`LOGCOMPAT`] option is set the compatibility logger is initialized as well.
///
/// A type implementing this trait has to implement [`Send`], [`Sync`] and [`Copy`].
///
/// ## What is a Scope?
///
/// The [`Scope`] is a container to store all initialized [`LogKey`]s. Each Scope is assigned a
/// unique [`ScopeKey`] which is used to crate a namespace for [`LogKey`]s in the context.
/// This way can be ensured that there are no identifier collisions in different crates or modules
/// when using the library.
///
/// A call to [`init`] will fetch a new [`ScopeKey`] from the context and initialize the loggers
/// within this container. The [`ScopeKey`] itself is constant at runtime and can not be changed
/// because it is used as an Index in the context.
///
/// ## Options of the Scope
///
/// On initialization default [`Level`], [`FacadeVariant`] and [`Options`] can be set for the
/// [`Scope`]. This values are passed to each newly initialized [`LogKey`] in this [`Scope`].
/// If the [`LogKey`] is initialized with a different set of options, the default options are
/// ignored.
///
/// # Examples
///
/// ### Manual Implementation
///
/// ```compile_fail
/// use hclog::{Scope, Level, FacadeVariant, Options, Result};
///
/// #[derive(Copy, Clone, Debug)]
/// enum MyLogKeys {
///   Foo,
///   Bar,
///   Baz,
/// }
///
/// // This example won't compile because the LogKeys and std::fmt::Display traits
/// // are not implemented but reqiuired. This is by intention to show the minimal
/// // implementation of the Scope trait.
///
/// impl Scope for MyLogKeys {
///     fn init<S: std::fmt::Display>(
///         name: S, level: Level, facade: FacadeVariant, options: Options
///     ) -> Result<()> {
///         hclog::init::<Self, S>(name, level, facade, options)?;
///         hclog::add_submodules(&[Self::Foo, Self::Bar, Self::Baz])
///     }
/// }
///
/// ```
///
/// ### Derive from `HCLog` via crate `hclog_macros`
///
/// For a detailed usage of the derive macro see the documentation of the crate `hclog_macros`.
///
/// ```rust
/// use hclog_macros::HCLog;
///
/// #[derive(HCLog, Copy, Clone)]
/// enum MyLogKeys {
///    Foo,
///    Bar,
///    Baz,
/// }
/// ```
///
/// # Errors
///
/// The initialzation of the log scope can fail if:
/// - the log scope is already initialized
/// - the initialization of the log scope fails
/// - the [`LOGCOMPAT`] option is set and the initialization of the
///   compatibility logger fails
///
/// # Panics
///
/// The `init` or `init_with_defaults` function might panic if the internal `RwLock` is already
/// held by the current thread as documented in [`std::sync::RwLock::write`].
///
pub trait Scope: Send + Sync + Copy {
    /// Initialization function for the log scope
    ///
    /// This function is intended to be called once from the application on the defined [`LogKey`]
    /// type. It initializes the library self - if not already happend - and creates a new
    /// [`Scope`] entry for the type it's implemented on.
    /// The passed `name` is used as the name of the log scope and is displayed in the log output
    /// if the [`BINNAME`] [`Options`] is set. The `level` is the default [`Level`] for the loggers
    /// in this scope. The `facade` is the default [`FacadeVariant`] for the loggers in this scope.
    fn init<S: Display>(
        name: S, level: Level, facade: FacadeVariant, options: Options
    ) -> Result<()>;

    /// Shortcut to [`init`] which initialize the library with default values taken from the
    /// `default_*` functions. Those functions can be overwritten by the implementing type or
    /// are attributable via the `hclog` attribute macro from the `hclog_macros` crate.
    fn init_with_defaults<S: Display>(name: S) -> Result<()> {
        Self::init(name, Self::default_level(), Self::default_facade(),
            Self::default_options())
    }

    /// Returns the [`ScopeKey`] which is used to access the [`LogKey`]s in the context.
    fn logscope() -> ScopeKey { ScopeKey::default() }
    /// default [`Options`] for the log scope
    ///
    /// if no Options are defined the [`Options::default`] is used.
    fn default_options() -> Options { Options::default() }
    /// default [`FacadeVariant`] for the log scope
    ///
    /// if no FacadeVariant is defined the [`FacadeVariant::default`] is used.
    fn default_facade() -> FacadeVariant { FacadeVariant::default() }
    /// default [`Level`] for the log scope
    ///
    /// if no Level is defined the [`Level::default`] is used.
    fn default_level() -> Level { Level::default() }
}

/// Trait for the LogKey
///
/// ## What is a LogKey?
///
/// A [`LogKey`] is a unique identifier for a set of loggers. It is used to access the loggers
/// in the context. The [`LogKey`] is used to initialize the loggers with a set of default values
/// like the [`Level`], [`FacadeVariant`] and [`Options`].
///
/// A type implementing `LogKey` must also implement the [`Scope`] and [`Display`] traits. The `Display`
/// trait is used to display the name of the `LogKey` in the log output. The `Scope` trait is used
/// store the associated `LogKey` in a defined namespace container.
///
/// # Usage
///
/// The basic usage of those Keys is to pass them to hclog provided functions and macros as an
/// unique identifier to the logger like this:
///
/// ```rust
/// use hclog::{Level, lEM};
/// # use hclog_macros::HCLog;
/// # #[derive(HCLog, Copy, Clone, Debug)]
/// enum MyLogKeys { LA }
///
/// use MyLogKeys::LA;
///
/// # MyLogKeys::init_with_defaults("MyLogKeys").unwrap();
/// let _ = hclog::set_level(LA, Level::Emerg);
/// lEM!(LA, "This is an emergency message");
/// ```
///
/// The `MyLogKeys` type is taken from the [`example`](trait@LogKey#manual-implementation) below.
///
/// # Examples
///
/// There are two ways how this trait can be implemented. Either manually or via the `HCLog` derive
/// macro from the `hclog_macros` crate:
///
/// ### Manual Implementation
///
/// ```compile_fail
/// use hclog::LogKey;
///
/// #[derive(Copy, Clone, Debug)]
/// enum MyLogKeys {
///     LA,
///     LB,
///     LC,
/// }
/// // Implement the Scope trait for the LogKeys is missing. Note that both
/// // traits (Scope + LogKey) have to be implemented for the LogKeys. Therefore
/// // this example won't compile
///
/// // Implement Display for the LogKeys because LogKey demands it
/// impl std::fmt::Display for MyLogKeys {
///     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
///         write!(f, "{}", match self {
///             Self::Foo => "foo",
///             Self::Bar => "bar",
///             Self::Baz => "baz",
///         })
///     }
/// }
///
/// // Implement the LogKey trait for the LogKeys
/// impl LogKey for MyLogKeys {
///    fn log_key(&self) -> ContextKey {
///         match self {
///             Self::LA => 0,
///             Self::LB => 1,
///             Self::LC => 2,
///         }
///     }
/// }
/// ```
///
/// ### Derive from `HCLog` via crate `hclog_macros`
///
/// For a detailed usage of the derive macro see the documentation of the crate `hclog_macros`.
///
/// ```rust
/// use hclog_macros::HCLog;
///
/// #[derive(HCLog, Copy, Clone)]
/// enum MyLogKeys {
///    Foo,
///    Bar,
///    Baz,
/// }
/// ```
///
pub trait LogKey: Scope + Display {
    /// Returns the Index key associated with the `LogKey` variant
    ///
    /// Since this trait should be usually implemented on enums this will be the enum discriminant
    fn log_key(&self) -> ContextKey;

    /// Initial [`Level`] of the LogKey
    ///
    /// If no Level is defined the [`Scope::default_level`] is used.
    fn init_level(&self) -> Option<Level> { None }
    /// Initial [`FacadeVariant`] of the LogKey
    ///
    /// If no FacadeVariant is defined the [`Scope::default_facade`] is used.
    fn init_facade(&self) -> Option<FacadeVariant> { None }
    // init_options is reserved right now and not derived automaticaly
    #[doc(hidden)]
    fn init_options(&self) -> Option<Options> { None }
}

