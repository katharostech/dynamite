use std::collections::HashMap;

use crate::{LanguageAdapter, ScriptApi, ScriptApiError, ScriptType, TypePath, Void};

/// A [`LanguageAdapter`] that uses the [`inventory`] crate to pull in API elements from the entire
/// crate graph.
pub struct Stockpile {
    api: ScriptApi,
    function_pointers: HashMap<TypePath, unsafe fn(args: &[*const Void]) -> *const Void>,
}

impl Stockpile {
    pub fn new() -> Result<Self, ScriptApiError> {
        let mut api = ScriptApi::new();
        let mut function_pointers = HashMap::new();

        // Loop through items in the stockpile and add them to the API
        for item in inventory::iter::<StockpileItem> {
            if api
                .insert(item.path.clone(), item.script_type.clone())
                .is_some()
            {
                // Return an error if we are defining the same type path twice
                return Err(ScriptApiError::TypeRedefined(item.path.clone()));
            }

            // Register the function pointer if present
            if let Some(pointer) = item.function_pointer {
                function_pointers.insert(item.path.clone(), pointer);
            }
        }

        Ok(Self {
            api,
            function_pointers,
        })
    }
}

impl LanguageAdapter for Stockpile {
    fn get_api(&self, _host_functions: &dyn crate::HostFunctions) -> crate::ScriptApi {
        self.api.clone()
    }

    unsafe fn call_function(
        &self,
        _host_functions: &dyn crate::HostFunctions,
        path: &str,
        args: &[*const crate::Void],
    ) -> *const crate::Void {
        if let Some(function_pointer) = self.function_pointers.get(path) {
            (function_pointer)(args)
        } else {
            panic!("Call to non-existent function");
        }
    }
}

/// An item in the Dynamite stockpile
#[derive(Clone)]
pub struct StockpileItem {
    /// The path of the item
    pub path: TypePath,
    /// The script type of the item
    pub script_type: ScriptType,
    /// A function pointer to register with this type, if it's a function
    pub function_pointer: Option<unsafe fn(args: &[*const Void]) -> *const Void>,
}

inventory::collect!(StockpileItem);

#[macro_export]
macro_rules! add_binding {
    ($item:expr) => {
        inventory::submit!($item);
    }
}