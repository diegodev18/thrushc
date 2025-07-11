use crate::{
    core::errors::standard::ThrushCompilerIssue,
    frontend::{
        lexer::{span::Span, token::Token, tokentype::TokenType},
        parser::ParserContext,
        types::{
            lexer::ThrushType,
            parser::{
                stmts::{
                    ident::ReferenceIdentificator,
                    stmt::ThrushStatement,
                    traits::{FoundSymbolEither, FoundSymbolExtension, TokenExtensions},
                },
                symbols::{
                    traits::{
                        ConstantSymbolExtensions, LLISymbolExtensions, LocalSymbolExtensions,
                    },
                    types::{
                        ConstantSymbol, FoundSymbolId, LLISymbol, LocalSymbol, ParameterSymbol,
                    },
                },
            },
        },
    },
};

pub fn build_reference<'instr>(
    parser_context: &mut ParserContext<'instr>,
    name: &'instr str,
    span: Span,
) -> Result<ThrushStatement<'instr>, ThrushCompilerIssue> {
    let symbol: FoundSymbolId = parser_context.get_symbols().get_symbols_id(name, span)?;

    if symbol.is_constant() {
        let const_id: &str = symbol.expected_constant(span)?;

        let constant: ConstantSymbol = parser_context
            .get_symbols()
            .get_const_by_id(const_id, span)?;

        let constant_type: ThrushType = constant.get_type();

        return Ok(ThrushStatement::Reference {
            name,
            kind: constant_type,
            span,
            identificator: ReferenceIdentificator::Constant,
            is_mutable: false,
            is_allocated: true,
        });
    }

    if symbol.is_parameter() {
        let parameter_id: &str = symbol.expected_parameter(span)?;

        let parameter: ParameterSymbol = parser_context
            .get_symbols()
            .get_parameter_by_id(parameter_id, span)?;

        let parameter_type: ThrushType = parameter.get_type();

        let is_mutable: bool = parameter.is_mutable();

        let is_allocated: bool = parameter_type.is_mut_type()
            || parameter_type.is_ptr_type()
            || parameter_type.is_address_type();

        return Ok(ThrushStatement::Reference {
            name,
            kind: parameter_type,
            span,
            is_mutable,
            identificator: ReferenceIdentificator::FunctionParameter,
            is_allocated,
        });
    }

    if symbol.is_lli() {
        let lli_id: (&str, usize) = symbol.expected_lli(span)?;

        let lli_name: &str = lli_id.0;
        let scope_idx: usize = lli_id.1;

        let parameter: &LLISymbol = parser_context
            .get_symbols()
            .get_lli_by_id(lli_name, scope_idx, span)?;

        let lli_type: ThrushType = parameter.get_type();

        let is_allocated: bool = lli_type.is_ptr_type() || lli_type.is_address_type();

        return Ok(ThrushStatement::Reference {
            name,
            kind: lli_type,
            span,
            is_mutable: false,
            identificator: ReferenceIdentificator::LowLevelInstruction,
            is_allocated,
        });
    }

    let local_position: (&str, usize) = symbol.expected_local(span)?;

    let local: &LocalSymbol =
        parser_context
            .get_symbols()
            .get_local_by_id(local_position.0, local_position.1, span)?;

    let is_mutable: bool = local.is_mutable();

    let local_type: ThrushType = local.get_type();

    let reference: ThrushStatement = ThrushStatement::Reference {
        name,
        kind: local_type.clone(),
        span,
        is_mutable,
        identificator: ReferenceIdentificator::Local,
        is_allocated: true,
    };

    if parser_context.match_token(TokenType::PlusPlus)?
        | parser_context.match_token(TokenType::MinusMinus)?
    {
        let operator_tk: &Token = parser_context.previous();
        let operator: TokenType = operator_tk.get_type();
        let span: Span = operator_tk.get_span();

        let unaryop: ThrushStatement = ThrushStatement::UnaryOp {
            operator,
            expression: reference.into(),
            kind: local_type,
            is_pre: false,
            span,
        };

        return Ok(unaryop);
    }

    Ok(reference)
}
