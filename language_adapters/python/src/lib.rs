use dynamite::*;
use std::collections::HashMap;

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
    unsafe fn call_function(
        &self,
        host_functions: &dyn HostFunctions,
        path: &str,
        args: &[*const dynamite::Void],
    ) -> *const dynamite::Void {
        if path == "python::test_function" {
            let arg1 = args[0];

            let number = &*(arg1 as *const f32);

            println!("The number is: {}", number);

            dbg!(host_functions.get_full_api());

            let a = &32;
            let b = &44;

            let ret = host_functions.call_function(
                &"hello_world::rust_func".to_string(),
                &[
                    a as *const i32 as *const Void,
                    b as *const i32 as *const Void,
                ],
            ) as *const i32;

            println!("Got number back: {}", &*ret);
        }

        std::ptr::null()
    }
}
