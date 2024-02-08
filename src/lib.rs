//! To generate [Display](core::fmt::Display) implementation and
//! documentation comments for error types.
//!
//! |     Concept      |            Example             |
//! |:----------------:|:------------------------------:|
//! |      Number      |             `1234`             |
//! |       Code       |            `E1234`             |
//! |     Category     |              `E`               |
//! |       Kind       |         `error[E1234]`         |
//! | Message Category |        `error[E1234]: `        |
//! |   Description    |        `Access denied.`        |
//! |     Message      | `error[E1234]: Access denied.` |

#![warn(rust_2021_compatibility, rustdoc::all, missing_docs)]

#[cfg(feature = "colored")]
use ansi_term::{Color, Style};
use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::{format_ident, quote, ToTokens};
use std::iter::once;
use syn::{
    braced,
    parse::{self, Parse},
    parse_macro_input, Attribute, Fields, Generics, Ident, LitInt, LitStr, Meta, Token, Variant,
    Visibility,
};

/// Configuration for each variant.
#[derive(Clone, Copy, Debug)]
struct Config {
    category: char,
    nested: bool,
    #[cfg(feature = "colored")]
    style_prefix: Style,
    #[cfg(feature = "colored")]
    style_message: Style,
}
impl Config {
    pub fn new(category: String) -> Self {
        assert_eq!(category.len(), 1, "Length of category can only be 1.");
        let category = category.chars().next().expect("Category can't be empty.");
        let nested = false;
        #[cfg(feature = "colored")]
        {
            let style_prefix = Style::default();
            let style_message = Style::default();
            Self {
                category,
                nested,
                style_prefix,
                style_message,
            }
        }
        #[cfg(not(feature = "colored"))]
        {
            Self { category, nested }
        }
    }
    #[cfg(feature = "colored")]
    pub fn prefix(&self) -> ansi_term::Prefix {
        self.style_prefix.prefix()
    }
    #[cfg(feature = "colored")]
    pub fn infix(&self) -> ansi_term::Infix {
        self.style_prefix.infix(self.style_message)
    }
    #[cfg(feature = "colored")]
    pub fn suffix(&self) -> ansi_term::Suffix {
        self.style_message.suffix()
    }
    pub fn on_category(&mut self, _category: &Ident) {
        #[cfg(feature = "colored")]
        {
            let category = _category;
            match category.to_string().as_str() {
                "E" => self.style_prefix = self.style_prefix.fg(Color::Red),
                "W" => self.style_prefix = self.style_prefix.fg(Color::Yellow),
                _ => {}
            }
        }
    }
    pub fn on_attrs(&mut self, attrs: &Vec<Attribute>) {
        for attr in attrs {
            self.on_attr(attr)
        }
    }
    #[cfg(feature = "colored")]
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
    #[cfg(feature = "colored")]
    fn rgb_tuple_to_color(tuple: &syn::ExprTuple) -> Color {
        assert!(
            tuple.elems.len() == 3,
            "RGB color should have 3 componenets."
        );
        let mut iter = tuple.elems.iter();
        let mut get_component = || -> u8 {
            let component = iter.next().unwrap();
            if let syn::Expr::Lit(syn::ExprLit {
                lit: syn::Lit::Int(int),
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
                if let Some(_ident) = name_value.path.get_ident() {
                    #[cfg(feature = "colored")]
                    {
                        let ident = _ident;
                        if ident == "color" || ident == "foreground" || ident == "fg" {
                            match &name_value.value {
                                syn::Expr::Lit(literal) => {
                                    if !literal.attrs.is_empty() {
                                        eprintln!("Attributes in literal is ignored.");
                                    }
                                    match &literal.lit {
                                        syn::Lit::Int(int) => {
                                            res.style_prefix = res.style_prefix.fg(Color::Fixed(
                                                int.base10_parse().expect("Invalid color."),
                                            ));
                                        }
                                        syn::Lit::Str(str) => {
                                            res.style_prefix =
                                                res.style_prefix.fg(Self::lit_str_to_color(str))
                                        }
                                        _ => {
                                            unimplemented!("Unsupported literal in MetaNameValue.")
                                        }
                                    }
                                }
                                syn::Expr::Tuple(tuple) => {
                                    res.style_prefix =
                                        res.style_prefix.fg(Self::rgb_tuple_to_color(tuple))
                                }
                                _ => unimplemented!("Unsupported expression in MetaNameValue."),
                            }
                        } else if ident == "background" || ident == "bg" {
                            match &name_value.value {
                                syn::Expr::Lit(literal) => {
                                    if !literal.attrs.is_empty() {
                                        eprintln!("Attributes in literal is ignored.");
                                    }
                                    match &literal.lit {
                                        syn::Lit::Int(int) => {
                                            res.style_prefix = res.style_prefix.on(Color::Fixed(
                                                int.base10_parse().expect("Invalid color."),
                                            ));
                                        }
                                        syn::Lit::Str(str) => {
                                            res.style_prefix =
                                                res.style_prefix.on(Self::lit_str_to_color(str));
                                        }
                                        _ => {
                                            unimplemented!("Unsupported literal in MetaNameValue.")
                                        }
                                    }
                                }
                                syn::Expr::Tuple(tuple) => {
                                    res.style_prefix =
                                        res.style_prefix.on(Self::rgb_tuple_to_color(tuple))
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
                                        res.style_message = res.style_message.$ident();
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
    fn fmt_desc(&self) -> TokenStream2 {
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
    fn get_desc(&self) -> TokenStream2 {
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
                        ::std::format!{#msg}
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
                        ::std::format!{#msg, #(#elements, )*}
                    }
                }
            }
            Fields::Unit => {
                quote! {
                    Self::#name => {
                        ::std::format!{#msg}
                    }
                }
            }
        }
    }
    fn get_category(&self, cfg: Config) -> TokenStream2 {
        let name = &self.variant.ident;
        if cfg.nested {
            quote! {
                Self::#name (nested) => nested.get_category(),
            }
        } else {
            let category = cfg.category;
            quote! {
                Self::#name {..} => #category,
            }
        }
    }
    fn get_number(&self, cfg: Config, number: String) -> TokenStream2 {
        let name = &self.variant.ident;
        if cfg.nested {
            quote! {
                Self::#name (nested) => {
                    let number = nested.get_number();
                    ::std::borrow::Cow::Owned(::std::format!("{}{}", #number, number))
                }
            }
        } else {
            quote! {
                Self::#name {..} => ::std::borrow::Cow::Borrowed(#number),
            }
        }
    }
    fn get_code(&self, cfg: Config, number: String) -> TokenStream2 {
        let name = &self.variant.ident;
        if cfg.nested {
            quote! {
                Self::#name (nested) => {
                    let number = nested.get_number();
                    let category = nested.get_category();
                    ::std::borrow::Cow::Owned(::std::format!("{}{}{}", category, #number, number))
                }
            }
        } else {
            let category = cfg.category;
            let code = format!("{category}{number}");
            quote! {
                Self::#name {..} => ::std::borrow::Cow::Borrowed(#code),
            }
        }
    }
    fn get_prefix(&self, cfg: Config, number: String) -> TokenStream2 {
        let name = &self.variant.ident;
        // eprintln!("{:?}", cfg);
        if cfg.nested {
            let prefix = quote! {::std::format!("error[{}{}]", #number, nested.get_code())};
            quote! {
                Self::#name (nested) => ::std::borrow::Cow::Owned(#prefix),
            }
        } else {
            let prefix = format!("error[{}]", &number);
            quote! {
                Self::#name {..} => ::std::borrow::Cow::Borrowed(#prefix),
            }
        }
    }
    #[cfg(feature = "colored")]
    fn write_str(s: impl std::fmt::Display) -> TokenStream2 {
        let s = s.to_string();
        quote! {f.write_str(#s)?;}
    }
    fn fmt(&self, number: String, cfg: Config) -> TokenStream2 {
        let name = &self.variant.ident;
        let msg = &self.msg;
        #[cfg(feature = "colored")]
        let prefix = Self::write_str(cfg.prefix());
        #[cfg(feature = "colored")]
        let infix = Self::write_str(cfg.infix());
        #[cfg(feature = "colored")]
        let suffix = Self::write_str(cfg.suffix());
        #[cfg(not(feature = "colored"))]
        let prefix = quote! {};
        #[cfg(not(feature = "colored"))]
        let infix = quote! {};
        #[cfg(not(feature = "colored"))]
        let suffix = quote! {};
        let get_code = if cfg.nested {
            quote! {&self.get_code()}
        } else {
            let code = format!("{}{}", cfg.category, number);
            quote! {#code}
        };
        match &self.variant.fields {
            Fields::Named(fields) => {
                assert!(!cfg.nested, "Named fields can't be nested error.");
                let fields = fields
                    .named
                    .iter()
                    .map(|field| field.ident.as_ref().unwrap());
                quote! {
                    Self::#name { #(#fields, )* } => {
                        #prefix
                        f.write_str(#get_code)?;
                        f.write_str(": ")?;
                        #infix
                        ::core::write!{f, #msg}?;
                        #suffix
                        ::core::result::Result::Ok(())
                    }
                }
            }
            Fields::Unnamed(unnamed) => {
                if cfg.nested {
                    assert_eq!(
                        unnamed.unnamed.len(),
                        1,
                        "Nested error can consists of one unnamed fields.",
                    );
                    quote! {
                        Self::#name ( nested ) => {
                            #prefix
                            f.write_str(#get_code)?;
                            f.write_str(": ")?;
                            #infix
                            ::core::write!{f, #msg, nested.get_desc()}?;
                            #suffix
                            ::core::result::Result::Ok(())
                        }
                    }
                } else {
                    let elements = unnamed
                        .unnamed
                        .iter()
                        .enumerate()
                        .map(|(i, _)| format_ident!("_{i}"))
                        .collect::<Vec<_>>();
                    quote! {
                        Self::#name ( #(#elements, )* ) => {
                            #prefix
                            f.write_str(#get_code)?;
                            f.write_str(": ")?;
                            #infix
                            ::core::write!{f, #msg, #(#elements, )*}?;
                            #suffix
                            ::core::result::Result::Ok(())
                        }
                    }
                }
            }
            Fields::Unit => {
                assert!(!cfg.nested, "Unit can't be nested error.");
                quote! {
                    Self::#name => {
                        #prefix
                        f.write_str(#get_code)?;
                        f.write_str(": ")?;
                        #infix
                        ::core::write!{f, #msg}?;
                        #suffix
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
        prenumber: &'_ str,
    ) -> impl Iterator<Item = (Config, String, &'s ErrorVariant)> {
        match self {
            Self::Prefix(attrs, postnumber, _desc, children) => children
                .iter()
                .flat_map(|node| {
                    let mut config = config.clone();
                    config.on_attrs(attrs);
                    node.get_variants(config, &format!("{prenumber}{postnumber}"))
                })
                .collect::<Vec<_>>()
                .into_iter(),
            Self::Variant(attrs, postnumber, var) => {
                let code = format!("{prenumber}{postnumber}");
                let mut config = config;
                config.on_attrs(attrs);
                vec![(config, code, var)].into_iter()
            }
        }
    }
    /// - Depth.
    /// - Prefix.
    /// - Variant name.
    /// - Message (for [Display](core::fmt::Display)).
    fn get_nodes<'s>(
        &'s self,
        config: Config,
        prenumber: &str,
        depth: usize,
    ) -> impl Iterator<Item = (Config, usize, String, Option<String>, String)> {
        match self {
            Self::Prefix(_attrs, postnumber, desc, children) => {
                let number = format!("{}{}", prenumber, postnumber);
                once((config.clone(), depth, number.clone(), None, desc.value()))
                    .chain(
                        children
                            .iter()
                            .flat_map(|node| node.get_nodes(config.clone(), &number, depth + 1)),
                    )
                    .collect::<Vec<_>>()
                    .into_iter()
            }
            Self::Variant(_attrs, postnumber, var) => {
                let number = format!("{}{}", prenumber, postnumber);
                vec![(
                    config,
                    depth,
                    number,
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
        self.variants.iter().flat_map(|(attrs, category, _, tree)| {
            let mut config = Config::new(category.to_string());
            config.on_attrs(attrs);
            config.on_category(category);
            tree.iter()
                .flat_map(move |node| node.get_variants(config, ""))
        })
    }
    /// - [Config].
    /// - Depth.
    /// - Prefix.
    /// - Variant name.
    /// - Message (for [Display](core::fmt::Display)).
    fn get_nodes<'s>(&'s self) -> Vec<(Config, usize, String, Option<String>, String)> {
        self.variants
            .iter()
            .flat_map(|(attrs, category, msg, tree)| {
                let mut config = Config::new(category.to_string());
                config.on_category(category);
                config.on_attrs(attrs);
                once((config, 0, String::new(), None, msg.value())).chain(
                    tree.iter()
                        .flat_map(|node| node.get_nodes(config, "", 1))
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
            .map(|(_config, depth, code, name, desc)| {
                let indent = "  ".repeat(depth);
                match name {
                    Some(name) => format!("{indent}- `{code}`(**{name}**): {desc}"),
                    None => format!("{indent}- `{code}`: {desc}"),
                }
            });
        let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

        let error_variants = self.get_variants().collect::<Vec<_>>();

        let variants = {
            let mut tokens = TokenStream2::new();
            error_variants.iter().for_each(|(cfg, number, var)| {
                var.to_tokens(&format!("{}{}", cfg.category, number), &mut tokens)
            });
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

        let fmt = self
            .get_variants()
            .map(|(cfg, code, variant)| variant.fmt(code, cfg));
        tokens.extend(quote! {
            impl #impl_generics ::core::fmt::Display for #name #ty_generics #where_clause {
                fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                    match self {
                        #(#fmt)*
                    }
                }
            }
        });

        let get_category = self
            .get_variants()
            .map(|(cfg, _number, variant)| variant.get_category(cfg));
        let get_number = self
            .get_variants()
            .map(|(cfg, number, variant)| variant.get_number(cfg, number));
        let get_code = self
            .get_variants()
            .map(|(cfg, number, variant)| variant.get_code(cfg, number));
        let get_prefix = self
            .get_variants()
            .map(|(cfg, number, variant)| variant.get_prefix(cfg, number));
        let fmt_desc = self
            .get_variants()
            .map(|(_cfg, _number, variant)| variant.fmt_desc());
        let get_desc = self
            .get_variants()
            .map(|(_cfg, _number, variant)| variant.get_desc());
        tokens.extend(quote! {
            impl #impl_generics #name #ty_generics #where_clause {
                /// Write error category like `E`.
                pub fn get_category(&self) -> ::core::primitive::char {
                    match self {
                        #(#get_category)*
                    }
                }
                /// Write error code number like `0000`.
                pub fn get_number(&self) -> ::std::borrow::Cow<'static, str> {
                    match self {
                        #(#get_number)*
                    }
                }
                /// Write error code like `E0000`.
                pub fn get_code(&self) -> ::std::borrow::Cow<'static, str> {
                    match self {
                        #(#get_code)*
                    }
                }
                /// Write error message prefix like `error[E0000]: `.
                pub fn get_prefix(&self) -> ::std::borrow::Cow<'static, str> {
                    match self {
                        #(#get_prefix)*
                    }
                }
                fn fmt_desc(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                    match self {
                        #(#fmt_desc)*
                    }
                }
                pub fn get_desc(&self) -> String {
                    match self {
                        #(#get_desc)*
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
    fn basic() {
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
    }

    #[test]
    #[cfg(feature = "colored")]
    fn colored() {
        let output: ErrorEnum = syn::parse2(quote! {
            FileSystemError
                #[color = (0xaf, 0, 0)]
                #[color = (0xa8, 0xa8, 0xa8)]
                E "错误" {
                    01 FileNotFound {path: std::path::Path}
                    "{path} not found.",
                }
                #[fg = 214]
                #[bg = 025]
                W "警告" {
                    01 FileTooLarge {path: std::path::Path}
                    "{path} is too large.",
                }
                #[color = "blue"]
                H "提示" {
                    01 FileNameSuggestion (std::path::Path)
                    "{0} may be what you want.",
                }
        })
        .unwrap();
        let output = output.into_token_stream();
        eprintln!("{:#}", output);
    }

    #[test]
    #[cfg(feature = "colored")]
    fn colorful() {
        let output: ErrorEnum = syn::parse2(quote! {
            ColoredError
                #[bold]
                #[dimmer]
                E "错误" {
                    #[fg = "black"]
                    0 BlackError (u8)
                        "{0} is not black.",
                    #[bg = "red"]
                    1 RedError (u8, u8)
                        "{0} and {1} is red.",
                    #[fg = "green"]
                    #[bg = "yellow"]
                    2 GreenYellowError
                        "Code is green, while description is yellow.",
                    #[color = "blue"]
                    3 BlueError
                        "I'm blue.",
                    #[foreground = "purple"]
                    #[background = "cyan"]
                    4 PurpleCyanError
                        "Purpule and cyan.",
                    #[color = "white"]
                    5 WhiteError { white: String }
                        "All in white.",
                }
        })
        .unwrap();
        let output = output.into_token_stream();
        eprintln!("{:#}", output);
    }

    #[test]
    fn deep() {
        let output: ErrorEnum = syn::parse2(quote! {
            FileSystemError
                #[color = (0xaf, 0, 0)]
                E "错误" {
                    0 "文件错误" {
                        0 AccessDenied
                        "无权限。",
                    }
                }
        })
        .unwrap();
        let output = output.into_token_stream();
        eprintln!("{:#}", output);
    }

    #[test]
    fn nested() {
        let output: ErrorEnum = syn::parse2(quote! {
            FileSystemError
                #[color = (0xaf, 0, 0)]
                E "错误" {
                    #[nested]
                    01 FileError (FileError)
                    "{0}",
                }
        })
        .unwrap();
        let output = output.into_token_stream();
        eprintln!("{:#}", output);
    }

    #[test]
    #[should_panic]
    fn rgb_2() {
        let output: ErrorEnum = syn::parse2(quote! {
            FileSystemError
                #[color = (0, 0)]
                E "错误" {
                    01 FileError (FileError)
                    "{0}",
                }
        })
        .unwrap();
        let output = output.into_token_stream();
        eprintln!("{:#}", output);
    }

    #[test]
    #[should_panic]
    fn path() {
        let output: ErrorEnum = syn::parse2(quote! {
            FileSystemError
                #[nest::ed]
                E "错误" {
                    01 FileError (FileError)
                    "{0}",
                }
        })
        .unwrap();
        let output = output.into_token_stream();
        eprintln!("{:#}", output);
    }
}
