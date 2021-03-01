# Dynamite

[![Crates.io](https://img.shields.io/crates/v/dynamite)](https://crates.io/crates/dynamite)
[![Docs.rs](https://docs.rs/dynamite/badge.svg)](https://docs.rs/dynamite)
[![Katharos License](https://img.shields.io/badge/License-Katharos-blue)](https://github.com/katharostech/katharos-license)

Dynamite is a language-agnostic scripting system for the Rust programming language. Dynamite
makes it easy to integrate scripting languages into your Rust program and is special in the way
that it orchestrates communication not only between the host program and the scripting
languages, but also allows each scripting language to interact with data in the other scripting
languages as well.

Dynamite is not currently usable, but is being developed as a component for the [Arsenal] game
engine.

## Example

### Host Application

```rust
use std::collections::HashMap;
use dynamite::*;

/// A Rust function that we want to create bindings to so that it can be called from other lanuguage
/// adapters.
fn rust_func() {
    println!("Hello from Rust!!");
}

/// The built-in "language adapter" that will provide bindings to our native Rust
struct NativeLanguageAdapter;

// We implement [`LanguageAdapter`] which is responsible for supplying the [`ScriptApi`] which
// details the available types layouts and functions provided by the adapter, and for calling
// functions provided by the adapter at the request of other adapters or the host.
//
// For this adapter
// we are just going to be providing a binding to the `rust_func` defined above. This function can
// then be called from other adapters such as the Python adapter loaded from a dynamic library
// below.
impl LanguageAdapter for NativeLanguageAdapter {
    fn get_api(&self, _host_functions: &dyn HostFunctions) -> ScriptApi {
        let mut api = ScriptApi::new();

        api.insert(
            "native::rust_func".into(),
            ScriptType::Function(FunctionDefinition {
                arguments: HashMap::new(),
                return_type: None,
            }),
        );

        api
    }

    fn call_function(
        &self,
        _host_functions: &dyn HostFunctions,
        path: &str,
        _args: &[*const Void],
    ) -> *const Void {
        if path == "native::rust_func" {
            rust_func();
            std::ptr::null()
        } else {
            panic!("Function not defined");
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize dynamite
    let mut dynamite = Dynamite::new();

    // Add our native Rust adapter
    dynamite.add_language_adapter(Box::new(NativeLanguageAdapter))?;

    // Load langauge adapter ( relatively safe, but still unsafe because dynamic libraries could do
    // _anything_ 👀 )
    unsafe {
        dynamite.load_dynamic_library_language_adapter("./target/debug/libdynamite_python.so")?
    };

    // Print discovered api
    dbg!(dynamite.get_full_api());

    // Call a function provided by the language adapter ( just assuming for this example that we
    // know ahead of time that this function exists, it would error if it didn't ). This is also
    // unsafe because your language adapter could mis-behave.
    let arg1 = &42f32;
    unsafe {
        dynamite.call_function(
            &"python::test_function".to_string(),
            &[arg1 as *const f32 as *const Void],
        );
    }

    Ok(())
}
```

### Language Adapter

_This isn't really a Python language adapter, it's really just Rust, but we'll add Python later 😉_

```rust
use std::collections::HashMap;
use dynamite::*;

/// The Dynamite Python language adapter
#[language_adapter]
struct PythonAdapter;

impl DynamicLibLanguageAdapter for PythonAdapter {
    /// Initialize adapter
    fn init_adapter() -> Self {
        PythonAdapter
    }
}

impl LanguageAdapter for PythonAdapter {
    /// Get the adapter's API
    fn get_api(&self, _host_functions: &dyn HostFunctions) -> ScriptApi {
        let mut components = ScriptApi::default();

        components.insert(
            "python::test_function".into(),
            ScriptType::Function(FunctionDefinition {
                arguments: {
                    let mut h = HashMap::new();

                    h.insert("number".into(), "std::f32".into());

                    h
                },
                return_type: None,
            }),
        );

        components
    }

    /// Call functions provided by this adapter
    fn call_function(
        &self,
        host_functions: &dyn HostFunctions,
        path: &str,
        args: &[*const dynamite::Void],
    ) -> *const dynamite::Void {
        if path == "python::test_function" {
            let arg1 = args[0];

            let number = unsafe { &*(arg1 as *const f32) };

            println!("The number is: {}", number);

            dbg!(host_functions.get_full_api());

            unsafe {
                host_functions.call_function(&"native::rust_func".to_string(), &[]);
            }
        }

        std::ptr::null()
    }
}
```

[Arsenal]: https://github.com/katharostech/arsenal
