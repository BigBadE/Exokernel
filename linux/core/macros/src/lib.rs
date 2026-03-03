//! Linux kernel export macros
//!
//! This crate provides `#[linux_export]` for generating C-compatible FFI functions
//! from safe Rust code.
//!
//! ## Usage
//!
//! ```ignore
//! #[linux_export]
//! fn atomic_read(v: &atomic_t) -> c_int {
//!     v.read()
//! }
//! ```
//!
//! Generates:
//!
//! ```ignore
//! #[unsafe(no_mangle)]
//! pub unsafe extern "C" fn atomic_read(v: *const atomic_t) -> c_int {
//!     if v.is_null() { return Default::default(); }
//!     let v: &atomic_t = &*v;
//!     v.read()
//! }
//! ```
//!
//! ## Supported patterns
//!
//! ### Arguments
//! - `&T` → `*const T` with null check
//! - `&mut T` → `*mut T` with null check
//! - Primitive types and raw pointers pass through unchanged
//!
//! ### Return types
//! - `&T` → `*const T`
//! - `&mut T` → `*mut T`
//! - `Option<&T>` → `*const T` (None = null)
//! - `Option<&mut T>` → `*mut T` (None = null)
//! - `KernelResult<()>` → `c_int` (Ok = 0, Err = -errno)
//! - `KernelResult<&T>` → `*const T` (Err = null)
//! - `KernelResult<&mut T>` → `*mut T` (Err = null)
//! - Primitive types pass through unchanged
//!
//! ## Unsafe functions
//!
//! For functions that need to perform unsafe operations (like pointer dereferences),
//! mark the function as `unsafe`:
//!
//! ```ignore
//! #[linux_export]
//! unsafe fn memcpy(dest: *mut c_void, src: *const c_void, n: usize) -> *mut c_void {
//!     ptr::copy_nonoverlapping(src as *const u8, dest as *mut u8, n);
//!     dest
//! }
//! ```

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::{quote, format_ident};
use syn::{
    parse_macro_input, FnArg, ItemFn, Pat, ReturnType, Type, PathArguments,
    GenericArgument,
};

/// Marks a function for export to C code.
///
/// The function should be written with safe Rust types (references instead of pointers),
/// and this macro will generate the unsafe FFI wrapper.
///
/// If the function needs to perform unsafe operations internally, mark it as `unsafe`.
#[proc_macro_attribute]
pub fn linux_export(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemFn);

    match generate_export(input) {
        Ok(tokens) => tokens.into(),
        Err(e) => e.to_compile_error().into(),
    }
}

fn generate_export(input: ItemFn) -> syn::Result<TokenStream2> {
    let vis = &input.vis;
    let fn_name = &input.sig.ident;
    let inner_name = format_ident!("__inner_{}", fn_name);
    let body = &input.block;
    let generics = &input.sig.generics;

    // Check if the original function is marked unsafe
    let is_unsafe = input.sig.unsafety.is_some();

    // Parse arguments and build conversions
    let mut extern_args = Vec::new();
    let mut null_checks = Vec::new();
    let mut arg_conversions = Vec::new();
    let mut inner_call_args = Vec::new();

    for arg in &input.sig.inputs {
        match arg {
            FnArg::Receiver(_) => {
                return Err(syn::Error::new_spanned(arg, "self not supported in linux_export"));
            }
            FnArg::Typed(pat_type) => {
                let pat = &pat_type.pat;
                let ty = &pat_type.ty;

                let arg_name = match pat.as_ref() {
                    Pat::Ident(ident) => &ident.ident,
                    _ => return Err(syn::Error::new_spanned(pat, "expected identifier")),
                };

                match ty.as_ref() {
                    // &mut T -> *mut T
                    Type::Reference(ref_ty) if ref_ty.mutability.is_some() => {
                        let inner = &ref_ty.elem;
                        extern_args.push(quote! { #arg_name: *mut #inner });
                        null_checks.push(quote! {
                            if #arg_name.is_null() { return Default::default(); }
                        });
                        arg_conversions.push(quote! {
                            let #arg_name: &mut #inner = &mut *#arg_name;
                        });
                        inner_call_args.push(quote! { #arg_name });
                    }
                    // &T -> *const T
                    Type::Reference(ref_ty) => {
                        let inner = &ref_ty.elem;
                        extern_args.push(quote! { #arg_name: *const #inner });
                        null_checks.push(quote! {
                            if #arg_name.is_null() { return Default::default(); }
                        });
                        arg_conversions.push(quote! {
                            let #arg_name: &#inner = &*#arg_name;
                        });
                        inner_call_args.push(quote! { #arg_name });
                    }
                    // Primitives, raw pointers, and other types pass through
                    _ => {
                        extern_args.push(quote! { #arg_name: #ty });
                        inner_call_args.push(quote! { #arg_name });
                    }
                }
            }
        }
    }

    // Parse return type and build conversion
    let (extern_ret, ret_conversion) = convert_return_type(&input.sig.output)?;

    // Build the inner function (original code)
    let inner_args = &input.sig.inputs;
    let inner_ret = &input.sig.output;

    // Generate different output based on whether the function is unsafe
    let output = if is_unsafe {
        // For unsafe functions, the inner function is also unsafe
        // and we call it with unsafe block but don't need extra conversions
        // since they operate on raw pointers directly
        quote! {
            #[unsafe(no_mangle)]
            #vis unsafe extern "C" fn #fn_name #generics (#(#extern_args),*) #extern_ret {
                #[inline(always)]
                unsafe fn #inner_name #generics (#inner_args) #inner_ret #body

                let __result = #inner_name(#(#inner_call_args),*);
                #ret_conversion
            }
        }
    } else {
        // For safe functions, generate null checks and reference conversions
        quote! {
            #[unsafe(no_mangle)]
            #vis unsafe extern "C" fn #fn_name #generics (#(#extern_args),*) #extern_ret {
                #[inline(always)]
                fn #inner_name #generics (#inner_args) #inner_ret #body

                #(#null_checks)*
                #(#arg_conversions)*

                let __result = #inner_name(#(#inner_call_args),*);
                #ret_conversion
            }
        }
    };

    Ok(output)
}

/// Convert return type from safe Rust to FFI
fn convert_return_type(ret: &ReturnType) -> syn::Result<(TokenStream2, TokenStream2)> {
    match ret {
        ReturnType::Default => {
            // () -> ()
            Ok((quote! {}, quote! {}))
        }
        ReturnType::Type(_, ty) => convert_return_type_inner(ty),
    }
}

fn convert_return_type_inner(ty: &Type) -> syn::Result<(TokenStream2, TokenStream2)> {
    match ty {
        // &mut T -> *mut T
        Type::Reference(ref_ty) if ref_ty.mutability.is_some() => {
            let inner = &ref_ty.elem;
            Ok((
                quote! { -> *mut #inner },
                quote! { __result as *mut #inner },
            ))
        }
        // &T -> *const T
        Type::Reference(ref_ty) => {
            let inner = &ref_ty.elem;
            Ok((
                quote! { -> *const #inner },
                quote! { __result as *const #inner },
            ))
        }
        // Check for Result<T, Errno>, KernelResult<T>, or Option<T>
        Type::Path(type_path) => {
            let segment = type_path.path.segments.last()
                .ok_or_else(|| syn::Error::new_spanned(ty, "expected type path"))?;

            let type_name = segment.ident.to_string();

            match type_name.as_str() {
                "KernelResult" | "Result" => {
                    if let PathArguments::AngleBracketed(args) = &segment.arguments {
                        if let Some(GenericArgument::Type(inner_ty)) = args.args.first() {
                            return convert_result_return(inner_ty);
                        }
                    }
                    Err(syn::Error::new_spanned(ty, "expected KernelResult<T>"))
                }
                "Option" => {
                    if let PathArguments::AngleBracketed(args) = &segment.arguments {
                        if let Some(GenericArgument::Type(inner_ty)) = args.args.first() {
                            return convert_option_return(inner_ty);
                        }
                    }
                    Err(syn::Error::new_spanned(ty, "expected Option<T>"))
                }
                _ => {
                    // Other types pass through
                    Ok((quote! { -> #ty }, quote! { __result }))
                }
            }
        }
        // Other types pass through
        _ => Ok((quote! { -> #ty }, quote! { __result })),
    }
}

fn convert_result_return(inner_ty: &Type) -> syn::Result<(TokenStream2, TokenStream2)> {
    match inner_ty {
        // KernelResult<()> -> c_int
        Type::Tuple(tuple) if tuple.elems.is_empty() => {
            Ok((
                quote! { -> core::ffi::c_int },
                quote! { __result.to_int() },
            ))
        }
        // KernelResult<&mut T> -> *mut T
        Type::Reference(ref_ty) if ref_ty.mutability.is_some() => {
            let inner = &ref_ty.elem;
            Ok((
                quote! { -> *mut #inner },
                quote! { __result.to_ptr_mut() },
            ))
        }
        // KernelResult<&T> -> *const T
        Type::Reference(ref_ty) => {
            let inner = &ref_ty.elem;
            Ok((
                quote! { -> *const #inner },
                quote! { __result.to_ptr() },
            ))
        }
        // KernelResult<primitive> -> c_int
        _ => {
            Ok((
                quote! { -> core::ffi::c_int },
                quote! { __result.to_int() },
            ))
        }
    }
}

fn convert_option_return(inner_ty: &Type) -> syn::Result<(TokenStream2, TokenStream2)> {
    match inner_ty {
        // Option<&mut T> -> *mut T (None = null)
        Type::Reference(ref_ty) if ref_ty.mutability.is_some() => {
            let inner = &ref_ty.elem;
            Ok((
                quote! { -> *mut #inner },
                quote! {
                    match __result {
                        Some(r) => r as *mut #inner,
                        None => core::ptr::null_mut(),
                    }
                },
            ))
        }
        // Option<&T> -> *const T (None = null)
        Type::Reference(ref_ty) => {
            let inner = &ref_ty.elem;
            Ok((
                quote! { -> *const #inner },
                quote! {
                    match __result {
                        Some(r) => r as *const #inner,
                        None => core::ptr::null(),
                    }
                },
            ))
        }
        // Option<T> for other types - pass through
        _ => {
            Ok((
                quote! { -> #inner_ty },
                quote! { __result.unwrap_or_default() },
            ))
        }
    }
}
