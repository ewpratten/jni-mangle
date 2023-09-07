#![doc = include_str!("../README.md")]
#![deny(unsafe_code)]

use args::{parse_macro_args, TOrTokens};
use darling::{FromMeta, ToTokens};
use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::Block;
use utils::{
    escape::escape_string,
    validators::{is_valid_class, is_valid_method, is_valid_package},
};
mod args;
mod utils;

/// Arguments accepted by the `#[mangle]` macro
#[derive(Debug, FromMeta)]
struct MangleArgs {
    /// Java package name
    package: String,
    /// Java class name
    class: String,
    /// Java method name (or just the Rust function name if not specified)
    method: Option<String>,
    /// Optional Java args (used to disambiguate overloaded functions)
    args: Option<String>,
    /// Whether to alias the function with the original name
    alias: Option<bool>,
}

/// Mangle a Rust function to be callable from Java through JNI
///
/// This will generate an `extern "system"` function that is correctly named to be called from Java.
///
/// ## Macro arguments
/// - `package`: The Java package name
/// - `class`: The Java class name this method belongs to
/// - `method` (optional): The Java method name (defaults to the Rust function name)
/// - `args` (optional): The Java method args (used to disambiguate overloaded functions)
/// - `alias` (optional): Whether to alias the function with the original name (defaults to `true`)
///
/// Aliasing allows the function to be called from Rust using its original name as well as from Java using
/// the mangled name. If Aliasing is disabled, the rust function name will not be callable from Rust.
///
/// ## Example
/// ```
/// use jni_mangle::mangle;
///
/// #[mangle(package="com.example", class="Example", method="addTwoNumbers")]
/// pub fn add_two_numbers(a: i32, b: i32) -> i32 {
///    a + b    
/// }
///
/// // This function is callable from rust using both the mangled name and
/// // the original name since `alias` is enabled by default
/// assert_eq!(
///     add_two_numbers(1, 2),
///     Java_com_example_Example_addTwoNumbers(1, 2)
/// );
/// ```
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

/// Arguments accepted by the `#[mangle_raw]` macro
#[derive(Debug, FromMeta)]
struct MangleRawArgs {
    /// The name to mangle the function to
    name: String,
    /// Whether to alias the function with the original name
    alias: bool,
}

/// # Warning: You probably don't want to use this unless you know what you're doing
/// Mangles a Rust function to be callable from Java through JNI with no validation on the name
///
/// ## Macro arguments
/// - `name`: The name to mangle the function to
/// - `alias`: Whether to alias the function with the original name
///
/// ## Example
/// ```
/// use jni_mangle::mangle_raw;
///
/// #[mangle_raw(name="Java_com_example_Example_addTwoNumbers", alias=true)]
/// pub fn add_two_numbers(a: i32, b: i32) -> i32 {
///   a + b
/// }
///
/// // This function is callable from rust using both the mangled name and
/// // the original name since `alias` is enabled by default
/// assert_eq!(
///    add_two_numbers(1, 2),   
///    Java_com_example_Example_addTwoNumbers(1, 2)
/// );
/// ```
#[proc_macro_attribute]
pub fn mangle_raw(args: TokenStream, input: TokenStream) -> TokenStream {
    // Parse the attribute arguments
    let args = match parse_macro_args::<MangleRawArgs>(args) {
        TOrTokens::T(args) => args,
        TOrTokens::Tokens(error) => return error,
    };

    // Parse the function
    let input_fn = syn::parse_macro_input!(input as syn::ItemFn);
    let mut output_fn = input_fn.clone();

    // Rename the function
    let rust_name_ident = output_fn.sig.ident.clone();
    output_fn.sig.ident = syn::Ident::new(&args.name, output_fn.sig.ident.span());

    // Set the function to be `extern "system"`
    output_fn.sig.abi = Some(syn::parse_quote! { extern "system" });

    // Wrap the function in needed attributes
    let mut output = quote! {
        #[no_mangle]
        #[allow(non_snake_case)]
        #output_fn
    };

    // If aliasing is enabled, add another function with the original name and args
    if args.alias {
        // Clone the input function again to modify into the aliased function. 
        // The reason for doing this is to avoid needing to copy over every generic, 
        // docstring, modifier, where clause, etc...
        let mut alias_fn = input_fn.clone();

        // Build a list of tokens to be the arguments for the inner function
        let inner_fn_args_list = alias_fn
            .sig
            .inputs
            .iter()
            .map(|arg| match arg {
                syn::FnArg::Receiver(_) => panic!("Cannot alias a method with a receiver"),
                syn::FnArg::Typed(pat_type) => pat_type.pat.clone(),
            })
            .map(|pat| quote! { #pat })
            .collect::<Vec<TokenStream2>>();

        // Replace the name with the original name again
        alias_fn.sig.ident = rust_name_ident.clone();

        // Replace the body with a function call
        alias_fn.block = Box::new(syn::parse_quote! {
            {
                #rust_name_ident (#(#inner_fn_args_list),*)
            }
        });

        // Extend the output with the alias function
        output.extend(quote! {
            #[no_mangle]
            #[allow(non_snake_case)]
            #alias_fn
        });
    }

    output.into()
}
