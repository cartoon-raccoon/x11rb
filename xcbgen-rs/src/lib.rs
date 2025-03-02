//! Structures for working with the xcb-proto XML descriptions.
//!
//! xcb-proto contains a machine readable description of the X11 protocol. This library contains
//! structures to read this XML description and to work with it. Basically, this is a Rust version
//! of xcb-proto's `xcbgen`.

#![deny(
    rust_2018_idioms,
    missing_docs,
    trivial_casts,
    trivial_numeric_casts,
    unsafe_code,
    unreachable_pub,
    unused,
    unused_qualifications,
    missing_copy_implementations,
    missing_debug_implementations,
    rustdoc::private_doc_tests,
    single_use_lifetimes,
    clippy::cast_lossless,
    clippy::needless_pass_by_value
)]
#![forbid(unsafe_code)]

pub mod defs;
mod parser;
mod resolver;

pub use parser::{ParseError, Parser};
pub use resolver::{resolve, ResolveError};
