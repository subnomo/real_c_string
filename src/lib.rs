#![feature(proc_macro_hygiene)]

extern crate proc_macro;

use quote::{quote};
use syn::{
    parse::{Parse, ParseStream, Result},
    parse_macro_input,
};

struct RealCString {
    string: String,
}

impl Parse for RealCString {
    fn parse(input: ParseStream) -> Result<Self> {
        if let syn::Lit::Str(str) = input.parse()? {
            Ok(RealCString {
                string: str.value(),
            })
        } else {
            Err(input.error("expected Str instead of ByteStr"))
        }
    }
}

/// Transforms passed string to same look as C strings at asm level
/// Used in vmprotect crate, because vmprotect disassembles code, and finds usages like this
/// 
/// ```rust
/// #![feature(proc_macro_hygiene)]
/// 
/// use real_c_string::real_c_string;
/// assert_eq!(0i8, unsafe{*real_c_string!("")});
/// 
/// let c_string = real_c_string!("Hello world!");
/// let same_as_array_of_bytes: [i8;13] = [72i8, 101i8, 108i8, 108i8, 111i8, 32i8, 119i8, 111i8, 114i8, 108i8, 100i8, 33i8, 0i8];
/// for i in 0..13 {
///     assert_eq!(same_as_array_of_bytes[i], unsafe{*c_string.offset(i as isize)})
/// }
/// ```
#[proc_macro]
pub fn real_c_string(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    if input.is_empty() {
        panic!("No passed tokens!");
    }
    let RealCString { string } = parse_macro_input!(input as RealCString);
    let bytes: Vec<proc_macro2::TokenStream> = string
        .as_bytes()
        .to_owned()
        .iter()
        .map(|e| {
            let f: i8 = *e as i8;
            quote! {#f,}
        })
        .collect();
    let expanded = quote! {
        &[#(#bytes)* 0i8,] as *const i8
    };

    proc_macro::TokenStream::from(expanded)
}
