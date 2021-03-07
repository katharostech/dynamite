use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::{format_ident, quote, quote_spanned};
use syn::spanned::Spanned;
use syn::{parse_macro_input, DeriveInput, ItemFn};

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
            unsafe fn call_function(
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

/// Attribute macro that can be used to automatically create bindings to
#[proc_macro_attribute]
pub fn stockpile_function(_args: TokenStream, input: TokenStream) -> TokenStream {
    let function = parse_macro_input!(input as ItemFn);
    impl_stockpile_function(function).into()
}

fn impl_stockpile_function(function: ItemFn) -> TokenStream2 {
    // Output the function unchanged
    let mut out = quote! {
        #function
    };

    // Output a proxy function with the signature required for FFI
    let function_name = function.sig.ident.clone();
    let proxy_function_name = format_ident!("{}_dynamite_ptr", function.sig.ident);

    struct ArgInfo {
        index: usize,
        ident: syn::Ident,
        argtype: syn::Type,
    }

    // Validate and collect argument_info
    let mut arg_infos = Vec::new();
    for (i, arg) in function.sig.inputs.iter().enumerate() {
        // Make sure arg is not `self`
        if let syn::FnArg::Typed(arg) = arg {
            // Make sure arg is a referenece
            if let syn::Type::Reference(argtype) = &*arg.ty {
                let argtype = &argtype.elem;

                // Make sure argument name is a simple identifier
                if let syn::Pat::Ident(ident) = &*arg.pat {
                    arg_infos.push(ArgInfo {
                        index: i,
                        ident: ident.ident.clone(),
                        argtype: (**argtype).clone(),
                    });
                } else {
                    let e = quote_spanned! { arg.pat.span() =>
                        compile_error!{"Arg name must be an identifier,"}
                    };
                    out = quote! { #out #e };
                }
            } else {
                let e = quote_spanned! { arg.ty.span() =>
                    compile_error!{"All args must be a reference"}
                };
                out = quote! { #out #e };
            }
        } else {
            let e = quote_spanned! { arg.span() =>
                compile_error!{"Methods with `self` args are no supported"}
            };
            out = quote! { #out #e };
        }
    }

    // Get the function return type
    let return_type = match function.sig.output {
        syn::ReturnType::Default => None,
        syn::ReturnType::Type(_, t) => {
            if let syn::Type::Reference(t) = &*t {
                let t = &*t.elem;
                Some(t.clone())
            } else {
                return quote_spanned! {t.span() =>
                    compile_error!{"Return type must be a reference"}
                };
            }
        }
    };

    // Create our FFI compatible proxy function
    let return_tokens = if let Some(return_type) = &return_type {
        quote! {
            as *const #return_type as *const ::dynamite::Void
        }
    } else {
        quote! {
            ; std::ptr::null()
        }
    };
    let cast_function_args = arg_infos
        .iter()
        .map(|x| {
            let argtype = x.argtype.clone();
            let index = x.index;
            quote! {
                &*(args[#index] as *const #argtype)
            }
        })
        .collect::<Vec<_>>();
    out = quote! {
        #out

        unsafe fn #proxy_function_name (args: &[*const ::dynamite::Void]) -> *const dynamite::Void {
           #function_name(#( #cast_function_args ),*) #return_tokens
        }
    };

    // Create the stockpile entry for the method
    let api_return_tokens = if let Some(return_type) = &return_type {
        quote! {
            Some(<#return_type as ::dynamite::HasScriptType>::script_path())
        }
    } else {
        quote! { None }
    };
    let function_arg_script_paths = arg_infos
        .iter()
        .map(|x| {
            let argtype = x.argtype.clone();
            let ident = x.ident.clone();
            quote_spanned! {argtype.span() =>
                h.insert(stringify!(#ident).into(), <#argtype as ::dynamite::HasScriptType>::script_path());
            }
        })
        .collect::<Vec<_>>();
    out = quote! {
        #out
        ::dynamite::_macros_private::inventory::submit!(
            ::dynamite::StockpileItem {
                path: ::dynamite::TypePath::from(
                    concat!(module_path!(), "::", stringify!(#function_name))
                ),
                script_type: ::dynamite::ScriptType::Function(::dynamite::FunctionDefinition {
                    arguments: {
                        let mut h = std::collections::HashMap::new();

                        #(#function_arg_script_paths)*

                        h
                    },
                    return_type: #api_return_tokens,
                }),
                function_pointer: Some(#proxy_function_name)
            }
        );
    };

    out
}
