#![doc = include_str!("../README.md")]
#![warn(missing_docs)]
#![warn(
    clippy::unwrap_in_result,
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic
)]
#![warn(
    rustdoc::invalid_codeblock_attributes,
    rustdoc::bare_urls,
    rustdoc::broken_intra_doc_links,
    rustdoc::invalid_html_tags,
    rustdoc::invalid_rust_codeblocks,
    rustdoc::unescaped_backticks
)]

use lazy_regex::{lazy_regex, Lazy, Regex};
use proc_macro::TokenStream;
use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::{format_ident, quote, ToTokens};
use syn::{
    braced,
    parse::{self, Parse},
    parse_macro_input, token, Attribute, DeriveInput, Error, Fields, Generics, Ident, LitInt,
    LitStr, Result, Token, Variant, Visibility,
};

#[cfg(test)]
mod tests;

/// Tree node of error definitions.
enum ErrorTree {
    /// Prefix node.
    Prefix(Span, Vec<Attribute>, Vec<ErrorTree>, Token![,]),
    /// Leaf node.
    ///
    /// See [`syn::Variant`] and [`syn::DataStruct`].
    Variant(Span, Vec<Attribute>, Ident, Fields, Token![,]),
}

impl ErrorTree {
    fn attrs(&self) -> &[Attribute] {
        match self {
            ErrorTree::Prefix(_, attrs, _, _) => attrs,
            ErrorTree::Variant(_, attrs, _, _, _) => attrs,
        }
    }
    fn ident(&self) -> Option<&Ident> {
        match self {
            ErrorTree::Prefix(_, _, _, _) => None,
            ErrorTree::Variant(_, _, ident, _, _) => Some(ident),
        }
    }
    fn fields(&self) -> Option<&Fields> {
        match self {
            ErrorTree::Prefix(_, _, _, _) => None,
            ErrorTree::Variant(_, _, _, fields, _) => Some(fields),
        }
    }
    fn span(&self) -> Span {
        match self {
            ErrorTree::Prefix(span, _, _, _) => *span,
            ErrorTree::Variant(span, _, _, _, _) => *span,
        }
    }
    #[expect(unused)]
    fn comma(&self) -> &Token![,] {
        match self {
            ErrorTree::Prefix(_, _, _, comma) => comma,
            ErrorTree::Variant(_, _, _, _, comma) => comma,
        }
    }
}

impl Parse for ErrorTree {
    /// See [`Variant::parse`].
    fn parse(input: parse::ParseStream) -> syn::Result<Self> {
        let attrs = input.call(Attribute::parse_outer)?;
        if input.peek(Ident) {
            let ident: Ident = input.parse()?;
            let fields = if input.peek(token::Brace) {
                Fields::Named(input.parse()?)
            } else if input.peek(token::Paren) {
                Fields::Unnamed(input.parse()?)
            } else {
                Fields::Unit
            };
            Ok(ErrorTree::Variant(
                ident.span(),
                attrs,
                ident,
                fields,
                input.parse()?,
            ))
        } else {
            let span = input.span();
            let children;
            braced!(children in input);
            let mut nodes = Vec::new();
            while !children.is_empty() {
                let node = children.parse()?;
                nodes.push(node);
            }
            Ok(ErrorTree::Prefix(span, attrs, nodes, input.parse()?))
        }
    }
}

#[derive(Clone, Copy)]
enum Kind {
    Error,
    Warn,
}

impl TryFrom<LitStr> for Kind {
    type Error = Error;

    fn try_from(value: LitStr) -> Result<Self> {
        match value.value().as_str() {
            "error" | "Error" => Ok(Kind::Error),
            "warn" | "Warn" => Ok(Kind::Warn),
            _ => Err(Error::new_spanned(
                value,
                "Kind must be either `Error` or `Warn`.",
            )),
        }
    }
}

/// Configuration for each variant.
#[derive(Clone)]
struct Config {
    #[expect(unused)]
    kind: Option<Kind>,
    code: String,
    msg: Option<LitStr>,
    attrs: Vec<Attribute>,
    ident: Option<Ident>,
    fields: Option<Fields>,
    depth: usize,
    #[expect(unused)]
    nested: bool,
    #[expect(unused)]
    span: Span,
}

impl Config {
    const fn new(span: Span) -> Self {
        Self {
            kind: None,
            code: String::new(),
            msg: None,
            attrs: Vec::new(),
            ident: None,
            fields: None,
            depth: 0,
            nested: false,
            span,
        }
    }
    fn process(
        &self,
        attrs: &[Attribute],
        ident: Option<&Ident>,
        fields: Option<&Fields>,
        span: Span,
    ) -> Result<Self> {
        let mut kind = None;
        let mut code = self.code.clone();
        let mut msg = None;
        let depth = self.depth + 1;
        let mut nested = false;
        let mut unused_attrs = Vec::new();

        for attr in attrs {
            if attr.path().is_ident("diag") {
                attr.parse_nested_meta(|meta| {
                    if meta.path.is_ident("kind") {
                        let value: LitStr = meta.value()?.parse()?;
                        kind = Some(value.try_into()?);
                    } else if meta.path.is_ident("code") {
                        let value: LitInt = meta.value()?.parse()?;
                        code.push_str(value.to_string().as_str());
                    } else if meta.path.is_ident("msg") {
                        let value: LitStr = meta.value()?.parse()?;
                        msg = Some(value);
                    } else if meta.path.is_ident("nested") {
                        nested = true;
                    } else {
                        return Err(Error::new_spanned(meta.path, "Unknown attribute key."));
                    }
                    Ok(())
                })?
            } else {
                unused_attrs.push(attr.clone());
            }
        }

        let ident = ident.cloned();
        let fields = fields.cloned();
        Ok(Self {
            kind,
            code,
            msg,
            attrs: unused_attrs,
            ident,
            fields,
            depth,
            nested,
            span,
        })
    }
}

struct ErrorTreeIter<'i> {
    stack: Vec<(&'i [ErrorTree], Config)>,
}

impl<'i> ErrorTreeIter<'i> {
    fn new(tree: &'i [ErrorTree], attrs: &[Attribute], span: Span) -> Result<Self> {
        Ok(Self {
            stack: vec![(tree, Config::new(span).process(attrs, None, None, span)?)],
        })
    }
    fn process_next(node: &'i ErrorTree, config: &Config, span: Span) -> Result<Config> {
        let new_config = config.process(node.attrs(), node.ident(), node.fields(), span)?;
        Ok(new_config)
    }
}

impl<'i> Iterator for ErrorTreeIter<'i> {
    type Item = Result<Config>;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some((slice, config)) = self.stack.last_mut() {
            if let Some((node, rest)) = slice.split_first() {
                *slice = rest;
                let config = Self::process_next(node, config, node.span())
                    .map(Some)
                    .transpose()?;
                if let Ok(config) = &config {
                    if let ErrorTree::Prefix(_, _, children, _) = node {
                        self.stack.push((children.as_slice(), config.clone()));
                    }
                }
                return Some(config);
            } else {
                self.stack.pop();
            }
        }
        None
    }
}

/// The entire error enum.
///
/// ```ignore
/// pub ErrorName {
///     // Variants...
/// }
/// ```
struct ErrorEnum {
    attrs: Vec<Attribute>,
    vis: Visibility,
    name: Ident,
    generics: Generics,
    roots: Vec<ErrorTree>,
}

impl ErrorEnum {
    fn iter(&self) -> Result<ErrorTreeIter<'_>> {
        ErrorTreeIter::new(self.roots.as_slice(), &self.attrs, self.name.span())
    }
}

impl Parse for ErrorEnum {
    fn parse(input: parse::ParseStream) -> syn::Result<Self> {
        let attrs = input.call(Attribute::parse_outer)?;
        let vis = input.parse()?;
        let name = input.parse()?;
        let generics = input.parse()?;
        let mut roots = Vec::new();
        while !input.is_empty() {
            roots.push(ErrorTree::parse(input)?);
        }
        Ok(Self {
            attrs,
            vis,
            generics,
            name,
            roots,
        })
    }
}

impl TryFrom<DeriveInput> for ErrorEnum {
    type Error = Error;

    fn try_from(value: DeriveInput) -> Result<Self> {
        let DeriveInput {
            attrs,
            vis,
            ident,
            generics,
            data,
        } = value;
        let mut roots = Vec::new();
        match data {
            syn::Data::Enum(data_enum) => {
                for variant in data_enum.variants {
                    let span = variant.ident.span();
                    let node = ErrorTree::Variant(
                        span,
                        variant.attrs,
                        variant.ident,
                        variant.fields,
                        Token![,](span),
                    );
                    roots.push(node);
                }
            }
            _ => {
                return Err(Error::new_spanned(
                    ident,
                    "ErrorEnum can only be derived for enums.",
                ))
            }
        }
        Ok(Self {
            attrs,
            vis,
            name: ident,
            generics,
            roots,
        })
    }
}

impl ErrorEnum {
    fn doc(&self) -> Result<Vec<String>> {
        self.iter()?
            .map(|config| {
                let Config {
                    code,
                    depth,
                    ident,
                    msg,
                    ..
                } = config?;
                let indent = "  ".repeat(depth - 2);
                let msg = msg.as_ref().map(|s| s.value());
                Ok(match (ident, msg) {
                    (Some(ident), Some(msg)) => format!("{indent}- `{code}`(**{ident}**): {msg}"),
                    (None, Some(msg)) => format!("{indent}- `{code}`: {msg}"),
                    (Some(ident), None) => format!("{indent}- `{code}`(**{ident}**)"),
                    (None, None) => format!("{indent}- `{code}`"),
                })
            })
            .collect()
    }
    fn variants(&self) -> Result<Vec<Variant>> {
        self.iter()?
            .filter_map(|config| {
                config
                    .map(
                        |Config {
                             attrs,
                             ident,
                             fields,
                             ..
                         }| { Some((attrs, ident?, fields?)) },
                    )
                    .transpose()
            })
            .map(|config| {
                let (attrs, ident, fields) = config?;
                Ok(Variant {
                    attrs,
                    ident,
                    fields,
                    discriminant: None,
                })
            })
            .collect()
    }
    fn display(&self) -> Result<Vec<TokenStream2>> {
        self.iter()?
            .filter_map(|config| {
                config
                    .map(
                        |Config {
                             msg, ident, fields, ..
                         }| { Some((msg, ident?, fields?)) },
                    )
                    .transpose()
            })
            .map(|config| {
                let (msg, ident, fields) = config?;
                match fields {
                    Fields::Named(named) => {
                        let members = named.named.iter().map(|f| f.ident.as_ref());
                        Ok(quote! {
                            #[allow(unused_variables)]
                            Self::#ident { #(#members,)* } => {
                                write!(f, #msg)
                            },
                        })
                    }
                    Fields::Unnamed(unnamed) => {
                        static ARG: Lazy<Regex> = lazy_regex!(r#"\{(\d+)(:[^\{\}]*)?\}"#);
                        let params = (0..unnamed.unnamed.len()).map(|i| format_ident!("_{}", i));
                        let args = msg
                            .as_ref()
                            .map(|msg| -> Result<Vec<Ident>> {
                                ARG.captures_iter(msg.value().as_str())
                                    .map(|cap| {
                                        let index = cap
                                            .get(1)
                                            .ok_or_else(|| {
                                                Error::new_spanned(msg, "Invalid argument index.")
                                            })?
                                            .as_str()
                                            .parse::<usize>()
                                            .map_err(|err| {
                                                Error::new_spanned(
                                                    msg,
                                                    format!("Invalid argument index: {err}"),
                                                )
                                            })?;
                                        Ok(format_ident!("_{}", index))
                                    })
                                    .collect()
                            })
                            .transpose()?
                            .unwrap_or_default();
                        Ok(quote! {
                            Self::#ident ( #(#params, )* ) => {
                                write!(f, #msg #(, #args)* )
                            },
                        })
                    }
                    Fields::Unit => Ok(quote! {
                        Self::#ident => {
                            write!(f, #msg)
                        },
                    }),
                }
            })
            .collect()
    }
    fn try_to_tokens(&self, tokens: &mut TokenStream2) -> Result<()> {
        let attrs = &self.attrs;
        let vis = &self.vis;
        let name = &self.name;
        let generics = &self.generics;

        let doc = self.doc()?;

        let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

        let variants = self.variants()?;

        tokens.extend(quote! {
            #[doc = "List of error variants:"]
            #(
                #[doc = #doc]
            )*
            #(#attrs)*
            #vis enum #name #generics {
                #(#variants, )*
            }
        });

        let display = self.display()?;
        tokens.extend(quote! {
            impl #impl_generics ::core::fmt::Display for #name #ty_generics #where_clause {
                fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                    match self {
                        #(#display)*
                    }
                }
            }
            impl #impl_generics ::core::error::Error for #name #ty_generics #where_clause {}
        });

        // let get_category = self
        //     .get_variants()
        //     .map(|(cfg, _number, variant)| variant.get_category(cfg));
        // let get_number = self
        //     .get_variants()
        //     .map(|(cfg, number, variant)| variant.get_number(cfg, number));
        // let get_code = self
        //     .get_variants()
        //     .map(|(cfg, number, variant)| variant.get_code(cfg, number));
        // let get_prefix = self
        //     .get_variants()
        //     .map(|(cfg, number, variant)| variant.get_prefix(cfg, number));
        // let fmt_desc = self
        //     .get_variants()
        //     .map(|(_cfg, _number, variant)| variant.fmt_desc());
        // let get_desc = self
        //     .get_variants()
        //     .map(|(_cfg, _number, variant)| variant.get_desc());
        // tokens.extend(quote! {
        //     impl #impl_generics #name #ty_generics #where_clause {
        //         /// Write error category like `E`.
        //         pub fn get_category(&self) -> &'static ::core::primitive::str {
        //             match self {
        //                 #(#get_category)*
        //             }
        //         }
        //         /// Write error code number like `0000`.
        //         pub fn get_number(&self) -> ::std::borrow::Cow<'static, str> {
        //             match self {
        //                 #(#get_number)*
        //             }
        //         }
        //         /// Write error code like `E0000`.
        //         pub fn get_code(&self) -> ::std::borrow::Cow<'static, str> {
        //             match self {
        //                 #(#get_code)*
        //             }
        //         }
        //         /// Write error message prefix like `error[E0000]: `.
        //         pub fn get_prefix(&self) -> ::std::borrow::Cow<'static, str> {
        //             match self {
        //                 #(#get_prefix)*
        //             }
        //         }
        //         fn fmt_desc(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
        //             match self {
        //                 #(#fmt_desc)*
        //             }
        //         }
        //         /// Get error description.
        //         pub fn get_desc(&self) -> String {
        //             match self {
        //                 #(#get_desc)*
        //             }
        //         }
        //     }
        // });

        Ok(())
    }
}

impl ToTokens for ErrorEnum {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        self.try_to_tokens(tokens).unwrap_or_else(|err| {
            let diag = err.to_compile_error();
            tokens.extend(diag);
        });
    }
}

/// Define a new layered error type.
///
/// Syntax:
///
/// ```ignore
/// $error_type =
///     $vis:vis $name:ident
///         $($variant:variant, )*
///
/// $variant =
///   // Prefix node.
///     #[diag(kind = $kind:lit_str)]
///     #[diag(code = $code:lit_int)]
///     #[diag(msg = $msg:lit_str)]
///     {
///         $($child_variant:variant, )*
///     }
///   // Leaf node (three forms).
///   | #[diag(kind = $kind:lit_str)]
///     #[diag(code = $code:lit_int)]
///     #[diag(msg = $msg:lit_str)]
///     $ident:ident ($($field_ty:ty),*)
///   | #[diag(kind = $kind:lit_str)]
///     #[diag(code = $code:lit_int)]
///     #[diag(msg = $msg:lit_str)]
///     $ident:ident { $($field_name:ident : $field_ty:ty),* }
///   | #[diag(kind = $kind:lit_str)]
///     #[diag(code = $code:lit_int)]
///     #[diag(msg = $msg:lit_str)]
///     $ident:ident
/// ```
#[proc_macro]
pub fn error_type(token: TokenStream) -> TokenStream {
    let error = parse_macro_input!(token as ErrorEnum);
    error.to_token_stream().into()
}

/// Implement error capabilities for an existing enum.
///
/// See [`error_type!`] for syntax details.
#[proc_macro_derive(ErrorType, attributes(diag))]
pub fn error_enum(token: TokenStream) -> TokenStream {
    let input: DeriveInput = parse_macro_input!(token as DeriveInput);
    let error = ErrorEnum::try_from(input)
        .map_or_else(|err| err.to_compile_error(), |e| e.to_token_stream());
    error.into()
}
