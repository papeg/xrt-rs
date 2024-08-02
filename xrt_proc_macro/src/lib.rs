extern crate proc_macro;
use proc_macro::TokenStream;
use syn::{self, parse_macro_input, AttributeArgs};

#[proc_macro_attribute]
pub fn kernel(attrs: TokenStream, items: TokenStream) -> TokenStream {
    let attributes = parse_macro_input!(attrs as AttributeArgs);    
    if attributes.len() != 1 {
        return syn::Error::new(attrs, "To create a kernel argument handler function provide exactly one argument: The name of the kernel: #[kernel(my_add_kernel)]").to_compile_error().into();
    }
    items
}