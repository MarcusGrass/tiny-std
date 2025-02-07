#![expect(dead_code)]
#![expect(clippy::struct_field_names)]
use std::panic;
use tiny_cli::{ArgParse, Subcommand};
use tiny_std::unix::cli::ArgParse;
use tiny_std::{unix_lit, UnixStr, UnixString};

#[derive(ArgParse)]
#[cli(help_path = "tiny-cli")]
pub struct SimplestStructWithOpt {
    #[cli(long = "one-req-field")]
    one_req_field: i32,
}

#[test]
fn simplest_struct_happy() {
    let values = [
        UnixStr::from_str_checked("--one-req-field\0"),
        UnixStr::from_str_checked("15\0"),
    ];
    let ss = SimplestStructWithOpt::arg_parse(&mut values.into_iter()).unwrap();
    assert_eq!(15, ss.one_req_field);
}
#[test]
fn simplest_struct_err() {
    let values = [UnixStr::from_str_checked("--one-req-field\0")];
    let ss = SimplestStructWithOpt::arg_parse(&mut values.into_iter());
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
#[cli(help_path = "tiny-cli")]
pub struct SimplestStructWithArg {
    one_req_field: i32,
}

#[test]
fn simplest_struct_arg_happy() {
    let values = [UnixStr::from_str_checked("15\0")];
    let ss = SimplestStructWithArg::arg_parse(&mut values.into_iter()).unwrap();
    assert_eq!(15, ss.one_req_field);
}
#[test]
fn simplest_struct_arg_err() {
    let values = [];
    let ss = SimplestStructWithArg::arg_parse(&mut values.into_iter());
    let Err(e) = ss else {
        panic!("Expected arg parse to fail on simple struct")
    };
    let string_out = e.to_string();
    assert_eq!(
        "\
Usage: tiny-cli [ONE_REQ_FIELD]

Arguments:
  [ONE_REQ_FIELD]

Required argument 'one_req_field' not supplied.",
        string_out
    );
}

#[derive(ArgParse)]
#[cli(help_path = "tiny-cli")]
pub struct SimplestStructWithOptArg {
    one_req_field: Option<i32>,
}

#[test]
fn simplest_struct_opt_arg_happy() {
    let values = [UnixStr::from_str_checked("15\0")];
    let ss = SimplestStructWithOptArg::arg_parse(&mut values.into_iter()).unwrap();
    assert_eq!(Some(15), ss.one_req_field);
    let values = [];
    let ss = SimplestStructWithOptArg::arg_parse(&mut values.into_iter()).unwrap();
    assert!(ss.one_req_field.is_none());
}

#[derive(ArgParse)]
pub struct SimpleStructWithAliases {
    #[cli(short = "s", long = "long")]
    one_req_field: i32,
}

#[test]
fn aliases_work() {
    let values = [
        UnixStr::from_str_checked("-s\0"),
        UnixStr::from_str_checked("15\0"),
    ];
    let ss = SimpleStructWithAliases::arg_parse(&mut values.into_iter()).unwrap();
    assert_eq!(15, ss.one_req_field);
    let values = [
        UnixStr::from_str_checked("--long\0"),
        UnixStr::from_str_checked("15\0"),
    ];
    let ss = SimpleStructWithAliases::arg_parse(&mut values.into_iter()).unwrap();
    assert_eq!(15, ss.one_req_field);
}

#[derive(ArgParse)]
pub struct SimplStructWithBool {
    #[cli(short = "b")]
    my_opt: bool,
}

#[test]
fn bool_parsing_works() {
    let values = [unix_lit!("-b")];
    let b = SimplStructWithBool::arg_parse(&mut values.into_iter()).unwrap();
    assert!(b.my_opt);
    let no_value = [];
    let b = SimplStructWithBool::arg_parse(&mut no_value.into_iter()).unwrap();
    assert!(!b.my_opt);
}

#[derive(ArgParse)]
#[cli(help_path = "tiny-cli")]
pub struct MultiArg {
    pos_one: String,
    pos_two: i64,
}

#[test]
fn parse_multi_arg() {
    let values = [unix_lit!("one"), unix_lit!("2")];
    let multi = MultiArg::arg_parse(&mut values.into_iter()).unwrap();
    assert_eq!("one", multi.pos_one);
    assert_eq!(2, multi.pos_two);
}

#[test]
fn parse_multi_arg_needs_both() {
    let values = [unix_lit!("one")];
    let Err(e) = MultiArg::arg_parse(&mut values.into_iter()) else {
        panic!("Expected parse failure on multiarg missing second arg");
    };
    assert_eq!(
        "\
Usage: tiny-cli [POS_ONE] [POS_TWO]

Arguments:
  [POS_ONE]

  [POS_TWO]

Required argument 'pos_two' not supplied.",
        e.to_string()
    );
}

#[derive(ArgParse)]
#[cli(help_path = "tiny-cli")]
pub struct MultiArgOptLast {
    pub pos_one: String,
    pub(crate) pos_two: i64,
    pos_three: Option<usize>,
}

#[test]
fn parse_multi_arg_opt_no_opt() {
    let values = [unix_lit!("one"), unix_lit!("2")];
    let multi = MultiArgOptLast::arg_parse(&mut values.into_iter()).unwrap();
    assert_eq!("one", multi.pos_one);
    assert_eq!(2, multi.pos_two);
}

#[test]
fn parse_multi_arg_opt_with_opt() {
    let values = [unix_lit!("one"), unix_lit!("2"), unix_lit!("1337")];
    let multi = MultiArgOptLast::arg_parse(&mut values.into_iter()).unwrap();
    assert_eq!("one", multi.pos_one);
    assert_eq!(2, multi.pos_two);
    assert_eq!(Some(1337), multi.pos_three);
}

#[derive(ArgParse)]
pub struct StructWithDifferentPackaging {
    #[cli(long = "req-field")]
    req_field: i32,
    #[cli(short = "o")]
    opt_field: Option<i32>,
    #[cli(long = "rep")]
    rep_field: Vec<i32>,
}

#[test]
fn required_optional_repeated() {
    let values = [
        UnixStr::from_str_checked("--req-field\0"),
        UnixStr::from_str_checked("15\0"),
    ];
    let ss = StructWithDifferentPackaging::arg_parse(&mut values.into_iter()).unwrap();
    assert_eq!(15, ss.req_field);
    assert!(ss.opt_field.is_none());
    assert!(ss.rep_field.is_empty());
    let values = [
        UnixStr::from_str_checked("--req-field\0"),
        UnixStr::from_str_checked("15\0"),
        UnixStr::from_str_checked("-o\0"),
        UnixStr::from_str_checked("30\0"),
    ];
    let ss = StructWithDifferentPackaging::arg_parse(&mut values.into_iter()).unwrap();
    assert_eq!(15, ss.req_field);
    assert_eq!(Some(30), ss.opt_field);
    assert!(ss.rep_field.is_empty());
    let values = [
        UnixStr::from_str_checked("--req-field\0"),
        UnixStr::from_str_checked("15\0"),
        UnixStr::from_str_checked("--rep\0"),
        UnixStr::from_str_checked("30\0"),
        UnixStr::from_str_checked("--rep\0"),
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
    #[cli(long = "field1")]
    field1: i32,
}

#[test]
fn simple_subcommand() {
    let values = [UnixStr::from_str_checked("cmd-one\0")];
    let ss = WithEnumSubcommand::arg_parse(&mut values.into_iter()).unwrap();
    assert_eq!(TestSubcommand::CmdOne, ss.sc);
    let values = [
        UnixStr::from_str_checked("cmd-two\0"),
        UnixStr::from_str_checked("--field1\0"),
        UnixStr::from_str_checked("7\0"),
    ];
    let ss = WithEnumSubcommand::arg_parse(&mut values.into_iter()).unwrap();
    assert_eq!(TestSubcommand::CmdTwo(TestSubTwo { field1: 7 }), ss.sc);
    let values = [UnixStr::from_str_checked("cmd-three\0")];
    let ss = WithEnumSubcommand::arg_parse(&mut values.into_iter()).unwrap();
    assert_eq!(TestSubcommand::CmdThree, ss.sc);
}

#[test]
fn simple_subcommand_help_print() {
    let values = [UnixStr::from_str_checked("cmd-one\0")];
    let ss = WithEnumSubcommand::arg_parse(&mut values.into_iter()).unwrap();
    assert_eq!(TestSubcommand::CmdOne, ss.sc);
    let values = [
        UnixStr::from_str_checked("cmd-two\0"),
        UnixStr::from_str_checked("-h\0"),
    ];
    let Err(e1) = WithEnumSubcommand::arg_parse(&mut values.into_iter()) else {
        panic!("Expected err on help");
    };
    let values = [
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
    #[cli(long = "my-field")]
    my_field: i32,
    #[cli(short = "s", long = "my-field-has-short")]
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
    Arg(SubCommandWithArg),
}

#[derive(ArgParse, Debug)]
#[cli(help_path = "tiny-cli, run")]
struct RunArgs {
    #[cli(short = "a")]
    arg_has_opt_str: Option<&'static str>,
    #[cli(short = "b")]
    arg_has_opt_unix_str: Option<&'static UnixStr>,
    #[cli(short = "c")]
    arg_has_rep_str: Vec<&'static str>,
    #[cli(short = "d")]
    arg_has_rep_unix_str: Vec<&'static UnixStr>,
}

#[derive(ArgParse, Debug)]
#[cli(help_path = "tiny-cli, list")]
struct ListArgs {
    #[cli(long = "arg-has-opt-string")]
    arg_has_opt_string: Option<String>,
    #[cli(long = "rep")]
    arg_has_rep_unix_string: Vec<UnixString>,
    /// This is required
    #[cli(long = "req")]
    arg_has_required_string: String,
}
#[derive(ArgParse, Debug)]
#[cli(help_path = "tiny-cli, other")]
struct OtherArgs {
    /// This field is required
    #[cli(long = "required-field")]
    required_field: i32,
    /// Also has optional subcommand
    #[cli(subcommand)]
    subc_opt: Option<OtherSubcommand>,
}

#[derive(ArgParse, Debug)]
#[cli(help_path = "tiny-cli, arg")]
struct SubCommandWithArg {
    /// Required positional argument
    subc_arg: String,
    /// Optional option
    #[cli(short = "o")]
    opt: Option<i32>,
}

#[derive(Subcommand, Debug)]
enum OtherSubcommand {
    OnlyOneOption(OptStruct),
}

#[derive(ArgParse, Debug)]
#[cli(help_path = "tiny-cli, other, only-one-option")]
pub struct OptStruct {
    /// This isn't required
    #[cli(long = "only-one-opt-owned-field")]
    only_one_opt_owned_field: Option<i128>,
}

#[test]
fn complex_run_no_opts() {
    let values = [
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
    let values = [
        UnixStr::from_str_checked("--my-field\0"),
        UnixStr::from_str_checked("1\0"),
        UnixStr::from_str_checked("-s\0"),
        UnixStr::from_str_checked("string-value\0"),
        UnixStr::from_str_checked("--long-field\0"),
        UnixStr::from_str_checked("unixstr\0"),
        UnixStr::from_str_checked("-c\0"),
        UnixStr::from_str_checked("remapped string field\0"),
        UnixStr::from_str_checked("run\0"),
        UnixStr::from_str_checked("-a\0"),
        UnixStr::from_str_checked("myval\0"),
        UnixStr::from_str_checked("-b\0"),
        UnixStr::from_str_checked("myunixval\0"),
        UnixStr::from_str_checked("-c\0"),
        UnixStr::from_str_checked("myrepval\0"),
        UnixStr::from_str_checked("-c\0"),
        UnixStr::from_str_checked("myrepval2\0"),
        UnixStr::from_str_checked("-d\0"),
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
    let values = [
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
fn complex_subcommand_with_list_args() {
    let values = [
        UnixStr::from_str_checked("--my-field\0"),
        UnixStr::from_str_checked("1\0"),
        UnixStr::from_str_checked("-s\0"),
        UnixStr::from_str_checked("string-value\0"),
        UnixStr::from_str_checked("--long-field\0"),
        UnixStr::from_str_checked("unixstr\0"),
        UnixStr::from_str_checked("-c\0"),
        UnixStr::from_str_checked("remapped string field\0"),
        UnixStr::from_str_checked("list\0"),
        UnixStr::from_str_checked("--rep\0"),
        UnixStr::from_str_checked("one\0"),
        UnixStr::from_str_checked("--rep\0"),
        UnixStr::from_str_checked("two\0"),
        UnixStr::from_str_checked("--req\0"),
        UnixStr::from_str_checked("req\0"),
    ];
    let res = ComplexUsesAllFeatures::arg_parse(&mut values.into_iter()).unwrap();
    assert_eq!(1, res.my_field);
    assert_eq!("string-value", res.my_field_has_short);
    assert_eq!(
        UnixStr::from_str_checked("unixstr\0"),
        res.my_field_has_long_remap
    );
    assert_eq!("remapped string field", res.my_field_has_double_remap);
    let ComplexSubcommand::List(o) = res.subcommand else {
        panic!("Expected other subcommand to have been invoked");
    };
    assert!(o.arg_has_opt_string.is_none());
    assert_eq!(unix_lit!("one"), o.arg_has_rep_unix_string[0].as_ref());
    assert_eq!(unix_lit!("two"), o.arg_has_rep_unix_string[1].as_ref());
    assert_eq!("req", &o.arg_has_required_string);
}

#[test]
fn complex_subcommand_full_nested() {
    let values = [
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
    let values = [UnixStr::from_str_checked("-h\0")];
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
  arg

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
    let values = [
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
  -a

  -b

  -c

  -d

",
        output
    );
}

#[test]
fn complex_list_help() {
    let values = [
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

      --rep

      --req
        This is required

",
        output
    );
}

#[test]
fn complex_other_help() {
    let values = [
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
    let values = [
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

#[test]
fn complex_accepts_arg() {
    let values = [
        UnixStr::from_str_checked("--my-field\0"),
        UnixStr::from_str_checked("1\0"),
        UnixStr::from_str_checked("-s\0"),
        UnixStr::from_str_checked("string-value\0"),
        UnixStr::from_str_checked("--long-field\0"),
        UnixStr::from_str_checked("unixstr\0"),
        UnixStr::from_str_checked("-c\0"),
        UnixStr::from_str_checked("remapped string field\0"),
        UnixStr::from_str_checked("arg\0"),
        UnixStr::from_str_checked("mystr\0"),
    ];
    let s = ComplexUsesAllFeatures::arg_parse(&mut values.into_iter()).unwrap();
    match s.subcommand {
        ComplexSubcommand::Arg(a) => {
            assert_eq!("mystr", a.subc_arg);
        }
        unk => panic!("Unexpected subcommand parsed: {unk:?}"),
    }
}

#[test]
fn complex_accepts_arg_help() {
    let values = [
        UnixStr::from_str_checked("--my-field\0"),
        UnixStr::from_str_checked("1\0"),
        UnixStr::from_str_checked("-s\0"),
        UnixStr::from_str_checked("string-value\0"),
        UnixStr::from_str_checked("--long-field\0"),
        UnixStr::from_str_checked("unixstr\0"),
        UnixStr::from_str_checked("-c\0"),
        UnixStr::from_str_checked("remapped string field\0"),
        UnixStr::from_str_checked("arg\0"),
        UnixStr::from_str_checked("-h\0"),
    ];
    let Err(e) = ComplexUsesAllFeatures::arg_parse(&mut values.into_iter()) else {
        panic!("Expect err on complex run help");
    };
    assert_eq!(0, e.cause.len());
    let output = e.to_string();
    assert_eq!(
        "\
Usage: tiny-cli arg [OPTIONS] [SUBC_ARG]

Arguments:
  [SUBC_ARG]
        Required positional argument

Options:
  -o
        Optional option

",
        output
    );
}
