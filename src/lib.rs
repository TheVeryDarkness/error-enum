//! To generate [core::fmt::Display] implementation and
//! documentation comments for error types.

#![warn(rust_2021_compatibility, rustdoc::all, missing_docs)]

use ansi_term::Color::Red;
use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::{format_ident, quote, ToTokens};
use std::iter::once;
use syn::{
    braced, parse, parse::Parse, parse_macro_input, Attribute, Fields, Generics, Ident, LitInt,
    LitStr, Token, Variant, Visibility,
};

struct ErrorVariant {
    variant: Variant,
    msg: LitStr,
}

impl Parse for ErrorVariant {
    fn parse(input: parse::ParseStream) -> syn::Result<Self> {
        let variant = input.parse()?;
        let msg = input.parse()?;
        Ok(Self { variant, msg })
    }
}

impl ErrorVariant {
    fn to_tokens(&self, code: &str, tokens: &mut TokenStream2) {
        let variant = &self.variant;
        let msg = &self.msg;
        let code = format!("{}: ", code);
        tokens.extend(quote! {
            #[doc = #code]
            #[doc = #msg]
            #variant,
        })
    }
}

impl ErrorVariant {
    fn fmt_self(&self) -> TokenStream2 {
        let name = &self.variant.ident;
        let msg = &self.msg;
        match &self.variant.fields {
            Fields::Named(fields) => {
                let fields = fields
                    .named
                    .iter()
                    .map(|field| field.ident.as_ref().unwrap());
                quote! {
                    Self::#name { #(#fields, )* } => {
                        ::core::write!{f, #msg}?;
                        ::core::result::Result::Ok(())
                    }
                }
            }
            Fields::Unnamed(unnamed) => {
                let elements = unnamed
                    .unnamed
                    .iter()
                    .enumerate()
                    .map(|(i, _)| format_ident!("_{i}"))
                    .collect::<Vec<_>>();
                quote! {
                    Self::#name ( #(#elements, )* ) => {
                        ::core::write!{f, #msg, #(#elements, )*}?;
                        ::core::result::Result::Ok(())
                    }
                }
            }
            Fields::Unit => {
                quote! {
                    Self::#name => {
                        ::core::write!{f, #msg}?;
                        ::core::result::Result::Ok(())
                    }
                }
            }
        }
    }
}

enum ErrorTree {
    Prefix(LitInt, LitStr, Vec<ErrorTree>),
    Variant(LitInt, ErrorVariant),
}

impl Parse for ErrorTree {
    fn parse(input: parse::ParseStream) -> syn::Result<Self> {
        if input.peek2(LitStr) {
            let code = input.parse()?;
            let desc = input.parse()?;
            let children;
            braced!(children in input);
            let mut nodes = Vec::new();
            while !children.is_empty() {
                let node = children.parse()?;
                nodes.push(node);
            }
            Ok(ErrorTree::Prefix(code, desc, nodes))
        } else {
            let code = input.parse()?;
            let variant = input.parse()?;
            let _comma: Token![,] = input.parse()?;
            Ok(ErrorTree::Variant(code, variant))
        }
    }
}

impl ErrorTree {
    fn get_variants<'s>(
        &'s self,
        prefix: String,
    ) -> impl Iterator<Item = (String, &'s ErrorVariant)> {
        match self {
            Self::Prefix(code, _desc, children) => children
                .iter()
                .flat_map(|node| node.get_variants(format!("{prefix}{}", code.to_string())))
                .collect::<Vec<_>>()
                .into_iter(),
            Self::Variant(code, var) => {
                let prefix = format!("{prefix}{code}");
                vec![(prefix, var)].into_iter()
            }
        }
    }
    fn get_nodes<'s>(
        &'s self,
        prefix: &str,
        depth: usize,
    ) -> impl Iterator<Item = (usize, String, Option<String>, String)> {
        match self {
            Self::Prefix(code, desc, children) => {
                let prefix = format!("{}{}", prefix, code);
                once((depth, prefix.clone(), None, desc.value()))
                    .chain(
                        children
                            .iter()
                            .flat_map(|node| node.get_nodes(&prefix, depth + 1)),
                    )
                    .collect::<Vec<_>>()
                    .into_iter()
            }
            Self::Variant(code, var) => {
                let prefix = format!("{}{}", prefix, code);
                vec![(
                    depth,
                    prefix,
                    Some(var.variant.ident.to_string()),
                    var.msg.value(),
                )]
                .into_iter()
            }
        }
    }
}

struct ErrorEnum {
    attrs: Vec<Attribute>,
    vis: Visibility,
    name: Ident,
    generics: Generics,
    variants: Vec<(Ident, LitStr, Vec<ErrorTree>)>,
}

impl ErrorEnum {
    fn get_variants<'s>(&'s self) -> impl Iterator<Item = (String, &'s ErrorVariant)> {
        self.variants.iter().flat_map(|(ident, _, tree)| {
            tree.iter()
                .flat_map(|node| node.get_variants(ident.to_string()))
        })
    }
    fn get_nodes<'s>(&'s self) -> Vec<(usize, String, Option<String>, String)> {
        self.variants
            .iter()
            .flat_map(|(ident, msg, tree)| {
                let prefix = ident.to_string();
                once((0, prefix.clone(), None, msg.value())).chain(
                    tree.iter()
                        .flat_map(|node| node.get_nodes(&prefix, 1))
                        .collect::<Vec<_>>()
                        .into_iter(),
                )
            })
            .collect()
    }
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
            let msg = input.parse()?;
            let inner;
            braced!(inner in input);
            let mut trees = Vec::new();
            while !inner.is_empty() {
                let tree = inner.parse()?;
                trees.push(tree);
            }
            assert!(inner.is_empty());
            variants.push((kind, msg, trees));
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
        let generics = &self.generics;
        let doc = self
            .get_nodes()
            .into_iter()
            .map(|(depth, code, name, desc)| {
                let indent = "  ".repeat(depth);
                if let Some(name) = name {
                    format!("{indent}- `{code}`(**{name}**): {desc}")
                } else {
                    format!("{indent}- `{code}`: {desc}")
                }
            });
        let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

        let variants = {
            let mut tokens = TokenStream2::new();
            self.get_variants()
                .for_each(|(code, var)| var.to_tokens(&code, &mut tokens));
            tokens
        };
        tokens.extend(quote! {
            #[doc = "List of error variants:"]
            #(
                #[doc = #doc]
            )*
            #(#attrs)*
            #vis enum #name #generics {
                #variants
            }
        });

        tokens.extend(quote! {
            impl #impl_generics ::core::fmt::Display for #name #ty_generics #where_clause {
                fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                    f.write_str(self.get_prefix())?;
                    self.fmt_self(f)?;
                    ::core::result::Result::Ok(())
                }
            }
        });

        let get_code = self.get_variants().map(|(code, variant)| {
            let name = &variant.variant.ident;
            quote! {
                Self::#name { .. } => #code,
            }
        });
        let get_prefix = self.get_variants().map(|(code, variant)| {
            let name = &variant.variant.ident;
            let prefix = format!("error[{}]", &code);
            #[cfg(feature = "colored")]
            let prefix = Red.paint(prefix).to_string();
            let prefix = format!("{prefix}: ");
            quote! {
                Self::#name {..} => #prefix,
            }
        });
        let fmt_self = self.get_variants().map(|(_, variant)| variant.fmt_self());
        tokens.extend(quote! {
            impl #impl_generics #name #ty_generics #where_clause {
                /// Get code.
                pub fn get_code(&self) -> &'static str {
                    match self {
                        #(#get_code)*
                    }
                }
                pub fn get_prefix(&self) -> &'static str {
                    match self {
                        #(#get_prefix)*
                    }
                }
                fn fmt_self(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                    match self {
                        #(#fmt_self)*
                    }
                }
            }
        });
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
                E "错误" {
                    01 FileNotFound {path: std::path::Path}
                    "{path} not found.",
                }
        })
        .unwrap();
        let output = output.into_token_stream();
        eprintln!("{:#}", output);
        // panic!();
    }
}
