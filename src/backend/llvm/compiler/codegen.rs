#![allow(clippy::collapsible_if)]

use std::fmt::Display;

use crate::backend::llvm::compiler::{
    binaryop, builtins, declarations, expressions, ptrgen, statements,
};
use crate::backend::types::{repr::LLVMFunction, traits::AssemblerFunctionExtensions};
use crate::core::console::logging::{self, LoggingType};
use crate::frontend::types::ast::metadata::local::LocalMetadata;
use crate::frontend::types::parser::repr::{
    AssemblerFunctionRepresentation, FunctionParameter, FunctionRepresentation,
};
use crate::frontend::types::parser::stmts::traits::ThrushAttributesExtensions;
use crate::frontend::types::parser::stmts::types::ThrushAttributes;
use crate::frontend::typesystem::types::Type;

use crate::frontend::types::ast::Ast;

use super::super::compiler::attributes::LLVMAttribute;

use super::{
    attributes::{AttributeBuilder, LLVMAttributeApplicant},
    context::LLVMCodeGenContext,
    conventions::CallConvention,
    typegen, valuegen,
};

use inkwell::InlineAsmDialect;
use inkwell::values::{BasicMetadataValueEnum, PointerValue};
use inkwell::{
    basic_block::BasicBlock,
    builder::Builder,
    context::Context,
    module::{Linkage, Module},
    types::FunctionType,
    values::{BasicValueEnum, FunctionValue},
};

pub struct LLVMCodegen<'a, 'ctx> {
    context: &'a mut LLVMCodeGenContext<'a, 'ctx>,
    ast: &'ctx [Ast<'ctx>],
}

impl<'a, 'ctx> LLVMCodegen<'a, 'ctx> {
    pub fn generate(context: &'a mut LLVMCodeGenContext<'a, 'ctx>, ast: &'ctx [Ast<'ctx>]) {
        Self { context, ast }.compile();
    }

    fn compile(&mut self) {
        self.declare_forward();

        self.ast.iter().for_each(|ast| {
            self.codegen(ast);
        });
    }

    fn codegen(&mut self, decl: &'ctx Ast) {
        self.codegen_declaration(decl);
    }

    pub fn codegen_declaration(&mut self, decl: &'ctx Ast) {
        /* ######################################################################


            LLVM CODEGEN | DECLARATIONS - START


        ########################################################################*/

        match decl {
            Ast::EntryPoint { body, .. } => {
                let entrypoint: FunctionValue = self.entrypoint();

                self.context.set_current_fn(entrypoint);

                self.codegen_block(body);
            }

            Ast::Function { body, .. } => {
                if body.is_null() {
                    return;
                }

                self.compile_function(decl);
            }

            Ast::GlobalAssembler { asm, .. } => {
                let llvm_module: &Module = self.context.get_llvm_module();

                llvm_module.set_inline_assembly(asm);
            }

            _ => (),
        }

        /* ######################################################################


            LLVM CODEGEN | DECLARATIONS - END


        ########################################################################*/
    }

    pub fn codegen_block(&mut self, stmt: &'ctx Ast) {
        /* ######################################################################


            LLVM CODEGEN | CODE BLOCK - START


        ########################################################################*/

        match stmt {
            Ast::Block { stmts, .. } => {
                self.context.begin_scope();

                stmts.iter().for_each(|stmt| {
                    self.codegen_block(stmt);
                });

                self.context.end_scope();
            }

            stmt => self.stmt(stmt),
        }

        /* ######################################################################


            LLVM CODEGEN | CODE BLOCK - END


        ########################################################################*/
    }

    fn stmt(&mut self, stmt: &'ctx Ast) {
        self.codegen_conditionals(stmt);
    }

    pub fn codegen_conditionals(&mut self, stmt: &'ctx Ast) {
        /* ######################################################################


            LLVM CODEGEN | IF - ELIF - ELSE - START


        ########################################################################*/

        match stmt {
            Ast::If { .. } => statements::conditional::compile(self, stmt),

            stmt => self.codegen_loops(stmt),
        }

        /* ######################################################################


            LLVM CODEGEN | IF - ELIF - ELSE - END


        ########################################################################*/
    }

    pub fn codegen_loops(&mut self, stmt: &'ctx Ast) {
        /* ######################################################################


            LLVM CODEGEN | LOOPS - START


        ########################################################################*/

        match stmt {
            // Loops
            Ast::While { .. } => statements::loops::whileloop::compile(self, stmt),
            Ast::Loop { .. } => statements::loops::infloop::compile(self, stmt),
            Ast::For { .. } => statements::loops::forloop::compile(self, stmt),

            // Control Flow
            Ast::Break { .. } => statements::loops::controlflow::loopbreak::compile(self, stmt),
            Ast::Continue { .. } => statements::loops::controlflow::loopjump::compile(self, stmt),

            stmt => self.codegen_variables(stmt),
        }

        /* ######################################################################


            LLVM CODEGEN | LOOPS - END


        ########################################################################*/
    }

    pub fn codegen_variables(&mut self, stmt: &'ctx Ast) {
        /* ######################################################################


            LLVM CODEGEN | VARIABLES - START


        ########################################################################*/

        match stmt {
            Ast::Local {
                name,
                ascii_name,
                kind,
                value,
                attributes,
                metadata,
                ..
            } => {
                let metadata: &LocalMetadata = metadata;

                if metadata.is_undefined() {
                    self.context.new_local(name, ascii_name, kind, attributes);
                    return;
                }

                statements::local::compile(
                    self.context,
                    (name, ascii_name, kind, value, attributes),
                );
            }

            Ast::Const {
                name,
                ascii_name,
                kind,
                value,
                ..
            } => {
                statements::constant::compile_local(self.context, (name, ascii_name, kind, value));
            }

            Ast::Static {
                name,
                ascii_name,
                kind,
                value,
                metadata,
                ..
            } => {
                statements::staticvar::compile_local(
                    self.context,
                    (name, ascii_name, kind, value, *metadata),
                );
            }

            Ast::LLI {
                name, kind, value, ..
            } => {
                statements::lli::compile(self.context, name, kind, value);
            }

            stmt => self.codegen_terminator(stmt),
        }

        /* ######################################################################


            LLVM CODEGEN | VARIABLES - END


        ########################################################################*/
    }

    pub fn codegen_terminator(&mut self, stmt: &'ctx Ast) {
        /* ######################################################################


            LLVM CODEGEN | TERMINATOR - START


        ########################################################################*/

        match stmt {
            Ast::Return { .. } => {
                statements::terminator::compile(self, stmt);
            }

            any => self.expressions(any),
        }

        /* ######################################################################


            LLVM CODEGEN | TERMINATOR - END


        ########################################################################*/
    }

    fn expressions(&mut self, stmt: &'ctx Ast) {
        self.codegen_loose(stmt);
    }

    pub fn codegen_loose(&mut self, stmt: &'ctx Ast) {
        /* ######################################################################


            LLVM CODEGEN | LOOSE EXPRESSIONS || STATEMENTS - START


        ########################################################################*/

        match stmt {
            Ast::UnaryOp {
                operator,
                kind,
                expression,
                ..
            } => {
                expressions::unaryop::compile(self.context, (operator, kind, expression), None);
            }

            Ast::BinaryOp {
                left,
                operator,
                right,
                kind,
                ..
            } => {
                if kind.is_integer_type() {
                    binaryop::integer::compile(self.context, (left, operator, right), None);
                }

                if kind.is_float_type() {
                    binaryop::float::compile(self.context, (left, operator, right), None);
                }

                if kind.is_bool_type() {
                    binaryop::boolean::compile(self.context, (left, operator, right), None);
                }

                if kind.is_ptr_type() {
                    binaryop::pointer::compile(self.context, (left, operator, right));
                }

                self::codegen_abort(format!(
                    "Could not compile binary operation with type '{}'.",
                    kind
                ));
            }

            Ast::Mut { .. } => {
                statements::mutation::compile(self.context, stmt);
            }

            Ast::Write { .. } => {
                valuegen::compile(self.context, stmt, None);
            }

            Ast::Call { .. } => {
                valuegen::compile(self.context, stmt, None);
            }

            Ast::AsmValue { .. } => {
                valuegen::compile(self.context, stmt, None);
            }

            Ast::Builtin { builtin, .. } => {
                builtins::compile(self.context, builtin, None);
            }

            _ => (),
        }

        /* ######################################################################


            LLVM CODEGEN | LOOSE EXPRESSIONS || STATEMENTS - END


        ########################################################################*/
    }

    fn entrypoint(&mut self) -> FunctionValue<'ctx> {
        let llvm_module: &Module = self.context.get_llvm_module();
        let llvm_context: &Context = self.context.get_llvm_context();
        let llvm_builder: &Builder = self.context.get_llvm_builder();

        let main_type: FunctionType = llvm_context.i32_type().fn_type(&[], false);
        let main: FunctionValue = llvm_module.add_function("main", main_type, None);

        let main_block: BasicBlock = llvm_context.append_basic_block(main, "");

        llvm_builder.position_at_end(main_block);

        main
    }

    fn compile_function_parameter(
        &mut self,
        llvm_function: FunctionValue<'ctx>,
        parameter: FunctionParameter<'ctx>,
    ) {
        let name: &str = parameter.0;
        let ascii_name: &str = parameter.1;

        let kind: &Type = parameter.2;
        let position: u32 = parameter.3;

        if let Some(value) = llvm_function.get_nth_param(position) {
            self.context.new_parameter(name, ascii_name, kind, value);
        } else {
            self::codegen_abort(
                "The value of a parameter of an LLVM function could not be obtained at code generation time.",
            );
        }
    }

    fn compile_function(&mut self, raw_function: &'ctx Ast) {
        let llvm_context: &Context = self.context.get_llvm_context();
        let llvm_builder: &Builder = self.context.get_llvm_builder();

        let function: FunctionRepresentation = raw_function.as_function_representation();

        let name: &str = function.0;
        let function_type: &Type = function.2;
        let parameters: &[Ast<'ctx>] = function.3;
        let body: &Ast = function.5;

        let get_llvm_function: LLVMFunction = self.context.get_table().get_function(name);

        let llvm_function_value: FunctionValue = get_llvm_function.0;

        let llvm_function: FunctionValue = llvm_function_value;

        let entry: BasicBlock = llvm_context.append_basic_block(llvm_function, "");

        llvm_builder.position_at_end(entry);

        self.context.set_current_fn(llvm_function);

        parameters.iter().for_each(|parameter| {
            if let Ast::FunctionParameter {
                name,
                ascii_name,
                kind,
                position,
                ..
            } = parameter
            {
                self.compile_function_parameter(llvm_function, (name, ascii_name, kind, *position));
            }
        });

        self.codegen_block(body);

        if !body.has_return() && function_type.is_void_type() {
            if llvm_builder.build_return(None).is_err() {
                self::codegen_abort(
                    "Unable to build the return instruction at code generation time.",
                );
            }
        }

        self.context.unset_current_function();
    }

    /* ######################################################################


        CODEGEN FORWARD DECLARATION | START


    ########################################################################*/

    fn declare_forward(&mut self) {
        self.ast.iter().for_each(|ast| {
            if ast.is_asm_function() {
                self.declare_asm_function(ast);
            }

            if ast.is_function() {
                self.declare_function(ast);
            }

            if ast.is_constant() {
                declarations::constant::compile_global(self.context, ast.as_global_constant());
            }

            if ast.is_static() {
                declarations::stativar::compile_global(self.context, ast.as_global_static());
            }
        });
    }

    /* ######################################################################


        CODEGEN FORWARD DECLARATION | END


    ########################################################################*/

    fn declare_asm_function(&mut self, stmt: &'ctx Ast) {
        let llvm_module: &Module = self.context.get_llvm_module();
        let llvm_context: &Context = self.context.get_llvm_context();
        let llvm_builder: &Builder = self.context.get_llvm_builder();

        let asm_function: AssemblerFunctionRepresentation = stmt.as_asm_function_representation();

        let asm_function_name: &str = asm_function.0;
        let asm_function_ascii_name: &str = asm_function.1;
        let asm_function_assembler: String = asm_function.2.to_string();
        let asm_function_constraints: String = asm_function.3.to_string();
        let asm_function_return_type: &Type = asm_function.4;
        let asm_function_parameters: &[Ast] = asm_function.5;
        let asm_function_parameters_types: &[Type] = asm_function.6;
        let asm_function_attributes: &ThrushAttributes = asm_function.7;

        let mut call_convention: u32 = CallConvention::Standard as u32;
        let mut syntax: InlineAsmDialect = InlineAsmDialect::Intel;

        let sideeffects: bool = asm_function_attributes.has_asmsideffects_attribute();
        let align_stack: bool = asm_function_attributes.has_asmalignstack_attribute();
        let can_throw: bool = asm_function_attributes.has_asmthrow_attribute();
        let is_public: bool = asm_function_attributes.has_public_attribute();

        asm_function_attributes.iter().for_each(|attribute| {
            if let LLVMAttribute::Convention(call_conv, _) = attribute {
                call_convention = (*call_conv) as u32;
            }

            if let LLVMAttribute::AsmSyntax(new_syntax, ..) = *attribute {
                syntax = str::assembler_syntax_attr_to_inline_assembler_dialect(new_syntax);
            }
        });

        let truly_function_name: String = format!("__asm_fn_{}", asm_function_ascii_name);

        let asm_function_type: FunctionType = typegen::function_type(
            self.context,
            asm_function_return_type,
            asm_function_parameters,
            false,
        );

        let asm_function_ptr: PointerValue = llvm_context.create_inline_asm(
            asm_function_type,
            asm_function_assembler,
            asm_function_constraints,
            sideeffects,
            align_stack,
            Some(syntax),
            can_throw,
        );

        let llvm_asm_function: FunctionValue =
            llvm_module.add_function(&truly_function_name, asm_function_type, None);

        if !is_public {
            llvm_asm_function.set_linkage(Linkage::LinkerPrivate);
        }

        let original_block: Option<BasicBlock> = llvm_builder.get_insert_block();

        let entry: BasicBlock = llvm_context.append_basic_block(llvm_asm_function, "");

        llvm_builder.position_at_end(entry);

        let args: Vec<BasicMetadataValueEnum> = llvm_asm_function
            .get_param_iter()
            .map(|param| param.into())
            .collect();

        if let Ok(asm_fn_call) =
            llvm_builder.build_indirect_call(asm_function_type, asm_function_ptr, &args, "")
        {
            match (
                asm_function_return_type.is_void_type(),
                asm_fn_call.try_as_basic_value().left(),
            ) {
                (false, Some(return_value)) => {
                    llvm_builder.build_return(Some(&return_value))
            .map_err(|_| {
                self::codegen_abort(
                    "Failed to create return terminator with value in assembly function generation.");
            })
            .ok();
                }
                _ => {
                    llvm_builder.build_return(None)
            .map_err(|_| {
                self::codegen_abort("Failed to create void return terminator in assembly function generation.",);
            })
            .ok();
                }
            }
        } else {
            self::codegen_abort("Unable to create indirect call for call assembly function.");
        }

        if let Some(original_block) = original_block {
            llvm_builder.position_at_end(original_block);
        }

        self.context.new_function(
            asm_function_name,
            (
                llvm_asm_function,
                asm_function_parameters_types,
                call_convention,
            ),
        );
    }

    fn declare_function(&mut self, stmt: &'ctx Ast) {
        let llvm_module: &Module = self.context.get_llvm_module();
        let llvm_context: &Context = self.context.get_llvm_context();

        let function: FunctionRepresentation = stmt.as_function_representation();

        let name: &str = function.0;
        let ascii_name: &str = function.1;
        let function_type: &Type = function.2;
        let function_parameters: &[Ast<'ctx>] = function.3;
        let function_parameters_types: &[Type] = function.4;
        let attributes: &ThrushAttributes = function.6;

        let ignore_args: bool = attributes.has_ignore_attribute();
        let is_public: bool = attributes.has_public_attribute();

        let mut extern_name: Option<&str> = None;
        let mut convention: u32 = CallConvention::Standard as u32;

        attributes.iter().for_each(|attribute| match attribute {
            LLVMAttribute::Extern(name, ..) => {
                extern_name = Some(name);
            }

            LLVMAttribute::Convention(conv, _) => {
                convention = (*conv) as u32;
            }
            _ => (),
        });

        let external_name: &str = if let Some(ffi_name) = extern_name {
            ffi_name
        } else {
            ascii_name
        };

        let function_type: FunctionType = typegen::function_type(
            self.context,
            function_type,
            function_parameters,
            ignore_args,
        );

        let llvm_function: FunctionValue =
            llvm_module.add_function(external_name, function_type, None);

        let mut attribute_builder: AttributeBuilder = AttributeBuilder::new(
            llvm_context,
            attributes,
            LLVMAttributeApplicant::Function(llvm_function),
        );

        attribute_builder.add_function_attributes(&mut convention);

        if !is_public && extern_name.is_none() {
            llvm_function.set_linkage(Linkage::LinkerPrivate);
        }

        self.context.set_current_fn(llvm_function);

        self.context
            .new_function(name, (llvm_function, function_parameters_types, convention));
    }
}

impl<'a, 'ctx> LLVMCodegen<'a, 'ctx> {
    pub fn get_mut_context(&mut self) -> &mut LLVMCodeGenContext<'a, 'ctx> {
        self.context
    }

    pub fn get_context(&self) -> &LLVMCodeGenContext<'a, 'ctx> {
        self.context
    }
}

pub fn compile_expr<'ctx>(
    context: &mut LLVMCodeGenContext<'_, 'ctx>,
    expr: &'ctx Ast,
    cast_type: Option<&Type>,
    hl_ptr: bool,
) -> BasicValueEnum<'ctx> {
    if cast_type.is_some_and(|cast| cast.is_ptr_type() || (cast.is_mut_type() && hl_ptr)) {
        ptrgen::compile(context, expr, cast_type)
    } else {
        valuegen::compile(context, expr, cast_type)
    }
}

fn codegen_abort<T: Display>(message: T) {
    logging::log(LoggingType::BackendBug, &format!("{}", message));
}
