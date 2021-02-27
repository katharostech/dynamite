//! Dynamite is a language-agnostic scripting system for the Rust programming language. Dynamite
//! makes it easy to integrate scripting languages into your Rust program and is special in the way
//! that it orchestrates communication not only between the host program and the scripting
//! languages, but also allows each scripting language to interact with data in the other scripting
//! languages as well.
//!
//! Dynamite is not currently usable, but is being developed as a component for the [Arsenal] game
//! engine.
//!
//! [Arsenal]: https://github.com/katharostech/arsenal

#[macro_use]
extern crate dlopen_derive;

mod language_adapter;
pub use language_adapter::*;

mod types;
pub use types::*;

#[cfg(feature = "derive")]
pub use dynamite_macros::*;

// Libs used by the macros when the derive feature is enabled
#[cfg(feature = "derive")]
#[doc(hidden)]
pub mod _macros_private {
    pub use once_cell;
    pub use serde_cbor;
}
