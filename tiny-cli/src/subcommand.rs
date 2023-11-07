use crate::{pascal_to_snake, pop_ident, snake_to_scream, try_extract_doc_comment};
use proc_macro::{Delimiter, Group, TokenStream, TokenTree};
use std::fmt::Write;
use std::str::FromStr;

#[inline]
pub(crate) fn do_derive(struct_candidate: TokenStream) -> TokenStream {
    let mut state = EnumTreeParse {
        doc_comments: vec![],
        state: EnumTreeParseState::None,
    };
    for tree in struct_candidate {
        check_tree(&mut state, tree);
        match state.state.clone() {
            EnumTreeParseState::None
            | EnumTreeParseState::SeenEnum
            | EnumTreeParseState::SeenName(_) => {}
            EnumTreeParseState::FoundGroup(name, group) => {
                let out = parse_inner_enum(&name, &group);
                return TokenStream::from_str(&out).expect(
                    "[ArgParse derive] Failed to produce a valid token stream (this is a bug)",
                );
            }
        }
    }
    TokenStream::new()
}

struct EnumTreeParse {
    doc_comments: Vec<String>,
    state: EnumTreeParseState,
}

#[derive(Debug, Clone)]
enum EnumTreeParseState {
    None,
    SeenEnum,
    SeenName(String),
    FoundGroup(String, Group),
}

fn check_tree(state: &mut EnumTreeParse, tree: TokenTree) {
    match tree {
        TokenTree::Group(g) => {
            if let EnumTreeParseState::SeenName(name) = state.state.clone() {
                state.state = EnumTreeParseState::FoundGroup(name, g);
            } else if let Some(cmnt) = try_extract_doc_comment(&g) {
                state.doc_comments.push(cmnt);
            }
        }
        TokenTree::Ident(id) => match state.state.clone() {
            EnumTreeParseState::None => {
                if id.to_string().as_str() == "enum" {
                    state.state = EnumTreeParseState::SeenEnum;
                }
            }
            EnumTreeParseState::SeenEnum => {
                state.state = EnumTreeParseState::SeenName(id.to_string());
            }
            EnumTreeParseState::SeenName(_) | EnumTreeParseState::FoundGroup(_, _) => {
                panic!("[ArgParse derive] Inconsistent state expected state to be `SeenName` (this is a bug).");
            }
        },
        TokenTree::Punct(_) | TokenTree::Literal(_) => {}
    }
}

#[derive(Debug, Clone)]
struct SubCommandParsed {
    doc_comments: Vec<String>,
    tag_name: String,
    inner_parse_type_name: Option<String>,
}

#[derive(Debug, Clone)]
enum ArgsParsedTreeParseState {
    Ready,
    ReadyOrInner(SubCommandParsed),
    WantsAnnotationGroup,
}

fn parse_inner_enum<'a>(name: &'a str, group: &'a Group) -> String {
    let mut state = ArgsParsedTreeParseState::Ready;
    let mut pending_doc_comments = Vec::new();
    let mut cw = CodeWriter::new(name);
    for tree in group.stream() {
        match tree {
            TokenTree::Group(g) => match state.clone() {
                ArgsParsedTreeParseState::ReadyOrInner(mut r) => {
                    match g.delimiter() {
                        Delimiter::Parenthesis => {
                            let mut inner_g_stream = g.stream().into_iter();
                            let ident = pop_ident(&mut inner_g_stream, "[ArgParse derive] Expected subcommand enum tag to contain a member \
                                on the form (Struct), found group: {g}");
                            r.inner_parse_type_name = Some(ident.to_string());
                        }
                        Delimiter::Brace | Delimiter::Bracket | Delimiter::None => {}
                    }
                    cw.push_cmd(r);
                    state = ArgsParsedTreeParseState::Ready;
                }
                ArgsParsedTreeParseState::WantsAnnotationGroup => {
                    if let Some(dc) = try_extract_doc_comment(&g) {
                        pending_doc_comments.push(dc);
                    }
                    state = ArgsParsedTreeParseState::Ready;
                }
                ArgsParsedTreeParseState::Ready => {
                    panic!("[ArgParse derive] Expected an ident when parsing enum inner, found group: {g}");
                }
            },
            TokenTree::Ident(id) => match state.clone() {
                ArgsParsedTreeParseState::Ready => {
                    let sc = SubCommandParsed {
                        doc_comments: core::mem::take(&mut pending_doc_comments),
                        tag_name: id.to_string(),
                        inner_parse_type_name: None,
                    };
                    state = ArgsParsedTreeParseState::ReadyOrInner(sc);
                }
                ArgsParsedTreeParseState::ReadyOrInner(sc) => {
                    cw.push_cmd(sc);
                    let sc = SubCommandParsed {
                        doc_comments: core::mem::take(&mut pending_doc_comments),
                        tag_name: id.to_string(),
                        inner_parse_type_name: None,
                    };
                    state = ArgsParsedTreeParseState::ReadyOrInner(sc);
                }
                ArgsParsedTreeParseState::WantsAnnotationGroup => {
                    panic!("[ArgParse derive] Expected a group when parsing enum inner, found ident: {id}");
                }
            },
            TokenTree::Punct(p) => {
                if p.as_char() == '#' {
                    if let ArgsParsedTreeParseState::ReadyOrInner(sc) = state.clone() {
                        cw.push_cmd(sc);
                    }
                    state = ArgsParsedTreeParseState::WantsAnnotationGroup;
                }
            }
            TokenTree::Literal(_) => {}
        }
    }
    if let ArgsParsedTreeParseState::ReadyOrInner(sc) = state {
        cw.push_cmd(sc.clone());
    }
    cw.finish()
}

/// Single pass code writer (as far as that's possible)
struct CodeWriter {
    longest_name: usize,
    printer_head: String,
    printer_cmd_out: Vec<(String, Option<String>)>,
    lit_matches_head: String,
    subcommand_head: String,
    subcommand_match_head: String,
}

impl CodeWriter {
    fn new(name: &str) -> Self {
        Self {
            longest_name: 0,
            printer_head: gen_printer_head(name),
            printer_cmd_out: vec![],
            lit_matches_head: gen_lit_matches_head(name),
            subcommand_head: gen_subcommand_head(name),
            subcommand_match_head: subcommand_match_head().to_string(),
        }
    }

    fn push_cmd(&mut self, cmd: SubCommandParsed) {
        let snake = pascal_to_snake(&cmd.tag_name);
        let scream = snake_to_scream(&snake);
        self.push_to_match_head(&scream, &cmd);
        self.push_to_subcommand(&snake, &scream);
        self.push_lit_matches(&snake);
        self.push_print_data(snake, cmd);
    }

    fn push_to_match_head(&mut self, scream: &str, cmd: &SubCommandParsed) {
        if let Some(inner) = &cmd.inner_parse_type_name {
            let _ = self.subcommand_match_head.write_fmt(format_args!(
                "            {} => {{ Self::{}(<{} as tiny_std::unix::cli::ArgParse>::arg_parse(args)?) }},\n",
                scream, cmd.tag_name, inner
            ));
        } else {
            let _ = self.subcommand_match_head.write_fmt(format_args!(
                "            {} => {{ Self::{} }},\n",
                scream, cmd.tag_name
            ));
        }
    }

    #[inline]
    fn push_to_subcommand(&mut self, snake: &str, scream: &str) {
        let _ = self.subcommand_head.write_fmt(format_args!(
            "\tconst {scream}: &[u8] = UnixStr::from_str_checked(\"{snake}\\0\").as_slice();\n",
        ));
    }

    #[inline]
    fn push_lit_matches(&mut self, snake: &str) {
        let _ = self.lit_matches_head.write_fmt(format_args!("{snake} | "));
    }
    fn push_print_data(&mut self, snake: String, mut cmd: SubCommandParsed) {
        if snake.len() > self.longest_name {
            self.longest_name = snake.len();
        }
        let mut use_comment = None;
        if !cmd.doc_comments.is_empty() {
            use_comment = Some(cmd.doc_comments.swap_remove(0));
        }
        self.printer_cmd_out.push((snake, use_comment));
    }

    fn finish(mut self) -> String {
        const PRINTER_TAIL: &str = printer_tail();
        const LIT_MATCHES_TAIL: &str = lit_matches_tail();
        const SUBCOMMAND_TAIL: &str = subcommand_tail();
        let _ = self
            .printer_head
            .write_fmt(format_args!("\tlet pad = {}usize;\n", self.longest_name));
        for (snake_name, doc_comment) in self.printer_cmd_out {
            if let Some(first_line_doc_comment) = doc_comment {
                let _ = self.printer_head.write_fmt(format_args!(
                    "\tf.write_fmt(format_args!(\"  {{:<pad_width$}} - {first_line_doc_comment}\\n\", \"{snake_name}\", pad_width = pad))?;\n"
                ));
            } else {
                let _ = self
                    .printer_head
                    .write_fmt(format_args!("\tf.write_str(\"  {snake_name}\\n\")?;\n"));
            }
        }
        for _i in 0..3 {
            self.lit_matches_head.pop();
        }
        let needs_additional_len = PRINTER_TAIL.len()
            + LIT_MATCHES_TAIL.len()
            + SUBCOMMAND_TAIL.len()
            + self.lit_matches_head.len()
            + self.subcommand_head.len()
            + self.subcommand_match_head.len();
        self.printer_head.reserve(needs_additional_len);
        let _ = self.printer_head.write_str(PRINTER_TAIL);
        let _ = self.printer_head.write_str(&self.lit_matches_head);
        let _ = self.printer_head.write_str(LIT_MATCHES_TAIL);
        let _ = self.printer_head.write_str(&self.subcommand_head);
        let _ = self.printer_head.write_str(&self.subcommand_match_head);
        let _ = self.printer_head.write_str(SUBCOMMAND_TAIL);
        self.printer_head
    }
}

fn gen_printer_head(name: &str) -> String {
    let help_printer_name = format!("__{name}HelpPrinterZst");
    format!(
        "\
pub struct {help_printer_name};
impl core::fmt::Display for {help_printer_name} {{
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {{
        f.write_str(\"Commands:\\n\")?;
    "
    )
}

const fn printer_tail() -> &'static str {
    "        Ok(())
    }
}
"
}

fn gen_lit_matches_head(name: &str) -> String {
    format!(
        "\
impl {name} {{
    pub const fn __command_lit_matches() -> &'static str {{
        \""
    )
}

const fn lit_matches_tail() -> &'static str {
    "\"
    }
}
"
}

fn gen_subcommand_head(name: &str) -> String {
    let help_printer_name = format!("__{name}HelpPrinterZst");
    format!("\
impl tiny_std::unix::cli::SubcommandParse for {name} {{
    type HelpPrinter = {help_printer_name};
    #[inline]
    fn help_printer() -> &'static Self::HelpPrinter {{
        &{help_printer_name}
    }}
    fn subcommand_parse(cmd: &'static UnixStr, args: &mut impl Iterator<Item = &'static UnixStr>) -> core::result::Result<Option<Self>, tiny_std::unix::cli::ArgParseError> {{
")
}

const fn subcommand_match_head() -> &'static str {
    "        Ok(Some(match cmd.as_slice() {
"
}

const fn subcommand_tail() -> &'static str {
    "            _val => { return Ok(None) }
        }))
    }
}
"
}
