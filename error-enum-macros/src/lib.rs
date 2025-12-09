//! # `error-enum-macros`
//!
//! A procedural macro crate for [`error-enum`](https://crates.io/crates/error-enum) to define error enums
//! with rich diagnostics support.
//!
//! Please refer to [`error-enum`](https://crates.io/crates/error-enum) and
//! [`its documentation`](https://docs.rs/error-enum/) for more details.

use lazy_regex::{lazy_regex, Lazy, Regex};
use proc_macro::TokenStream;
use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::{format_ident, quote, ToTokens};
use syn::{
    braced,
    parse::{self, Parse},
    parse_macro_input,
    punctuated::{self, Punctuated},
    token::{self, Brace},
    Attribute, DeriveInput, Error, Fields, Generics, Ident, LitStr, Result, Token, Variant,
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
    label: Option<LitStr>,
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
            number: String::new(),
            msg: None,
            attrs: Vec::new(),
            ident: None,
            fields: None,
            span_field: None,
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
        let mut msg = None;
        let mut label = self.label.clone();
        let depth = self.depth + 1;
        let mut nested = false;
        let mut unused_attrs = Vec::new();

        for attr in attrs {
            if attr.path().is_ident("diag") {
                attr.parse_nested_meta(|meta| {
                    if meta.path.is_ident("kind") {
                        let value: LitStr = meta.value()?.parse()?;
                        kind = Some(value.try_into()?);
                    } else if meta.path.is_ident("number") {
                        let value: LitStr = meta.value()?.parse()?;
                        number.push_str(value.value().as_str());
                    } else if meta.path.is_ident("msg") {
                        let value: LitStr = meta.value()?.parse()?;
                        msg = Some(value);
                    } else if meta.path.is_ident("label") {
                        let value: LitStr = meta.value()?.parse()?;
                        label = Some(value);
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
            number,
            msg,
            attrs: unused_attrs,
            ident,
            fields,
            span_field,
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
    brace: Brace,
    roots: Punctuated<ErrorTree, Token![,]>,
    config: Config,
}

impl ErrorEnum {
    fn iter(&self) -> Result<ErrorTreeIter<'_>> {
        ErrorTreeIter::new(self.roots.iter(), self.config.clone())
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
        Ok(Self {
            attrs,
            vis,
            generics,
            name,
            brace,
            roots,
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
                Ok(Self {
                    attrs,
                    vis,
                    name: ident,
                    generics,
                    brace: data_enum.brace_token,
                    roots,
                    config,
                })
            }
            _ => Err(Error::new_spanned(
                ident,
                "ErrorEnum can only be derived for enums.",
            )),
        }
    }
}

impl ErrorEnum {
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
    fn display_branch(ident: &Ident, fields: &Fields, msg: &LitStr) -> Result<TokenStream2> {
        match fields {
            Fields::Named(named) => {
                let members = named.named.iter().map(|f| f.ident.as_ref());
                Ok(quote! {
                    #[allow(unused_variables)]
                    Self::#ident { #(#members),* } => ::core::write!(f, #msg),
                })
            }
            Fields::Unnamed(unnamed) => {
                let params = (0..unnamed.unnamed.len()).map(|i| format_ident!("_{}", i));
                let args = Self::used_unnamed_fields(msg)?;
                Ok(quote! {
                    Self::#ident ( #(#params),* ) => ::core::write!(f, #msg #(, #args)* ),
                })
            }
            Fields::Unit => Ok(quote! {
                Self::#ident => ::core::write!(f, #msg),
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
                Self::display_branch(&ident, &fields, &msg)
            })
            .collect()
    }
    fn primary_label_branch(
        ident: &Ident,
        fields: &Fields,
        label: &LitStr,
    ) -> Result<TokenStream2> {
        match fields {
            Fields::Named(named) => {
                let members = named.named.iter().map(|f| f.ident.as_ref());
                Ok(quote! {
                    #[allow(unused_variables)]
                    Self::#ident { #(#members),* } => ::std::format!(#label),
                })
            }
            Fields::Unnamed(unnamed) => {
                let params = (0..unnamed.unnamed.len()).map(|i| format_ident!("_{}", i));
                let args = Self::used_unnamed_fields(label)?;
                Ok(quote! {
                    Self::#ident ( #(#params),* ) => ::std::format!(#label #(, #args)* ),
                })
            }
            Fields::Unit => Ok(quote! {
                Self::#ident => ::std::format!(#label),
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
                Self::primary_label_branch(&ident, &fields, &label)
            })
            .collect()
    }
    fn impl_error_enum_branch(
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

        let kind = quote! {
            Self::#ident #branch_ignored => #kind,
        };
        let number = quote! {
            Self::#ident #branch_ignored => #number,
        };
        let code = quote! {
            Self::#ident #branch_ignored => #code,
        };
        let span = if let Some(span_field) = span_field {
            quote! {<::error_enum::SimpleSpan as ::core::convert::From<_>>::from(#span_field)}
        } else {
            quote! {<::error_enum::SimpleSpan as ::core::default::Default>::default()}
        };
        let primary_span = match fields {
            Fields::Named(named) => {
                let members = named.named.iter().map(|f| f.ident.as_ref());
                quote! {
                    #[allow(unused_variables)]
                    Self::#ident { #(#members),* } => #span,
                }
            }
            Fields::Unnamed(unnamed) => {
                let params = (0..unnamed.unnamed.len()).map(|i| format_ident!("_{}", i));
                quote! {
                    #[allow(unused_variables)]
                    Self::#ident ( #(#params),* ) => #span,
                }
            }
            Fields::Unit => quote! {
                Self::#ident => #span,
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
                            Some((ident?, fields?, span_field, kind?, number))
                        },
                    )
                    .transpose()
            })
            .map(|config| {
                let (ident, fields, span_field, kind, number) = config?;
                Self::impl_error_enum_branch(&ident, &fields, span_field, &kind, &number)
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
            #(#attrs)*
            #[doc = "List of error variants:"]
            #(
                #[doc = #doc]
            )*
            #vis enum #name #generics
        });
        self.brace.surround(tokens, |tokens| {
            tokens.extend(quote! { #(#variants, )* });
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

        let (kind, number, code, primary_span) = self.impl_error_enum()?;
        let primary_label = self.primary_label()?;
        tokens.extend(quote! {
            impl #impl_generics ::error_enum::ErrorEnum for #name #ty_generics #where_clause {
                type Span = ::error_enum::SimpleSpan;
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
                fn primary_span(&self) -> ::error_enum::SimpleSpan {
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
