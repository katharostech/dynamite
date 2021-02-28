use std::collections::HashMap;

use dynamite::*;

/// The Dynamite Python language adapter
#[language_adapter]
struct PythonAdapter;

impl DynamicLibLanguageAdapter for PythonAdapter {
    /// Initialize adapter
    fn init_adapter(_host_functions: &HostFunctions) -> Self {
        PythonAdapter
    }
}

impl LanguageAdapter for PythonAdapter {
    /// Get the adapter's API
    fn get_api(&self, _host_functions: &HostFunctions) -> ScriptApi {
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
        _host_functions: &HostFunctions,
        path: &str,
        args: &[*const dynamite::Erased],
    ) -> *const dynamite::Erased {
        if path == "python::test_function" {
            let arg1 = args[0];

            let number = unsafe { &*(arg1 as *const i32) };

            println!("The number is: {}", number);
        }

        std::ptr::null()
    }
}
