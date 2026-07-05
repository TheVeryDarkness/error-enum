//! # `error-enum-macros`
//!
//! A procedural macro crate for [`error-enum`](https://crates.io/crates/error-enum) to define error enums
//! with rich diagnostics support.
//!
//! Please refer to [`error-enum`](https://crates.io/crates/error-enum) and
//! [`its documentation`](https://docs.rs/error-enum/) for more details.
#![warn(unused_crate_dependencies)]

use alloc::borrow::Cow;
use either::Either;
use lazy_regex::{lazy_regex, Captures, Lazy, Regex};
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

extern crate alloc;

#[cfg(test)]
mod tests;

/// A tuple type with 4 identical types.
///
/// For `impl_error_enum_branch` and `impl_error_enum`,
/// it means `(kind, number, code, primary_span)`.
type Tuple4<T> = (T, T, T, T);

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
            let fields = if input.peek(token::Brace) {
                Fields::Named(input.parse()?)
            } else if input.peek(token::Paren) {
                Fields::Unnamed(input.parse()?)
            } else {
                Fields::Unit
            };
            Ok(ErrorTree::Variant {
                span: ident.span(),
                attrs,
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
enum SubDiagKind {
    Note,
    Help,
}

#[derive(Clone)]
struct LabelEntry {
    field: Ident,
    text: LitStr,
    order: usize,
}

#[derive(Clone)]
struct SubDiagnosticUnit {
    kind: SubDiagKind,
    message: LitStr,
    field: Option<Ident>,
    labels: Vec<LabelEntry>,
    order: usize,
}

#[derive(Clone)]
enum PendingItem {
    Note {
        field: Option<Ident>,
        message: LitStr,
        label_override: Option<LitStr>,
        order: usize,
    },
    Help {
        field: Option<Ident>,
        message: LitStr,
        label_override: Option<LitStr>,
        order: usize,
    },
    SecondaryLabel {
        field: Ident,
        text: LitStr,
        order: usize,
    },
}

impl PendingItem {
    const fn order(&self) -> usize {
        match self {
            Self::Note { order, .. }
            | Self::Help { order, .. }
            | Self::SecondaryLabel { order, .. } => *order,
        }
    }
}

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
    pending: Vec<PendingItem>,
    depth: usize,
    nested: bool,
    #[expect(unused)]
    span: Span,
}

impl Config {
    fn parse_subdiagnostic(
        meta: &syn::meta::ParseNestedMeta,
        kind: SubDiagKind,
        field: Option<Ident>,
        order: usize,
        pending: &mut Vec<PendingItem>,
    ) -> Result<()> {
        if meta.input.peek(Token![=]) {
            let key = match kind {
                SubDiagKind::Note => "note",
                SubDiagKind::Help => "help",
            };
            return Err(meta.error(format!(
                "use `#[diag({key}(\"...\"))]` instead of `#[diag({key} = \"...\")]`"
            )));
        }
        let content;
        syn::parenthesized!(content in meta.input);
        let message: LitStr = content.parse()?;
        let mut label_override = None;
        while !content.is_empty() {
            content.parse::<Token![,]>()?;
            let key: Ident = content.parse()?;
            content.parse::<Token![=]>()?;
            if key == "label" {
                if label_override.is_some() {
                    return Err(syn::Error::new(key.span(), "duplicate `label` key"));
                }
                label_override = Some(content.parse()?);
            } else {
                return Err(syn::Error::new(key.span(), "unknown subdiagnostic key"));
            }
        }
        pending.push(match kind {
            SubDiagKind::Note => PendingItem::Note {
                field,
                message,
                label_override,
                order,
            },
            SubDiagKind::Help => PendingItem::Help {
                field,
                message,
                label_override,
                order,
            },
        });
        Ok(())
    }
    fn parse_secondary_label(
        meta: &syn::meta::ParseNestedMeta,
        field: Ident,
        order: usize,
        pending: &mut Vec<PendingItem>,
    ) -> Result<()> {
        if meta.input.peek(Token![=]) {
            return Err(meta.error("use `#[diag(label(\"...\"))]` for secondary labels on fields"));
        }
        let content;
        syn::parenthesized!(content in meta.input);
        let text: LitStr = content.parse()?;
        if !content.is_empty() {
            return Err(syn::Error::new(
                content.span(),
                "unexpected tokens in `#[diag(label(\"...\"))]`",
            ));
        }
        pending.push(PendingItem::SecondaryLabel { field, text, order });
        Ok(())
    }
    fn finalize_diags(
        &self,
        span_field: Option<&Ident>,
        label: &Option<LitStr>,
        msg: &Option<LitStr>,
        ident: &Ident,
    ) -> Result<(Vec<LabelEntry>, Vec<SubDiagnosticUnit>)> {
        let primary_text = label.clone().or_else(|| msg.clone()).ok_or_else(|| {
            Error::new_spanned(
                ident,
                "Missing label or message. Consider using `#[diag(label = \"...\")]`",
            )
        })?;
        let primary_field = span_field
            .cloned()
            .unwrap_or_else(|| format_ident!("_primary"));
        let mut primary_labels = vec![LabelEntry {
            field: primary_field,
            text: primary_text,
            order: 0,
        }];
        let mut units: Vec<SubDiagnosticUnit> = Vec::new();
        let mut sorted = self.pending.clone();
        sorted.sort_by_key(PendingItem::order);
        for item in sorted {
            match item {
                PendingItem::Note {
                    field,
                    message,
                    label_override,
                    order,
                } => {
                    let anchor = field.clone().unwrap_or_else(|| {
                        span_field
                            .cloned()
                            .unwrap_or_else(|| format_ident!("_primary"))
                    });
                    let label_text = label_override.unwrap_or_else(|| message.clone());
                    units.push(SubDiagnosticUnit {
                        kind: SubDiagKind::Note,
                        message,
                        field,
                        labels: vec![LabelEntry {
                            field: anchor,
                            text: label_text,
                            order,
                        }],
                        order,
                    });
                }
                PendingItem::Help {
                    field,
                    message,
                    label_override,
                    order,
                } => {
                    let anchor = field.clone().unwrap_or_else(|| {
                        span_field
                            .cloned()
                            .unwrap_or_else(|| format_ident!("_primary"))
                    });
                    let label_text = label_override.unwrap_or_else(|| message.clone());
                    units.push(SubDiagnosticUnit {
                        kind: SubDiagKind::Help,
                        message,
                        field,
                        labels: vec![LabelEntry {
                            field: anchor,
                            text: label_text,
                            order,
                        }],
                        order,
                    });
                }
                PendingItem::SecondaryLabel { field, text, order } => {
                    if let Some(unit) = units
                        .iter_mut()
                        .find(|unit| unit.field.as_ref() == Some(&field))
                    {
                        if unit.labels.iter().any(|entry| entry.field != field) {
                            return Err(Error::new_spanned(
                                &field,
                                "additional labels on a note/help must use the same field",
                            ));
                        }
                        unit.labels.push(LabelEntry { field, text, order });
                    } else {
                        primary_labels.push(LabelEntry { field, text, order });
                    }
                }
            }
        }
        primary_labels.sort_by_key(|entry| entry.order);
        units.sort_by_key(|unit| unit.order);
        for unit in &units {
            if unit.labels.is_empty() {
                return Err(Error::new_spanned(
                    ident,
                    "subdiagnostic must have at least one label",
                ));
            }
            if let Some(field) = &unit.field {
                if unit.labels.iter().any(|entry| entry.field != *field) {
                    return Err(Error::new_spanned(
                        field,
                        "all labels in one note/help must refer to the same field",
                    ));
                }
            }
        }
        Ok((primary_labels, units))
    }
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
            pending: Vec::new(),
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
        let mut kind = self.kind;
        let mut number = self.number.clone();
        let mut msg = self.msg.clone();
        let mut label = self.label.clone();
        let mut pending = self.pending.clone();
        let mut span_field = self.span_field.clone();
        let mut span_type = self.span_type.clone();
        let depth = self.depth + 1;
        let mut nested = self.nested;
        let mut unused_attrs = Vec::new();
        let mut item_order = 0usize;

        for attr in attrs {
            if attr.path().is_ident("diag") {
                attr.parse_nested_meta(|meta| {
                    if meta.path.is_ident("kind") {
                        let value: LitStr = meta.value()?.parse()?;
                        kind = Some(value.try_into()?);
                    } else if meta.path.is_ident("label") {
                        if meta.input.peek(Token![=]) {
                            let value: LitStr = meta.value()?.parse()?;
                            label = Some(value);
                        } else {
                            return Err(meta.error(
                                "`#[diag(label(\"...\"))]` on variants is invalid; use `#[diag(label = \"...\")]` for the primary label",
                            ));
                        }
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
                    } else if meta.path.is_ident("note") {
                        let order = item_order;
                        item_order += 1;
                        Self::parse_subdiagnostic(
                            &meta,
                            SubDiagKind::Note,
                            None,
                            order,
                            &mut pending,
                        )?;
                    } else if meta.path.is_ident("help") {
                        let order = item_order;
                        item_order += 1;
                        Self::parse_subdiagnostic(
                            &meta,
                            SubDiagKind::Help,
                            None,
                            order,
                            &mut pending,
                        )?;
                    } else {
                        return Err(meta.error("Unknown attribute key."));
                    }
                    Ok(())
                })?
            } else {
                unused_attrs.push(attr.clone());
            }
        }

        if let Some(fields) = fields {
            for (idx, field) in fields.iter().enumerate() {
                let field_ident = field.ident.clone().unwrap_or(format_ident!("_{idx}"));
                for attr in &field.attrs {
                    if attr.path().is_ident("diag") {
                        attr.parse_nested_meta(|meta| {
                            if meta.path.is_ident("span") {
                                span_field = Some(field_ident.clone());
                            } else if meta.path.is_ident("note") {
                                let order = item_order;
                                item_order += 1;
                                Self::parse_subdiagnostic(
                                    &meta,
                                    SubDiagKind::Note,
                                    Some(field_ident.clone()),
                                    order,
                                    &mut pending,
                                )?;
                            } else if meta.path.is_ident("help") {
                                let order = item_order;
                                item_order += 1;
                                Self::parse_subdiagnostic(
                                    &meta,
                                    SubDiagKind::Help,
                                    Some(field_ident.clone()),
                                    order,
                                    &mut pending,
                                )?;
                            } else if meta.path.is_ident("label") {
                                let order = item_order;
                                item_order += 1;
                                Self::parse_secondary_label(
                                    &meta,
                                    field_ident.clone(),
                                    order,
                                    &mut pending,
                                )?;
                            } else {
                                return Err(meta.error("Unknown attribute key."));
                            }
                            Ok(())
                        })?
                    }
                }
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
            pending,
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
        let new_config = config.process(node.attrs(), node.ident(), node.fields(), span)?;
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
                let iter = Either::Right(core::iter::once(ErrorTreeIter::process_next(
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
        let attrs = input.call(Attribute::parse_outer)?;
        let vis = input.parse()?;
        let name: Ident = input.parse()?;
        let generics = input.parse()?;
        let children;
        let brace = braced!(children in input);
        let config = Config::new(name.span()).process(&attrs, None, None, name.span())?;

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
                    let (variant, comma) = pair.into_tuple();
                    let span = variant.ident.span();
                    let node = ErrorTree::Variant {
                        span,
                        attrs: variant.attrs,
                        ident: variant.ident,
                        fields: variant.fields,
                    };
                    roots.push_value(node);
                    if let Some(comma) = comma {
                        roots.push_punct(comma);
                    }
                }
                let config = Config::new(ident.span()).process(&attrs, None, None, ident.span())?;
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
            syn::Data::Struct(data_struct) => {
                let span = ident.span();
                let config = Config::new(ident.span()).process(&attrs, None, None, ident.span())?;
                attrs.retain(|attr| !attr.path().is_ident("diag"));

                let node = ErrorTree::Variant {
                    span,
                    attrs,
                    ident: ident.clone(),
                    fields: data_struct.fields,
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
                let (kind, msg, number, mut attrs, ident, mut fields) = config?;

                let kind = kind.unwrap_or_default();
                let code = format!("{}{}", kind.short_str(), number);

                let doc = match msg {
                    Some(msg) => {
                        format!("`{code}`: {msg}", msg = msg.value())
                    }
                    None => format!("`{code}`"),
                };

                attrs.retain(|attr| !attr.path().is_ident("diag"));
                attrs.push(syn::parse_quote! {
                    #[doc = #doc]
                });
                attrs.push(syn::parse_quote! {
                    #[doc(alias = #code)]
                });

                for field in fields.iter_mut() {
                    field.attrs.retain(|attr| !attr.path().is_ident("diag"));
                }

                Ok(Variant {
                    attrs,
                    ident,
                    fields,
                    discriminant: None,
                })
            })
            .collect()
    }
    fn process_unnamed_fields(msg: &str) -> Cow<'_, str> {
        static ARG: Lazy<Regex> =
            lazy_regex!(r#"(?<prefix>(^|[^\{])(\{\{)*)\{(?<index>\d+)(?<optional>:[^\{\}]*)?\}"#);
        ARG.replace_all(msg, |cap: &Captures| {
            let prefix = &cap["prefix"].replace("{", "{{");
            let index = &cap["index"];
            if let Some(optional) = &cap.name("optional") {
                format!("{}{{_{}{}}}", prefix, index, optional.as_str())
            } else {
                format!("{}{{_{}}}", prefix, index)
            }
        })
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
                let msg = msg.value();
                let msg = Self::process_unnamed_fields(&msg);
                Ok(quote! {
                    #prefix ( #(#params),* ) => ::core::write!(f, #msg),
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
    fn label_vec1_codegen(
        &self,
        entries: &[LabelEntry],
        unnamed: bool,
        spanless: bool,
    ) -> TokenStream2 {
        let span_type = self.span_type();
        let pairs = entries.iter().map(|entry| {
            let field = &entry.field;
            let text = &entry.text;
            let span_expr = if spanless {
                quote! { <#span_type as ::core::default::Default>::default() }
            } else {
                quote! { <#span_type as ::core::convert::From<_>>::from(#field) }
            };
            if unnamed {
                let value = text.value();
                let value = Self::process_unnamed_fields(&value);
                quote! { (#span_expr, ::error_enum::format!(#value)) }
            } else {
                quote! { (#span_expr, ::error_enum::format!(#text)) }
            }
        });
        quote! { ::error_enum::vec1![ #(#pairs),* ] }
    }
    fn primary_labels(&self) -> Result<Vec<TokenStream2>> {
        self.iter()?
            .filter_map(|config| {
                config
                    .map(
                        |Config {
                             msg,
                             ident,
                             fields,
                             label,
                             span_field,
                             pending,
                             ..
                         }| {
                            Some((msg, ident?, fields?, label, span_field, pending))
                        },
                    )
                    .transpose()
            })
            .map(|config| {
                let (msg, ident, fields, label, span_field, pending) = config?;
                let config = Config {
                    pending,
                    ..Config::new(ident.span())
                };
                let (primary_labels, _) =
                    config.finalize_diags(span_field.as_ref(), &label, &msg, &ident)?;
                self.primary_labels_branch(&ident, &fields, span_field.as_ref(), &primary_labels)
            })
            .collect()
    }
    fn primary_labels_branch(
        &self,
        ident: &Ident,
        fields: &Fields,
        span_field: Option<&Ident>,
        entries: &[LabelEntry],
    ) -> Result<TokenStream2> {
        let prefix = self.variant(ident);
        let labels = self.label_vec1_codegen(
            entries,
            matches!(fields, Fields::Unnamed(_)),
            span_field.is_none(),
        );
        match fields {
            Fields::Named(named) => {
                let members = named.named.iter().map(|f| f.ident.as_ref());
                Ok(quote! {
                    #[allow(unused_variables)]
                    #prefix { #(#members),* } => #labels,
                })
            }
            Fields::Unnamed(unnamed) => {
                let params = (0..unnamed.unnamed.len()).map(|i| format_ident!("_{}", i));
                Ok(quote! {
                    #prefix ( #(#params),* ) => #labels,
                })
            }
            Fields::Unit => Ok(quote! {
                #prefix => #labels,
            }),
        }
    }
    fn additional_unit_tokens(&self, unit: &SubDiagnosticUnit, unnamed: bool) -> TokenStream2 {
        let spanless = unit.field.is_none();
        let labels = self.label_vec1_codegen(&unit.labels, unnamed, spanless);
        let message = &unit.message;
        let message_fmt = if unnamed {
            let value = message.value();
            let value = Self::process_unnamed_fields(&value);
            quote! { ::error_enum::format!(#value) }
        } else {
            quote! { ::error_enum::format!(#message) }
        };
        let kind = match unit.kind {
            SubDiagKind::Note => quote! { ::error_enum::AdditionalKind::Note },
            SubDiagKind::Help => quote! { ::error_enum::AdditionalKind::Help },
        };
        quote! {
            (
                #message_fmt,
                #labels,
                #kind,
            )
        }
    }
    fn additional_branch(
        &self,
        ident: &Ident,
        fields: &Fields,
        units: &[SubDiagnosticUnit],
    ) -> Result<TokenStream2> {
        let prefix = self.variant(ident);
        let box_type: syn::Expr = parse_quote!(::error_enum::Box);
        let unnamed = matches!(fields, Fields::Unnamed(_));
        let additional = units
            .iter()
            .map(|unit| self.additional_unit_tokens(unit, unnamed));
        match fields {
            Fields::Named(named) => {
                let members = named.named.iter().map(|f| f.ident.as_ref());
                Ok(quote! {
                    #[allow(unused_variables)]
                    #prefix { #(#members),* } => #box_type::new([
                        #(#additional,)*
                    ].into_iter()),
                })
            }
            Fields::Unnamed(unnamed_fields) => {
                let params = (0..unnamed_fields.unnamed.len()).map(|i| format_ident!("_{}", i));
                Ok(quote! {
                    #prefix ( #(#params),* ) => #box_type::new([
                        #(#additional,)*
                    ].into_iter()),
                })
            }
            Fields::Unit => Ok(quote! {
                #prefix => #box_type::new([].into_iter()),
            }),
        }
    }
    fn additional(&self) -> Result<Vec<TokenStream2>> {
        self.iter()?
            .filter_map(|config| {
                config
                    .map(
                        |Config {
                             ident,
                             fields,
                             msg,
                             label,
                             span_field,
                             pending,
                             ..
                         }| {
                            Some((ident?, fields?, msg, label, span_field, pending))
                        },
                    )
                    .transpose()
            })
            .map(|config| {
                let (ident, fields, msg, label, span_field, pending) = config?;
                let config = Config {
                    pending,
                    ..Config::new(ident.span())
                };
                let (_, units) =
                    config.finalize_diags(span_field.as_ref(), &label, &msg, &ident)?;
                self.additional_branch(&ident, &fields, &units)
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
            quote! {::core::option::Option::Some(<#span_type as ::core::convert::From<_>>::from(#span_field))}
        } else {
            quote! {::core::option::Option::None}
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
                             kind,
                             number,
                             span_field,
                             ..
                         }| {
                            Some((ident?, fields?, kind, number, span_field))
                        },
                    )
                    .transpose()
            })
            .map(|config| {
                let (ident, fields, kind, number, span_field) = config?;
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
        let attrs: Vec<&Attribute> = self
            .attrs
            .iter()
            .filter(|attr| !attr.path().is_ident("diag"))
            .collect();
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
        let primary_labels = self.primary_labels()?;
        let additional = self.additional()?;
        let span_type = self.span_type();
        let option_span_type: Type = parse_quote!(::core::option::Option<#span_type>);
        let msg_type: Type = parse_quote!(::error_enum::String);
        let box_type: Type = parse_quote!(::error_enum::Box);
        let iterator_trait: Type = parse_quote!(::core::iter::Iterator);
        tokens.extend(quote! {
            impl #impl_generics ::error_enum::ErrorType for #name #ty_generics #where_clause {
                type Span = #span_type;
                type Message = #msg_type;
                type Label = #msg_type;

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
                fn primary_span(&self) -> #option_span_type {
                    match self {
                        #(#primary_span)*
                    }
                }
                fn primary_message(&self) -> #msg_type {
                    ::error_enum::format!("{self}")
                }
                fn primary_labels(&self) -> ::error_enum::LabelVec1<#span_type, #msg_type> {
                    match self {
                        #(#primary_labels)*
                    }
                }
                fn additional(&self) -> #box_type<dyn #iterator_trait<Item = (#msg_type, ::error_enum::LabelVec1<#span_type, #msg_type>, ::error_enum::AdditionalKind)>> {
                    match self {
                        #(#additional)*
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
/// # Syntax
///
/// ```ignore
/// $error_type =
///     $vis:vis $name:ident {
///         $($variant:variant, )*
///     }
///
/// $variant =
///   // Prefix node.
///     {
///         $($child_variant:variant, )*
///     }
///   // Leaf node (three forms, just the same as `syn::Variant`).
///   | $ident:ident (
///         $(
///             $field_ty:ty
///         ),*
///     )
///   | $ident:ident {
///         $(
///             $field_name:ident: $field_ty:ty
///         ),*
///     }
///   | $ident:ident
/// ```
///
#[doc = include_str!("../attributes.md")]
#[proc_macro]
pub fn error_type(token: TokenStream) -> TokenStream {
    let error = parse_macro_input!(token as ErrorEnum);
    error.to_token_stream().into()
}

/// Implement error capabilities for an existing enum.
///
#[doc = include_str!("../attributes.md")]
#[proc_macro_derive(ErrorType, attributes(diag))]
pub fn error_enum(token: TokenStream) -> TokenStream {
    let input: DeriveInput = parse_macro_input!(token as DeriveInput);
    let error = ErrorEnum::try_from(input)
        .map_or_else(|err| err.to_compile_error(), |e| e.to_token_stream());
    error.into()
}
