use crate::derive_struct::impl_struct::CodeWriter;
use crate::{
    pop_expect_ident, pop_expect_punct, pop_group, pop_ident, pop_lit, try_extract_doc_comment,
};
use proc_macro::{Group, Ident, Punct, TokenStream, TokenTree};
use std::collections::VecDeque;
use std::fmt::Write;
use std::str::FromStr;

mod impl_struct;

#[inline]
pub(crate) fn do_derive(struct_candidate: TokenStream) -> TokenStream {
    let mut state = StructTreeParse::new();
    for tree in struct_candidate {
        check_state(&mut state, tree);
        if let StructTreeParseState::FoundGroup(name, group) = &state.state {
            return parse_group(name, &state.metadata, group);
        }
    }
    panic!("[ArgParse derive] Failed to derive arg_parse, failed to find struct information");
}

#[derive(Debug)]
struct StructTreeParse {
    metadata: StructMetadata,
    state: StructTreeParseState,
}

#[derive(Debug, Clone)]
pub(crate) struct StructMetadata {
    doc_comments: Vec<String>,
    help_info: Vec<String>,
}

impl StructTreeParse {
    fn new() -> Self {
        Self {
            metadata: StructMetadata {
                doc_comments: vec![],
                help_info: vec![],
            },
            state: StructTreeParseState::None,
        }
    }
}

#[derive(Debug, Clone)]
enum StructTreeParseState {
    None,
    SeenStruct,
    SeenName(String),
    FoundGroup(String, Group),
}

fn check_state(state: &mut StructTreeParse, tree: TokenTree) {
    match tree {
        TokenTree::Group(g) => match state.state.clone() {
            StructTreeParseState::SeenName(n) => {
                state.state = StructTreeParseState::FoundGroup(n.clone(), g);
            }
            StructTreeParseState::None
            | StructTreeParseState::SeenStruct
            | StructTreeParseState::FoundGroup(_, _) => {
                try_extract_more_metadata(&mut state.metadata, &g);
            }
        },
        TokenTree::Ident(i) => match state.state.clone() {
            StructTreeParseState::None => {
                if matches!(i.to_string().as_str(), "struct") {
                    state.state = StructTreeParseState::SeenStruct;
                }
            }
            StructTreeParseState::SeenStruct => {
                state.state = StructTreeParseState::SeenName(i.to_string());
            }
            StructTreeParseState::SeenName(_) | StructTreeParseState::FoundGroup(_, _) => {
                panic!("[ArgParse derive] Inconsistent state expected state to be `SeenName` (this is a bug).");
            }
        },
        TokenTree::Punct(_) | TokenTree::Literal(_) => {}
    }
}

fn try_extract_more_metadata(meta: &mut StructMetadata, group: &Group) {
    if let Some(cmnt) = try_extract_doc_comment(group) {
        meta.doc_comments.push(cmnt);
        return;
    }
    let mut stream = group.stream().into_iter();
    if let Some(TokenTree::Ident(ident)) = stream.next() {
        let to_string = ident.to_string();
        if "cli" == to_string {
            let inner_group = pop_group(
                &mut stream,
                format!("[ArgParse derive] Expected to find a group following #[cli in {group}"),
            );
            let mut inner_group_stream = inner_group.stream().into_iter();
            while let Some((k, v)) = extract_struct_level_cli_properties(&mut inner_group_stream) {
                match k.as_str() {
                    "help_path" => {
                        let help_info = v
                            .split(',')
                            .filter_map(|s| {
                                let trimmed = s.trim();
                                if trimmed.is_empty() {
                                    None
                                } else {
                                    Some(trimmed.to_string())
                                }
                            })
                            .collect::<Vec<String>>();
                        meta.help_info = help_info;
                    }
                    val => {
                        panic!("[ArgParse derive] Unrecognized argument placed in struct declaration #[cli(... expected 'help_path' found {val}");
                    }
                }
            }
        }
    }
}

fn extract_struct_level_cli_properties(
    g_it: &mut impl Iterator<Item = TokenTree>,
) -> Option<(String, String)> {
    if let Some(mut next_token_tree) = g_it.next() {
        if let TokenTree::Punct(p) = next_token_tree {
            if p.as_char() == ',' {
                if let Some(next) = g_it.next() {
                    next_token_tree = next;
                } else {
                    // Trailing comma
                    return None;
                }
            } else {
                panic!("[ArgParse derive] only punctuation expected within #[cli(... is '=' between keys and values, found {}", p.as_char());
            }
        }
        let TokenTree::Ident(key) = next_token_tree else {
            panic!("[ArgParse derive] Expected to find arguments inside #[cli on form #[cli(k = \"val\", ...)")
        };
        pop_expect_punct(g_it, '=', "[ArgParse derive] Expected to find punctuation '=' after ident when parsing #[cli(k ('=')...");
        let val = pop_lit(g_it, "[ArgParse derive] Expected ident and '=' to be followed by a literal in the form #[cli(k = \"val\"...");
        let val_string = val.to_string();
        let val_string = val_string.trim().trim_matches('"').trim().to_string();
        return Some((key.to_string(), val_string));
    }
    None
}

#[derive(Debug, Clone)]
enum ArgsParsedTreeParseState {
    Ready,
    WantsAnnotation,
    WantsSubcommand,
    WantsMember(CliPreferences),
}
#[expect(clippy::too_many_lines)]
fn parse_group(name: &str, metadata: &StructMetadata, g: &Group) -> TokenStream {
    let mut stream = g.stream().into_iter();
    let mut state = ArgsParsedTreeParseState::Ready;
    let mut doc_comments_for_next = Vec::new();
    let mut parsed_fields = VecDeque::new();
    let mut subcommand = None;
    let mut c = CodeWriter::new(name, metadata);
    while let Some(tree) = stream.next() {
        match &tree {
            TokenTree::Group(g) => match state {
                ArgsParsedTreeParseState::WantsAnnotation => {
                    let res = parse_annotation_group(g);
                    match res {
                        GroupParseResult::Ignore => {}
                        GroupParseResult::DocComment(com) => {
                            doc_comments_for_next.push(com);
                        }
                        GroupParseResult::SubCommand => {
                            state = ArgsParsedTreeParseState::WantsSubcommand;
                            continue;
                        }
                        GroupParseResult::FieldPreferences(prefs) => {
                            state = ArgsParsedTreeParseState::WantsMember(prefs);
                            continue;
                        }
                    }
                    state = ArgsParsedTreeParseState::Ready;
                    continue;
                }
                ArgsParsedTreeParseState::WantsSubcommand
                | ArgsParsedTreeParseState::WantsMember(_)
                | ArgsParsedTreeParseState::Ready => {
                    panic!("[ArgParse derive] Bad state ready encountering group, expected ReadyParseAnnotation")
                }
            },
            TokenTree::Ident(ident) => match state.clone() {
                ArgsParsedTreeParseState::WantsAnnotation => {
                    panic!("[ArgParse derive] Bad state encountering ident, expected Ready")
                }
                ArgsParsedTreeParseState::WantsSubcommand => {
                    let mem = parse_member(ident, &mut stream);
                    let field_ty = match mem.ty {
                        FieldTy::UnixStr | FieldTy::Str | FieldTy::Bool => {
                            panic!("[ArgParse derive] Invalid type for subcommand");
                        }
                        FieldTy::Unknown(ty) => ty,
                    };
                    assert!(subcommand.is_none(), "Found multiple subcommands");
                    doc_comments_for_next = Vec::new();
                    let sc = ParsedSubcommand {
                        field_name: mem.name,
                        field_ty,
                        is_opt: matches!(mem.package, FieldPackageKind::Option),
                    };
                    c.push_subcommand(&sc);
                    subcommand = Some(sc);
                    state = ArgsParsedTreeParseState::Ready;
                }
                ArgsParsedTreeParseState::Ready => {
                    let mem = parse_member(ident, &mut stream);
                    let pf = ParsedField::new_check_consistency(
                        core::mem::take(&mut doc_comments_for_next),
                        mem.name.clone(),
                        mem.ty,
                        mem.is_ref,
                        mem.package,
                        None,
                        None,
                        None,
                    );
                    c.push_field(&pf);
                    parsed_fields.push_back(pf);
                }
                ArgsParsedTreeParseState::WantsMember(p) => {
                    let mem = parse_member(ident, &mut stream);
                    let pf = ParsedField::new_check_consistency(
                        core::mem::take(&mut doc_comments_for_next),
                        mem.name.clone(),
                        mem.ty,
                        mem.is_ref,
                        mem.package,
                        p.arg,
                        p.long,
                        p.short,
                    );
                    c.push_field(&pf);
                    parsed_fields.push_back(pf);
                    state = ArgsParsedTreeParseState::Ready;
                }
            },
            TokenTree::Punct(p) => {
                if p.as_char() == '#' {
                    state = ArgsParsedTreeParseState::WantsAnnotation;
                }
            }
            TokenTree::Literal(_) => {}
        }
    }
    // Subcommands and positional arguments are hard to parse together (though not impossible)
    assert!(
        !(subcommand.is_some() && parsed_fields.iter().any(|pf| pf.positional.is_some())),
        "Struct has both a subcommand and a positional argument, unparseable"
    );

    // Jumbled optional and required positionals may be impossible to parse correctly
    let mut seen_opt_positional = false;
    for pf in &parsed_fields {
        if pf.positional.is_some() && matches!(pf.package, FieldPackageKind::Option) {
            assert!(
                !seen_opt_positional,
                "If an optional positional argument is specified, it must come last"
            );
            seen_opt_positional = true;
        }
    }
    let out = c.finish(parsed_fields);
    TokenStream::from_str(&out)
        .expect("[ArgParse derive] Failed to convert generated struct to token stream")
}

#[expect(clippy::too_many_lines)]
fn parse_annotation_group(g: &Group) -> GroupParseResult {
    let mut group_stream = g.stream().into_iter();
    let first = group_stream
        .next()
        .expect("[ArgParse derive] Expected at least one item in group");
    let TokenTree::Ident(ident) = first else {
        panic!("[ArgParse derive] Expected first item in gorup to be an ident");
    };
    match ident.to_string().as_str() {
        "doc" => {
            pop_expect_punct(
                &mut group_stream,
                '=',
                "Expected doc group with a '=' punctuation".to_string(),
            );
            let lit = pop_lit(
                &mut group_stream,
                "Expected doc group to contain a literal after '='",
            );
            GroupParseResult::DocComment(
                lit.to_string()
                    .trim_matches(|ch: char| ch == '"' || ch.is_whitespace())
                    .to_string(),
            )
        }
        "cli" => {
            let g = pop_group(&mut group_stream, "Expected cli to be followed by a group starting with (. ex: #[cli( long= \"my-arg\")]");
            let mut g_stream = g.stream().into_iter();
            let mut preferred_arg = None;
            let mut preferred_short = None;
            let mut preferred_long = None;
            while let Some(next_item) = g_stream.next() {
                let ident = match next_item {
                    TokenTree::Ident(i) => i,
                    TokenTree::Group(_) | TokenTree::Punct(_) | TokenTree::Literal(_) => {
                        continue;
                    }
                };
                match ident.to_string().trim() {
                    "short" => {
                        assert!(
                            preferred_short.is_none(),
                            "Found multiple cli(short) in struct"
                        );
                        pop_expect_punct(
                            &mut g_stream,
                            '=',
                            "Expected 'short' in #[cli(short... to be followed by an =",
                        );
                        let short_lit = pop_lit(
                            &mut g_stream,
                            "Expected a literal in #[cli(short = \"<lit>\"...",
                        )
                        .to_string()
                        .trim_matches('\"')
                        .to_string();
                        assert_eq!(1, short_lit.chars().count(), "Expected short literal in #[cli(short = \"<lit>\"... to be a single character, got {short_lit}");
                        preferred_short = Some(short_lit.to_string());
                    }
                    "long" => {
                        assert!(
                            preferred_long.is_none(),
                            "Found multiple cli(long) in struct"
                        );
                        pop_expect_punct(
                            &mut g_stream,
                            '=',
                            "Expected 'long' in #[cli(long... to be followed by an =",
                        );
                        let long_lit = pop_lit(
                            &mut g_stream,
                            "Expected a literal in #[cli(long = \"<lit>\"...",
                        )
                        .to_string()
                        .trim_matches('\"')
                        .to_string();
                        preferred_long = Some(long_lit);
                    }
                    "subcommand" => {
                        assert!(!(preferred_long.is_some() || preferred_short.is_some()), "Found both subcommand and long/short on the same field, subcommands are named by their enum tags");
                        return GroupParseResult::SubCommand;
                    }
                    "arg" => {
                        assert!(
                            preferred_arg.is_none(),
                            "Found multiple cli(arg) in struct"
                        );
                        pop_expect_punct(
                            &mut g_stream,
                            '=',
                            "Expected 'long' in #[cli(arg... to be followed by an =",
                        );
                        let arg_lit = pop_lit(
                            &mut g_stream,
                            "Expected a literal in #[cli(arg = \"<lit>\"...",
                        )
                        .to_string()
                        .trim_matches('\"')
                        .to_string();
                        preferred_arg = Some(arg_lit);
                    }
                    v => panic!("[ArgParse derive] Expected cli group ident to be either 'long', 'short', subcommand, or 'arg' got {v}"),
                }
            }

            GroupParseResult::FieldPreferences(CliPreferences {
                arg: preferred_arg,
                long: preferred_long,
                short: preferred_short,
            })
        }
        _ => GroupParseResult::Ignore,
    }
}

#[expect(clippy::too_many_lines)]
fn parse_member<I: Iterator<Item = TokenTree>>(ident: &Ident, it: &mut I) -> ParsedMember {
    let mut field_name = ident.to_string();
    if field_name == "pub" {
        match it.next() {
            None => {
                panic!("Expected visibility to be followed by an ident found nothing");
            }
            Some(tt) => match tt {
                TokenTree::Group(g) => {
                    let count = g.stream().into_iter().count();
                    assert_eq!(
                        1, count,
                        "Expected a single identifier after 'pub(', found {g}"
                    );
                    field_name =
                        pop_ident(it, "Expected to find an ident after visibility").to_string();
                }
                TokenTree::Ident(id) => {
                    field_name = id.to_string();
                }
                TokenTree::Punct(p) => {
                    panic!("Found punctuation '{p}' after visibility specifier, expected an ident");
                }
                TokenTree::Literal(l) => {
                    panic!("Found literal {l} after visibility specifier, expected an ident");
                }
            },
        }
    }
    pop_expect_punct(it, ':', "Failed to parse member, expected ':' punctuation");
    let next = it
        .next()
        .expect("[ArgParse derive] Failed to parse member, expected a type");
    match next {
        TokenTree::Ident(id) => {
            let trimmed_name = id.to_string();

            match trimmed_name.as_str() {
                "Vec" => {
                    pop_expect_punct(
                        it,
                        '<',
                        format!("Expected a '<' following vec declaration for {field_name}"),
                    );
                    let next = it.next().unwrap_or_else(|| {
                        panic!(
                            "[ArgParse derive] Expected something following '<' in vec declaration for {field_name}"
                        )
                    });
                    match next {
                        TokenTree::Ident(ident) => {
                            let ty = FieldTy::from_ident(&ident);
                            ParsedMember {
                                name: field_name,
                                ty,
                                is_ref: false,
                                package: FieldPackageKind::Vec,
                            }
                        }
                        TokenTree::Punct(p) => {
                            parse_static_ref(it, &p, field_name, FieldPackageKind::Vec)
                        }
                        t => {
                            panic!("Found unexpected token {t:?} parsing {field_name}");
                        }
                    }
                }
                "Option" => {
                    pop_expect_punct(
                        it,
                        '<',
                        format!("Expected a '<' following optional declaration for {field_name}"),
                    );
                    let next = it.next().unwrap_or_else(|| panic!("Expected something following '<' in optional declaration for {field_name}"));
                    match next {
                        TokenTree::Ident(ident) => {
                            let ty = FieldTy::from_ident(&ident);
                            ParsedMember {
                                name: field_name,
                                ty,
                                is_ref: false,
                                package: FieldPackageKind::Option,
                            }
                        }
                        TokenTree::Punct(p) => {
                            parse_static_ref(it, &p, field_name, FieldPackageKind::Option)
                        }
                        t => {
                            panic!("Found unexpected token {t:?} parsing {field_name}");
                        }
                    }
                }
                _ty => {
                    let ty = FieldTy::from_ident(&id);
                    ParsedMember {
                        name: field_name,
                        ty,
                        is_ref: false,
                        package: FieldPackageKind::None,
                    }
                }
            }
        }
        TokenTree::Punct(p) => parse_static_ref(it, &p, field_name, FieldPackageKind::None),
        TokenTree::Group(_) | TokenTree::Literal(_) => {
            panic!("Expected ident or punct when parsing tiny-cli annotated struct, found group or literal")
        }
    }
}

fn parse_static_ref<I: Iterator<Item = TokenTree>>(
    it: &mut I,
    p: &Punct,
    field_name: String,
    package: FieldPackageKind,
) -> ParsedMember {
    assert_eq!(
        '&',
        p.as_char(),
        "Expected a reference '&' or an ident after ':' for {field_name}"
    );
    pop_expect_punct(
        it,
        '\'',
        format!("Expected a 'static after '&' for {field_name}"),
    );
    pop_expect_ident(
        it,
        "static",
        format!("Expected a &'static after '&' for {field_name}"),
    );
    let ident = pop_ident(
        it,
        format!("Expected an ident after &'static for {field_name}"),
    );
    let ty = FieldTy::from_ident(&ident);
    ParsedMember {
        name: field_name,
        ty,
        is_ref: true,
        package,
    }
}

#[derive(Clone, Debug)]
enum GroupParseResult {
    Ignore,
    DocComment(String),
    SubCommand,
    FieldPreferences(CliPreferences),
}

#[derive(Debug, Clone)]
struct CliPreferences {
    arg: Option<String>,
    long: Option<String>,
    short: Option<String>,
}

#[derive(Clone, Debug)]
pub(crate) struct ParsedMember {
    name: String,
    ty: FieldTy,
    is_ref: bool,
    package: FieldPackageKind,
}

#[derive(Clone, Debug)]
pub(crate) struct ParsedField {
    doc_comments: Vec<String>,
    name: String,
    ty: FieldTy,
    is_ref: bool,
    package: FieldPackageKind,
    positional: Option<String>,
    long_match: Option<String>,
    short_match: Option<String>,
}

impl ParsedField {
    pub(crate) fn long_const_ident(&self) -> Option<String> {
        self.long_match
            .as_ref()
            .map(|s| s.replace('-', "_").to_uppercase())
    }
    pub(crate) fn long_match_lit(&self) -> Option<String> {
        self.long_match
            .as_ref()
            .map(|s| s.to_lowercase().replace('_', "-"))
    }

    pub(crate) fn short_const_ident(&self) -> Option<String> {
        self.short_match
            .as_ref()
            .map(|s| s.replace('-', "_").to_uppercase())
    }

    pub(crate) fn short_match_lit(&self) -> Option<String> {
        self.short_match
            .as_ref()
            .map(|s| s.to_lowercase().replace('_', "-"))
    }

    pub(crate) fn as_const_match(&self) -> Option<String> {
        let id = match (self.short_const_ident(), self.long_const_ident()) {
            (Some(short), Some(long)) => {
                format!("{short} | {long}")
            }
            (Some(id), None) | (None, Some(id)) => id,
            (None, None) => {
                return None;
            }
        };
        Some(id)
    }

    pub(crate) fn type_decl(&self) -> String {
        match &self.ty {
            FieldTy::UnixStr => "&'static tiny_std::UnixStr".to_string(),
            FieldTy::Str => "&'static str".to_string(),
            FieldTy::Unknown(ty) => {
                if self.is_ref {
                    format!("&'static {ty}")
                } else {
                    ty.clone()
                }
            }
            FieldTy::Bool => "bool".to_string(),
        }
    }

    pub(crate) fn as_lit_match(&self) -> Option<String> {
        let help_row = match (self.short_match_lit(), self.long_match_lit()) {
            (Some(short), Some(long)) => {
                format!("-{short} | --{long}")
            }
            (Some(short), None) => {
                format!("-{short}")
            }
            (None, Some(long)) => {
                format!("--{long}")
            }
            (None, None) => {
                return None;
            }
        };
        Some(help_row)
    }

    pub(crate) fn write_into_help(&self, help_row: &mut String) {
        match (self.short_match_lit(), self.long_match_lit()) {
            (Some(short), Some(long)) => {
                let _ = help_row.write_fmt(format_args!("  -{short}, --{long}\n",));
            }
            (Some(short), None) => {
                let _ = help_row.write_fmt(format_args!("  -{short}\n"));
            }
            (None, Some(long)) => {
                let _ = help_row.write_fmt(format_args!("      --{long}\n"));
            }
            (None, None) => {}
        }
        for dc in &self.doc_comments {
            let _ = help_row.write_fmt(format_args!("        {dc}\n"));
        }
        let _ = help_row.write_char('\n');
    }

    pub(crate) fn write_into_args_help(&self, arg: &str, help_row: &mut String) {
        let _ = help_row.write_fmt(format_args!("  [{}]\n", arg.to_uppercase()));
        for dc in &self.doc_comments {
            let _ = help_row.write_fmt(format_args!("        {dc}\n"));
        }
        let _ = help_row.write_char('\n');
    }

    #[expect(clippy::too_many_arguments)]
    pub(crate) fn new_check_consistency(
        doc_comments: Vec<String>,
        name: String,
        ty: FieldTy,
        is_ref: bool,
        package: FieldPackageKind,
        mut positional: Option<String>,
        long_match: Option<String>,
        short_match: Option<String>,
    ) -> Self {
        assert!(!(ty == FieldTy::Bool && matches!(package, FieldPackageKind::Option)), "[ArgParse Derive] Failed to derive, got field with name={name} specified as Option<bool> bool defaults to false and are always optional");
        assert!(!(positional.is_some() && (long_match.is_some() || short_match.is_some())), "[ArgParse Derive] Failed to derive, got field with name={name} with both a positional (arg) and options specified");
        if positional.is_none() && long_match.is_none() && short_match.is_none() {
            positional = Some(name.clone());
        }
        assert!(!(positional.is_some() && (ty == FieldTy::Bool || matches!(package, FieldPackageKind::Vec))), "[ArgParse Derive] Failed to derive, got field with name={name} with an invalid type, arguments can't be booleans or vectors");
        Self {
            doc_comments,
            name,
            ty,
            is_ref,
            package,
            positional,
            long_match,
            short_match,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum FieldTy {
    UnixStr,
    Str,
    Bool,
    Unknown(String),
}

impl FieldTy {
    fn from_ident(ident: &Ident) -> Self {
        let trimmed_ident = ident.to_string().trim().to_string();
        match trimmed_ident.as_str() {
            "UnixStr" => Self::UnixStr,
            "str" => Self::Str,
            "bool" => Self::Bool,
            &_ => Self::Unknown(trimmed_ident),
        }
    }
}

#[derive(Clone, Debug)]
pub(crate) struct ParsedSubcommand {
    field_name: String,
    field_ty: String,
    is_opt: bool,
}

#[derive(Debug, Copy, Clone)]
pub(crate) enum FieldPackageKind {
    None,
    Vec,
    Option,
}
