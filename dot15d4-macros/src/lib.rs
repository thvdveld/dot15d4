use proc_macro::TokenStream;
use quote::{quote, ToTokens};
use syn::{parse_macro_input, ItemStruct};

#[proc_macro_attribute]
pub fn frame(_attr: TokenStream, item: TokenStream) -> TokenStream {
    // Get the name of the frame element.
    let input = parse_macro_input!(item as ItemStruct);

    let item_attr = input.attrs;
    let name = input.ident;

    let mut f = quote! {
        #(#item_attr)*
        pub struct #name<T: AsRef<[u8]>> {
            buffer: T,
        }
    };

    let mut impls = vec![];

    impls.push(quote! {
            pub fn new(buffer: T) -> Self {
                Self { buffer }
            }
    });

    let mut offset = 0;
    let mut bits_offset = 0;

    for field in input.fields {
        let fnname = field.ident.unwrap();
        let ty = field.ty;

        let doc = field.attrs.iter().find(|attr| attr.path().is_ident("doc"));

        let condition = field
            .attrs
            .iter()
            .find(|attr| attr.path().is_ident("condition"))
            .map(|attr| attr.parse_args::<syn::Expr>().unwrap());

        let bytes = field
            .attrs
            .iter()
            .find(|attr| attr.path().is_ident("bytes"))
            .map(|attr| {
                attr.parse_args::<syn::LitInt>()
                    .unwrap()
                    .base10_parse::<usize>()
                    .unwrap()
            });

        let bytes = if bytes.is_none() {
            match ty.to_token_stream().to_string().as_str() {
                "bool" => Some(1),
                "u8" => Some(1),
                "u16" => Some(2),
                "i16" => Some(2),
                "u32" => Some(4),
                "i32" => Some(4),
                "u64" => Some(8),
                _ => None,
            }
        } else {
            bytes
        };

        let bits = field
            .attrs
            .iter()
            .find(|attr| attr.path().is_ident("bits"))
            .map(|attr| {
                attr.parse_args::<syn::LitInt>()
                    .unwrap()
                    .base10_parse::<usize>()
                    .unwrap()
            });

        if !fnname.to_string().contains("reserved") {
            let getter = match ty.to_token_stream().to_string().as_str() {
                "bool" => quote! {
                    let buffer = &self.buffer.as_ref()[#offset..];
                    ((buffer[0] >> #bits_offset) & 0b1) != 0
                },
                "u8" => {
                    if let Some(bits) = bits {
                        quote! {
                            let buffer = &self.buffer.as_ref()[#offset..];
                            (buffer[0] >> #bits_offset) & ((1 << #bits) - 1)
                        }
                    } else {
                        quote! {
                            self.buffer.as_ref()[#offset..][0]
                        }
                    }
                }
                "u16" => {
                    quote! {
                        let buffer = &self.buffer.as_ref()[#offset..];
                        u16::from_le_bytes([buffer[0], buffer[1]])
                    }
                }
                "i16" => {
                    quote! {
                        let buffer = &self.buffer.as_ref()[#offset..];
                        i16::from_le_bytes([buffer[0], buffer[1]])
                    }
                }
                "u32" => {
                    if bytes == Some(3) {
                        quote! {
                            let buffer = &self.buffer.as_ref()[#offset..];
                            u32::from_le_bytes([0, buffer[0], buffer[1], buffer[2]])
                        }
                    } else {
                        quote! {
                            let buffer = &self.buffer.as_ref()[#offset..];
                            u32::from_le_bytes([buffer[0], buffer[1], buffer[2], buffer[3]])
                        }
                    }
                }
                "i32" => {
                    quote! {
                        let buffer = &self.buffer.as_ref()[#offset..];
                        i32::from_le_bytes([buffer[0], buffer[1], buffer[2], buffer[3]])
                    }
                }
                "u64" => {
                    quote! {
                        let buffer = &self.buffer.as_ref()[#offset..];
                        u64::from_le_bytes([
                           buffer[0],
                           buffer[1],
                           buffer[2],
                           buffer[3],
                           buffer[4],
                           buffer[5],
                           buffer[6],
                           buffer[7],
                        ])
                    }
                }
                "& [u8]" => {
                    if bytes == Some(0) {
                        quote! {
                            &self.buffer.as_ref()[#offset..]
                        }
                    } else {
                        quote! {
                            &self.buffer.as_ref()[#offset..][..#bytes]
                        }
                    }
                }
                _ => {
                    quote! {
                        #ty::new(&self.buffer.as_ref()[#offset..][..#ty::<&[u8]>::size()])
                    }
                }
            };

            let getter = if let Some(ref condition) = condition {
                quote! {
                    if #condition {
                        Some({
                            #getter
                        })
                    } else {
                        None
                    }
                }
            } else {
                getter
            };

            let return_type = match ty.to_token_stream().to_string().as_str() {
                "bool" | "u8" | "u16" | "u32" | "u64" | "& [u8]" => quote! { #ty },
                _ => quote! { #ty<&[u8]> },
            };

            if condition.is_some() {
                impls.push(quote! {
                    #doc
                    pub fn #fnname(&self) -> Option<#return_type> {
                        #getter
                    }
                });
            } else {
                impls.push(quote! {
                    #doc
                    pub fn #fnname(&self) -> #return_type {
                        #getter
                    }
                });
            }
        }

        for attr in field.attrs {
            if attr.path().is_ident("bytes") {
                offset += attr
                    .parse_args::<syn::LitInt>()
                    .unwrap()
                    .base10_parse::<usize>()
                    .unwrap();
            } else if attr.path().is_ident("bits") {
                bits_offset += attr
                    .parse_args::<syn::LitInt>()
                    .unwrap()
                    .base10_parse::<usize>()
                    .unwrap();

                if bits_offset % 8 == 0 && bits_offset != 0 {
                    offset += 1;
                    bits_offset = 0;
                }
            }
        }
    }

    f.extend(quote! {
        impl<T: AsRef<[u8]>> #name<T> {
            #(#impls)*

            /// Returns the size of this structure in bytes.
            pub const fn size() -> usize {
                #offset
            }
        }
    });

    f.into()
}
