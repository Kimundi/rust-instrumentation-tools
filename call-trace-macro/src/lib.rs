#![warn(missing_docs)]

//! Provides the `#[trace]` macro. See the `callback-trace` crate for
//! more details.

extern crate proc_macro;
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, ItemFn, parse_quote, Block, Expr, Token, punctuated::Punctuated};
use syn::parse::Parser;

/// The `#[trace]` macro. Needs to be applied to function definitions,
/// and will invoke callbacks managed by the the `callback-trace`
/// before and after the functions body.
///
/// # Example
/// ```notest
/// use call_trace::trace;
///
/// #[trace]
/// fn foo() {
///     // ...
/// }
/// ```
#[proc_macro_attribute]
pub fn trace(attr: TokenStream, input: TokenStream) -> TokenStream {
    assert!(attr.is_empty());
    let input = parse_macro_input!(input as ItemFn);

    // Hand the output tokens back to the compiler
    TokenStream::from(quote! {
        #[::call_trace_macro::trace_with(::call_trace::on_trace)]
        #input
    })
}

/// The `#[trace_with]` macro. Needs to be applied to function definitions,
/// and will call a user-provided expression with the function body wrapped
/// in a closure, and a `CallContext` parameter.
///
/// The attribute accepts additional expression arguments that will
/// be inserted after the context paramter.
///
/// # Example
/// ```notest
/// use call_trace::{trace_with, CallContext};
///
/// impl MyType {
///     #[trace_with(self.trace())]
///     fn foo(&mut self) {
///         // ...
///     }
///
///     fn trace<T, F: FnOnce() -> T>(&mut self) -> impl FnOnce(F, CallContext) -> T {
///         |f, _ctx| {
///             f()
///         }
///     }
/// }
/// ```
#[proc_macro_attribute]
pub fn trace_with(attr: TokenStream, input: TokenStream) -> TokenStream {
    let parser = Punctuated::<Expr, Token![,]>::parse_terminated;
    let mut with_args: Vec<_> = parser.parse(attr).unwrap().into_iter().collect();
    let with = with_args.remove(0);

    let input = parse_macro_input!(input as ItemFn);
    let fn_name = input.ident.clone();

    // Hand the output tokens back to the compiler
    TokenStream::from(quote! {
        #[::call_trace_macro::inject_with(#with, ::call_trace::CallContext {
            file: file!(),
            line: line!(),
            fn_name: stringify!(#fn_name),
        } #(,#with_args)*)]
        #input
    })
}

/// The `#[inject_with]` macro. Needs to be applied to function definitions,
/// and will call a user-provided expression with the function body wrapped
/// in a closure.
///
/// The attribute accepts additional expression arguments that will
/// be passed to the user-provided function as extra arguments.
///
/// # Example
/// ```notest
/// use call_trace::inject_with;
///
/// impl MyType {
///     #[inject_with(self.trace())]
///     fn foo(&mut self) {
///         // ...
///     }
///
///     fn trace<T, F: FnOnce() -> T>(&mut self) -> impl FnOnce(F) -> T {
///         |f| {
///             f()
///         }
///     }
/// }
/// ```
#[proc_macro_attribute]
pub fn inject_with(attr: TokenStream, input: TokenStream) -> TokenStream {
    let parser = Punctuated::<Expr, Token![,]>::parse_terminated;
    let mut with_args: Vec<_> = parser.parse(attr).unwrap().into_iter().collect();
    let with = with_args.remove(0);

    let mut input = parse_macro_input!(input as ItemFn);

    let inner_block = input.block;
    let trace_wrapper: Block = parse_quote! {
        {
            #with(move || #inner_block #(,#with_args)*)
        }
    };
    input.block = Box::new(trace_wrapper);

    // Hand the output tokens back to the compiler
    TokenStream::from(quote! { #input })

}
