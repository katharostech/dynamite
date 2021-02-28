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

```rust
use dynamite::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize dynamite
    let mut dynamite = Dynamite::new();

    // Load langauge adapter ( relatively safe, but still unsafe because dynamic libraries
    // could do _anything_ 👀 )
    unsafe {
        dynamite.load_dynamic_library_language_adapter("./target/debug/libdynamite_python.so")?
    };

    // Print discovered api. This is a full description of the combined dynamic API loaded from
    // all scripting adapters ( but currrently just the Python one ).
    dbg!(dynamite.get_full_api());

    // Call a function provided by the language adapter ( just assuming for this example that we
    // know ahead of time that this function exists, it would error if it didn't ). This is also
    // unsafe because your language adapter could mis-behave.
    let arg1 = &42f32;
    unsafe {
        dynamite.call_function(
            &"python::test_function".to_string(),
            &[arg1 as *const f32 as *const Void],
        )?;
    }

    Ok(())
}
```

[Arsenal]: https://github.com/katharostech/arsenal
