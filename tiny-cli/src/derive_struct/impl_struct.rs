use crate::derive_struct::{
    FieldPackageKind, FieldTy, ParsedField, ParsedSubcommand, StructMetadata,
};
use std::fmt::Write;

pub(crate) struct CodeWriter {
    printer_head: String,
    printer_mid: String,
    printer_opts: Option<String>,
    sc_print_fmt: Option<String>,
    impl_head: String,
    var_decl_head: String,
    match_head: String,
    match_tail: Option<String>,
    struct_out: String,
}

impl CodeWriter {
    pub(crate) fn new(name: &str, pkg_meta: &StructMetadata) -> Self {
        let help_printer_name = format!("__{name}HelpPrinterZst");
        let mut printer_mid = String::new();
        for dc in &pkg_meta.doc_comments {
            let _ = printer_mid.write_fmt(format_args!("{dc}\n"));
        }
        if !pkg_meta.doc_comments.is_empty() {
            printer_mid.push('\n');
        }
        let _ = printer_mid.write_str("Usage:");
        for info in &pkg_meta.help_info {
            let _ = printer_mid.write_fmt(format_args!(" {info}"));
        }
        Self {
            printer_head: gen_printer_head(&help_printer_name),
            printer_mid,
            printer_opts: None,
            sc_print_fmt: None,
            impl_head: gen_impl_head(name, &help_printer_name),
            var_decl_head: String::new(),
            match_head: String::new(),
            match_tail: None,
            struct_out: String::new(),
        }
    }
    pub(crate) fn push_field(&mut self, field: &ParsedField) {
        self.field_push_const_match(field);
        self.field_push_var_decl(field);
        self.field_push_to_match(field);
        self.field_push_to_out(field);
        self.field_push_printer(field);
    }

    pub(crate) fn push_subcommand(&mut self, sc: &ParsedSubcommand) {
        self.subcommand_push_var_decl(sc);
        self.subcommand_push_match_tail(sc);
        self.subcommand_push_to_out(sc);
        self.sc_print_fmt = Some(sc.field_ty.clone());
    }

    fn field_push_printer(&mut self, field: &ParsedField) {
        if self.printer_opts.is_none() {
            self.printer_opts = Some("\nOptions:\n".to_string());
        }
        let Some(opts) = self.printer_opts.as_mut() else {
            unreachable!()
        };
        field.write_into_help(opts);
    }

    fn field_push_const_match(&mut self, field: &ParsedField) {
        let _ = self.impl_head.write_fmt(format_args!(
            "\t\tconst {}: &[u8] = tiny_std::UnixStr::from_str_checked(\"--{}\\0\").as_slice();\n",
            field.long_const_ident(),
            field.long_match_lit()
        ));
        if let (Some(short_const), Some(short_match)) =
            (field.short_const_ident(), field.short_match_lit())
        {
            let _ = self.impl_head.write_fmt(format_args!("\t\tconst {short_const}: &[u8] = tiny_std::UnixStr::from_str_checked(\"-{short_match}\\0\").as_slice();\n"));
        }
    }

    fn field_push_var_decl(&mut self, field: &ParsedField) {
        match field.package {
            FieldPackageKind::None | FieldPackageKind::Option => {
                let _ = self.var_decl_head.write_fmt(format_args!(
                    "\t\tlet mut {}: Option<{}> = None;\n",
                    field.name,
                    field.type_decl()
                ));
            }
            FieldPackageKind::Vec => {
                let _ = self.var_decl_head.write_fmt(format_args!(
                    "\t\tlet mut {}: Vec<{}> = Vec::new();\n",
                    field.name,
                    field.type_decl()
                ));
            }
        }
    }

    #[inline]
    fn subcommand_push_var_decl(&mut self, sc: &ParsedSubcommand) {
        let _ = self.var_decl_head.write_fmt(format_args!(
            "\t\tlet mut {}: Option<{}> = None;\n",
            sc.field_name, sc.field_ty
        ));
    }

    fn field_push_to_match(&mut self, field: &ParsedField) {
        let asgn = member_try_assign(field);
        let _ = self.match_head.write_fmt(format_args!(
            "\
\t\t\t\t{} => {{
                    {asgn};
                }},
",
            field.as_const_match()
        ));
    }

    fn subcommand_push_match_tail(&mut self, sc: &ParsedSubcommand) {
        let match_tail = format!("\
                    if let Some(sc_parsed) = <{} as tiny_std::unix::cli::SubcommandParse>::subcommand_parse(next, args)? {{
                        {} = Some(sc_parsed);
                    }} else {{
                        return Err(tiny_std::unix::cli::ArgParseError::new_cause_fmt(Self::help_printer(), format_args!(\"Unrecognized argument: {{:?}}\", core::str::from_utf8(no_match)))?);
                    }}
        ", sc.field_ty, sc.field_name);
        self.match_tail = Some(match_tail);
    }

    fn field_push_to_out(&mut self, field: &ParsedField) {
        if matches!(field.package, FieldPackageKind::None) {
            let _ = self.struct_out.write_fmt(format_args!("\
            {}: {{
                if let Some(found_arg) = {} {{
                    found_arg
                }} else {{
                    return Err(tiny_std::unix::cli::ArgParseError::new_cause_str(Self::help_printer(), \"Required option '{}' not supplied.\")?);
                }}
            }},
            ", field.name, field.name, field.as_lit_match()));
        } else {
            let _ = self.struct_out.write_fmt(format_args!(
                "\
{},
",
                field.name
            ));
        }
    }

    fn subcommand_push_to_out(&mut self, sc: &ParsedSubcommand) {
        if sc.is_opt {
            let _ = self
                .struct_out
                .write_fmt(format_args!("\t\t\t{},\n", sc.field_name));
        } else {
            let _ = self.struct_out.write_fmt(format_args!("\
                {}: {{
                    if let Some(found_arg) = {} {{
                        found_arg
                    }} else {{
                        return Err(tiny_std::unix::cli::ArgParseError::new_cause_fmt(Self::help_printer(), format_args!(\"Required command '{{}}' not supplied.\", {}::__command_lit_matches()))?);
                    }}
                }},
", sc.field_name, sc.field_name, sc.field_ty));
        }
    }

    pub(crate) fn finish(mut self) -> String {
        let mut output = String::new();
        if self.printer_opts.is_some() {
            let _ = self.printer_mid.write_str(" [OPTIONS]");
        }
        let has_subcommand = self.match_tail.is_some();
        if has_subcommand {
            let _ = self.printer_mid.write_str(" [COMMAND]\n\n{}");
        } else {
            let _ = self.printer_mid.write_str("\n");
        }

        let _ = output.write_str(&self.printer_head);
        if let Some(sc_ty) = self.sc_print_fmt {
            if let Some(opts) = &self.printer_opts {
                let _ = output.write_fmt(format_args!("\t\tf.write_fmt(format_args!(\"{}{}\", <{} as tiny_std::unix::cli::SubcommandParse>::help_printer()))\n", self.printer_mid, opts, sc_ty));
            } else {
                let _ = output.write_fmt(format_args!("\t\tf.write_fmt(format_args!(\"{}\", <{} as tiny_std::unix::cli::SubcommandParse>::help_printer()))\n", self.printer_mid, sc_ty));
            }
        } else if let Some(opts) = &self.printer_opts {
            let _ = output.write_fmt(format_args!(
                "\t\tf.write_str(\"{}{}\")",
                self.printer_mid, opts
            ));
        } else {
            let _ = output.write_fmt(format_args!("\t\tf.write_str(\"{}\")", self.printer_mid));
        }
        let _ = output.write_str("\n\t}\n}\n");
        let _ = output.write_str(&self.impl_head);
        let _ = output.write_str(&self.var_decl_head);
        let _ = output
            .write_str("\t\twhile let Some(next) = args.next() {\n\t\t\tmatch next.as_slice() {\n");
        if let Some(mt) = &self.match_tail {
            let _ = self.match_head.write_fmt(format_args!("\
\t\t\t\tb\"-h\\0\" | b\"--help\\0\" => {{ return Err(tiny_std::unix::cli::ArgParseError::new_cause_str(Self::help_printer(), \"\")?)}},
                no_match => {{
                    {mt}
                }},
"));
        } else {
            let _ = self.match_head.write_str("\
\t\t\t\tb\"-h\\0\" | b\"--help\\0\" => { return Err(tiny_std::unix::cli::ArgParseError::new_cause_str(Self::help_printer(), \"\")?) },
                no_match => {
                    return Err(tiny_std::unix::cli::ArgParseError::new_cause_fmt(Self::help_printer(), format_args!(\"Unrecognized argument: {:?}\", core::str::from_utf8(no_match)))?);
                },
");
        }
        let _ = output.write_str(&self.match_head);
        let _ = output.write_str("\t\t\t}\n\t\t}\n");
        let _ = output.write_fmt(format_args!(
            "\t\tOk(Self {{\n\t\t\t{}\t\t}})\n\t}}\n}}\n",
            &self.struct_out
        ));
        output
    }
}
fn gen_printer_head(help_printer_name: &str) -> String {
    format!(
        "\
pub struct {help_printer_name};
impl core::fmt::Display for {help_printer_name} {{
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {{
"
    )
}

fn gen_impl_head(name: &str, help_printer_name: &str) -> String {
    format!("\
impl tiny_std::unix::cli::ArgParse for {name} {{
    type HelpPrinter = {help_printer_name};
    #[inline]
    fn help_printer() -> &'static Self::HelpPrinter {{
        &{help_printer_name}
    }}
    fn arg_parse(args: &mut impl Iterator<Item=&'static tiny_std::UnixStr>) -> core::result::Result<Self, tiny_std::unix::cli::ArgParseError> {{
")
}
fn member_try_assign(m: &ParsedField) -> String {
    match m.package {
        FieldPackageKind::None | FieldPackageKind::Option => {
            format!("{} = Some({})", m.name, member_as_convert(m))
        }
        FieldPackageKind::Vec => {
            format!("{}.push({})", m.name, member_as_convert(m))
        }
    }
}

fn member_as_convert(m: &ParsedField) -> String {
    let mut out = String::from("{\n");
    let _ = out.write_str("\t\t\t\t\t\tlet Some(next_arg) = args.next() else {\n");
    let _ = out.write_fmt(format_args!(
        "\t\t\t\t\t\t\treturn Err(tiny_std::unix::cli::ArgParseError::new_cause_str(Self::help_printer(), \"Expected argument following '{}'.\")?);\n",
        m.as_lit_match()
    ));
    let _ = out.write_str("\t\t\t\t\t\t};\n");
    match &m.ty {
        FieldTy::UnixStr => {
            let _ = out.write_str("\t\t\t\t\t\tnext_arg\n");
            let _ = out.write_str("\t\t\t\t\t\t}");
        }
        FieldTy::Str => {
            let _ = out.write_str("\t\t\t\t\t\tmatch next_arg.as_str() {\n");
            let _ = out.write_str("\t\t\t\t\t\t\tOk(s) => s,\n");
            let _ = out.write_str("\t\t\t\t\t\t\tErr(e) => {\n");
            let _ = out.write_fmt(format_args!(
                "\t\t\t\t\t\t\t\treturn Err(tiny_std::unix::cli::ArgParseError::new_cause_str(Self::help_printer(), \"Failed to parse argument at '{}' as utf8-str\")?);\n",
                m.as_lit_match()
            ));
            let _ = out.write_str("\t\t\t\t\t\t\t},\n");

            let _ = out.write_str("\t\t\t\t\t\t}\n");
            let _ = out.write_str("\t\t\t\t\t}");
        }
        FieldTy::Unknown(ty) => {
            let _ = out.write_str("\t\t\t\t\t\tlet next_str_arg = match next_arg.as_str() {\n");
            let _ = out.write_str("\t\t\t\t\t\t\tOk(s) => s,\n");
            let _ = out.write_str("\t\t\t\t\t\t\tErr(e) => {\n");
            let _ = out.write_fmt(format_args!(
                "\t\t\t\t\t\t\t\treturn Err(tiny_std::unix::cli::ArgParseError::new_cause_str(Self::help_printer(), \"Failed to parse argument at '{}' as utf8-str\")?);\n",
                m.as_lit_match()
            ));
            let _ = out.write_str("\t\t\t\t\t\t\t},\n");
            let _ = out.write_str("\t\t\t\t\t\t};\n");
            let _ = out.write_fmt(format_args!(
                "\t\t\t\t\t\tmatch <{ty} as core::str::FromStr>::from_str(next_str_arg) {{\n"
            ));
            let _ = out.write_str("\t\t\t\t\t\t\tOk(s) => s,\n");
            let _ = out.write_str("\t\t\t\t\t\t\tErr(e) => {\n");
            let _ = out.write_fmt(format_args!(
                "\t\t\t\t\t\t\t\treturn Err(tiny_std::unix::cli::ArgParseError::new_cause_fmt(Self::help_printer(), format_args!(\"Failed to convert argument at '{}' from str: {{e}}\"))?);\n",
                m.as_lit_match()
            ));
            let _ = out.write_str("\t\t\t\t\t\t\t}\n");
            let _ = out.write_str("\t\t\t\t\t\t}\n\t\t\t\t\t}");
        }
    }
    out
}
