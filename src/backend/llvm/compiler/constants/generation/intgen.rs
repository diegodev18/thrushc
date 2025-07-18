use inkwell::{context::Context, values::IntValue};

use crate::{
    core::console::logging::{self, LoggingType},
    frontend::typesystem::types::Type,
};

pub fn const_int<'ctx>(
    context: &'ctx Context,
    kind: &Type,
    number: u64,
    signed: bool,
) -> IntValue<'ctx> {
    match kind {
        Type::Char => context.i8_type().const_int(number, signed).const_neg(),
        Type::S8 if signed => context.i8_type().const_int(number, signed).const_neg(),
        Type::S8 => context.i8_type().const_int(number, signed),
        Type::S16 if signed => context.i16_type().const_int(number, signed).const_neg(),
        Type::S16 => context.i16_type().const_int(number, signed),
        Type::S32 if signed => context.i32_type().const_int(number, signed).const_neg(),
        Type::S32 => context.i32_type().const_int(number, signed),
        Type::S64 if signed => context.i64_type().const_int(number, signed).const_neg(),
        Type::S64 => context.i64_type().const_int(number, signed),
        Type::U8 => context.i8_type().const_int(number, false),
        Type::U16 => context.i16_type().const_int(number, false),
        Type::U32 => context.i32_type().const_int(number, false),
        Type::U64 => context.i64_type().const_int(number, false),
        Type::Bool => context.bool_type().const_int(number, false),

        what => {
            logging::log(
                LoggingType::BackendBug,
                &format!("Unsupported integer type: '{:#?}'.", what),
            );

            unreachable!()
        }
    }
}
