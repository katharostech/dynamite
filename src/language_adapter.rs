use std::ffi::OsStr;

use crate::{Dynamite, ScriptApi, Void};

/// Type implementing this trait can be loaded as dynamite language adapters wgeb
pub trait LanguageAdapter {
    /// Get the [`ScriptApi`] provided by this language adapter
    fn get_api(&self, host_functions: &dyn HostFunctions) -> ScriptApi;

    /// Call a function provided by the language adapter
    fn call_function(
        &self,
        host_functions: &dyn HostFunctions,
        path: &str,
        args: &[*const Void],
    ) -> *const Void;
}

pub trait DynamicLibLanguageAdapter {
    /// Initialize the language adapter
    fn init_adapter() -> Self;
}

/// A language adapter loaded from a dynamic library
pub struct LoadedDynamicLibLanguageAdapter {
    /// the container for the adapter's C API
    api: Container<LanguageAdapterCApi>,
}

impl<'a> LoadedDynamicLibLanguageAdapter {
    /// Load a dynamic lib language adapter
    pub unsafe fn load<P: AsRef<OsStr>>(
        path: P,
        host_functions: CHostFunctionPointers,
    ) -> Result<Self, dlopen::Error> {
        // Load the dynamic library
        let api: Container<LanguageAdapterCApi> = Container::load(path)?;

        // Initialize the adapter
        api.init_adapter(host_functions);

        Ok(Self { api })
    }
}

impl LanguageAdapter for LoadedDynamicLibLanguageAdapter {
    fn get_api(&self, host_functions: &dyn HostFunctions) -> ScriptApi {
        let bytes = self
            .api
            .get_api(host_functions.as_dynamite() as *const Dynamite as *const Void);

        serde_cbor::from_slice(&bytes).expect("Could not parse CBOR api data from language adapter")
    }

    fn call_function(
        &self,
        host_functions: &dyn HostFunctions,
        path: &str,
        args: &[*const Void],
    ) -> *const Void {
        self.api.call_function(
            host_functions.as_dynamite() as *const Dynamite as *const Void,
            path.into(),
            args.into(),
        )
    }
}

/// Functions provided by the host
pub trait HostFunctions {
    fn as_dynamite(&self) -> &Dynamite;
    fn get_full_api(&self) -> ScriptApi;
}

pub use capi::*;
use dlopen::wrapper::Container;
#[allow(missing_docs)]
mod capi {
    use crate::{Dynamite, HostFunctions, Void};
    use dlopen::wrapper::WrapperApi;
    use safer_ffi::prelude::*;

    /// Pointers to the C functions that the host provides for use by the language adapters
    #[derive_ReprC]
    #[repr(C)]
    #[derive(Clone)]
    pub struct CHostFunctionPointers {
        /// Get the full CBOR serialized [`ScriptApi`] including components discovered and
        /// implemented by other language adapters or the dynamite host.
        pub get_full_api: extern "C" fn(dynamite: *const Void) -> repr_c::Vec<u8>,
    }

    /// A wrapper that allows idiomatic access to the Rust host functions from a dynamically loaded
    /// language adapter.
    #[derive_ReprC]
    #[repr(C)]
    pub struct RemoteHostFunctions {
        pub dynamite: *const Void,
        pub pointers: CHostFunctionPointers,
    }

    impl HostFunctions for RemoteHostFunctions {
        fn get_full_api(&self) -> crate::ScriptApi {
            // Get the bytes of the API
            let bytes = (self.pointers.get_full_api)(self.dynamite);

            // Parse the btyes as a ScriptAPI
            serde_cbor::from_slice(&bytes)
                .expect("Could not parse CBOR API definition from language adapter")
        }

        fn as_dynamite(&self) -> &Dynamite {
            unsafe { &*(self.dynamite as *const Dynamite) }
        }
    }

    /// The C API implemented by language adapters
    #[derive(WrapperApi)]
    pub struct LanguageAdapterCApi {
        /// Initialize the language adapter
        init_adapter: fn(host_functions: CHostFunctionPointers),

        /// Get a catalog of all of the components discovered by the adapter. The return value of
        /// the function must be a vector of bytes in the CBOR format corresponding to a serialized
        /// [`ScriptApi`].
        get_api: fn(dynamite: *const Void) -> safer_ffi::Vec<u8>,

        /// Execute a function that is hosted by the language adapter.
        call_function: fn(
            dynamite: *const Void,
            path: str::Ref,
            args: c_slice::Ref<*const Void>,
        ) -> *const Void,
    }
}
