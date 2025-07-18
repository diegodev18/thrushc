use std::fmt::Display;

use crate::backend::llvm::compiler::constgen::{self};
use crate::backend::llvm::compiler::context::LLVMCodeGenContext;
use crate::core::console::logging::{self, LoggingType};
use crate::frontend::lexer::tokentype::TokenType;
use crate::frontend::types::ast::Ast;
use crate::frontend::types::parser::repr::UnaryOperation;
use crate::frontend::typesystem::types::Type;

use inkwell::AddressSpace;
use inkwell::types::FloatType;
use inkwell::values::{BasicValueEnum, FloatValue, IntValue};

pub fn compile<'ctx>(
    context: &mut LLVMCodeGenContext<'_, 'ctx>,
    unary: UnaryOperation<'ctx>,
    cast: &Type,
) -> BasicValueEnum<'ctx> {
    match unary {
        (TokenType::PlusPlus | TokenType::MinusMinus, _, expr) => {
            self::compile_increment_decrement(context, unary.0, expr, cast)
        }

        (TokenType::Bang, _, expr) => self::compile_logical_negation(context, expr, cast),
        (TokenType::Minus, _, expr) => self::compile_arithmetic_negation(context, expr, cast),

        _ => {
            self::codegen_abort("Unsupported unary operation pattern encountered.");
            self::compile_null_ptr(context)
        }
    }
}

fn compile_increment_decrement<'ctx>(
    context: &mut LLVMCodeGenContext<'_, 'ctx>,
    operator: &TokenType,
    expression: &'ctx Ast,
    cast: &Type,
) -> BasicValueEnum<'ctx> {
    let value: BasicValueEnum = constgen::compile(context, expression, cast);
    let kind: &Type = expression.get_type_unwrapped();

    match kind {
        kind if kind.is_integer_type() => {
            let int: IntValue = value.into_int_value();

            let modifier: IntValue = int.get_type().const_int(1, false);

            match operator {
                TokenType::PlusPlus => int.const_add(modifier).into(),
                TokenType::MinusMinus => int.const_sub(modifier).into(),

                _ => {
                    self::codegen_abort(
                        "Unknown operator compared to increment and decrement in unary operation.",
                    );
                    self::compile_null_ptr(context)
                }
            }
        }
        _ => {
            let float: FloatValue = value.into_float_value();

            match operator {
                TokenType::PlusPlus => {
                    if let Some(constant_float) = float.get_constant() {
                        let value: f64 = constant_float.0;
                        let new_value: f64 = value + 1.0;

                        return float.get_type().const_float(new_value).into();
                    }

                    float.into()
                }

                TokenType::MinusMinus => {
                    if let Some(constant_float) = float.get_constant() {
                        let value: f64 = constant_float.0;
                        let new_value: f64 = value - 1.0;

                        return float.get_type().const_float(new_value).into();
                    }

                    float.into()
                }

                _ => {
                    self::codegen_abort(
                        "Unknown operator compared to increment and decrement in unary operation.",
                    );

                    self::compile_null_ptr(context)
                }
            }
        }
    }
}

fn compile_logical_negation<'ctx>(
    context: &mut LLVMCodeGenContext<'_, 'ctx>,
    expr: &'ctx Ast,
    cast: &Type,
) -> BasicValueEnum<'ctx> {
    let value: BasicValueEnum = constgen::compile(context, expr, cast);
    let kind: &Type = expr.get_type_unwrapped();

    match kind {
        kind if kind.is_bool_type() => {
            let int: IntValue = value.into_int_value();
            int.const_not().into()
        }

        _ => {
            self::codegen_abort("Cannot perform a logical negation.");
            self::compile_null_ptr(context)
        }
    }
}

fn compile_arithmetic_negation<'ctx>(
    context: &mut LLVMCodeGenContext<'_, 'ctx>,
    expr: &'ctx Ast,
    cast: &Type,
) -> BasicValueEnum<'ctx> {
    let value: BasicValueEnum = constgen::compile(context, expr, cast);
    let kind: &Type = expr.get_type_unwrapped();

    match kind {
        kind if kind.is_integer_type() => {
            let int: IntValue = value.into_int_value();

            int.const_neg().into()
        }

        _ => {
            let mut float: FloatValue = value.into_float_value();
            let float_type: FloatType = float.get_type();

            if let Some(float_value) = float.get_constant() {
                float = float_type.const_float(-float_value.0);
            }

            float.into()
        }
    }
}

fn compile_null_ptr<'ctx>(context: &LLVMCodeGenContext<'_, 'ctx>) -> BasicValueEnum<'ctx> {
    context
        .get_llvm_context()
        .ptr_type(AddressSpace::default())
        .const_null()
        .into()
}

fn codegen_abort<T: Display>(message: T) {
    logging::log(LoggingType::BackendBug, &format!("{}", message));
}
