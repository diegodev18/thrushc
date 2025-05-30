use crate::middle::types::backend::llvm::types::LLVMUnaryOp;
use crate::middle::types::frontend::lexer::tokenkind::TokenKind;
use crate::middle::types::frontend::lexer::types::ThrushType;

use super::{Instruction, context::LLVMCodeGenContext, memory::SymbolAllocated, valuegen};

use super::typegen;

use inkwell::{
    builder::Builder,
    context::Context,
    values::{BasicValueEnum, FloatValue, IntValue},
};

pub fn unary_op<'ctx>(
    context: &LLVMCodeGenContext<'_, 'ctx>,
    unary: LLVMUnaryOp<'ctx>,
) -> BasicValueEnum<'ctx> {
    let llvm_builder: &Builder = context.get_llvm_builder();
    let llvm_context: &Context = context.get_llvm_context();

    if let (
        TokenKind::PlusPlus | TokenKind::MinusMinus,
        _,
        Instruction::LocalRef {
            name,
            kind: ref_type,
            ..
        },
    ) = unary
    {
        let symbol: SymbolAllocated = context.get_allocated_symbol(name);

        if ref_type.is_integer_type() {
            let int: IntValue = symbol.load(context).into_int_value();

            let modifier: IntValue =
                typegen::type_int_to_llvm_int_type(llvm_context, ref_type).const_int(1, false);

            let result: IntValue = if unary.0.is_plusplus_operator() {
                llvm_builder.build_int_nsw_add(int, modifier, "").unwrap()
            } else {
                llvm_builder.build_int_nsw_sub(int, modifier, "").unwrap()
            };

            symbol.store(context, result.into());

            return result.into();
        }

        let float: FloatValue = symbol.load(context).into_float_value();

        let modifier: FloatValue =
            typegen::type_float_to_llvm_float_type(llvm_context, ref_type).const_float(1.0);

        let result: FloatValue = if unary.0.is_plusplus_operator() {
            llvm_builder.build_float_add(float, modifier, "").unwrap()
        } else {
            llvm_builder.build_float_sub(float, modifier, "").unwrap()
        };

        symbol.store(context, result.into());

        return result.into();
    }

    if let (
        TokenKind::Bang,
        _,
        Instruction::LocalRef {
            name: ref_name,
            kind: ref_type,
            ..
        }
        | Instruction::ConstRef {
            name: ref_name,
            kind: ref_type,
            ..
        },
    ) = unary
    {
        let symbol: SymbolAllocated = context.get_allocated_symbol(ref_name);

        if ref_type.is_integer_type() || ref_type.is_bool_type() {
            let int: IntValue = symbol.load(context).into_int_value();
            let result: IntValue = llvm_builder.build_not(int, "").unwrap();

            return result.into();
        }

        let float: FloatValue = symbol.load(context).into_float_value();
        let result: FloatValue = llvm_builder.build_float_neg(float, "").unwrap();

        return result.into();
    }

    if let (
        TokenKind::Minus,
        _,
        Instruction::LocalRef {
            name,
            kind: ref_type,
            ..
        }
        | Instruction::ConstRef {
            name,
            kind: ref_type,
            ..
        },
    ) = unary
    {
        let symbol: SymbolAllocated = context.get_allocated_symbol(name);

        if ref_type.is_integer_type() {
            let int: IntValue = symbol.load(context).into_int_value();
            let result: IntValue = llvm_builder.build_not(int, "").unwrap();

            return result.into();
        }

        let float: FloatValue = symbol.load(context).into_float_value();
        let result: FloatValue = llvm_builder.build_float_neg(float, "").unwrap();

        return result.into();
    }

    if let (TokenKind::Bang, _, Instruction::Boolean(_, bool, _)) = unary {
        let value: IntValue =
            valuegen::integer(llvm_context, &ThrushType::Bool, *bool as u64, false);
        let result: IntValue = llvm_builder.build_not(value, "").unwrap();

        return result.into();
    }

    if let (TokenKind::Minus, _, Instruction::Integer(kind, num, is_signed, _)) = unary {
        let value: IntValue = valuegen::integer(llvm_context, kind, *num as u64, *is_signed);

        if !is_signed {
            let result: IntValue = llvm_builder.build_not(value, "").unwrap();
            return result.into();
        }

        return value.into();
    }

    if let (TokenKind::Minus, _, Instruction::Float(kind, number, is_signed, _)) = unary {
        let value: FloatValue =
            valuegen::float(llvm_builder, llvm_context, kind, *number, *is_signed);

        if !is_signed {
            let result: FloatValue = llvm_builder.build_float_neg(value, "").unwrap();
            return result.into();
        }

        return value.into();
    }

    unreachable!()
}
