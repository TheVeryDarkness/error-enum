//! To generate [Display](core::fmt::Display) implementation and
//! documentation comments for error types.

#![warn(rust_2021_compatibility, rustdoc::all, missing_docs)]

use ansi_term::Color;
#[cfg(feature = "colored")]
use ansi_term::Style;
use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::{format_ident, quote, ToTokens};
use std::iter::once;
use syn::{
    braced,
    parse::{self, Parse},
    parse_macro_input, Attribute, Expr, ExprLit, ExprTuple, Fields, Generics, Ident, Lit, LitInt,
    LitStr, Meta, Token, Variant, Visibility,
};

/// Configuration for each variant.
#[derive(Clone, Copy, Debug, Default)]
struct Config {
    nested: bool,
    #[cfg(feature = "colored")]
    style: Style,
}
impl Config {
    pub fn from_attrs(attrs: &Vec<Attribute>) -> Self {
        let mut res = Self::default();
        res.on_attrs(attrs);
        res
    }
    pub fn on_attrs(&mut self, attrs: &Vec<Attribute>) {
        for attr in attrs {
            self.on_attr(attr)
        }
    }
    fn lit_str_to_color(str: &LitStr) -> Color {
        let str = str.value();
        match str.as_str() {
            "black" => Color::Black,
            "red" => Color::Red,
            "green" => Color::Green,
            "yellow" => Color::Yellow,
            "blue" => Color::Blue,
            "purple" => Color::Purple,
            "cyan" => Color::Cyan,
            "white" => Color::White,
            _ => panic!("Unexpected color `{}`.", str),
        }
    }
    fn rgb_tuple_to_color(tuple: &ExprTuple) -> Color {
        assert!(
            tuple.elems.len() == 3,
            "RGB color should has 3 componenets."
        );
        let mut iter = tuple.elems.iter();
        let mut get_component = || -> u8 {
            let component = iter.next().unwrap();
            if let Expr::Lit(ExprLit {
                lit: Lit::Int(int),
                attrs: _attrs,
            }) = component
            {
                int.base10_parse().expect("Invalid RGB code")
            } else {
                panic!("Unsupported expression in RGB code.");
            }
        };
        Color::RGB(get_component(), get_component(), get_component())
    }
    fn on_attr(&mut self, attr: &Attribute) {
        let res = self;
        match &attr.meta {
            Meta::List(_list) => {
                unimplemented!("Attribute list.");
            }
            Meta::NameValue(name_value) => {
                if let Some(ident) = name_value.path.get_ident() {
                    #[cfg(feature = "colored")]
                    {
                        if ident == "color" || ident == "foreground" || ident == "fg" {
                            match &name_value.value {
                                Expr::Lit(literal) => {
                                    if !literal.attrs.is_empty() {
                                        eprintln!("Attributes in literal is ignored.");
                                    }
                                    match &literal.lit {
                                        Lit::Int(int) => {
                                            res.style = res.style.fg(Color::Fixed(
                                                int.base10_parse().expect("Invalid color."),
                                            ));
                                        }
                                        Lit::Str(str) => {
                                            res.style = res.style.fg(Self::lit_str_to_color(str))
                                        }
                                        _ => {
                                            unimplemented!("Unsupported literal in MetaNameValue.")
                                        }
                                    }
                                }
                                Expr::Tuple(tuple) => {
                                    res.style = res.style.fg(Self::rgb_tuple_to_color(tuple))
                                }
                                _ => unimplemented!("Unsupported expression in MetaNameValue."),
                            }
                        } else if ident == "background" || ident == "bg" {
                            match &name_value.value {
                                Expr::Lit(literal) => {
                                    if !literal.attrs.is_empty() {
                                        eprintln!("Attributes in literal is ignored.");
                                    }
                                    match &literal.lit {
                                        Lit::Int(int) => {
                                            res.style = res.style.on(Color::Fixed(
                                                int.base10_parse().expect("Invalid color."),
                                            ));
                                        }
                                        Lit::Str(str) => {
                                            res.style = res.style.on(Self::lit_str_to_color(str));
                                        }
                                        _ => {
                                            unimplemented!("Unsupported literal in MetaNameValue.")
                                        }
                                    }
                                }
                                Expr::Tuple(tuple) => {
                                    res.style = res.style.on(Self::rgb_tuple_to_color(tuple))
                                }
                                _ => unimplemented!("Unsupported expression in MetaNameValue."),
                            }
                        }
                    }
                    #[cfg(not(feature = "colored"))]
                    unimplemented!("Path in MetaNameValue.");
                } else {
                    unimplemented!("Path in MetaNameValue.");
                }
            }
            Meta::Path(path) => {
                if let Some(ident) = path.get_ident() {
                    if ident == "nested" {
                        res.nested = true;
                    } else {
                        #[cfg(feature = "colored")]
                        {
                            macro_rules! set_config {
                                ($ident:ident) => {
                                    if ident == stringify!($ident) {
                                        res.style = res.style.$ident();
                                    }
                                };
                            }
                            set_config!(bold);
                            set_config!(dimmed);
                            set_config!(italic);
                            set_config!(underline);
                            set_config!(blink);
                            set_config!(reverse);
                            set_config!(hidden);
                            set_config!(strikethrough);
                        }
                        #[cfg(not(feature = "colored"))]
                        unimplemented!("Path in MetaNameValue.");
                    }
                } else {
                    unimplemented!("Path in attribute.");
                }
            }
        }
    }
}

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
    Prefix(Vec<Attribute>, LitInt, LitStr, Vec<ErrorTree>),
    Variant(Vec<Attribute>, LitInt, ErrorVariant),
}

impl Parse for ErrorTree {
    fn parse(input: parse::ParseStream) -> syn::Result<Self> {
        if input.peek2(LitStr) {
            let attrs = input.call(Attribute::parse_outer)?;
            let code = input.parse()?;
            let desc = input.parse()?;
            let children;
            braced!(children in input);
            let mut nodes = Vec::new();
            while !children.is_empty() {
                let node = children.parse()?;
                nodes.push(node);
            }
            Ok(ErrorTree::Prefix(attrs, code, desc, nodes))
        } else {
            let attrs = input.call(Attribute::parse_outer)?;
            let code = input.parse()?;
            let variant = input.parse()?;
            let _comma: Token![,] = input.parse()?;
            Ok(ErrorTree::Variant(attrs, code, variant))
        }
    }
}

impl ErrorTree {
    /// - [Config].
    /// - Code.
    /// - [ErrorVariant].
    fn get_variants<'s>(
        &'s self,
        config: Config,
        prefix: String,
    ) -> impl Iterator<Item = (Config, String, &'s ErrorVariant)> {
        match self {
            Self::Prefix(attrs, code, _desc, children) => children
                .iter()
                .flat_map(|node| {
                    let mut config = config.clone();
                    config.on_attrs(attrs);
                    node.get_variants(config, format!("{prefix}{}", code.to_string()))
                })
                .collect::<Vec<_>>()
                .into_iter(),
            Self::Variant(attrs, code, var) => {
                let prefix = format!("{prefix}{code}");
                let mut config = config;
                config.on_attrs(attrs);
                vec![(config, prefix, var)].into_iter()
            }
        }
    }
    /// - Depth.
    /// - Prefix.
    /// - Variant name.
    /// - Message (for [Display](core::fmt::Display)).
    fn get_nodes<'s>(
        &'s self,
        prefix: &str,
        depth: usize,
    ) -> impl Iterator<Item = (usize, String, Option<String>, String)> {
        match self {
            Self::Prefix(_attrs, code, desc, children) => {
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
            Self::Variant(_attrs, code, var) => {
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
    variants: Vec<(Vec<Attribute>, Ident, LitStr, Vec<ErrorTree>)>,
}

impl ErrorEnum {
    /// - [Config].
    /// - Code.
    /// - [ErrorVariant].
    fn get_variants<'s>(&'s self) -> impl Iterator<Item = (Config, String, &'s ErrorVariant)> {
        self.variants.iter().flat_map(|(attrs, ident, _, tree)| {
            tree.iter().flat_map(|node| {
                let mut config = Config::from_attrs(&self.attrs);
                config.on_attrs(attrs);
                node.get_variants(config, ident.to_string())
            })
        })
    }
    /// - Depth.
    /// - Prefix.
    /// - Variant name.
    /// - Message (for [Display](core::fmt::Display)).
    fn get_nodes<'s>(&'s self) -> Vec<(usize, String, Option<String>, String)> {
        self.variants
            .iter()
            .flat_map(|(_, ident, msg, tree)| {
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
            let attrs = input.call(Attribute::parse_outer)?;
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
            variants.push((attrs, kind, msg, trees));
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
                .for_each(|(_cfg, code, var)| var.to_tokens(&code, &mut tokens));
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

        let get_code = self.get_variants().map(|(_cfg, code, variant)| {
            let name = &variant.variant.ident;
            quote! {
                Self::#name { .. } => #code,
            }
        });
        let get_prefix = self.get_variants().map(|(cfg, code, variant)| {
            let name = &variant.variant.ident;
            let prefix = format!("error[{}]", &code);
            // eprintln!("{:?}", cfg);
            #[cfg(feature = "colored")]
            let prefix = cfg.style.paint(prefix).to_string();
            let prefix = format!("{prefix}: ");
            quote! {
                Self::#name {..} => #prefix,
            }
        });
        let fmt_self = self
            .get_variants()
            .map(|(_cfg, _code, variant)| variant.fmt_self());
        tokens.extend(quote! {
            impl #impl_generics #name #ty_generics #where_clause {
                /// Get error code like `[E0000]`.
                pub fn get_code(&self) -> &'static str {
                    match self {
                        #(#get_code)*
                    }
                }
                /// Get error message prefix like `error[E0000]:`.
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
/// 2. Attributes (optional).
///     - `#[bold]` for bold text.
///     - `#[dimmed]` for dimmed text.
///     - `#[italic]` for italic text.
///     - `#[underline]` for text with an underline.
///     - `#[blink]` for blinking text.
///     - `#[reverse]` for text with reversed color.
///     - `#[hidden]` for hidden text.
///     - `#[strikethrough]` for text with strikethrough.
///     - `#[color = "<color name>"]` for overriding default color. Only 8 colors below are supported:
///
///         - `black`.
///         - `red`.
///         - `green`.
///         - `yellow`.
///         - `blue`.
///         - `purple`.
///         - `cyan`.
///         - `white`.
///
///     - `#[color = FIXED_COLOR]` for overriding default color with a color code.
///         See [ansi_term::Color].
///     - `#[color = (R, G, B)]` for overriding default color with RGB.
///     - `#[nested]` if it's a nested error generated by [error_enum](crate).
/// 3. Name.
/// 4. Fields.
/// 5. Message.
/// 6. A trailing comma.
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
                #[color = (0xaf, 0, 0)]
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
