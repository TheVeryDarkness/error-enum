//! To generate [core::fmt::Display] implementation and
//! documentation comments for error types.

#![warn(rust_2021_compatibility, rustdoc::all, missing_docs)]

use ansi_term::Color::Red;
use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::{quote, ToTokens};
use syn::{
    parse, parse::Parse, parse_macro_input, Attribute, FieldsNamed, Generics, Ident, LitStr, Token,
    Visibility,
};

struct ErrorVariant {
    code: Ident,
    name: Ident,
    fields: FieldsNamed,
    msg: LitStr,
}

impl Parse for ErrorVariant {
    fn parse(input: parse::ParseStream) -> syn::Result<Self> {
        let code = input.parse()?;
        let name = input.parse()?;
        let fields = input.parse()?;
        let msg = input.parse()?;
        let _comma: Token![,] = input.parse()?;
        Ok(Self {
            code,
            name,
            fields,
            msg,
        })
    }
}

impl ToTokens for ErrorVariant {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        let name = &self.name;
        let fields = &self.fields;
        let msg = &self.msg;
        let code = format!("{}: ", self.code);
        tokens.extend(quote! {
            #[doc = #code]
            #[doc = #msg]
            #name #fields,
        })
    }
}

impl ErrorVariant {
    fn doc(&self) -> String {
        format!("- `{}` ({}): {}", self.name, self.code, self.msg.value())
    }
    fn format(&self) -> TokenStream2 {
        let name = &self.name;
        let fields = self
            .fields
            .named
            .iter()
            .map(|field| field.ident.as_ref().unwrap());
        let code = format!("error[{}]", &self.code);
        #[cfg(feature = "colored")]
        let code = Red.paint(code).to_string();
        let msg = &self.msg;
        quote! {
            Self::#name { #(#fields, )* } => {
                ::core::write!{f, "{}: ", #code}?;
                ::core::write!{f, #msg}?;
                ::core::result::Result::Ok(())
            }
        }
    }
}

struct ErrorEnum {
    attrs: Vec<Attribute>,
    vis: Visibility,
    name: Ident,
    generics: Generics,
    variants: Vec<ErrorVariant>,
}

impl Parse for ErrorEnum {
    fn parse(input: parse::ParseStream) -> syn::Result<Self> {
        let attrs = input.call(Attribute::parse_outer)?;
        let vis = input.parse()?;
        let name = input.parse()?;
        let generics = input.parse()?;
        let mut variants = Vec::new();
        while !input.is_empty() {
            let kind = input.parse()?;
            variants.push(kind);
        }
        Ok(Self {
            attrs,
            vis,
            generics,
            name,
            variants,
        })
    }
}

impl ToTokens for ErrorEnum {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        let attrs = &self.attrs;
        let vis = &self.vis;
        let name = &self.name;
        let variants = &self.variants;
        let generics = &self.generics;
        let doc = self.variants.iter().map(|v| v.doc());
        tokens.extend(quote! {
            #[doc = "List of error variants:"]
            #(
                #[doc = #doc]
            )*
            #(#attrs)*
            #vis enum #name #generics {
                #(
                    #variants
                )*
            }
        });
        let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
        let branches = self.variants.iter().map(ErrorVariant::format);
        tokens.extend(quote! {
            impl #impl_generics core::fmt::Display for #name #ty_generics #where_clause {
                fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                    match self {
                        #(#branches)*
                    }
                }
            }
        });
        /*
        let mut vars = self.variants.iter();
        if let Some(var) = vars.next() {
            fn get_fields(var: &ErrorVariant) -> BTreeSet<&Ident> {
                var.fields
                    .named
                    .iter()
                    .flat_map(|name| name.ident.iter())
                    .collect()
            }
            let mut fields = get_fields(var);
            for var in vars {
                fields = fields.intersection(&get_fields(var)).cloned().collect();
            }
            let accesser = fields.into_iter().map(|field| {
                let branches = self
                    .variants
                    .iter()
                    .map(|var| {
                        let var = &var.name;
                        quote! {
                            #var { #field, .. } => {
                                #field
                            }
                        }
                    })
                    .collect::<Vec<_>>();
                quote! {
                    fn field(&self) ->
                }
            });
            tokens.extend(quote! {
                impl #impl_generics core::fmt::Display for #name #ty_generics #where_clause {
                    #(#accesser)*
                }
            })
        }
        */
    }
}

/// Define a new error type.
///
/// First, provide the name of the type.
/// Second, provide each variant of the error by following order:
/// 1. Code.
/// 2. Name.
/// 3. Fields.
/// 4. Message.
/// 5. A trailing comma.
#[proc_macro]
pub fn error_type(token: TokenStream) -> TokenStream {
    let error = parse_macro_input!(token as ErrorEnum);
    error.to_token_stream().into()
}

#[cfg(test)]
mod tests {
    use crate::ErrorEnum;
    use quote::{quote, ToTokens};

    #[test]
    fn test() {
        let output: ErrorEnum = syn::parse2(quote! {
            FileSystemError
                E01 FileNotFound {path: std::path::Path}
                    "{path} not found.",
        })
        .unwrap();
        let output = output.into_token_stream();
        eprintln!("{:#}", output);
        // panic!();
    }
}
