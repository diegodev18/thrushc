use crate::middle::types::frontend::{
    lexer::types::BindingsApplicant,
    parser::{
        stmts::{
            instruction::Instruction,
            types::{CustomType, Enum},
        },
        symbols::types::{
            Bindings, Constant, Constants, CustomTypes, Enums, FoundSymbolId, Function, Functions,
            Local, Locals, Struct, Structs,
        },
    },
};

use super::{super::standard::error::ThrushCompilerIssue, lexer::Span};

use ahash::AHashMap as HashMap;

const MINIMAL_CUSTOM_TYPE_CAPACITY: usize = 255;
const MINIMAL_CONSTANTS_CAPACITY: usize = 255;
const MINIMAL_STRUCTURE_CAPACITY: usize = 255;
const MINIMAL_ENUMS_CAPACITY: usize = 255;
const MINIMAL_LOCAL_SCOPE_CAPACITY: usize = 255;

#[derive(Clone, Debug, Default)]
pub struct SymbolsTable<'instr> {
    custom_types: CustomTypes<'instr>,
    constants: Constants<'instr>,
    locals: Locals<'instr>,
    structs: Structs<'instr>,
    functions: Functions<'instr>,
    enums: Enums<'instr>,

    lifts: Vec<Instruction<'instr>>,
}

impl<'instr> SymbolsTable<'instr> {
    pub fn with_functions(functions: HashMap<&'instr str, Function<'instr>>) -> Self {
        Self {
            custom_types: HashMap::with_capacity(MINIMAL_CUSTOM_TYPE_CAPACITY),
            constants: HashMap::with_capacity(MINIMAL_CONSTANTS_CAPACITY),
            locals: Vec::with_capacity(MINIMAL_LOCAL_SCOPE_CAPACITY),
            functions,
            structs: HashMap::with_capacity(MINIMAL_STRUCTURE_CAPACITY),
            enums: HashMap::with_capacity(MINIMAL_ENUMS_CAPACITY),
            lifts: Vec::with_capacity(10),
        }
    }

    pub fn get_symbols_id(
        &self,
        name: &'instr str,
        span: Span,
    ) -> Result<FoundSymbolId<'instr>, ThrushCompilerIssue> {
        if self.custom_types.contains_key(name) {
            return Ok((None, None, None, None, Some(name), None));
        }

        if self.constants.contains_key(name) {
            return Ok((None, None, None, Some(name), None, None));
        }

        if self.structs.contains_key(name) {
            return Ok((Some(name), None, None, None, None, None));
        }

        if self.enums.contains_key(name) {
            return Ok((None, None, Some(name), None, None, None));
        }

        if self.functions.contains_key(name) {
            return Ok((None, Some(name), None, None, None, None));
        }

        for (idx, scope) in self.locals.iter().enumerate().rev() {
            if scope.contains_key(name) {
                return Ok((None, None, None, None, None, Some((name, idx))));
            }
        }

        Err(ThrushCompilerIssue::Error(
            String::from("Structure/Function/Local/Constant/Type not found"),
            format!("'{}' is not declared or defined.", name),
            None,
            span,
        ))
    }

    #[inline]
    pub fn get_struct_by_id(
        &self,
        struct_id: &'instr str,
        span: Span,
    ) -> Result<Struct<'instr>, ThrushCompilerIssue> {
        if let Some(structure) = self.structs.get(struct_id).cloned() {
            return Ok(structure);
        }

        Err(ThrushCompilerIssue::Error(
            String::from("Expected struct reference"),
            String::from("Expected struct but found something else."),
            None,
            span,
        ))
    }

    #[inline]
    pub fn get_function_by_id(
        &self,
        span: Span,
        func_id: &'instr str,
    ) -> Result<Function<'instr>, ThrushCompilerIssue> {
        if let Some(function) = self.functions.get(func_id).cloned() {
            return Ok(function);
        }

        Err(ThrushCompilerIssue::Error(
            String::from("Expected function reference"),
            String::from("Expected function but found something else."),
            None,
            span,
        ))
    }

    #[inline]
    pub fn get_enum_by_id(
        &self,
        enum_id: &'instr str,
        span: Span,
    ) -> Result<Enum<'instr>, ThrushCompilerIssue> {
        if let Some(enum_found) = self.enums.get(enum_id).cloned() {
            return Ok(enum_found);
        }

        Err(ThrushCompilerIssue::Error(
            String::from("Expected enum reference"),
            String::from("Expected enum but found something else."),
            None,
            span,
        ))
    }

    #[inline]
    pub fn get_custom_type_by_id(
        &self,
        custom_type_id: &'instr str,
        span: Span,
    ) -> Result<CustomType<'instr>, ThrushCompilerIssue> {
        if let Some(custom_type) = self.custom_types.get(custom_type_id).cloned() {
            return Ok(custom_type);
        }

        Err(ThrushCompilerIssue::Error(
            String::from("Expected custom type reference"),
            String::from("Expected custom type but found something else."),
            None,
            span,
        ))
    }

    #[inline]
    pub fn get_local_by_id(
        &self,
        local_id: &'instr str,
        scope_idx: usize,
        span: Span,
    ) -> Result<&Local<'instr>, ThrushCompilerIssue> {
        if let Some(local) = self.locals[scope_idx].get(local_id) {
            return Ok(local);
        }

        Err(ThrushCompilerIssue::Error(
            String::from("Expected local reference"),
            String::from("Expected local but found something else."),
            None,
            span,
        ))
    }

    #[inline]
    pub fn get_const_by_id(
        &self,
        const_id: &'instr str,
        span: Span,
    ) -> Result<Constant<'instr>, ThrushCompilerIssue> {
        if let Some(constant) = self.constants.get(const_id).cloned() {
            return Ok(constant);
        }

        Err(ThrushCompilerIssue::Error(
            String::from("Expected constant reference"),
            String::from("Expected constant but found something else."),
            None,
            span,
        ))
    }

    #[inline]
    pub fn get_struct(
        &self,
        name: &str,
        span: Span,
    ) -> Result<Struct<'instr>, ThrushCompilerIssue> {
        if let Some(struct_fields) = self.structs.get(name).cloned() {
            return Ok(struct_fields);
        }

        Err(ThrushCompilerIssue::Error(
            String::from("Structure not found"),
            format!("'{}' structure not defined.", name),
            None,
            span,
        ))
    }

    #[inline]
    pub fn get_struct_mut(
        &mut self,
        name: &str,
        span: Span,
    ) -> Result<&mut Struct<'instr>, ThrushCompilerIssue> {
        if let Some(struct_fields) = self.structs.get_mut(name) {
            return Ok(struct_fields);
        }

        Err(ThrushCompilerIssue::Error(
            String::from("Structure not found"),
            format!("'{}' structure not defined.", name),
            None,
            span,
        ))
    }

    pub fn new_local(
        &mut self,
        scope_pos: usize,
        name: &'instr str,
        value: Local<'instr>,
        span: Span,
    ) -> Result<(), ThrushCompilerIssue> {
        if self.locals[scope_pos - 1].contains_key(name) {
            return Err(ThrushCompilerIssue::Error(
                String::from("Local variable already declared"),
                format!("'{}' local variable already declared before.", name),
                None,
                span,
            ));
        }

        self.locals[scope_pos - 1].insert(name, value);

        Ok(())
    }

    pub fn new_constant(
        &mut self,
        name: &'instr str,
        constant: Constant<'instr>,
        span: Span,
    ) -> Result<(), ThrushCompilerIssue> {
        if self.constants.contains_key(name) {
            return Err(ThrushCompilerIssue::Error(
                String::from("Constant already declared"),
                format!("'{}' constant already declared before.", name),
                None,
                span,
            ));
        }

        self.constants.insert(name, constant);

        Ok(())
    }

    pub fn new_custom_type(
        &mut self,
        name: &'instr str,
        custom_type: CustomType<'instr>,
        span: Span,
    ) -> Result<(), ThrushCompilerIssue> {
        if self.constants.contains_key(name) {
            return Err(ThrushCompilerIssue::Error(
                String::from("Custom type already declared"),
                format!("'{}' custom type already declared before.", name),
                None,
                span,
            ));
        }

        self.custom_types.insert(name, custom_type);

        Ok(())
    }

    pub fn new_struct(
        &mut self,
        name: &'instr str,
        field_types: Struct<'instr>,
        span: Span,
    ) -> Result<(), ThrushCompilerIssue> {
        if self.structs.contains_key(name) {
            return Err(ThrushCompilerIssue::Error(
                String::from("Structure already declared"),
                format!("'{}' structure already declared before.", name),
                None,
                span,
            ));
        }

        self.structs.insert(name, field_types);

        Ok(())
    }

    pub fn new_enum(
        &mut self,
        name: &'instr str,
        union: Enum<'instr>,
        span: Span,
    ) -> Result<(), ThrushCompilerIssue> {
        if self.enums.contains_key(name) {
            return Err(ThrushCompilerIssue::Error(
                String::from("Enum already declared"),
                format!("'{}' enum already declared before.", name),
                None,
                span,
            ));
        }

        self.enums.insert(name, union);

        Ok(())
    }

    pub fn new_function(
        &mut self,
        name: &'instr str,
        function: Function<'instr>,
        span: Span,
    ) -> Result<(), ThrushCompilerIssue> {
        if self.functions.contains_key(name) {
            return Err(ThrushCompilerIssue::Error(
                String::from("Function already declared"),
                format!("'{}' function already declared before.", name),
                None,
                span,
            ));
        }

        self.functions.insert(name, function);

        Ok(())
    }

    pub fn set_bindings(
        &mut self,
        name: &str,
        bindings: Bindings<'instr>,
        applicant: BindingsApplicant,
        span: Span,
    ) -> Result<(), ThrushCompilerIssue> {
        match applicant {
            BindingsApplicant::Struct => {
                let structure: &mut Struct = self.get_struct_mut(name, span)?;
                structure.3 = bindings;
            }
        }

        Ok(())
    }

    pub fn contains_structure(&self, name: &str, span: Span) -> Result<(), ThrushCompilerIssue> {
        if !self.structs.contains_key(name) {
            return Err(ThrushCompilerIssue::Error(
                String::from("Structure not found"),
                format!("'{}' structure not defined or declared yet.", name),
                None,
                span,
            ));
        }

        Ok(())
    }

    pub fn lift_instructions(&mut self, scope_pos: usize) -> Result<(), ThrushCompilerIssue> {
        for parameter in self.clone().lifts.iter() {
            if let Instruction::FunctionParameter {
                name,
                kind,
                is_mutable,
                span,
                ..
            }
            | Instruction::BindParameter {
                name,
                kind,
                is_mutable,
                span,
                ..
            } = parameter
            {
                self.new_local(
                    scope_pos,
                    name,
                    (kind.clone(), *is_mutable, false, *span),
                    *span,
                )?;
            }

            if let Instruction::This {
                kind,
                is_mutable,
                span,
            } = parameter
            {
                self.new_local(
                    scope_pos,
                    "this",
                    (kind.clone(), *is_mutable, false, *span),
                    *span,
                )?;
            }

            if let Instruction::Local {
                name,
                kind,
                span,
                is_mutable,
                ..
            } = parameter
            {
                self.new_local(
                    scope_pos,
                    name,
                    (kind.clone(), *is_mutable, false, *span),
                    *span,
                )?;
            }
        }

        self.lifts.clear();
        Ok(())
    }

    pub fn add_lift_instruction(&mut self, instruction: Instruction<'instr>) {
        self.lifts.push(instruction);
    }

    #[inline]
    pub fn begin_local_scope(&mut self) {
        self.locals
            .push(HashMap::with_capacity(MINIMAL_LOCAL_SCOPE_CAPACITY));
    }

    #[inline]
    pub fn end_local_scope(&mut self) {
        self.locals.pop();
    }

    pub fn clear_all_scopes(&mut self) {
        self.locals.clear();
    }
}
