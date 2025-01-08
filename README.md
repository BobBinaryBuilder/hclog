# hclog

hclog stands for highly configurable logging library for Rust.

# Compatibility

hclog is compatible with rustc >= 1.74.1 (MSRV).

# Overview

hclog is designed and intended to be as flexible and configurable as possible.

## Logging Level
hclog allows you to use a fine granular logging with Loglevels compatible to the unix
syslog. Instead of a single DEBUG Stage hclog allows you to use 10 different DEBUG level.

## The Log Message

The generated logline can be configured depending on your needs and may include things
like Rust module-path, filename, linenumber or the function name. This options can be
set on intialization of the library or on demand.

# Documentation

A complete documentantion is available at [docs.rs](https://docs.rs/hclog/latest/hclog).


## Include hclog in your project

```toml
[dependencies]
hclog = "0.1.0"
hclog_macros = "0.1.0"

# with the "derive" feature you can use the macros directly from hclog
hclog = { version = "0.1.0", features = ["derive"] }
```

## Macros

The hclog_macro crate provides a <code>#[derive(HCLog)]</code> procmacro to be used with
(currently only) enum declarations to derive all required traits.
All log calls are encapsulated in declarative macros like <code>lE</code>, <code>lI</code>
or <code>lD1</code>

# Community, discussion, contribution, and support

## Security

We take security seriously.
Please read our [security policy](SECURITY.md) for information on how to report security issues.
