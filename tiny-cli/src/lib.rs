#![warn(clippy::pedantic)]
mod derive_struct;
mod subcommand;

extern crate proc_macro;

use proc_macro::{Group, Ident, Literal, TokenStream, TokenTree};
use std::fmt::Display;

#[proc_macro_derive(ArgParse, attributes(cli))]
pub fn derive_arg_parse(struct_candidate: TokenStream) -> TokenStream {
    derive_struct::do_derive(struct_candidate)
}

#[proc_macro_derive(Subcommand, attributes(cli))]
pub fn derive_sc_parse(struct_candidate: TokenStream) -> TokenStream {
    subcommand::do_derive(struct_candidate)
}

fn pop_expect_punct<I: Iterator<Item = TokenTree>, D: Display>(
    stream: &mut I,
    expect: char,
    err_msg: D,
) {
    let punct = stream
        .next()
        .unwrap_or_else(|| panic!("[ArgParse derive] {err_msg}"));
    if let TokenTree::Punct(p) = punct {
        assert_eq!(p.as_char(), expect, "{err_msg}");
    } else {
        panic!(
            "[ArgParse derive] Expected punctation with {expect}, found: {punct:?}, ctx: {err_msg}"
        );
    }
}

fn pop_expect_ident<I: Iterator<Item = TokenTree>, D: Display>(
    stream: &mut I,
    expect: &str,
    err_msg: D,
) {
    let ident = pop_ident(stream, &err_msg);
    assert_eq!(
        expect,
        ident.to_string().trim(),
        "[ArgParse derive] Ident {ident} didn't match expected {expect}, ctx: {err_msg}"
    );
}

fn pop_ident<I: Iterator<Item = TokenTree>, D: Display>(stream: &mut I, err_msg: D) -> Ident {
    let ident = stream
        .next()
        .unwrap_or_else(|| panic!("[ArgParse derive] {err_msg}"));
    if let TokenTree::Ident(ident) = ident {
        ident
    } else {
        panic!("[ArgParse derive] Expected ident, found {ident:?}, ctx: {err_msg}");
    }
}

fn pop_lit<I: Iterator<Item = TokenTree>, D: Display>(stream: &mut I, err_msg: D) -> Literal {
    let lit = stream
        .next()
        .unwrap_or_else(|| panic!("[ArgParse derive] {err_msg}"));
    if let TokenTree::Literal(l) = lit {
        l
    } else {
        panic!("[ArgParse derive] Expected literal found: {lit:?}, ctx: {err_msg}");
    }
}

fn pop_group<I: Iterator<Item = TokenTree>, D: Display>(stream: &mut I, err_msg: D) -> Group {
    let group = stream
        .next()
        .unwrap_or_else(|| panic!("[ArgParse derive] {err_msg}"));
    if let TokenTree::Group(g) = group {
        g
    } else {
        panic!("[ArgParse derive] Expected group, found: {group:?}, ctx: {err_msg}");
    }
}

pub(crate) fn try_extract_doc_comment(g: &Group) -> Option<String> {
    let mut stream = g.stream().into_iter();
    if let Some(TokenTree::Ident(id)) = stream.next() {
        if id.to_string().trim() == "doc" {
            pop_expect_punct(&mut stream, '=', "Expected a '=' after #[doc");
            let ident = pop_lit(&mut stream, "Expected #[doc = <literal>...");
            let id_str = ident.to_string();
            let id_trimmed = id_str.trim().trim_matches('"').trim().to_string();
            return Some(id_trimmed);
        }
    }
    None
}

pub(crate) fn pascal_to_snake(prev: &str) -> String {
    let mut new = String::new();
    let mut chars = prev.chars();
    if let Some(next) = chars.next() {
        for lc in next.to_lowercase() {
            new.push(lc);
        }
    } else {
        return new;
    }
    for char in chars {
        if char.is_uppercase() {
            new.push('-');
        }
        for lc in char.to_lowercase() {
            new.push(lc);
        }
    }
    new
}

#[inline]
pub(crate) fn snake_to_scream(prev: &str) -> String {
    prev.replace('-', "_").to_uppercase()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn switch_case() {
        let orig = "MyStructDecl";
        assert_eq!("my-struct-decl", pascal_to_snake(orig));
        let orig = "M";
        assert_eq!("m", pascal_to_snake(orig));
        assert_eq!("", pascal_to_snake(""));
        assert_eq!("m-m", pascal_to_snake("MM"));
    }
}
