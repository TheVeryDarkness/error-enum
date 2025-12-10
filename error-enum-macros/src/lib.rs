//! # `error-enum-macros`
//!
//! A procedural macro crate for [`error-enum`](https://crates.io/crates/error-enum) to define error enums
//! with rich diagnostics support.
//!
//! Please refer to [`error-enum`](https://crates.io/crates/error-enum) and
//! [`its documentation`](https://docs.rs/error-enum/) for more details.
#![warn(unused_crate_dependencies)]

use std::borrow::Cow;

use either::Either;
use lazy_regex::{lazy_regex, Lazy, Regex};
use proc_macro::TokenStream;
use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::{format_ident, quote, ToTokens};
use syn::{
    braced,
    parse::{self, Parse},
    parse_macro_input, parse_quote,
    punctuated::{self, Punctuated},
    token::{self, Brace},
    Attribute, DeriveInput, Error, Fields, Generics, Ident, LitStr, Result, Token, Type, Variant,
    Visibility,
};

#[cfg(test)]
mod tests;

/// A tuple type with 4 identical types.
///
/// For `impl_error_enum_branch` and `impl_error_enum`,
/// it means `(kind, number, code, primary_span)`.
type Tuple4<T> = (T, T, T, T);

fn split_fields_attrs(fields: &mut Fields) -> Result<Option<Ident>> {
    let mut span_ident = None;
    for (idx, field) in fields.iter_mut().enumerate() {
        for attr in &field.attrs {
            if attr.meta.path().is_ident("diag") {
                attr.parse_nested_meta(|meta| {
                    if meta.path.is_ident("span") {
                        span_ident = Some(field.ident.clone().unwrap_or(format_ident!("_{idx}")))
                    }
                    Ok(())
                })?
            }
        }
        field.attrs.retain(|attr| !attr.path().is_ident("diag"));
    }
    Ok(span_ident)
}

/// Tree node of error definitions.
enum ErrorTree {
    /// Prefix node.
    Prefix {
        span: Span,
        attrs: Vec<Attribute>,
        nodes: Punctuated<ErrorTree, Token![,]>,
    },
    /// Leaf node.
    ///
    /// See [`syn::Variant`] and [`syn::DataStruct`].
    Variant {
        span: Span,
        attrs: Vec<Attribute>,
        span_ident: Option<Ident>,
        ident: Ident,
        fields: Fields,
    },
}

impl ErrorTree {
    fn attrs(&self) -> &[Attribute] {
        match self {
            ErrorTree::Prefix { attrs, .. } => attrs,
            ErrorTree::Variant { attrs, .. } => attrs,
        }
    }
    fn ident(&self) -> Option<&Ident> {
        match self {
            ErrorTree::Prefix { .. } => None,
            ErrorTree::Variant { ident, .. } => Some(ident),
        }
    }
    fn fields(&self) -> Option<&Fields> {
        match self {
            ErrorTree::Prefix { .. } => None,
            ErrorTree::Variant { fields, .. } => Some(fields),
        }
    }
    fn span_ident(&self) -> Option<Ident> {
        match self {
            ErrorTree::Prefix { .. } => None,
            ErrorTree::Variant { span_ident, .. } => span_ident.clone(),
        }
    }
    fn span(&self) -> Span {
        match self {
            ErrorTree::Prefix { span, .. } => *span,
            ErrorTree::Variant { span, .. } => *span,
        }
    }
}

impl Parse for ErrorTree {
    /// See [`Variant::parse`].
    fn parse(input: parse::ParseStream) -> syn::Result<Self> {
        let attrs = input.call(Attribute::parse_outer)?;

        if input.peek(Ident) {
            let ident: Ident = input.parse()?;
            let mut fields = if input.peek(token::Brace) {
                Fields::Named(input.parse()?)
            } else if input.peek(token::Paren) {
                Fields::Unnamed(input.parse()?)
            } else {
                Fields::Unit
            };
            let span_ident = split_fields_attrs(&mut fields)?;
            Ok(ErrorTree::Variant {
                span: ident.span(),
                attrs,
                span_ident,
                ident,
                fields,
            })
        } else {
            let span = input.span();
            let children;
            braced!(children in input);
            let nodes = Punctuated::parse_terminated(&children)?;
            Ok(ErrorTree::Prefix { span, attrs, nodes })
        }
    }
}

#[derive(Clone, Copy, Default)]
enum Kind {
    #[default]
    Error,
    Warn,
}

impl Kind {
    fn short_str(&self) -> &'static str {
        match self {
            Kind::Error => "E",
            Kind::Warn => "W",
        }
    }
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

impl ToTokens for Kind {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        let kind = match self {
            Kind::Error => quote! { ::error_enum::Kind::Error },
            Kind::Warn => quote! { ::error_enum::Kind::Warn },
        };
        tokens.extend(kind);
    }
}

/// Configuration for each variant.
#[derive(Clone)]
struct Config {
    kind: Option<Kind>,
    number: String,
    msg: Option<LitStr>,
    attrs: Vec<Attribute>,
    ident: Option<Ident>,
    fields: Option<Fields>,
    span_field: Option<Ident>,
    // FIXME: move to `ErrorEnum` for better performance?
    span_type: Option<Type>,
    label: Option<LitStr>,
    depth: usize,
    nested: bool,
    #[expect(unused)]
    span: Span,
}

impl Config {
    const fn new(span: Span) -> Self {
        Self {
            kind: None,
            number: String::new(),
            msg: None,
            attrs: Vec::new(),
            ident: None,
            fields: None,
            span_field: None,
            span_type: None,
            label: None,
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
        span_field: Option<Ident>,
        span: Span,
    ) -> Result<Self> {
        let mut kind = self.kind;
        let mut number = self.number.clone();
        let mut msg = self.msg.clone();
        let mut label = self.label.clone();
        let mut span_type = self.span_type.clone();
        let depth = self.depth + 1;
        let mut nested = self.nested;
        let mut unused_attrs = Vec::new();

        for attr in attrs {
            if attr.path().is_ident("diag") {
                attr.parse_nested_meta(|meta| {
                    if meta.path.is_ident("kind") {
                        let value: LitStr = meta.value()?.parse()?;
                        kind = Some(value.try_into()?);
                    } else if meta.path.is_ident("label") {
                        let value: LitStr = meta.value()?.parse()?;
                        label = Some(value);
                    } else if meta.path.is_ident("msg") {
                        let value: LitStr = meta.value()?.parse()?;
                        msg = Some(value);
                    } else if meta.path.is_ident("nested") {
                        nested = true;
                    } else if meta.path.is_ident("number") {
                        let value: LitStr = meta.value()?.parse()?;
                        number.push_str(value.value().as_str());
                    } else if meta.path.is_ident("span_type") {
                        let value: LitStr = meta.value()?.parse()?;
                        span_type = Some(value.parse()?);
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
            number,
            msg,
            attrs: unused_attrs,
            ident,
            fields,
            span_field,
            span_type,
            label,
            depth,
            nested,
            span,
        })
    }
}

struct ErrorTreeIter<'i> {
    stack: Vec<(punctuated::Iter<'i, ErrorTree>, Config)>,
}

impl<'i> ErrorTreeIter<'i> {
    fn new(tree: punctuated::Iter<'i, ErrorTree>, config: Config) -> Result<Self> {
        Ok(Self {
            stack: vec![(tree, config)],
        })
    }
    fn process_next(node: &'i ErrorTree, config: &Config, span: Span) -> Result<Config> {
        let new_config = config.process(
            node.attrs(),
            node.ident(),
            node.fields(),
            node.span_ident(),
            span,
        )?;
        Ok(new_config)
    }
}

impl<'i> Iterator for ErrorTreeIter<'i> {
    type Item = Result<Config>;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some((slice, config)) = self.stack.last_mut() {
            if let Some(node) = slice.next() {
                let config = Self::process_next(node, config, node.span())
                    .map(Some)
                    .transpose()?;
                if let Ok(config) = &config {
                    if let ErrorTree::Prefix { nodes, .. } = node {
                        self.stack.push((nodes.iter(), config.clone()));
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

enum ErrorEnumInner {
    Multiple {
        brace: Brace,
        roots: Punctuated<ErrorTree, Token![,]>,
        body: bool,
    },
    Single {
        node: ErrorTree,
    },
}

impl ErrorEnumInner {
    fn iter(&self, config: Config) -> Result<impl Iterator<Item = Result<Config>> + '_> {
        match self {
            ErrorEnumInner::Multiple { roots, .. } => {
                Ok(Either::Left(ErrorTreeIter::new(roots.iter(), config)?))
            }
            ErrorEnumInner::Single { node } => {
                let iter = Either::Right(std::iter::once(ErrorTreeIter::process_next(
                    node,
                    &config,
                    node.span(),
                )));
                Ok(iter)
            }
        }
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
    inner: ErrorEnumInner,
    config: Config,
}

impl ErrorEnum {
    fn iter(&self) -> Result<impl Iterator<Item = Result<Config>> + '_> {
        self.inner.iter(self.config.clone())
    }
    fn is_enum(&self) -> bool {
        matches!(self.inner, ErrorEnumInner::Multiple { .. })
    }
}

impl Parse for ErrorEnum {
    fn parse(input: parse::ParseStream) -> syn::Result<Self> {
        let mut attrs = input.call(Attribute::parse_outer)?;
        let vis = input.parse()?;
        let name: Ident = input.parse()?;
        let generics = input.parse()?;
        let children;
        let brace = braced!(children in input);
        let config = Config::new(name.span()).process(&attrs, None, None, None, name.span())?;
        attrs.retain(|attr| !attr.path().is_ident("diag"));

        let roots = Punctuated::parse_terminated(&children)?;
        let inner = ErrorEnumInner::Multiple {
            body: true,
            brace,
            roots,
        };
        Ok(Self {
            attrs,
            vis,
            generics,
            name,
            inner,
            config,
        })
    }
}

impl TryFrom<DeriveInput> for ErrorEnum {
    type Error = Error;

    fn try_from(value: DeriveInput) -> Result<Self> {
        let DeriveInput {
            mut attrs,
            vis,
            ident,
            generics,
            data,
        } = value;
        match data {
            syn::Data::Enum(data_enum) => {
                let mut roots = Punctuated::new();
                for pair in data_enum.variants.into_pairs() {
                    let (mut variant, comma) = pair.into_tuple();
                    let span = variant.ident.span();
                    let span_ident = split_fields_attrs(&mut variant.fields)?;
                    let node = ErrorTree::Variant {
                        span,
                        attrs: variant.attrs,
                        ident: variant.ident,
                        fields: variant.fields,
                        span_ident,
                    };
                    roots.push_value(node);
                    if let Some(comma) = comma {
                        roots.push_punct(comma);
                    }
                }
                let config =
                    Config::new(ident.span()).process(&attrs, None, None, None, ident.span())?;
                attrs.retain(|attr| !attr.path().is_ident("diag"));

                let inner = ErrorEnumInner::Multiple {
                    body: false,
                    brace: data_enum.brace_token,
                    roots,
                };
                Ok(Self {
                    attrs,
                    vis,
                    name: ident,
                    generics,
                    inner,
                    config,
                })
            }
            syn::Data::Struct(mut data_struct) => {
                let span = ident.span();
                let span_ident = split_fields_attrs(&mut data_struct.fields)?;
                let config =
                    Config::new(ident.span()).process(&attrs, None, None, None, ident.span())?;
                attrs.retain(|attr| !attr.path().is_ident("diag"));

                let node = ErrorTree::Variant {
                    span,
                    attrs,
                    ident: ident.clone(),
                    fields: data_struct.fields,
                    span_ident,
                };

                let inner = ErrorEnumInner::Single { node };

                Ok(Self {
                    attrs: Vec::new(),
                    vis,
                    name: ident,
                    generics,
                    inner,
                    config,
                })
            }
            _ => Err(Error::new_spanned(
                ident,
                "ErrorEnum can only be derived for enums or structs.",
            )),
        }
    }
}

impl ErrorEnum {
    fn variant<'a>(&'a self, ident: &'a Ident) -> impl ToTokens + 'a {
        struct VariantPrefix<'a> {
            ident: &'a Ident,
            is_enum: bool,
        }
        impl<'a> ToTokens for VariantPrefix<'a> {
            fn to_tokens(&self, tokens: &mut TokenStream2) {
                let ident = self.ident;
                if self.is_enum {
                    tokens.extend(quote! { Self::#ident });
                } else {
                    tokens.extend(quote! { Self });
                }
            }
        }

        VariantPrefix {
            ident,
            is_enum: self.is_enum(),
        }
    }
    fn doc(&self) -> Result<Vec<String>> {
        self.iter()?
            .map(|config| {
                let Config {
                    number,
                    depth,
                    ident,
                    msg,
                    kind,
                    ..
                } = config?;
                let indent = "  ".repeat(depth - 2);
                let msg = msg.as_ref().map(|s| s.value());
                let kind = kind.unwrap_or_default().short_str();
                Ok(match (ident, msg) {
                    (Some(ident), Some(msg)) => {
                        format!("{indent}- `{kind}{number}`(**{ident}**): {msg}")
                    }
                    (None, Some(msg)) => format!("{indent}- `{kind}{number}`: {msg}"),
                    (Some(ident), None) => format!("{indent}- `{kind}{number}`(**{ident}**)"),
                    (None, None) => format!("{indent}- `{kind}{number}`"),
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
                             kind,
                             msg,
                             number,
                             attrs,
                             ident,
                             fields,
                             ..
                         }| {
                            Some((kind, msg, number, attrs, ident?, fields?))
                        },
                    )
                    .transpose()
            })
            .map(|config| {
                let (kind, msg, number, mut attrs, ident, fields) = config?;

                let kind = kind.unwrap_or_default();
                let code = format!("{}{}", kind.short_str(), number);

                let doc = match msg {
                    Some(msg) => {
                        format!("`{code}`: {msg}", msg = msg.value())
                    }
                    None => format!("`{code}`"),
                };

                attrs.push(syn::parse_quote! {
                    #[doc = #doc]
                });
                attrs.push(syn::parse_quote! {
                    #[doc(alias = #code)]
                });

                Ok(Variant {
                    attrs,
                    ident,
                    fields,
                    discriminant: None,
                })
            })
            .collect()
    }
    fn used_unnamed_fields(msg: &LitStr) -> Result<Vec<Ident>> {
        static ARG: Lazy<Regex> = lazy_regex!(r#"(^|[^\{])(\{\{)*\{(?<index>\d+)(:[^\{\}]*)?\}"#);
        ARG.captures_iter(msg.value().as_str())
            .map(|cap| {
                let index = cap
                    .name("index")
                    .ok_or_else(|| Error::new_spanned(msg, "Invalid argument index."))?
                    .as_str()
                    .parse::<usize>()
                    .map_err(|err| {
                        Error::new_spanned(msg, format!("Invalid argument index: {err}"))
                    })?;
                Ok(format_ident!("_{}", index))
            })
            .collect()
    }
    fn display_branch(&self, ident: &Ident, fields: &Fields, msg: &LitStr) -> Result<TokenStream2> {
        let prefix = self.variant(ident);
        match fields {
            Fields::Named(named) => {
                let members = named.named.iter().map(|f| f.ident.as_ref());
                Ok(quote! {
                    #[allow(unused_variables)]
                    #prefix { #(#members),* } => ::core::write!(f, #msg),
                })
            }
            Fields::Unnamed(unnamed) => {
                let params = (0..unnamed.unnamed.len()).map(|i| format_ident!("_{}", i));
                let args = Self::used_unnamed_fields(msg)?;
                Ok(quote! {
                    #prefix ( #(#params),* ) => ::core::write!(f, #msg #(, #args)* ),
                })
            }
            Fields::Unit => Ok(quote! {
                #prefix => ::core::write!(f, #msg),
            }),
        }
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
                let msg = msg.ok_or_else(|| {
                    Error::new_spanned(
                        &ident,
                        "Missing message. Consider using `#[diag(msg = \"...\")]`",
                    )
                })?;
                self.display_branch(&ident, &fields, &msg)
            })
            .collect()
    }
    fn primary_label_branch(
        &self,
        ident: &Ident,
        fields: &Fields,
        label: &LitStr,
    ) -> Result<TokenStream2> {
        let prefix = self.variant(ident);
        match fields {
            Fields::Named(named) => {
                let members = named.named.iter().map(|f| f.ident.as_ref());
                Ok(quote! {
                    #[allow(unused_variables)]
                    #prefix { #(#members),* } => ::std::format!(#label),
                })
            }
            Fields::Unnamed(unnamed) => {
                let params = (0..unnamed.unnamed.len()).map(|i| format_ident!("_{}", i));
                let args = Self::used_unnamed_fields(label)?;
                Ok(quote! {
                    #prefix ( #(#params),* ) => ::std::format!(#label #(, #args)* ),
                })
            }
            Fields::Unit => Ok(quote! {
                #prefix => ::std::format!(#label),
            }),
        }
    }
    fn primary_label(&self) -> Result<Vec<TokenStream2>> {
        self.iter()?
            .filter_map(|config| {
                config
                    .map(
                        |Config {
                             msg,
                             ident,
                             fields,
                             label,
                             ..
                         }| { Some((msg, ident?, fields?, label)) },
                    )
                    .transpose()
            })
            .map(|config| {
                let (msg, ident, fields, label) = config?;
                let label = label.or(msg).ok_or_else(|| {
                    Error::new_spanned(
                        &ident,
                        "Missing label or message. Consider using `#[diag(label = \"...\")]`",
                    )
                })?;
                self.primary_label_branch(&ident, &fields, &label)
            })
            .collect()
    }
    fn impl_error_enum_branch(
        &self,
        ident: &Ident,
        fields: &Fields,
        span_field: Option<Ident>,
        kind: &Kind,
        number: &str,
    ) -> Result<Tuple4<TokenStream2>> {
        let branch_ignored = match fields {
            Fields::Named(_) => quote! { { .. } },
            Fields::Unnamed(_) => quote! { (..) },
            Fields::Unit => quote! {},
        };
        let code = format!("{}{}", kind.short_str(), number);

        let prefix = self.variant(ident);

        let kind = quote! {
            #prefix #branch_ignored => #kind,
        };
        let number = quote! {
            #prefix #branch_ignored => #number,
        };
        let code = quote! {
            #prefix #branch_ignored => #code,
        };
        let span_type = self.span_type();
        let span = if let Some(span_field) = span_field {
            quote! {<#span_type as ::core::convert::From<_>>::from(#span_field)}
        } else {
            quote! {<#span_type as ::core::default::Default>::default()}
        };
        let primary_span = match fields {
            Fields::Named(named) => {
                let members = named.named.iter().map(|f| f.ident.as_ref());
                quote! {
                    #[allow(unused_variables)]
                    #prefix { #(#members),* } => #span,
                }
            }
            Fields::Unnamed(unnamed) => {
                let params = (0..unnamed.unnamed.len()).map(|i| format_ident!("_{}", i));
                quote! {
                    #[allow(unused_variables)]
                    #prefix ( #(#params),* ) => #span,
                }
            }
            Fields::Unit => quote! {
                #prefix => #span,
            },
        };
        Ok((kind, number, code, primary_span))
    }
    fn impl_error_enum(&self) -> Result<Tuple4<Vec<TokenStream2>>> {
        self.iter()?
            .filter_map(|config| {
                config
                    .map(
                        |Config {
                             ident,
                             fields,
                             span_field,
                             kind,
                             number,
                             ..
                         }| {
                            Some((ident?, fields?, span_field, kind, number))
                        },
                    )
                    .transpose()
            })
            .map(|config| {
                let (ident, fields, span_field, kind, number) = config?;
                let kind = kind.unwrap_or_default();
                self.impl_error_enum_branch(&ident, &fields, span_field, &kind, &number)
            })
            .collect()
    }
    fn span_type(&self) -> Cow<'_, Type> {
        self.config.span_type.as_ref().map_or_else(
            || {
                Cow::Owned(parse_quote! {
                    ::error_enum::SimpleSpan
                })
            },
            Cow::Borrowed,
        )
    }
    fn try_to_tokens(&self, tokens: &mut TokenStream2) -> Result<()> {
        let attrs = &self.attrs;
        let vis = &self.vis;
        let name = &self.name;
        let generics = &self.generics;

        let doc = self.doc()?;

        let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

        let variants = self.variants()?;

        if let ErrorEnumInner::Multiple {
            body: true, brace, ..
        } = self.inner
        {
            tokens.extend(quote! {
                #(#attrs)*
                #[doc = "List of error variants:"]
                #(
                    #[doc = #doc]
                )*
                #vis enum #name #generics
            });
            brace.surround(tokens, |tokens| {
                tokens.extend(quote! { #(#variants, )* });
            });
        }

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

        let (kind, number, code, primary_span) = self.impl_error_enum()?;
        let primary_label = self.primary_label()?;
        let span_type = self.span_type();
        tokens.extend(quote! {
            impl #impl_generics ::error_enum::ErrorType for #name #ty_generics #where_clause {
                type Span = #span_type;
                type Message = ::std::string::String;

                fn kind(&self) -> ::error_enum::Kind {
                    match self {
                        #(#kind)*
                    }
                }
                fn number(&self) -> &::core::primitive::str {
                    match self {
                        #(#number)*
                    }
                }
                fn code(&self) -> &::core::primitive::str {
                    match self {
                        #(#code)*
                    }
                }
                fn primary_span(&self) -> #span_type {
                    match self {
                        #(#primary_span)*
                    }
                }
                fn primary_message(&self) -> ::std::string::String {
                    ::std::format!("{self}")
                }
                fn primary_label(&self) -> ::std::string::String {
                    match self {
                        #(#primary_label)*
                    }
                }
            }
        });

        Ok(())
    }
}

impl ToTokens for ErrorEnum {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        let mut buffer = TokenStream2::new();
        self.try_to_tokens(&mut buffer)
            .inspect(|()| tokens.extend(buffer))
            .unwrap_or_else(|err| {
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
///     $vis:vis $name:ident {
///         $($variant:variant, )*
///     }
///
/// $variant =
///   // Prefix node.
///     #[diag(kind   = $kind:lit_str)]
///     #[diag(number = $number:lit_int)]
///     #[diag(msg    = $msg:lit_str)]
///     {
///         $($child_variant:variant, )*
///     }
///   // Leaf node (three forms).
///   | #[diag(kind   = $kind:lit_str)]
///     #[diag(number = $number:lit_int)]
///     #[diag(msg    = $msg:lit_str)]
///     $ident:ident (
///         $(
///             $(#[diag(span)])?
///             $field_ty:ty
///         ),*
///     )
///   | #[diag(kind   = $kind:lit_str)]
///     #[diag(number = $number:lit_int)]
///     #[diag(msg    = $msg:lit_str)]
///     $ident:ident {
///         $(
///             $(#[diag(span)])?
///             $field_name:ident: $field_ty:ty
///         ),*
///     }
///   | #[diag(kind   = $kind:lit_str)]
///     #[diag(number = $number:lit_int)]
///     #[diag(msg    = $msg:lit_str)]
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
