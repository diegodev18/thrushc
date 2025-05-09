use crate::middle::statement::StructFields;
use crate::middle::statement::traits::{FoundSymbolEither, FoundSymbolExtension, StructExtensions};
use crate::middle::symbols::types::{Bindings, Struct};

use super::super::common::error::ThrushCompilerError;

use super::{super::middle::types::*, lexer::Span, symbols::FoundSymbolId};

impl<'a> StructExtensions<'a> for Struct<'a> {
    fn contains_field(&self, name: &str) -> bool {
        self.1.iter().any(|field| field.0 == name)
    }

    fn get_field_type(&self, name: &str) -> Option<Type> {
        if let Some(field) = self.1.iter().find(|field| field.0 == name) {
            let field_type: Type = field.1.clone();
            return Some(field_type);
        }

        None
    }

    fn get_fields(&self) -> StructFields<'a> {
        (self.0, self.1.clone())
    }

    fn get_bindings(&self) -> Bindings<'a> {
        self.3.clone()
    }
}

impl FoundSymbolExtension for FoundSymbolId<'_> {
    fn is_structure(&self) -> bool {
        self.0.is_some()
    }

    fn is_function(&self) -> bool {
        self.1.is_some()
    }

    fn is_enum(&self) -> bool {
        self.2.is_some()
    }

    fn is_constant(&self) -> bool {
        self.3.is_some()
    }

    fn is_custom_type(&self) -> bool {
        self.4.is_some()
    }
}

impl<'instr> FoundSymbolEither<'instr> for FoundSymbolId<'instr> {
    fn expected_custom_type(&self, span: Span) -> Result<&'instr str, ThrushCompilerError> {
        if let Some(type_id) = self.4 {
            return Ok(type_id);
        }

        Err(ThrushCompilerError::Error(
            String::from("Expected custom type reference"),
            String::from("Expected custom type but found something else."),
            String::default(),
            span,
        ))
    }

    fn expected_constant(&self, span: Span) -> Result<&'instr str, ThrushCompilerError> {
        if let Some(const_id) = self.3 {
            return Ok(const_id);
        }

        Err(ThrushCompilerError::Error(
            String::from("Expected constant reference"),
            String::from("Expected constant but found something else."),
            String::default(),
            span,
        ))
    }

    fn expected_enum(&self, span: Span) -> Result<&'instr str, ThrushCompilerError> {
        if let Some(name) = self.2 {
            return Ok(name);
        }

        Err(ThrushCompilerError::Error(
            String::from("Expected enum reference"),
            String::from("Expected enum but found something else."),
            String::default(),
            span,
        ))
    }

    fn expected_struct(&self, span: Span) -> Result<&'instr str, ThrushCompilerError> {
        if let Some(name) = self.0 {
            return Ok(name);
        }

        Err(ThrushCompilerError::Error(
            String::from("Expected struct reference"),
            String::from("Expected struct but found something else."),
            String::default(),
            span,
        ))
    }

    fn expected_function(&self, span: Span) -> Result<&'instr str, ThrushCompilerError> {
        if let Some(name) = self.1 {
            return Ok(name);
        }

        Err(ThrushCompilerError::Error(
            String::from("Expected function reference"),
            String::from("Expected function but found something else."),
            String::default(),
            span,
        ))
    }

    fn expected_local(&self, span: Span) -> Result<(&'instr str, usize), ThrushCompilerError> {
        if let Some((name, scope_idx)) = self.5 {
            return Ok((name, scope_idx));
        }

        Err(ThrushCompilerError::Error(
            String::from("Expected local reference"),
            String::from("Expected local but found something else."),
            String::default(),
            span,
        ))
    }
}
