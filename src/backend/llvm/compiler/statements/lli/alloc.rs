use inkwell::values::BasicValueEnum;

use crate::{
    backend::llvm::compiler::{
        context::LLVMCodeGenContext,
        memory::{self, LLVMAllocationSite},
    },
    frontend::{types::parser::stmts::sites::AllocationSite, typesystem::types::Type},
};

pub fn compile<'ctx>(
    context: &mut LLVMCodeGenContext<'_, 'ctx>,
    alloc: &Type,
    site_allocation: &AllocationSite,
) -> BasicValueEnum<'ctx> {
    let site: LLVMAllocationSite = site_allocation.to_llvm_allocation_site();

    memory::alloc_anon(site, context, alloc).into()
}
