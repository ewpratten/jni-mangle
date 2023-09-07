#![doc = include_str!("../README.md")]
#![deny(unsafe_code)]

use args::{parse_macro_args, TOrTokens};
use darling::{FromMeta, ToTokens};
use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use utils::{
    escape::escape_string,
    validators::{is_valid_class, is_valid_method, is_valid_package},
};
mod args;
mod utils;

#[derive(Debug, FromMeta)]
struct MangleArgs {
    package: String,
    class: String,
    method: Option<String>,
    args: Option<String>,
    alias: Option<bool>,
}

#[proc_macro_attribute]
pub fn mangle(args: TokenStream, input: TokenStream) -> TokenStream {
    // Parse the attribute arguments
    let args = match parse_macro_args::<MangleArgs>(args) {
        TOrTokens::T(args) => args,
        TOrTokens::Tokens(error) => return error,
    };

    // Parse the function
    let input = syn::parse_macro_input!(input as syn::ItemFn);
    let input_function_name = input.sig.ident.to_string();
    let output_function_name = args.method.unwrap_or(input_function_name);

    // We need valid inputs
    if !is_valid_package(&args.package) {
        return syn::Error::new_spanned(args.package, "Invalid Java package name")
            .to_compile_error()
            .into();
    }
    if !is_valid_class(&args.class) {
        return syn::Error::new_spanned(args.class, "Invalid Java class name")
            .to_compile_error()
            .into();
    }
    if !is_valid_method(&output_function_name) {
        return syn::Error::new_spanned(output_function_name, "Invalid Java method name")
            .to_compile_error()
            .into();
    }

    // Build the mangled function name
    let mut mangled_fn_name = format!(
        "Java_{}_{}_{}",
        escape_string(&args.package),
        escape_string(&args.class),
        escape_string(&output_function_name)
    );

    // If we have args, append them to the mangled name
    if args.args.is_some() {
        mangled_fn_name.push_str(&format!("__{}", escape_string(&args.args.unwrap())));
    }

    // Hand off to the raw mangle macro for the main processing logic
    let should_alias = args.alias.unwrap_or(true);
    mangle_raw(
        quote! {name=#mangled_fn_name, alias=#should_alias}.into(),
        input.into_token_stream().into(),
    )
}

#[derive(Debug, FromMeta)]
struct MangleRawArgs {
    name: String,
    alias: bool,
}

#[proc_macro_attribute]
pub fn mangle_raw(args: TokenStream, input: TokenStream) -> TokenStream {
    // Parse the attribute arguments
    let args = match parse_macro_args::<MangleRawArgs>(args) {
        TOrTokens::T(args) => args,
        TOrTokens::Tokens(error) => return error,
    };

    // Parse the function
    let mut input = syn::parse_macro_input!(input as syn::ItemFn);

    // Rename the function
    let rust_name_ident = input.sig.ident.clone();
    input.sig.ident = syn::Ident::new(&args.name, input.sig.ident.span());

    // Set the function to be `extern "system"`
    input.sig.abi = Some(syn::parse_quote! { extern "system" });

    // Wrap the function in needed attributes
    let mut output = quote! {
        #[no_mangle]
        #[allow(non_snake_case)]
        #input
    };

    // If aliasing is enabled, add another function with the original name and args
    if args.alias {
        // Create the needed identifiers
        let java_name_ident = input.sig.ident; // This was defined earlier
        let rust_name_ident = rust_name_ident; // This was defined earlier
        let fn_args = input.sig.inputs.clone();
        let fn_args = fn_args.iter();
        let fn_return = input.sig.output;
        let fn_visiblity = input.vis;
        let fn_attrs = input.attrs;

        // Convert fn_args to a string of arg names
        let inner_fn_args_list = input
            .sig
            .inputs
            .iter()
            .map(|arg| match arg {
                syn::FnArg::Receiver(_) => panic!("Cannot alias a method with a receiver"),
                syn::FnArg::Typed(pat_type) => pat_type.pat.clone(),
            })
            .map(|pat| quote! { #pat })
            .collect::<Vec<TokenStream2>>();

        // Build the alias function
        let alias_fn_output = quote! {
            #(#fn_attrs)*
            #fn_visiblity fn #rust_name_ident (#(#fn_args),*) #fn_return {
                #java_name_ident (#(#inner_fn_args_list),*)
            }
        };

        // Extend the output with the alias function
        output.extend(alias_fn_output);
    }

    output.into()
}
