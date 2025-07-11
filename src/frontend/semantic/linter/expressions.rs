use crate::{
    core::errors::{position::CompilationPosition, standard::ThrushCompilerIssue},
    frontend::{
        lexer::span::Span,
        semantic::linter::Linter,
        types::{lexer::ThrushType, parser::stmts::stmt::ThrushStatement},
    },
};

pub fn analyze_expression<'linter>(linter: &mut Linter<'linter>, node: &'linter ThrushStatement) {
    match node {
        ThrushStatement::Group { expression, .. } => {
            linter.analyze_stmt(expression);
        }

        ThrushStatement::BinaryOp { left, right, .. } => {
            linter.analyze_stmt(left);
            linter.analyze_stmt(right);
        }

        ThrushStatement::UnaryOp { expression, .. } => {
            linter.analyze_stmt(expression);
        }

        ThrushStatement::AsmValue { args, .. } => {
            args.iter().for_each(|arg| {
                linter.analyze_stmt(arg);
            });
        }

        ThrushStatement::Index {
            index_to,
            indexes,
            span,
            ..
        } => {
            indexes.iter().for_each(|indexe| {
                linter.analyze_stmt(indexe);
            });

            if let Some(any_reference) = &index_to.0 {
                let name = any_reference.0;

                if let Some(local) = linter.symbols.get_local_info(name) {
                    local.1 = true;
                    return;
                }

                if let Some(parameter) = linter.symbols.get_parameter_info(name) {
                    parameter.1 = true;
                    return;
                }

                if let Some(lli) = linter.symbols.get_lli_info(name) {
                    lli.1 = true;
                    return;
                }

                if let Some(constant) = linter.symbols.get_constant_info(name) {
                    constant.1 = true;
                    return;
                }

                linter.add_bug(ThrushCompilerIssue::Bug(
                    String::from("Reference not caught"),
                    format!("Could not get reference with name '{}'.", name),
                    *span,
                    CompilationPosition::Linter,
                    line!(),
                ));
            }

            if let Some(expr) = &index_to.1 {
                linter.analyze_stmt(expr);
            }
        }

        ThrushStatement::Constructor {
            name,
            arguments,
            span,
            ..
        } => {
            let constructor_args: &[(&str, ThrushStatement, ThrushType, u32)] = &arguments.1;

            constructor_args.iter().for_each(|arg| {
                let stmt: &ThrushStatement = &arg.1;
                linter.analyze_stmt(stmt);
            });

            if let Some(structure) = linter.symbols.get_struct_info(name) {
                structure.2 = true;
                return;
            }

            linter.add_bug(ThrushCompilerIssue::Bug(
                String::from("Structure not caught"),
                format!("Could not get named struct with name '{}'.", name),
                *span,
                CompilationPosition::Linter,
                line!(),
            ));
        }
        ThrushStatement::Call {
            name, span, args, ..
        } => {
            if let Some(function) = linter.symbols.get_function_info(name) {
                function.1 = true;

                args.iter().for_each(|arg| {
                    linter.analyze_stmt(arg);
                });

                return;
            }

            if let Some(asm_function) = linter.symbols.get_asm_function_info(name) {
                asm_function.1 = true;

                args.iter().for_each(|arg| {
                    linter.analyze_stmt(arg);
                });

                return;
            }

            linter.add_bug(ThrushCompilerIssue::Bug(
                String::from("Call not caught"),
                format!("Could not get named function '{}'.", name),
                *span,
                CompilationPosition::Linter,
                line!(),
            ));
        }

        ThrushStatement::Reference { name, span, .. } => {
            if let Some(local) = linter.symbols.get_local_info(name) {
                local.1 = true;
                return;
            }

            if let Some(parameter) = linter.symbols.get_parameter_info(name) {
                parameter.1 = true;
                return;
            }

            if let Some(lli) = linter.symbols.get_lli_info(name) {
                lli.1 = true;
                return;
            }

            if let Some(constant) = linter.symbols.get_constant_info(name) {
                constant.1 = true;
                return;
            }

            linter.add_bug(ThrushCompilerIssue::Bug(
                String::from("Reference not caught"),
                format!("Could not get reference with name '{}'.", name),
                *span,
                CompilationPosition::Linter,
                line!(),
            ));
        }

        ThrushStatement::FixedArray { items, .. } | ThrushStatement::Array { items, .. } => {
            items.iter().for_each(|item| {
                linter.analyze_stmt(item);
            });
        }

        ThrushStatement::Mut { source, span, .. } => {
            if let Some(any_reference) = &source.0 {
                let name: &str = any_reference.0;

                if let Some(local) = linter.symbols.get_local_info(name) {
                    local.1 = true;
                    return;
                }

                if let Some(parameter) = linter.symbols.get_parameter_info(name) {
                    parameter.1 = true;
                    return;
                }

                if let Some(lli) = linter.symbols.get_lli_info(name) {
                    lli.1 = true;
                    return;
                }

                if let Some(constant) = linter.symbols.get_constant_info(name) {
                    constant.1 = true;
                    return;
                }

                linter.add_bug(ThrushCompilerIssue::Bug(
                    String::from("Mutable expression not caught"),
                    format!("Could not mutable reference with name '{}'.", name),
                    *span,
                    CompilationPosition::Linter,
                    line!(),
                ));
            }

            if let Some(expr) = &source.1 {
                linter.analyze_stmt(expr);
            }
        }

        ThrushStatement::EnumValue {
            name, value, span, ..
        } => {
            if let Some((enum_name, field_name)) = linter.symbols.split_enum_field_name(name) {
                if let Some(union) = linter.symbols.get_enum_info(enum_name) {
                    union.2 = true;
                }

                if let Some(enum_field) = linter.symbols.get_enum_field_info(enum_name, field_name)
                {
                    enum_field.1 = true;
                }

                linter.analyze_stmt(value);

                return;
            }

            linter.add_bug(ThrushCompilerIssue::Bug(
                String::from("Enum value not caught"),
                format!("Could not get correct name of the enum field '{}'.", name),
                *span,
                CompilationPosition::Linter,
                line!(),
            ));
        }

        _ => {
            let span: Span = node.get_span();

            linter.add_bug(ThrushCompilerIssue::Bug(
                "Expression not caught".into(),
                "Expression could not be caught for processing.".into(),
                span,
                CompilationPosition::Linter,
                line!(),
            ));
        }
    }
}
