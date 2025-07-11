#![allow(clippy::type_complexity)]

use std::{fmt::Display, rc::Rc};

use inkwell::{
    AddressSpace,
    values::{BasicValueEnum, PointerValue},
};

use crate::{
    backend::llvm::compiler::{context::LLVMCodeGenContext, memory, rawgen, valuegen},
    core::console::logging::{self, LoggingType},
    frontend::types::{lexer::ThrushType, parser::stmts::stmt::ThrushStatement},
};

pub fn compile<'ctx>(
    context: &mut LLVMCodeGenContext<'_, 'ctx>,
    write_to: &'ctx (
        Option<(&'ctx str, Rc<ThrushStatement<'ctx>>)>,
        Option<Rc<ThrushStatement<'ctx>>>,
    ),
    write_type: &'ctx ThrushType,
    write_value: &'ctx ThrushStatement,
) -> BasicValueEnum<'ctx> {
    let value: BasicValueEnum = valuegen::compile(context, write_value, Some(write_type));

    match write_to {
        (Some((name, _)), _) => {
            let symbol = context.get_allocated_symbol(name);
            symbol.store(context, value);
            self::compile_null_ptr(context)
        }
        (_, Some(expr)) => {
            let ptr: PointerValue = rawgen::compile(context, expr, None).into_pointer_value();

            memory::store_anon(context, ptr, value);

            self::compile_null_ptr(context)
        }
        _ => {
            self::codegen_abort("Invalid write target in expression");
            self::compile_null_ptr(context)
        }
    }
}

fn codegen_abort<T: Display>(message: T) {
    logging::log(
        LoggingType::Bug,
        &format!("CODE GENERATION: '{}'.", message),
    );
}

fn compile_null_ptr<'ctx>(context: &LLVMCodeGenContext<'_, 'ctx>) -> BasicValueEnum<'ctx> {
    context
        .get_llvm_context()
        .ptr_type(AddressSpace::default())
        .const_null()
        .into()
}
