extern crate proc_macro;
use proc_macro::TokenStream;
use quote::quote;
use syn::{self, parse_macro_input, spanned::Spanned, token::Token, AttributeArgs, DataStruct, ItemStruct};

#[proc_macro_attribute]
pub fn kernel(attrs: TokenStream, items: TokenStream) -> TokenStream {
    let attributes = parse_macro_input!(attrs as AttributeArgs);    
    if attributes.len() != 1 {
        return syn::Error::new(attributes[0].span(), "To create a kernel argument handler function provide exactly one argument: The name of the kernel: #[kernel(my_add_kernel)]").to_compile_error().into();
    }

    let parsed_struct = parse_macro_input!(items as ItemStruct);
    let struct_name = parsed_struct.ident;

    let result = quote! {
        impl #struct_name {
            fn ans() -> i32 {
                return 42;
            }
        }
    };
    let mut final_stream = TokenStream::new();
    final_stream.extend(&items.clone().into_iter());
    final_stream.extend::<proc_macro::TokenStream>(result.into());
    final_stream
}