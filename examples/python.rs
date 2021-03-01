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
