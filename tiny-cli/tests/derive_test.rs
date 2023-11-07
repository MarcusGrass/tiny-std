#![allow(unused)]

use std::any::{Any, TypeId};
use std::panic;
use std::sync::{Arc, Mutex};
use tiny_cli::{ArgParse, Subcommand};
use tiny_std::unix::cli::ArgParse;
use tiny_std::{UnixStr, UnixString};

#[derive(ArgParse)]
#[cli(help_path = "tiny-cli")]
pub struct SimplestStruct {
    one_req_field: i32,
}

#[test]
fn simplest_struct_happy() {
    let mut values = [
        UnixStr::from_str_checked("--one-req-field\0"),
        UnixStr::from_str_checked("15\0"),
    ];
    let ss = SimplestStruct::arg_parse(&mut values.into_iter()).unwrap();
    assert_eq!(15, ss.one_req_field);
}
#[test]
fn simplest_struct_err() {
    let mut values = [UnixStr::from_str_checked("--one-req-field\0")];
    let ss = SimplestStruct::arg_parse(&mut values.into_iter());
    let Err(e) = ss else {
        panic!("Expected arg parse to fail on simple struct")
    };
    let string_out = e.to_string();
    assert_eq!(
        "\
Usage: tiny-cli [OPTIONS]

Options:
      --one-req-field

Expected argument following '--one-req-field'.",
        string_out
    );
}

#[derive(ArgParse)]
pub struct SimpleStructWithAliases {
    #[cli(short = "s", long = "long")]
    one_req_field: i32,
}

#[test]
fn aliases_work() {
    let mut values = [
        UnixStr::from_str_checked("-s\0"),
        UnixStr::from_str_checked("15\0"),
    ];
    let ss = SimpleStructWithAliases::arg_parse(&mut values.into_iter()).unwrap();
    assert_eq!(15, ss.one_req_field);
    let mut values = [
        UnixStr::from_str_checked("--long\0"),
        UnixStr::from_str_checked("15\0"),
    ];
    let ss = SimpleStructWithAliases::arg_parse(&mut values.into_iter()).unwrap();
    assert_eq!(15, ss.one_req_field);
}

#[derive(ArgParse)]
pub struct StructWithDifferentPackaging {
    req_field: i32,
    opt_field: Option<i32>,
    rep_field: Vec<i32>,
}

#[test]
fn required_optional_repeated() {
    let mut values = [
        UnixStr::from_str_checked("--req-field\0"),
        UnixStr::from_str_checked("15\0"),
    ];
    let ss = StructWithDifferentPackaging::arg_parse(&mut values.into_iter()).unwrap();
    assert_eq!(15, ss.req_field);
    assert!(ss.opt_field.is_none());
    assert!(ss.rep_field.is_empty());
    let mut values = [
        UnixStr::from_str_checked("--req-field\0"),
        UnixStr::from_str_checked("15\0"),
        UnixStr::from_str_checked("--opt-field\0"),
        UnixStr::from_str_checked("30\0"),
    ];
    let ss = StructWithDifferentPackaging::arg_parse(&mut values.into_iter()).unwrap();
    assert_eq!(15, ss.req_field);
    assert_eq!(Some(30), ss.opt_field);
    assert!(ss.rep_field.is_empty());
    let mut values = [
        UnixStr::from_str_checked("--req-field\0"),
        UnixStr::from_str_checked("15\0"),
        UnixStr::from_str_checked("--rep-field\0"),
        UnixStr::from_str_checked("30\0"),
        UnixStr::from_str_checked("--rep-field\0"),
        UnixStr::from_str_checked("45\0"),
    ];
    let ss = StructWithDifferentPackaging::arg_parse(&mut values.into_iter()).unwrap();
    assert_eq!(15, ss.req_field);
    assert!(ss.opt_field.is_none());
    assert_eq!(vec![30, 45], ss.rep_field);
}

/// Doc comment on struct
#[derive(ArgParse)]
#[cli(help_path = "tiny-cli")]
pub struct WithEnumSubcommand {
    /// Doc comment on field
    #[cli(subcommand)]
    sc: TestSubcommand,
}

/// Doc comment on cmd
#[derive(Subcommand, Debug, Eq, PartialEq)]
pub enum TestSubcommand {
    /// Doc comment on tag
    CmdOne,
    CmdTwo(TestSubTwo),
    CmdThree,
}

#[derive(ArgParse, Debug, Eq, PartialEq)]
#[cli(help_path = "tiny-cli, cmd-two")]
pub struct TestSubTwo {
    field1: i32,
}

#[test]
fn simple_subcommand() {
    let mut values = [UnixStr::from_str_checked("cmd-one\0")];
    let ss = WithEnumSubcommand::arg_parse(&mut values.into_iter()).unwrap();
    assert_eq!(TestSubcommand::CmdOne, ss.sc);
    let mut values = [
        UnixStr::from_str_checked("cmd-two\0"),
        UnixStr::from_str_checked("--field1\0"),
        UnixStr::from_str_checked("7\0"),
    ];
    let ss = WithEnumSubcommand::arg_parse(&mut values.into_iter()).unwrap();
    assert_eq!(TestSubcommand::CmdTwo(TestSubTwo { field1: 7 }), ss.sc);
    let mut values = [UnixStr::from_str_checked("cmd-three\0")];
    let ss = WithEnumSubcommand::arg_parse(&mut values.into_iter()).unwrap();
    assert_eq!(TestSubcommand::CmdThree, ss.sc);
}

#[test]
fn simple_subcommand_help_print() {
    let mut values = [UnixStr::from_str_checked("cmd-one\0")];
    let ss = WithEnumSubcommand::arg_parse(&mut values.into_iter()).unwrap();
    assert_eq!(TestSubcommand::CmdOne, ss.sc);
    let mut values = [
        UnixStr::from_str_checked("cmd-two\0"),
        UnixStr::from_str_checked("-h\0"),
    ];
    let Err(e1) = WithEnumSubcommand::arg_parse(&mut values.into_iter()) else {
        panic!("Expected err on help");
    };
    let mut values = [
        UnixStr::from_str_checked("cmd-two\0"),
        UnixStr::from_str_checked("--help\0"),
    ];
    let Err(e2) = WithEnumSubcommand::arg_parse(&mut values.into_iter()) else {
        panic!("Expected err on help");
    };
    assert_eq!(e1.to_string(), e2.to_string());
    assert_eq!(0, e1.cause.len());
    assert_eq!(
        "\
Usage: tiny-cli cmd-two [OPTIONS]

Options:
      --field1

",
        e1.to_string()
    );
}

#[derive(ArgParse, Debug, Eq, PartialEq)]
struct NestedCommands {
    #[cli(subcommand)]
    command: NestedCommandSubcommand,
}

#[derive(Subcommand, Debug, Eq, PartialEq)]
enum NestedCommandSubcommand {
    MyTag(Nest),
}

#[derive(ArgParse, Debug, Eq, PartialEq)]
struct Nest {
    #[cli(subcommand)]
    inner: Option<NestedInner>,
}

#[derive(Subcommand, Debug, Eq, PartialEq)]
enum NestedInner {
    A,
    B,
}

#[test]
fn nested_optional_subcommand() {
    let values = [UnixStr::from_str_checked("my-tag\0")];
    assert_eq!(
        NestedCommands {
            command: NestedCommandSubcommand::MyTag(Nest { inner: None })
        },
        NestedCommands::arg_parse(&mut values.into_iter()).unwrap()
    );
    let values = [
        UnixStr::from_str_checked("my-tag\0"),
        UnixStr::from_str_checked("a\0"),
    ];
    assert_eq!(
        NestedCommands {
            command: NestedCommandSubcommand::MyTag(Nest {
                inner: Some(NestedInner::A)
            })
        },
        NestedCommands::arg_parse(&mut values.into_iter()).unwrap()
    );
    let values = [
        UnixStr::from_str_checked("my-tag\0"),
        UnixStr::from_str_checked("b\0"),
    ];
    assert_eq!(
        NestedCommands {
            command: NestedCommandSubcommand::MyTag(Nest {
                inner: Some(NestedInner::B)
            })
        },
        NestedCommands::arg_parse(&mut values.into_iter()).unwrap()
    );
}

/// My complex cli tool
#[derive(ArgParse, Debug)]
#[cli(help_path = "tiny-cli")]
struct ComplexUsesAllFeatures {
    /// Naked field, but has comment
    my_field: i32,
    #[cli(short = "s")]
    my_field_has_short: String,
    #[cli(long = "long-field")]
    my_field_has_long_remap: &'static UnixStr,
    #[cli(short = "c", long = "long-double")]
    my_field_has_double_remap: &'static str,
    #[cli(subcommand)]
    subcommand: ComplexSubcommand,
}

#[derive(Subcommand, Debug)]
enum ComplexSubcommand {
    /// For running
    Run(RunArgs),
    // We won't be looking at this
    List(ListArgs),
    /// No comment
    Other(OtherArgs),
}

#[derive(ArgParse, Debug)]
#[cli(help_path = "tiny-cli, run")]
struct RunArgs {
    arg_has_opt_str: Option<&'static str>,
    arg_has_opt_unix_str: Option<&'static UnixStr>,
    arg_has_rep_str: Vec<&'static str>,
    arg_has_rep_unix_str: Vec<&'static UnixStr>,
}

#[derive(ArgParse, Debug)]
#[cli(help_path = "tiny-cli, list")]
struct ListArgs {
    arg_has_opt_string: Option<String>,
    arg_has_rep_unix_string: Vec<UnixString>,
    /// This is required
    arg_has_required_string: String,
}
#[derive(ArgParse, Debug)]
#[cli(help_path = "tiny-cli, other")]
struct OtherArgs {
    /// This field is required
    required_field: i32,
    /// Also has optional subcommand
    #[cli(subcommand)]
    subc_opt: Option<OtherSubcommand>,
}

#[derive(Subcommand, Debug)]
enum OtherSubcommand {
    OnlyOneOption(OptStruct),
}

#[derive(ArgParse, Debug)]
#[cli(help_path = "tiny-cli, other, only-one-option")]
pub struct OptStruct {
    /// This isn't required
    only_one_opt_owned_field: Option<i128>,
}

#[test]
fn complex_run_no_opts() {
    let mut values = [
        UnixStr::from_str_checked("--my-field\0"),
        UnixStr::from_str_checked("1\0"),
        UnixStr::from_str_checked("-s\0"),
        UnixStr::from_str_checked("string-value\0"),
        UnixStr::from_str_checked("--long-field\0"),
        UnixStr::from_str_checked("unixstr\0"),
        UnixStr::from_str_checked("-c\0"),
        UnixStr::from_str_checked("remapped string field\0"),
        UnixStr::from_str_checked("run\0"),
    ];
    let res = ComplexUsesAllFeatures::arg_parse(&mut values.into_iter()).unwrap();
    assert_eq!(1, res.my_field);
    assert_eq!("string-value", res.my_field_has_short);
    assert_eq!(
        UnixStr::from_str_checked("unixstr\0"),
        res.my_field_has_long_remap
    );
    assert_eq!("remapped string field", res.my_field_has_double_remap);
    let ComplexSubcommand::Run(r) = res.subcommand else {
        panic!("Expected run subcommand to have been invoked");
    };
    assert!(r.arg_has_opt_str.is_none());
    assert!(r.arg_has_opt_unix_str.is_none());
    assert!(r.arg_has_rep_str.is_empty());
    assert!(r.arg_has_rep_unix_str.is_empty());
}

#[test]
fn complex_run_full_opts() {
    let mut values = [
        UnixStr::from_str_checked("--my-field\0"),
        UnixStr::from_str_checked("1\0"),
        UnixStr::from_str_checked("-s\0"),
        UnixStr::from_str_checked("string-value\0"),
        UnixStr::from_str_checked("--long-field\0"),
        UnixStr::from_str_checked("unixstr\0"),
        UnixStr::from_str_checked("-c\0"),
        UnixStr::from_str_checked("remapped string field\0"),
        UnixStr::from_str_checked("run\0"),
        UnixStr::from_str_checked("--arg-has-opt-str\0"),
        UnixStr::from_str_checked("myval\0"),
        UnixStr::from_str_checked("--arg-has-opt-unix-str\0"),
        UnixStr::from_str_checked("myunixval\0"),
        UnixStr::from_str_checked("--arg-has-rep-str\0"),
        UnixStr::from_str_checked("myrepval\0"),
        UnixStr::from_str_checked("--arg-has-rep-str\0"),
        UnixStr::from_str_checked("myrepval2\0"),
        UnixStr::from_str_checked("--arg-has-rep-unix-str\0"),
        UnixStr::from_str_checked("myrepunixval\0"),
    ];
    let res = ComplexUsesAllFeatures::arg_parse(&mut values.into_iter()).unwrap();
    assert_eq!(1, res.my_field);
    assert_eq!("string-value", res.my_field_has_short);
    assert_eq!(
        UnixStr::from_str_checked("unixstr\0"),
        res.my_field_has_long_remap
    );
    assert_eq!("remapped string field", res.my_field_has_double_remap);
    let ComplexSubcommand::Run(r) = res.subcommand else {
        panic!("Expected run subcommand to have been invoked");
    };
    assert_eq!(Some("myval"), r.arg_has_opt_str);
    assert_eq!(
        Some(UnixStr::from_str_checked("myunixval\0")),
        r.arg_has_opt_unix_str
    );
    assert_eq!(vec!["myrepval", "myrepval2"], r.arg_has_rep_str);
    assert_eq!(
        vec![UnixStr::from_str_checked("myrepunixval\0")],
        r.arg_has_rep_unix_str
    );
}

#[test]
fn complex_subcommand_skip_nested() {
    let mut values = [
        UnixStr::from_str_checked("--my-field\0"),
        UnixStr::from_str_checked("1\0"),
        UnixStr::from_str_checked("-s\0"),
        UnixStr::from_str_checked("string-value\0"),
        UnixStr::from_str_checked("--long-field\0"),
        UnixStr::from_str_checked("unixstr\0"),
        UnixStr::from_str_checked("-c\0"),
        UnixStr::from_str_checked("remapped string field\0"),
        UnixStr::from_str_checked("other\0"),
        UnixStr::from_str_checked("--required-field\0"),
        UnixStr::from_str_checked("2\0"),
    ];
    let res = ComplexUsesAllFeatures::arg_parse(&mut values.into_iter()).unwrap();
    assert_eq!(1, res.my_field);
    assert_eq!("string-value", res.my_field_has_short);
    assert_eq!(
        UnixStr::from_str_checked("unixstr\0"),
        res.my_field_has_long_remap
    );
    assert_eq!("remapped string field", res.my_field_has_double_remap);
    let ComplexSubcommand::Other(o) = res.subcommand else {
        panic!("Expected other subcommand to have been invoked");
    };
    assert_eq!(2, o.required_field);
}

#[test]
fn complex_subcommand_full_nested() {
    let mut values = [
        UnixStr::from_str_checked("--my-field\0"),
        UnixStr::from_str_checked("1\0"),
        UnixStr::from_str_checked("-s\0"),
        UnixStr::from_str_checked("string-value\0"),
        UnixStr::from_str_checked("--long-field\0"),
        UnixStr::from_str_checked("unixstr\0"),
        UnixStr::from_str_checked("-c\0"),
        UnixStr::from_str_checked("remapped string field\0"),
        UnixStr::from_str_checked("other\0"),
        UnixStr::from_str_checked("--required-field\0"),
        UnixStr::from_str_checked("2\0"),
        UnixStr::from_str_checked("only-one-option\0"),
        UnixStr::from_str_checked("--only-one-opt-owned-field\0"),
        UnixStr::from_str_checked("57287493014712903472878465\0"),
    ];
    let res = ComplexUsesAllFeatures::arg_parse(&mut values.into_iter()).unwrap();
    assert_eq!(1, res.my_field);
    assert_eq!("string-value", res.my_field_has_short);
    assert_eq!(
        UnixStr::from_str_checked("unixstr\0"),
        res.my_field_has_long_remap
    );
    assert_eq!("remapped string field", res.my_field_has_double_remap);
    let ComplexSubcommand::Other(o) = res.subcommand else {
        panic!("Expected other subcommand to have been invoked");
    };
    assert_eq!(2, o.required_field);
    let Some(OtherSubcommand::OnlyOneOption(opt)) = o.subc_opt else {
        panic!("Expected nested subcommand to have been invoked");
    };
    assert_eq!(
        57287493014712903472878465,
        opt.only_one_opt_owned_field.unwrap()
    );
}

#[test]
fn complex_top_level_help() {
    let mut values = [UnixStr::from_str_checked("-h\0")];
    let Err(e) = ComplexUsesAllFeatures::arg_parse(&mut values.into_iter()) else {
        panic!("Expect err on complex help");
    };
    assert_eq!(0, e.cause.len());
    let output = e.to_string();
    assert_eq!(
        "\
My complex cli tool

Usage: tiny-cli [OPTIONS] [COMMAND]

Commands:
  run   - For running
  list
  other - No comment

Options:
      --my-field
        Naked field, but has comment

  -s, --my-field-has-short

      --long-field

  -c, --long-double

",
        output
    );
}

#[test]
fn complex_run_help() {
    let mut values = [
        UnixStr::from_str_checked("run\0"),
        UnixStr::from_str_checked("-h\0"),
    ];
    let Err(e) = ComplexUsesAllFeatures::arg_parse(&mut values.into_iter()) else {
        panic!("Expect err on complex run help");
    };
    assert_eq!(0, e.cause.len());
    let output = e.to_string();
    assert_eq!(
        "\
Usage: tiny-cli run [OPTIONS]

Options:
      --arg-has-opt-str

      --arg-has-opt-unix-str

      --arg-has-rep-str

      --arg-has-rep-unix-str

",
        output
    );
}

#[test]
fn complex_list_help() {
    let mut values = [
        UnixStr::from_str_checked("list\0"),
        UnixStr::from_str_checked("--help\0"),
    ];
    let Err(e) = ComplexUsesAllFeatures::arg_parse(&mut values.into_iter()) else {
        panic!("Expect err on complex run help");
    };
    assert_eq!(0, e.cause.len());
    let output = e.to_string();
    assert_eq!(
        "\
Usage: tiny-cli list [OPTIONS]

Options:
      --arg-has-opt-string

      --arg-has-rep-unix-string

      --arg-has-required-string
        This is required

",
        output
    );
}

#[test]
fn complex_other_help() {
    let mut values = [
        UnixStr::from_str_checked("other\0"),
        UnixStr::from_str_checked("--help\0"),
    ];
    let Err(e) = ComplexUsesAllFeatures::arg_parse(&mut values.into_iter()) else {
        panic!("Expect err on complex run help");
    };
    assert_eq!(0, e.cause.len());
    let output = e.to_string();
    assert_eq!(
        "\
Usage: tiny-cli other [OPTIONS] [COMMAND]

Commands:
  only-one-option

Options:
      --required-field
        This field is required

",
        output
    );
}

#[test]
fn complex_other_opt_help() {
    let mut values = [
        UnixStr::from_str_checked("other\0"),
        UnixStr::from_str_checked("only-one-option\0"),
        UnixStr::from_str_checked("-h\0"),
    ];
    let Err(e) = ComplexUsesAllFeatures::arg_parse(&mut values.into_iter()) else {
        panic!("Expect err on complex run help");
    };
    assert_eq!(0, e.cause.len());
    let output = e.to_string();
    assert_eq!(
        "\
Usage: tiny-cli other only-one-option [OPTIONS]

Options:
      --only-one-opt-owned-field
        This isn't required

",
        output
    );
}
