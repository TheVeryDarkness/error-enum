#![expect(clippy::unwrap_used, clippy::panic)]

use crate::ErrorEnum;
use prettydiff::diff_lines;
use proc_macro2::TokenStream;
use quote::ToTokens;
use std::{
    io::Write,
    process::{Command, Stdio},
};
use syn::DeriveInput;

#[track_caller]
fn assert_eq_source(actual: &str, expected: &str) {
    if expected != actual {
        let diff = diff_lines(expected, actual);
        panic!(
            "---------- Source DIFF ----------\n{}\n--------- ACTUAL CODE ----------\n{}",
            diff, actual
        );
    }
}

fn format_str(source: &str) -> String {
    let path = if cfg!(target_os = "windows") {
        "rustfmt.exe"
    } else {
        "rustfmt"
    };
    let mut rustfmt = Command::new(path)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    let stdin = rustfmt.stdin.as_mut().unwrap();
    stdin.write_all(source.as_bytes()).unwrap();
    let output = rustfmt.wait_with_output().unwrap();
    String::from_utf8(output.stdout).unwrap()
}

#[track_caller]
fn test_error_type(tokens: TokenStream, expected: TokenStream) {
    let input: ErrorEnum = syn::parse2(tokens).unwrap();
    let output = input.into_token_stream();
    let output = format_str(&output.to_string());
    let expected = format_str(&expected.to_string());
    assert_eq_source(&output, &expected);
}

#[track_caller]
fn test_error_type_derive(tokens: TokenStream, expected: TokenStream) {
    let input: DeriveInput = syn::parse2(tokens).unwrap();
    let input = ErrorEnum::try_from(input).unwrap();
    let output = input.into_token_stream();
    let output = format_str(&output.to_string());
    let expected = format_str(&expected.to_string());
    assert_eq_source(&output, &expected);
}

mod basic;
mod derive;
mod nested;
