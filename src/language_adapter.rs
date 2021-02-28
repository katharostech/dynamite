use crate::{Erased, ScriptApi};

/// Type implementing this trait can be loaded as dynamite language adapters wgeb
pub trait LanguageAdapter {
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

pub trait DynamicLibLanguageAdapter {
    /// Initialize the language adapter
    fn init_adapter(host_functions: &HostFunctions) -> Self;
}

/// A language adapter loaded from a dynamic library
pub struct LoadedDynamicLibLanguageAdapter {
    /// the container for the adapter's C API
    api: Container<LanguageAdapterCApi>,
}

impl LoadedDynamicLibLanguageAdapter {
    pub fn new(api: Container<LanguageAdapterCApi>) -> Self {
        Self { api }
    }
}

impl LanguageAdapter for LoadedDynamicLibLanguageAdapter {
    fn get_api(&self, host_functions: &HostFunctions) -> ScriptApi {
        serde_cbor::from_slice(&self.api.get_api(host_functions.as_ref()))
            .expect("Could not parse CBOR api data from language adapter")
    }

    fn call_function(
        &self,
        host_functions: &HostFunctions,
        path: &str,
        args: &[*const Erased],
    ) -> *const Erased {
        self.api
            .call_function(host_functions.as_ref(), path.into(), args.into())
    }
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

impl AsRef<CHostFunctions> for HostFunctions {
    fn as_ref(&self) -> &CHostFunctions {
        &self.0
    }
}

pub use capi::*;
use dlopen::wrapper::Container;
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
        init_adapter: fn(host_functions: &CHostFunctions),

        /// Get a catalog of all of the components discovered by the adapter. The return value of
        /// the function must be a vector of bytes in the CBOR format corresponding to a serialized
        /// [`ScriptApi`].
        get_api: fn(host_functions: &CHostFunctions) -> safer_ffi::Vec<u8>,

        /// Execute a function that is hosted by the language adapter.
        call_function: fn(
            host_functions: &CHostFunctions,
            path: str::Ref,
            args: c_slice::Ref<*const Erased>,
        ) -> *const Erased,
    }
}
