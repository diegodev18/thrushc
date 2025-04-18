use {
    super::{
        super::{
            super::super::frontend::lexer::{TokenKind, Type},
            Instruction,
            objects::CompilerObjects,
            types::BinaryOp,
        },
        float::float_binaryop,
        integer::{build_int_op, compile_integer_binaryop},
    },
    inkwell::{builder::Builder, context::Context, module::Module, values::BasicValueEnum},
};

pub fn bool_binaryop<'ctx>(
    module: &Module<'ctx>,
    builder: &Builder<'ctx>,
    context: &'ctx Context,
    binary: BinaryOp<'ctx>,
    target_basic_type: &Type,
    compiler_objects: &mut CompilerObjects<'ctx>,
) -> BasicValueEnum<'ctx> {
    if let (
        Instruction::Integer(_, _, _) | Instruction::Float(_, _, _) | Instruction::Boolean(_),
        TokenKind::BangEq
        | TokenKind::EqEq
        | TokenKind::LessEq
        | TokenKind::Less
        | TokenKind::Greater
        | TokenKind::GreaterEq
        | TokenKind::And
        | TokenKind::Or,
        Instruction::Integer(_, _, _) | Instruction::Float(_, _, _) | Instruction::Boolean(_),
    ) = binary
    {
        if binary.0.get_basic_type().is_float_type() {
            return float_binaryop(
                module,
                builder,
                context,
                binary,
                target_basic_type,
                compiler_objects,
            );
        } else if binary.0.get_basic_type().is_integer_type()
            || binary.0.get_basic_type().is_bool_type()
        {
            return compile_integer_binaryop(
                module,
                builder,
                context,
                binary,
                target_basic_type,
                compiler_objects,
            );
        }

        unreachable!()
    }

    if let (
        Instruction::Call { .. },
        TokenKind::BangEq
        | TokenKind::EqEq
        | TokenKind::LessEq
        | TokenKind::Less
        | TokenKind::Greater
        | TokenKind::GreaterEq
        | TokenKind::And
        | TokenKind::Or,
        Instruction::Call { .. },
    ) = binary
    {
        if binary.0.get_basic_type().is_float_type() {
            return float_binaryop(
                module,
                builder,
                context,
                binary,
                target_basic_type,
                compiler_objects,
            );
        } else if binary.0.get_basic_type().is_integer_type()
            || binary.0.get_basic_type().is_bool_type()
        {
            return compile_integer_binaryop(
                module,
                builder,
                context,
                binary,
                target_basic_type,
                compiler_objects,
            );
        }

        unreachable!()
    }

    if let (
        Instruction::LocalRef { .. },
        TokenKind::BangEq
        | TokenKind::EqEq
        | TokenKind::LessEq
        | TokenKind::Less
        | TokenKind::Greater
        | TokenKind::GreaterEq
        | TokenKind::And
        | TokenKind::Or,
        Instruction::LocalRef { .. },
    ) = binary
    {
        if binary.0.get_basic_type().is_float_type() {
            return float_binaryop(
                module,
                builder,
                context,
                binary,
                target_basic_type,
                compiler_objects,
            );
        } else if binary.0.get_basic_type().is_integer_type()
            || binary.0.get_basic_type().is_bool_type()
        {
            return compile_integer_binaryop(
                module,
                builder,
                context,
                binary,
                target_basic_type,
                compiler_objects,
            );
        }

        unreachable!()
    }

    if let (
        Instruction::Integer(_, _, _) | Instruction::Float(_, _, _) | Instruction::Boolean(_),
        TokenKind::BangEq
        | TokenKind::EqEq
        | TokenKind::LessEq
        | TokenKind::Less
        | TokenKind::Greater
        | TokenKind::GreaterEq
        | TokenKind::And
        | TokenKind::Or,
        Instruction::LocalRef { .. },
    ) = binary
    {
        if binary.0.get_basic_type().is_float_type() {
            return float_binaryop(
                module,
                builder,
                context,
                binary,
                target_basic_type,
                compiler_objects,
            );
        } else if binary.0.get_basic_type().is_integer_type()
            || binary.0.get_basic_type().is_bool_type()
        {
            return compile_integer_binaryop(
                module,
                builder,
                context,
                binary,
                target_basic_type,
                compiler_objects,
            );
        }

        unreachable!()
    }

    if let (
        Instruction::Integer(_, _, _) | Instruction::Float(_, _, _) | Instruction::Boolean(_),
        TokenKind::BangEq
        | TokenKind::EqEq
        | TokenKind::LessEq
        | TokenKind::Less
        | TokenKind::Greater
        | TokenKind::GreaterEq
        | TokenKind::And
        | TokenKind::Or,
        Instruction::Call { .. },
    ) = binary
    {
        if binary.0.get_basic_type().is_float_type() {
            return float_binaryop(
                module,
                builder,
                context,
                binary,
                target_basic_type,
                compiler_objects,
            );
        } else if binary.0.get_basic_type().is_integer_type()
            || binary.0.get_basic_type().is_bool_type()
        {
            return compile_integer_binaryop(
                module,
                builder,
                context,
                binary,
                target_basic_type,
                compiler_objects,
            );
        }

        unreachable!()
    }

    if let (
        Instruction::Call { .. },
        TokenKind::BangEq
        | TokenKind::EqEq
        | TokenKind::LessEq
        | TokenKind::Less
        | TokenKind::Greater
        | TokenKind::GreaterEq
        | TokenKind::And
        | TokenKind::Or,
        Instruction::Integer(_, _, _) | Instruction::Float(_, _, _) | Instruction::Boolean(_),
    ) = binary
    {
        if binary.2.get_basic_type().is_float_type() {
            return float_binaryop(
                module,
                builder,
                context,
                binary,
                target_basic_type,
                compiler_objects,
            );
        } else if binary.2.get_basic_type().is_integer_type()
            || binary.2.get_basic_type().is_bool_type()
        {
            return compile_integer_binaryop(
                module,
                builder,
                context,
                binary,
                target_basic_type,
                compiler_objects,
            );
        }
    }

    if let (
        Instruction::LocalRef { .. },
        TokenKind::BangEq
        | TokenKind::EqEq
        | TokenKind::LessEq
        | TokenKind::Less
        | TokenKind::Greater
        | TokenKind::GreaterEq
        | TokenKind::And
        | TokenKind::Or,
        Instruction::Integer(_, _, _) | Instruction::Float(_, _, _) | Instruction::Boolean(_),
    ) = binary
    {
        if binary.2.get_basic_type().is_float_type() {
            return float_binaryop(
                module,
                builder,
                context,
                binary,
                target_basic_type,
                compiler_objects,
            );
        } else if binary.2.get_basic_type().is_integer_type()
            || binary.2.get_basic_type().is_bool_type()
        {
            return compile_integer_binaryop(
                module,
                builder,
                context,
                binary,
                target_basic_type,
                compiler_objects,
            );
        }
    }

    if let (
        Instruction::LocalRef { .. },
        TokenKind::BangEq
        | TokenKind::EqEq
        | TokenKind::LessEq
        | TokenKind::Less
        | TokenKind::Greater
        | TokenKind::GreaterEq
        | TokenKind::And
        | TokenKind::Or,
        Instruction::Call { .. },
    ) = binary
    {
        if binary.2.get_basic_type().is_float_type() {
            return float_binaryop(
                module,
                builder,
                context,
                binary,
                target_basic_type,
                compiler_objects,
            );
        } else if binary.2.get_basic_type().is_integer_type()
            || binary.2.get_basic_type().is_bool_type()
        {
            return compile_integer_binaryop(
                module,
                builder,
                context,
                binary,
                target_basic_type,
                compiler_objects,
            );
        }
    }

    if let (
        Instruction::Call { .. },
        TokenKind::BangEq
        | TokenKind::EqEq
        | TokenKind::LessEq
        | TokenKind::Less
        | TokenKind::Greater
        | TokenKind::GreaterEq
        | TokenKind::And
        | TokenKind::Or,
        Instruction::LocalRef { .. },
    ) = binary
    {
        if binary.0.get_basic_type().is_float_type() {
            return float_binaryop(
                module,
                builder,
                context,
                binary,
                target_basic_type,
                compiler_objects,
            );
        } else if binary.0.get_basic_type().is_integer_type()
            || binary.0.get_basic_type().is_bool_type()
        {
            return compile_integer_binaryop(
                module,
                builder,
                context,
                binary,
                target_basic_type,
                compiler_objects,
            );
        }

        unreachable!()
    }

    if let (
        Instruction::BinaryOp { .. },
        TokenKind::And | TokenKind::Or,
        Instruction::BinaryOp { .. },
    ) = binary
    {
        if binary.0.get_basic_type().is_float_type() {
            let left_compiled: BasicValueEnum = float_binaryop(
                module,
                builder,
                context,
                binary.0.as_binary(),
                target_basic_type,
                compiler_objects,
            );

            let right_compiled: BasicValueEnum = float_binaryop(
                module,
                builder,
                context,
                binary.2.as_binary(),
                target_basic_type,
                compiler_objects,
            );

            return build_int_op(
                context,
                builder,
                left_compiled.into_int_value(),
                right_compiled.into_int_value(),
                (false, false),
                binary.1,
            );
        }

        return compile_integer_binaryop(
            module,
            builder,
            context,
            binary,
            target_basic_type,
            compiler_objects,
        );
    }

    if let (Instruction::Group { .. }, TokenKind::And | TokenKind::Or, Instruction::Group { .. }) =
        binary
    {
        if binary.0.get_basic_type().is_float_type() {
            let left_compiled: BasicValueEnum = float_binaryop(
                module,
                builder,
                context,
                binary.0.as_binary(),
                target_basic_type,
                compiler_objects,
            );

            let right_compiled: BasicValueEnum = float_binaryop(
                module,
                builder,
                context,
                binary.2.as_binary(),
                target_basic_type,
                compiler_objects,
            );

            return build_int_op(
                context,
                builder,
                left_compiled.into_int_value(),
                right_compiled.into_int_value(),
                (false, false),
                binary.1,
            );
        }

        return compile_integer_binaryop(
            module,
            builder,
            context,
            binary,
            target_basic_type,
            compiler_objects,
        );
    }

    if let (
        Instruction::Group { .. },
        TokenKind::And | TokenKind::Or,
        Instruction::BinaryOp { .. },
    ) = binary
    {
        if binary.0.get_basic_type().is_float_type() {
            let left_compiled: BasicValueEnum = float_binaryop(
                module,
                builder,
                context,
                binary.0.as_binary(),
                target_basic_type,
                compiler_objects,
            );

            let right_compiled: BasicValueEnum = float_binaryop(
                module,
                builder,
                context,
                binary.2.as_binary(),
                target_basic_type,
                compiler_objects,
            );

            return build_int_op(
                context,
                builder,
                left_compiled.into_int_value(),
                right_compiled.into_int_value(),
                (false, false),
                binary.1,
            );
        }

        return compile_integer_binaryop(
            module,
            builder,
            context,
            binary,
            target_basic_type,
            compiler_objects,
        );
    }

    if let (
        Instruction::BinaryOp { .. },
        TokenKind::And | TokenKind::Or,
        Instruction::Group { .. },
    ) = binary
    {
        if binary.0.get_basic_type().is_float_type() {
            let left_compiled: BasicValueEnum = float_binaryop(
                module,
                builder,
                context,
                binary.0.as_binary(),
                target_basic_type,
                compiler_objects,
            );

            let right_compiled: BasicValueEnum = float_binaryop(
                module,
                builder,
                context,
                binary.2.as_binary(),
                target_basic_type,
                compiler_objects,
            );

            return build_int_op(
                context,
                builder,
                left_compiled.into_int_value(),
                right_compiled.into_int_value(),
                (false, false),
                binary.1,
            );
        }

        return compile_integer_binaryop(
            module,
            builder,
            context,
            binary,
            target_basic_type,
            compiler_objects,
        );
    }

    println!("{:#?}", binary);
    unimplemented!()
}
