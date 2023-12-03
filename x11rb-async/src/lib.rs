// This code is dual licensed under MIT OR Apache 2.0.

//! Asynchronous X11 rust bindings.
//!
//! This library allows to interact with an X11 server from rust code. A connection to an X11
//! server is represented by an implementation of the `Connection` trait.
//!
//! The client can interact with the server by sending requests. The server can answer requests and
//! can also generate events.
//!
//! The examples that come with this library might be a good starting point for new users.
//!
//!
//! # Feature flags
//!
//! This crate uses [feature
//! flags](https://doc.rust-lang.org/cargo/reference/manifest.html#the-features-section) to reduce
//! the amount of compiled code. There are two kinds of feature flags available:
//!
//! * Feature flags for specific X11 extensions
//! * Feature flags for additional functionality
//!
//! ## Feature flags for specific X11 extensions
//!
//! By default, only the core X11 protocol and X11 extensions that are needed internally are
//! enabled. These are the `bigreq`, `ge` and `xc_misc` extensions. Further extensions need to be
//! explicitly enabled via their feature flag:
//!
//! `composite`, `damage`, `dpms`, `dri2`, `dri3`, `glx`, `present`, `randr`, `record`, `render`,
//! `res`, `screensaver`, `shape`, `shm`, `sync`, `xevie`, `xf86dri`, `xf86vidmode`, `xfixes`,
//! `xinerama`, `xinput`, `xkb`, `xprint`, `xselinux`, `xtest`, `xv`, `xvmc`.
//!
//! If you want to take the "I do not want to think about this"-approach, you can enable the
//! `all-extensions` feature to just enable, well, all extensions.
//!
//! ## Feature flags for additional functionality
//!
//! Additionally, the following flags exist:
//! * `allow-unsafe-code`: Enable the same feature in x11rb and implement
//! [`blocking::BlockingConnection`] for [`x11rb::xcb_ffi::XCBConnection`]
//! * `extra-traits`: Implement extra traits for X11 types. This improves the output of the `Debug`
//!   impl and adds `PartialEq`, `Eq`, `PartialOrd`, `Ord`, and `Hash` where possible.

// -- Public Modules --

pub mod blocking;
pub mod connection;
#[allow(clippy::type_complexity)]
#[rustfmt::skip]
pub mod protocol;
pub mod rust_connection;

#[doc(inline)]
pub use x11rb::{errors, x11_utils};

#[doc(inline)]
pub use x11rb_protocol::SequenceNumber;

// -- Private Modules --

mod cookie;

pub use cookie::{Cookie, CookieWithFds, VoidCookie};

pub mod utils {
    pub use x11rb::utils::RawFdContainer;
}
