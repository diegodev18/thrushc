use std::sync::Arc;

use inkwell::{context::Context, targets::TargetData};

use crate::{
    backend::llvm::compiler::typegen,
    frontend::{lexer::Span, symbols::SymbolsTable},
    middle::types::frontend::parser::{
        stmts::{instruction::Instruction, traits::StructExtensions, types::StructFields},
        symbols::types::{Bindings, Struct},
    },
    standard::error::ThrushCompilerIssue,
};

pub type ThrushStructType = (String, Vec<Arc<ThrushType>>);

#[derive(Debug, Clone, Copy)]
pub enum BindingsApplicant {
    Struct,
}

#[derive(Debug, Clone)]
pub enum ThrushType {
    // Signed Integer Type
    S8,
    S16,
    S32,
    S64,

    // Unsigned Integer Type
    U8,
    U16,
    U32,
    U64,

    // Floating Point Type
    F32,
    F64,

    // Boolean Type
    Bool,

    // Char Type
    Char,

    // Str Type
    Str,

    // Mutable Type
    Mut(Arc<ThrushType>),

    // Ptr Type
    Ptr(Option<Arc<ThrushType>>),

    // Struct Type
    Struct(String, Vec<Arc<ThrushType>>),

    // Me (Self Type)
    Me(Option<Arc<ThrushType>>),

    // Address
    Address,

    // Void Type
    Void,
}

impl ThrushType {
    #[must_use]
    pub fn precompute_type(&self, other: &ThrushType) -> &ThrushType {
        match (self, other) {
            (ThrushType::S64, _) | (_, ThrushType::S64) => &ThrushType::S64,
            (ThrushType::S32, _) | (_, ThrushType::S32) => &ThrushType::S32,
            (ThrushType::S16, _) | (_, ThrushType::S16) => &ThrushType::S16,
            (ThrushType::S8, _) | (_, ThrushType::S8) => &ThrushType::S8,

            (ThrushType::U64, _) | (_, ThrushType::U64) => &ThrushType::U64,
            (ThrushType::U32, _) | (_, ThrushType::U32) => &ThrushType::U32,
            (ThrushType::U16, _) | (_, ThrushType::U16) => &ThrushType::U16,
            (ThrushType::U8, _) | (_, ThrushType::U8) => &ThrushType::U8,

            (ThrushType::F64, _) | (_, ThrushType::F64) => &ThrushType::F64,
            (ThrushType::F32, _) | (_, ThrushType::F32) => &ThrushType::F32,

            (ThrushType::Mut(a_subtype), ThrushType::Mut(b_subtype)) => {
                a_subtype.precompute_type(b_subtype)
            }

            _ => self,
        }
    }

    pub fn is_heap_allocated(&self, llvm_context: &Context, target_data: &TargetData) -> bool {
        target_data.get_abi_size(&typegen::generate_type(llvm_context, self)) >= 128
            || self.is_recursive_type()
    }

    pub fn is_mut_ptr_type(&self) -> bool {
        if let ThrushType::Mut(subtype) = self {
            if let ThrushType::Ptr(_) = &**subtype {
                return true;
            }
        }

        false
    }

    pub fn is_mut_numeric_type(&self) -> bool {
        if let ThrushType::Mut(subtype) = self {
            return subtype.is_integer_type() || subtype.is_float_type();
        }

        false
    }

    pub fn into_structure_type(self) -> ThrushStructType {
        if let ThrushType::Struct(name, types) = self {
            return (name, types);
        }

        unreachable!()
    }

    #[inline(always)]
    pub const fn is_char_type(&self) -> bool {
        matches!(self, ThrushType::Char)
    }

    #[inline(always)]
    pub const fn is_void_type(&self) -> bool {
        matches!(self, ThrushType::Void)
    }

    #[inline(always)]
    pub const fn is_bool_type(&self) -> bool {
        matches!(self, ThrushType::Bool)
    }

    #[inline(always)]
    pub const fn is_struct_type(&self) -> bool {
        matches!(self, ThrushType::Struct(..))
    }

    #[inline(always)]
    pub const fn is_float_type(&self) -> bool {
        matches!(self, ThrushType::F32 | ThrushType::F64)
    }

    #[inline(always)]
    pub const fn is_ptr_type(&self) -> bool {
        matches!(self, ThrushType::Ptr(_))
    }

    #[inline(always)]
    pub const fn is_address_type(&self) -> bool {
        matches!(self, ThrushType::Address)
    }

    #[inline(always)]
    pub const fn is_mut_type(&self) -> bool {
        matches!(self, ThrushType::Mut(_))
    }

    #[inline(always)]
    pub const fn is_str_type(&self) -> bool {
        matches!(self, ThrushType::Str)
    }

    #[inline(always)]
    pub const fn is_me_type(&self) -> bool {
        matches!(self, ThrushType::Me(_))
    }

    #[must_use]
    #[inline(always)]
    pub const fn is_signed_integer_type(&self) -> bool {
        matches!(
            self,
            ThrushType::S8 | ThrushType::S16 | ThrushType::S32 | ThrushType::S64
        )
    }

    #[inline(always)]
    pub const fn is_integer_type(&self) -> bool {
        matches!(
            self,
            ThrushType::S8
                | ThrushType::S16
                | ThrushType::S32
                | ThrushType::S64
                | ThrushType::U8
                | ThrushType::U16
                | ThrushType::U32
                | ThrushType::U64
                | ThrushType::Char
        )
    }

    pub fn narrowing_cast(&self) -> ThrushType {
        match self {
            ThrushType::U8 => ThrushType::S8,
            ThrushType::U16 => ThrushType::S16,
            ThrushType::U32 => ThrushType::S32,
            ThrushType::U64 => ThrushType::S64,
            _ => self.clone(),
        }
    }

    pub fn is_recursive_type(&self) -> bool {
        if let ThrushType::Struct(_, fields) = self {
            fields.iter().any(|tp| tp.is_me_type())
        } else {
            false
        }
    }

    pub fn create_structure_type(name: String, fields: &[ThrushType]) -> ThrushType {
        ThrushType::Struct(
            name,
            fields.iter().map(|field| Arc::new(field.clone())).collect(),
        )
    }
}

impl PartialEq for ThrushType {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (ThrushType::Struct(_, fields1), ThrushType::Struct(_, fields2)) => {
                fields1.len() == fields2.len()
                    && fields1
                        .iter()
                        .zip(fields2.iter())
                        .all(|(f1, f2)| f1.as_ref() == f2.as_ref())
            }

            (ThrushType::Mut(target), ThrushType::Mut(from)) => target == from,
            (ThrushType::Char, ThrushType::Char) => true,
            (ThrushType::S8, ThrushType::S8) => true,
            (ThrushType::S16, ThrushType::S16) => true,
            (ThrushType::S32, ThrushType::S32) => true,
            (ThrushType::S64, ThrushType::S64) => true,
            (ThrushType::U8, ThrushType::U8) => true,
            (ThrushType::U16, ThrushType::U16) => true,
            (ThrushType::U32, ThrushType::U32) => true,
            (ThrushType::U64, ThrushType::U64) => true,
            (ThrushType::F32, ThrushType::F32) => true,
            (ThrushType::F64, ThrushType::F64) => true,
            (ThrushType::Ptr(None), ThrushType::Ptr(None)) => true,
            (ThrushType::Ptr(Some(target)), ThrushType::Ptr(Some(from))) => target == from,
            (ThrushType::Void, ThrushType::Void) => true,
            (ThrushType::Str, ThrushType::Str) => true,
            (ThrushType::Me(Some(target)), ThrushType::Me(Some(from))) => target == from,
            (ThrushType::Me(None), ThrushType::Me(None)) => true,
            (ThrushType::Bool, ThrushType::Bool) => true,

            _ => false,
        }
    }
}

pub fn generate_bindings(original_bindings: Vec<Instruction>) -> Bindings {
    let mut bindings: Bindings = Vec::with_capacity(original_bindings.len());

    for binding in original_bindings {
        bindings.push((
            binding.get_binding_name(),
            binding.get_binding_type(),
            binding.get_binding_parameters(),
        ));
    }

    bindings
}

pub fn decompose_struct_property(
    mut position: usize,
    property_names: Vec<&'_ str>,
    struct_type: ThrushType,
    symbols_table: &SymbolsTable<'_>,
    span: Span,
) -> Result<(ThrushType, Vec<(ThrushType, u32)>), ThrushCompilerIssue> {
    let mut gep_indices: Vec<(ThrushType, u32)> = Vec::with_capacity(10);

    if position >= property_names.len() {
        return Ok((struct_type.clone(), gep_indices));
    }

    if let ThrushType::Struct(name, _) = &struct_type {
        let structure: Struct = symbols_table.get_struct(name, span)?;
        let fields: StructFields = structure.get_fields();

        let field_name: &str = property_names[position];

        let field_with_index: Option<(usize, &(&str, ThrushType, u32))> = fields
            .1
            .iter()
            .enumerate()
            .find(|field| field.1.0 == field_name);

        if let Some((index, (_, field_type, _))) = field_with_index {
            gep_indices.push((field_type.clone(), index as u32));

            position += 1;

            let (result_type, mut nested_indices) = decompose_struct_property(
                position,
                property_names,
                field_type.clone(),
                symbols_table,
                span,
            )?;

            gep_indices.append(&mut nested_indices);

            return Ok((result_type, gep_indices));
        }

        return Err(ThrushCompilerIssue::Error(
            String::from("Syntax error"),
            format!("Expected existing property, not '{}'.", field_name,),
            None,
            span,
        ));
    }

    if position < property_names.len() {
        return Err(ThrushCompilerIssue::Error(
            String::from("Syntax error"),
            format!(
                "Existing property '{}' is not a structure.",
                property_names[position]
            ),
            None,
            span,
        ));
    }

    Ok((struct_type.clone(), gep_indices))
}
