//! Dynamite is a language-agnostic scripting system for the Rust programming language. Dynamite
//! makes it easy to integrate scripting languages into your Rust program and is special in the way
//! that it orchestrates communication not only between the host program and the scripting
//! languages, but also allows each scripting language to interact with data in the other scripting
//! languages as well.
//!
//! Dynamite is not currently usable, but is being developed as a component for the [Arsenal] game
//! engine.
//!
//! # Example
//!
//! ## Host Application
//!
//! ```no_run
//! use std::collections::HashMap;
//! use dynamite::*;
//!
//! /// A Rust function that we want to create bindings to so that it can be called from other lanuguage
//! /// adapters.
//! fn rust_func() {
//!     println!("Hello from Rust!!");
//! }
//!
//! /// The built-in "language adapter" that will provide bindings to our native Rust
//! struct NativeLanguageAdapter;
//!
//! // We implement [`LanguageAdapter`] which is responsible for supplying the [`ScriptApi`] which
//! // details the available types layouts and functions provided by the adapter, and for calling
//! // functions provided by the adapter at the request of other adapters or the host.
//! //
//! // For this adapter
//! // we are just going to be providing a binding to the `rust_func` defined above. This function can
//! // then be called from other adapters such as the Python adapter loaded from a dynamic library
//! // below.
//! impl LanguageAdapter for NativeLanguageAdapter {
//!     fn get_api(&self, _host_functions: &dyn HostFunctions) -> ScriptApi {
//!         let mut api = ScriptApi::new();
//!
//!         api.insert(
//!             "native::rust_func".into(),
//!             ScriptType::Function(FunctionDefinition {
//!                 arguments: HashMap::new(),
//!                 return_type: None,
//!             }),
//!         );
//!
//!         api
//!     }
//!
//!     fn call_function(
//!         &self,
//!         _host_functions: &dyn HostFunctions,
//!         path: &str,
//!         _args: &[*const Void],
//!     ) -> *const Void {
//!         if path == "native::rust_func" {
//!             rust_func();
//!             std::ptr::null()
//!         } else {
//!             panic!("Function not defined");
//!         }
//!     }
//! }
//!
//! fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Initialize dynamite
//!     let mut dynamite = Dynamite::new();
//!
//!     // Add our native Rust adapter
//!     dynamite.add_language_adapter(Box::new(NativeLanguageAdapter))?;
//!
//!     // Load langauge adapter ( relatively safe, but still unsafe because dynamic libraries could do
//!     // _anything_ 👀 )
//!     unsafe {
//!         dynamite.load_dynamic_library_language_adapter("./target/debug/libdynamite_python.so")?
//!     };
//!
//!     // Print discovered api
//!     dbg!(dynamite.get_full_api());
//!
//!     // Call a function provided by the language adapter ( just assuming for this example that we
//!     // know ahead of time that this function exists, it would error if it didn't ). This is also
//!     // unsafe because your language adapter could mis-behave.
//!     let arg1 = &42f32;
//!     unsafe {
//!         dynamite.call_function(
//!             &"python::test_function".to_string(),
//!             &[arg1 as *const f32 as *const Void],
//!         );
//!     }
//!
//!     Ok(())
//! }
//! ```
//!
//! ## Language Adapter
//!
//! _This isn't really a Python language adapter, it's really just Rust, but we'll add Python later 😉_
//!
//! ```ignore
//! use std::collections::HashMap;
//! use dynamite::*;
//!
//! /// The Dynamite Python language adapter
//! #[language_adapter]
//! struct PythonAdapter;
//!
//! impl DynamicLibLanguageAdapter for PythonAdapter {
//!     /// Initialize adapter
//!     fn init_adapter() -> Self {
//!         PythonAdapter
//!     }
//! }
//!
//! impl LanguageAdapter for PythonAdapter {
//!     /// Get the adapter's API
//!     fn get_api(&self, _host_functions: &dyn HostFunctions) -> ScriptApi {
//!         let mut components = ScriptApi::default();
//!
//!         components.insert(
//!             "python::test_function".into(),
//!             ScriptType::Function(FunctionDefinition {
//!                 arguments: {
//!                     let mut h = HashMap::new();
//!
//!                     h.insert("number".into(), "std::f32".into());
//!
//!                     h
//!                 },
//!                 return_type: None,
//!             }),
//!         );
//!
//!         components
//!     }
//!
//!     /// Call functions provided by this adapter
//!     fn call_function(
//!         &self,
//!         host_functions: &dyn HostFunctions,
//!         path: &str,
//!         args: &[*const dynamite::Void],
//!     ) -> *const dynamite::Void {
//!         if path == "python::test_function" {
//!             let arg1 = args[0];
//!
//!             let number = unsafe { &*(arg1 as *const f32) };
//!
//!             println!("The number is: {}", number);
//!
//!             dbg!(host_functions.get_full_api());
//!
//!             unsafe {
//!                 host_functions.call_function(&"native::rust_func".to_string(), &[]);
//!             }
//!         }
//!
//!         std::ptr::null()
//!     }
//! }
//! ```
//!
//! [Arsenal]: https://github.com/katharostech/arsenal

#[macro_use]
extern crate dlopen_derive;

use std::{collections::HashMap, ffi::OsStr};

// Language adapter traits and types
mod language_adapter;
pub use language_adapter::*;

// Script api types
mod script_api;
pub use script_api::*;

// Macros
pub use dynamite_macros::*;

// Libs used by the macros but not a part of the public API
#[doc(hidden)]
pub mod _macros_private {
    pub use once_cell;
    pub use serde_cbor;
}

/// The main struct used to create a Dynamite host and load language adapters
#[derive(Default)]
pub struct Dynamite {
    /// The set of language adapters
    adapters: Vec<Box<dyn LanguageAdapter>>,

    /// A cache of the APIs provided by loaded adapters
    api_cache: Vec<ScriptApi>,

    /// Mapping of [`TypePath`]s to the adapter/api_cache index that provides that type
    type_adapter_index: HashMap<TypePath, usize>,
}

impl Dynamite {
    /// Create a new dynamite host
    pub fn new() -> Self {
        Default::default()
    }

    /// Load a language adapter from a dynamically linked library
    ///
    /// This allows you to load language adapters from .dll ( Windows ), .so ( Linux ), or .dylib (
    /// Mac ) files.
    pub unsafe fn load_dynamic_library_language_adapter<P: AsRef<OsStr>>(
        &mut self,
        path: P,
    ) -> Result<(), DynamiteError> {
        // Create the C function pointers used to call dynamite functions from the dynamic library
        let pointers = CHostFunctionPointers {
            get_full_api: ffi::dynamite_get_full_api,
            call_function: ffi::dynamite_call_function,
        };

        // Add the language adapter
        self.add_language_adapter(Box::new(LoadedDynamicLibLanguageAdapter::load(
            path, pointers,
        )?))?;

        Ok(())
    }

    /// Add a language adapter from any type implementing [`LanguageAdapter`]
    ///
    /// This can be used to easily add native Rust bindings to the scripting API.
    pub fn add_language_adapter(
        &mut self,
        adapter: Box<dyn LanguageAdapter>,
    ) -> Result<(), ScriptApiError> {
        // Load the adapter API
        let api = adapter.get_api(self);

        // Check for conflicting types
        for path in api.keys() {
            if self.type_adapter_index.contains_key(path) {
                return Err(ScriptApiError::TypeRedefined(path.clone()));
            }
        }

        // Add types to the adapter type_adapter_index
        for path in api.keys() {
            // Map the type path to the the index of this adapter
            self.type_adapter_index
                .insert(path.clone(), self.api_cache.len());
        }

        // Add the types to the cache
        self.api_cache.push(api);

        // Add the adapter to the list
        self.adapters.push(adapter);

        Ok(())
    }
}

impl HostFunctions for Dynamite {
    fn get_full_api(&self) -> ScriptApi {
        let mut full_api = ScriptApi::new();

        for api in &self.api_cache {
            full_api.extend(api.clone().into_iter());
        }

        full_api
    }

    fn as_dynamite(&self) -> &Dynamite {
        self
    }

    unsafe fn call_function(&self, path: &TypePath, args: &[*const Void]) -> *const Void {
        let adapter = self
            .adapters
            .get(
                *self
                    .type_adapter_index
                    .get(path)
                    .ok_or(ScriptApiError::NotFound(path.clone()))
                    .expect("TODO"),
            )
            .expect("Internal error finding adapter");

        adapter.call_function(self, path, args)
    }
}

mod ffi {
    use super::*;
    use safer_ffi::prelude::*;

    /// C function for getting the full dynamite API
    pub(super) extern "C" fn dynamite_get_full_api(dynamite: *const Void) -> repr_c::Vec<u8> {
        let dynamite = unsafe { &*(dynamite as *const Dynamite) };

        serde_cbor::to_vec(&dynamite.get_full_api())
            .expect("Could not serialize script API")
            .into()
    }

    /// C function for calling an API function
    pub(super) extern "C" fn dynamite_call_function(
        dynamite: *const Void,
        path: str::Ref,
        args: c_slice::Ref<*const Void>,
    ) -> *const Void {
        let dynamite = unsafe { &*(dynamite as *const Dynamite) };

        // TODO: Get rid of this `to_string` call
        unsafe { dynamite.call_function(&path.as_ref().to_string(), &args) }
    }
}

pub use error::*;
mod error {
    use super::*;

    /// An error that ocurred when trying to access the scripting API
    #[derive(thiserror::Error, Debug)]
    pub enum DynamiteError {
        #[error("Script API error: {0}")]
        ApiError(#[from] ScriptApiError),
        #[error("Error loading dynamic library: {0}")]
        DynamicLibError(#[from] dlopen::Error),
    }

    /// An error that ocurred when trying to access the scripting API
    #[derive(thiserror::Error, Debug)]
    pub enum ScriptApiError {
        #[error("The requested API element was not found: {0}")]
        NotFound(TypePath),
        #[error("Loaded adapter re-defineds type already defined by another adapter: {0}")]
        TypeRedefined(TypePath),
    }
}
