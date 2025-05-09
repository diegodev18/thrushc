pub mod impls;
pub mod traits;

use crate::backend::llvm::compiler::attributes::LLVMAttribute;

use inkwell::values::FunctionValue;

use super::{
    instruction::Instruction,
    types::{TokenKind, Type},
};

pub type BinaryOp<'ctx> = (
    &'ctx Instruction<'ctx>,
    &'ctx TokenKind,
    &'ctx Instruction<'ctx>,
);

pub type UnaryOp<'ctx> = (&'ctx TokenKind, &'ctx Type, &'ctx Instruction<'ctx>);

pub type Local<'ctx> = (&'ctx str, &'ctx Type, &'ctx Instruction<'ctx>);

pub type FunctionCall<'ctx> = (&'ctx str, &'ctx Type, &'ctx [Instruction<'ctx>]);

pub type FunctionPrototype<'ctx> = (
    &'ctx str,
    &'ctx Type,
    &'ctx [Instruction<'ctx>],
    &'ctx [Type],
    &'ctx Instruction<'ctx>,
    &'ctx ThrushAttributes<'ctx>,
);

pub type Function<'ctx> = (FunctionValue<'ctx>, &'ctx [Type], u32);
pub type FunctionParameter<'ctx> = (&'ctx str, &'ctx Type, u32, bool);

pub type StructFields<'ctx> = (&'ctx str, Vec<(&'ctx str, Type, u32)>);

pub type Enum<'ctx> = (EnumFields<'ctx>, ThrushAttributes<'ctx>);

pub type EnumFields<'ctx> = Vec<(&'ctx str, Instruction<'ctx>)>;
pub type EnumField<'ctx> = (&'ctx str, Instruction<'ctx>);

pub type CustomType<'ctx> = (CustomTypeFields<'ctx>, ThrushAttributes<'ctx>);
pub type CustomTypeField<'ctx> = Type;
pub type CustomTypeFields<'ctx> = Vec<CustomTypeField<'ctx>>;

pub type Constructor<'instr> = (
    &'instr str,
    Vec<(&'instr str, Instruction<'instr>, Type, u32)>,
);

pub type ThrushAttributes<'ctx> = Vec<LLVMAttribute<'ctx>>;
