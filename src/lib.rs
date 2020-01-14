extern crate proc_macro;

use quote::quote;
use syn::{
    parse::{Parse, ParseStream, Result},
    parse_macro_input,
};

/// Contains string parsed from tokens passed to proc macro
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

enum TransformType {
    CString,
    CWString,
}
impl TransformType {
    /// Returns max character that can fit into this transform
    fn max_char(&self) -> u32 {
        match self {
            TransformType::CString => 0xff,
            TransformType::CWString => 0xffff,
        }
    }
}

use TransformType::{CString, CWString};

/// Transforms passed string to needed form, used by proc macro at bottom
fn transform(
    input: RealCString,
    default_ch: char,
    transform_type: TransformType,
) -> proc_macro::TokenStream {
    let stream = match transform_type {
        CString | CWString => {
            let bytes: Vec<proc_macro2::TokenStream> = input
                .string
                .chars()
                .to_owned()
                .map(|cur_char| {
                    let out: char = if cur_char as u32 <= transform_type.max_char() {
                        cur_char
                    } else {
                        default_ch
                    };
                    match transform_type {
                        CString => {
                            let res: i8 = out as i8;
                            quote! {#res,}
                        }
                        CWString => {
                            let res: i16 = out as i16;
                            quote! {#res,}
                        }
                    }
                })
                .collect();
            match transform_type {
                CString => quote! {
                    &[#(#bytes)* 0i8,] as *const i8
                },
                CWString => quote! {
                    &[#(#bytes)* 0i16,] as *const i16
                },
            }
        }
    };
    proc_macro::TokenStream::from(stream)
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
///
/// let c_string = real_c_string!("Привет world!");
/// // 63i8 == '?', russian characters are not fitting in 1 byte in utf-8 character set
/// let same_as_array_of_bytes: [i8;14] = [63i8, 63i8, 63i8, 63i8, 63i8, 63i8, 32i8, 119i8, 111i8, 114i8, 108i8, 100i8, 33i8, 0i8];
/// for i in 0..13 {
///     assert_eq!(same_as_array_of_bytes[i], unsafe{*c_string.offset(i as isize)})
/// }
/// ```
#[proc_macro]
pub fn real_c_string(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    if input.is_empty() {
        panic!("No passed tokens!");
    }
    transform(
        parse_macro_input!(input as RealCString),
        '?',
        TransformType::CString,
    )
}

/// Same as `real_c_string`, but used for wchar_t* strings
///
/// ```rust
/// #![feature(proc_macro_hygiene)]
///
/// use real_c_string::real_c_wstring;
/// assert_eq!(0i16, unsafe{*real_c_wstring!("")});
///
/// let c_wstring = real_c_wstring!("Hello world!");
/// let same_as_array_of_bytes: [i16;13] = [72i16, 101i16, 108i16, 108i16, 111i16, 32i16, 119i16, 111i16, 114i16, 108i16, 100i16, 33i16, 0i16];
/// for i in 0..13 {
///     assert_eq!(same_as_array_of_bytes[i], unsafe{*c_wstring.offset(i as isize)})
/// }
///
/// let c_wstring = real_c_wstring!("Привет world!");
/// let same_as_array_of_bytes: [i16;14] = [1055i16, 1088i16, 1080i16, 1074i16, 1077i16, 1090i16, 32i16, 119i16, 111i16, 114i16, 108i16, 100i16, 33i16, 0i16];
/// for i in 0..13 {
///     assert_eq!(same_as_array_of_bytes[i], unsafe{*c_wstring.offset(i as isize)})
/// }
/// ```
#[proc_macro]
pub fn real_c_wstring(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    if input.is_empty() {
        panic!("No passed tokens!");
    }
    transform(
        parse_macro_input!(input as RealCString),
        '?',
        TransformType::CWString,
    )
}
