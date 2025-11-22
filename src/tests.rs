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
            #[fg = (0xaf, 0, 0)]
            #[bg = (0xa8, 0xa8, 0xa8)]
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
fn deep() {
    let output: ErrorEnum = syn::parse2(quote! {
        FileSystemError
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
fn check_config() {
    let output: ErrorEnum = syn::parse2(quote! {
        FileSystemError
            Error "错误" {
                #[nested]
                01 FileError (FileError)
                "{0}",
            }
    })
    .unwrap();
    for (config, number, _variant) in output.get_variants() {
        assert_eq!(config.category, "Error");
        #[cfg(feature = "colored")]
        assert_eq!(
            format!("{config:?}"),
            r#"Config { category: "Error", nested: true, style_prefix: Style { fg(Red) }, style_message: Style {} }"#,
        );
        #[cfg(not(feature = "colored"))]
        assert_eq!(
            format!("{config:?}"),
            "Config { category: 'E', nested: true }",
        );
        assert_eq!(number, "01");
    }
}

#[test]
#[cfg(feature = "colored")]
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
#[cfg(feature = "colored")]
#[should_panic]
fn rgb_wrong() {
    let output: ErrorEnum = syn::parse2(quote! {
        FileSystemError
            #[color = (4usize, -3, 2*5)]
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
#[cfg(feature = "colored")]
#[should_panic]
fn color_bool() {
    let output: ErrorEnum = syn::parse2(quote! {
        FileSystemError
            #[color = true]
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
fn attribute_list() {
    let output: ErrorEnum = syn::parse2(quote! {
        FileSystemError
            #[nested(true)]
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

#[test]
#[should_panic]
fn unsupported_value() {
    let output: ErrorEnum = syn::parse2(quote! {
        FileSystemError
            #[fg = true || false]
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
#[cfg(feature = "colored")]
#[should_panic]
fn wrong_color() {
    let output: ErrorEnum = syn::parse2(quote! {
        FileSystemError
            #[color = "blite"]
            E "错误" {
                01 FileError (FileError)
                "{0}",
            }
    })
    .unwrap();
    let output = output.into_token_stream();
    eprintln!("{:#}", output);
}
