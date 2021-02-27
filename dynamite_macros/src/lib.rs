use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_attribute]
pub fn language_adapter(_args: TokenStream, input: TokenStream) -> TokenStream {
    let input_ = input.clone();
    let derive_input = parse_macro_input!(input_ as DeriveInput);
    impl_language_adapter(derive_input, input.into()).into()
}

fn impl_language_adapter(derive_input: DeriveInput, raw_input: TokenStream2) -> TokenStream2 {
    let macros_private = quote! { ::dynamite::_macros_private };
    let adapter_ty = derive_input.ident;

    let out = quote! {
        // Output the input unchanged
        #raw_input

        // Create cell for holding the host functions that we will get upon adapter initialization
        static HOST_FUNCTIONS: #macros_private::once_cell::sync::OnceCell<HostFunctions>
            = #macros_private::once_cell::sync::OnceCell::new();

        // Create cell for the adapter
        static ADAPTER: #macros_private::once_cell::sync::OnceCell<#adapter_ty>
            = #macros_private::once_cell::sync::OnceCell::new();

        #[safer_ffi::ffi_export]
        fn init_adapter(c_host_functions: dynamite::CHostFunctions) {
            // Create more Rusty HostFunctions from CHostFunctions
            let host_funcs = dynamite::HostFunctions::new(c_host_functions);

            // Initialize host functions cell
            HOST_FUNCTIONS.set(host_funcs).map_err(|_| "Cannot initialize cell twice!").unwrap();

            // Initialize adapter
            ADAPTER.set(#adapter_ty::init_adapter(HOST_FUNCTIONS.get().unwrap()))
                .map_err(|_| "Cannot initialize cell twice!").unwrap();
        }

        #[safer_ffi::ffi_export]
        fn get_api() -> safer_ffi::prelude::repr_c::Vec<u8> {
            let e = "Adapter not initialized";
            // Get the adapter
            let adapter = ADAPTER.get().expect(e);

            // get the api from the adapter
            let api = adapter.get_api(HOST_FUNCTIONS.get().expect(e));

            // Serialize the API and return the bytes
            #macros_private::serde_cbor::to_vec(&api)
                .expect("Could not serialize language adapter API").into()
        }

        #[safer_ffi::ffi_export]
        fn call_function(
            path: safer_ffi::prelude::str::Ref,
            args: safer_ffi::prelude::c_slice::Ref<*const Erased>
        ) -> *const dynamite::Erased {
            let e = "Adapter not initialized";
            // Get the adapter
            let adapter = ADAPTER.get().expect(e);

            // Forward the call to the adapter
            adapter.call_function(
                HOST_FUNCTIONS.get().expect(e),
                path.as_str(),
                args.as_slice(),
            )
        }
    };

    out
}
