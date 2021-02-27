use std::path::Path;

use crate::{Erased, ScriptApi};

/// The main struct used to create a Dynamite host and load language adapters
#[derive(Default)]
pub struct Dynamite {
    // _adapters: Vec<LanguageAdapter>,
}

impl Dynamite {
    /// Create a new dynamite host
    pub fn new() -> Self {
        Default::default()
    }

    /// Load a language adapter
    pub fn load_adapter<P: AsRef<Path>>(_path: P) -> Result<(), ModuleLoadError> {
        Ok(())
    }
}

#[derive(thiserror::Error, Clone, Debug)]
#[error("Error loading module")]
pub enum ModuleLoadError {}

/// Type implementing this trait can be loaded as dynamite language adapters wgeb
pub trait LanguageAdapter {
    /// Initialize the language adapter
    fn init_adapter(host_functions: &HostFunctions) -> Self;

    /// Get the [`ScriptApi`] provided by this language adapter
    fn get_api(&self, host_functions: &HostFunctions) -> ScriptApi;

    /// Call a function provided by the language adapter
    fn call_function(
        &self,
        host_functions: &HostFunctions,
        path: &str,
        args: &[*const Erased],
    ) -> *const Erased;
}

/// Functions provided by the host
pub struct HostFunctions(CHostFunctions);

impl HostFunctions {
    pub fn new(c_funcs: CHostFunctions) -> Self {
        Self(c_funcs)
    }

    /// Get the [`ScriptApi`] for the whole app
    pub fn get_full_api(&self) -> ScriptApi {
        let bytes = (self.0.get_full_api)();
        serde_cbor::from_reader(&*bytes)
            .expect("Could not parse CBOR component data from language adapter")
    }
}

pub use capi::*;
#[allow(missing_docs)]
mod capi {
    use crate::Erased;
    use dlopen::wrapper::WrapperApi;
    use safer_ffi::prelude::*;

    #[derive_ReprC]
    #[repr(C)]
    pub struct CHostFunctions {
        /// Get the full CBOR serialized [`ScriptApi`] including components discovered and
        /// implemented by other language adapters or the dynamite host.
        pub get_full_api: extern "C" fn() -> repr_c::Vec<u8>,
    }

    /// The C API implemented by language adapters
    #[derive(WrapperApi)]
    pub struct LanguageAdapterCApi {
        /// Initialize the language adapter. The implementation is required to be idempotent and
        /// should be allowed to be called multiple times without negative side-effects.
        init_adapter: fn(args: CHostFunctions),

        /// Get a catalog of all of the components discovered by the adapter. The return value of
        /// the function must be a vector of bytes in the CBOR format corresponding to a serialized
        /// [`ScriptApi`].
        get_api: fn() -> safer_ffi::Vec<u8>,

        /// Execute a function that is hosted by the language adapter.
        call_function: fn(path: str::Ref, args: c_slice::Ref<*const Erased>) -> *const Erased,
    }
}
