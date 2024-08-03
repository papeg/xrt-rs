extern crate proc_macro;
use proc_macro::TokenStream;
use quote::quote;
use syn::{self, parse_macro_input, NestedMeta, Lit, LitStr, spanned::Spanned, token::Token, AttributeArgs, DataStruct, ItemStruct};

mod xclbin_reader;

#[proc_macro_attribute]
pub fn kernel(attrs: TokenStream, items: TokenStream) -> TokenStream {
    let attributes = parse_macro_input!(attrs as AttributeArgs);    
    if attributes.len() != 2 {
        return syn::Error::new(attributes[0].span(), "To create a kernel argument handler function provide exactly one argument: The name of the kernel: #[kernel(my_add_kernel)]").to_compile_error().into();
    }

    let mut xclbin_path: Option<String> = None;

    if let NestedMeta::Lit(lit) = &attributes[0] {
        if let Lit::Str(lit_str) = lit {
            println!("{}", lit_str.value());
            xclbin_path = Some(lit_str.value());
        }
    }

    if xclbin_path.is_none() {
        return syn::Error::new(attributes[0].span(), "unable to read the xclbinpath as first attribute argument").to_compile_error().into();
    }

    let mut kernel_name: Option<String> = None;

    if let NestedMeta::Lit(lit) = &attributes[1] {
        if let Lit::Str(lit_str) = lit {
            println!("{}", lit_str.value());
            kernel_name = Some(lit_str.value());
        }
    }
    
    if kernel_name.is_none() {
        return syn::Error::new(attributes[1].span(), "unable to read the kernel name as second attribute argument").to_compile_error().into();
    }

    let args = xclbin_reader::get_arguments(&xclbin_path.unwrap(), &kernel_name.unwrap()).unwrap();

    for arg in args {
        println!("{}  |  {}  |  {}", arg["name"], arg["type"], arg["size"]);
    }

    let parsed_struct = parse_macro_input!(items as ItemStruct);
    let struct_name = &parsed_struct.ident;

    let result = quote! {
        #parsed_struct
        impl #struct_name {
            fn ans(self) -> i32 {
                return 42;
            }
        }
    };
    println!("MACRO CODE: {}", result);
    result.into()
}