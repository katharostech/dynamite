use dynamite::*;
use lazy_static::lazy_static;
use safer_ffi::{prelude::*, string::str_ref};

use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

/// Helper to get the adapter state and panic if the adapter is uninitialized. Implemented as a
/// macro to make the RwLock read guard work easily.
macro_rules! get_adapter {
    ($a:ident) => {
        let $a;
        let guard;
        guard = ADAPTER.read().unwrap();
        $a = guard.as_ref().expect("Adapter not initialized");
    };
}

/// The adapter state
struct AdapterState {
    log_info: extern "C" fn(repr_c::String),
}

lazy_static! {
    /// The adapter global state
    static ref ADAPTER: Arc<RwLock<Option<AdapterState>>> = Arc::new(RwLock::new(None));
}

/// Initialize the adapter
#[ffi_export]
fn init_adapter<'a>(args: &'a mut LanguageAdapterInitArgs) {
    if ADAPTER.read().unwrap().is_none() {
        (args.log_info)(repr_c::String::from(
            "Initialized Python adapter".to_string(),
        ));

        *ADAPTER.write().unwrap() = Some(AdapterState {
            log_info: args.log_info,
        });
    }
}

/// Get the components discovered by this adapter
#[ffi_export]
fn get_components() -> repr_c::Vec<u8> {
    get_adapter!(adapter);

    (adapter.log_info)(repr_c::String::from(
        "Loading Python components".to_string(),
    ));

    let mut components = ScriptApi::new();

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

    serde_cbor::to_vec(&components).unwrap().into()
}

#[ffi_export]
fn get_function_pointer(path: str_ref) -> *const Erased {
    get_adapter!(adapter);

    let s = path.as_str();

    (adapter.log_info)(repr_c::String::from(format!(
        "getting function pointer for: {}",
        s
    )));

    match s {
        "python::test_function" => test_function as *const Erased,
        _ => panic!("Unidentified function"),
    }
}

extern "C" fn test_function(args: c_slice::Ref<*const Erased>) -> *const Erased {
    println!("Test Function!");

    let arg1 = args[0];

    let number = unsafe { &*(arg1 as *const i32) };

    println!("The number is: {}", number);

    std::ptr::null()
}
