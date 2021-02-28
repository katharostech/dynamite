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
use std::ffi::OsStr;

use dlopen::wrapper::Container;
pub use language_adapter::*;

mod types;
pub use types::*;

pub use dynamite_macros::*;

// Libs used by the macros
#[doc(hidden)]
pub mod _macros_private {
    pub use once_cell;
    pub use serde_cbor;
}

/// The main struct used to create a Dynamite host and load language adapters
#[derive(Default)]
pub struct Dynamite {
    adapters: Vec<Box<dyn LanguageAdapter>>,
}

impl Dynamite {
    /// Create a new dynamite host
    pub fn new() -> Self {
        Default::default()
    }

    /// Load a language adapter
    pub fn load_dynamic_library<P: AsRef<OsStr>>(&mut self, path: P) -> Result<(), dlopen::Error> {
        let api: Container<LanguageAdapterCApi> = unsafe { Container::load(path)? };

        self.adapters
            .push(Box::new(LoadedDynamicLibLanguageAdapter::new(api)));

        Ok(())
    }
}
