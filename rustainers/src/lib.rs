#![warn(missing_docs)]
#![forbid(unsafe_code)]
#![warn(clippy::perf)]
#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]
#![doc = include_str!("../README.md")]

mod error;
pub use self::error::*;

mod container;
pub use self::container::*;

mod image;
pub use self::image::*;

mod port;
pub use self::port::*;

mod id;
pub(crate) use self::id::Id;

pub(crate) mod cmd;

pub(crate) mod version;

pub(crate) mod io;

/// Runners like docker, podman, ...
pub mod runner;

/// Provided images like postgres, redis, ...
pub mod images;

/// Provide support of compose
pub mod compose;

/// Provide tools
pub mod tools;
