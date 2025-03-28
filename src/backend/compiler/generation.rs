use {
    super::{
        super::super::frontend::lexer::Type, binaryop, call, instruction::Instruction,
        objects::CompilerObjects, unaryop, utils,
    },
    inkwell::{
        AddressSpace,
        builder::Builder,
        context::Context,
        module::Module,
        values::{BasicValueEnum, FloatValue, IntValue, PointerValue},
    },
};

pub fn build_expression<'ctx>(
    module: &Module<'ctx>,
    builder: &Builder<'ctx>,
    context: &'ctx Context,
    instruction: &'ctx Instruction,
    casting_target: &Type,
    compiler_objects: &mut CompilerObjects<'ctx>,
) -> BasicValueEnum<'ctx> {
    if let Instruction::NullT = instruction {
        return context
            .ptr_type(AddressSpace::default())
            .const_null()
            .into();
    }

    if let Instruction::Str(str) = instruction {
        return utils::build_string_constant(module, builder, context, str).into();
    }

    if let Instruction::Float(kind, num, is_signed) = instruction {
        let mut float: FloatValue =
            utils::build_const_float(builder, context, kind, *num, *is_signed);

        if let Some(casted_float) =
            utils::float_autocast(kind, casting_target, None, float.into(), builder, context)
        {
            float = casted_float.into_float_value();
        }

        return float.into();
    }

    if let Instruction::Integer(kind, num, is_signed) = instruction {
        let mut integer: IntValue =
            utils::build_const_integer(context, kind, *num as u64, *is_signed);

        if let Some(casted_integer) =
            utils::integer_autocast(casting_target, kind, None, integer.into(), builder, context)
        {
            integer = casted_integer.into_int_value();
        }

        return integer.into();
    }

    if let Instruction::Char(char) = instruction {
        return context.i8_type().const_int(*char as u64, false).into();
    }

    if let Instruction::Boolean(bool) = instruction {
        return context.bool_type().const_int(*bool as u64, false).into();
    }

    if let Instruction::GEP { name, index } = instruction {
        let local: PointerValue = compiler_objects.get_local(name);

        let mut compiled_index: BasicValueEnum = build_expression(
            module,
            builder,
            context,
            index,
            &Type::U64,
            compiler_objects,
        );

        if let Some(casted_index) = utils::integer_autocast(
            &Type::U64,
            &index.get_data_type(),
            None,
            compiled_index,
            builder,
            context,
        ) {
            compiled_index = casted_index;
        }

        return unsafe {
            builder
                .build_in_bounds_gep(
                    context.ptr_type(AddressSpace::default()),
                    local,
                    &[compiled_index.into_int_value()],
                    "",
                )
                .unwrap()
                .into()
        };
    }

    if let Instruction::LocalRef { name, kind, .. } = instruction {
        let local: PointerValue = compiler_objects.get_local(name);

        if kind.is_float_type() {
            return builder
                .build_load(
                    utils::type_float_to_llvm_float_type(context, kind),
                    local,
                    "",
                )
                .unwrap();
        }

        if kind.is_integer_type() || kind.is_bool_type() {
            return builder
                .build_load(utils::type_int_to_llvm_int_type(context, kind), local, "")
                .unwrap();
        }

        if kind.is_static_str() {
            return local.into();
        }

        if kind.is_struct_type() {
            return builder.build_load(local.get_type(), local, "").unwrap();
        }

        if kind.is_raw_ptr() {
            return local.into();
        }

        unreachable!()
    }

    if let Instruction::BinaryOp {
        left,
        op,
        right,
        kind: binary_op_type,
        ..
    } = instruction
    {
        if binary_op_type.is_float_type() {
            return binaryop::float_binaryop(
                module,
                builder,
                context,
                (left, op, right),
                casting_target,
                compiler_objects,
            );
        }

        if binary_op_type.is_integer_type() {
            return binaryop::integer_binaryop(
                module,
                builder,
                context,
                (left, op, right),
                casting_target,
                compiler_objects,
            );
        }

        if binary_op_type.is_bool_type() {
            return binaryop::bool_binaryop(
                module,
                builder,
                context,
                (left, op, right),
                casting_target,
                compiler_objects,
            );
        }

        println!("{:?}", instruction);
        unreachable!()
    }

    if let Instruction::UnaryOp {
        op, value, kind, ..
    } = instruction
    {
        return unaryop::compile_unary_op(builder, context, (op, value, kind), compiler_objects);
    }

    if let Instruction::LocalMut { name, kind, value } = instruction {
        let ptr: PointerValue = compiler_objects.get_local(name);

        let compiled_expression: BasicValueEnum =
            build_expression(module, builder, context, value, kind, compiler_objects);

        builder.build_store(ptr, compiled_expression).unwrap();

        compiler_objects.insert(name, ptr);

        return compiled_expression;
    }

    if let Instruction::Call {
        name: call_name,
        args: call_arguments,
        kind: call_type,
        ..
    } = instruction
    {
        return call::build_call(
            module,
            builder,
            context,
            (call_name, call_type, call_arguments),
            compiler_objects,
        )
        .unwrap();
    }

    if let Instruction::Return(instruction, kind) = instruction {
        if kind.is_void_type() {
            builder.build_return(None).unwrap();

            return context
                .ptr_type(AddressSpace::default())
                .const_null()
                .into();
        }

        builder
            .build_return(Some(&build_expression(
                module,
                builder,
                context,
                instruction,
                kind,
                compiler_objects,
            )))
            .unwrap();

        return context
            .ptr_type(AddressSpace::default())
            .const_null()
            .into();
    }

    println!("{:?}", instruction);
    unreachable!()
}
