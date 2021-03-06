use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

/// Attribute macro that can be used to implement a Dynamite language adapter
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

        mod ffi {
            use dynamite::{DynamicLibLanguageAdapter, LanguageAdapter};

            // Create cell for holding the host functions that we will get upon adapter initialization
            static HOST_FUNCTION_POINTERS:
                #macros_private::once_cell::sync::OnceCell<dynamite::CHostFunctionPointers>
                = #macros_private::once_cell::sync::OnceCell::new();

            // Create cell for the adapter
            static ADAPTER: #macros_private::once_cell::sync::OnceCell<super::#adapter_ty>
                = #macros_private::once_cell::sync::OnceCell::new();

            #[safer_ffi::ffi_export]
            fn init_adapter(c_host_functions: dynamite::CHostFunctionPointers) {
                // Initialize host functions cell
                HOST_FUNCTION_POINTERS.set(c_host_functions).map_err(|_| "Cannot initialize cell twice!").unwrap();

                // Initialize adapter
                ADAPTER.set(super::#adapter_ty::init_adapter())
                    .map_err(|_| "Cannot initialize cell twice!").unwrap();
            }

            #[safer_ffi::ffi_export]
            fn get_api(dynamite: *const dynamite::Void) -> safer_ffi::prelude::repr_c::Vec<u8> {
                let e = "Adapter not initialized";
                // Get the adapter
                let adapter = ADAPTER.get().expect(e);

                // Get host functions
                let pointers = HOST_FUNCTION_POINTERS.get().expect(e);
                let host_funcs = dynamite::RemoteHostFunctions {
                    dynamite,
                    pointers: pointers.clone(),
                };

                // get the api from the adapter
                let api = adapter.get_api(&host_funcs);

                // Serialize the API and return the bytes
                #macros_private::serde_cbor::to_vec(&api)
                    .expect("Could not serialize language adapter API").into()
            }

            #[safer_ffi::ffi_export]
            fn call_function(
                dynamite: *const dynamite::Void,
                path: safer_ffi::prelude::str::Ref,
                args: safer_ffi::prelude::c_slice::Ref<*const dynamite::Void>
            ) -> *const dynamite::Void {
                let e = "Adapter not initialized";
                // Get the adapter
                let adapter = ADAPTER.get().expect(e);

                // Get host functions
                let pointers = HOST_FUNCTION_POINTERS.get().expect(e);
                let host_funcs = dynamite::RemoteHostFunctions {
                    dynamite,
                    pointers: pointers.clone()
                };

                // Forward the call to the adapter
                adapter.call_function(
                    &host_funcs,
                    path.as_str(),
                    args.as_slice(),
                )
            }
        }
    };

    out
}
