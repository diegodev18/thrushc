pub mod attributes;
pub mod binaryop;
pub mod builtins;
pub mod call;
pub mod codegen;
pub mod conventions;
pub mod generation;
pub mod impls;
pub mod instruction;
pub mod local;
pub mod misc;
pub mod objects;
pub mod traits;
pub mod types;
pub mod unaryop;
pub mod utils;

use {
    codegen::Codegen,
    inkwell::{builder::Builder, context::Context, module::Module},
    instruction::Instruction,
};

pub struct Compiler;

impl<'a, 'ctx> Compiler {
    #[inline]
    pub fn compile(
        module: &'a Module<'ctx>,
        builder: &'a Builder<'ctx>,
        context: &'ctx Context,
        instructions: &'ctx [Instruction<'ctx>],
    ) {
        Codegen::generate(module, builder, context, instructions);
    }
}
