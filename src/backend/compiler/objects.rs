use {
    super::{
        super::super::logging,
        memory::AllocatedObject,
        types::{CompilerFunction, Struct},
    },
    ahash::AHashMap as HashMap,
};

const FUNCTION_MINIMAL_CAPACITY: usize = 255;
const STRUCTURE_MINIMAL_CAPACITY: usize = 255;
const SCOPE_MINIMAL_CAPACITY: usize = 155;

#[derive(Debug)]
pub struct CompilerObjects<'ctx> {
    pub functions: HashMap<&'ctx str, CompilerFunction<'ctx>>,
    pub structs: HashMap<&'ctx str, &'ctx Struct<'ctx>>,
    pub blocks: Vec<HashMap<&'ctx str, AllocatedObject<'ctx>>>,
    pub scope_position: usize,
}

impl<'ctx> CompilerObjects<'ctx> {
    pub fn new() -> Self {
        Self {
            functions: HashMap::with_capacity(FUNCTION_MINIMAL_CAPACITY),
            structs: HashMap::with_capacity(STRUCTURE_MINIMAL_CAPACITY),
            blocks: Vec::with_capacity(SCOPE_MINIMAL_CAPACITY),
            scope_position: 0,
        }
    }

    #[inline]
    pub fn begin_scope(&mut self) {
        self.blocks
            .push(HashMap::with_capacity(SCOPE_MINIMAL_CAPACITY));
        self.scope_position += 1;
    }

    #[inline]
    pub fn end_scope(&mut self) {
        self.blocks.pop();
        self.scope_position -= 1;
    }

    #[inline]
    pub fn alloc_local_object(&mut self, name: &'ctx str, object: AllocatedObject<'ctx>) {
        self.blocks[self.scope_position - 1].insert(name, object);
    }

    #[inline]
    pub fn insert_function(&mut self, name: &'ctx str, function: CompilerFunction<'ctx>) {
        self.functions.insert(name, function);
    }

    #[inline]
    pub fn insert_struct(&mut self, name: &'ctx str, fields_types: &'ctx Struct) {
        self.structs.insert(name, fields_types);
    }

    #[inline]
    pub fn get_struct(&self, name: &str) -> Option<&Struct> {
        self.structs.get(name).map(|structure| &**structure)
    }

    #[inline]
    pub fn get_allocated_object(&self, name: &str) -> AllocatedObject<'ctx> {
        for position in (0..self.scope_position).rev() {
            if let Some(allocated_object) = self.blocks[position].get(name) {
                return *allocated_object;
            }
        }

        logging::log(
            logging::LogType::Panic,
            &format!(
                "Unable to get '{}' allocated object at frame pointer number #{}.",
                name, self.scope_position
            ),
        );

        unreachable!()
    }

    #[inline]
    pub fn get_function(&self, name: &str) -> CompilerFunction<'ctx> {
        if let Some(function) = self.functions.get(name) {
            return *function;
        }

        logging::log(
            logging::LogType::Panic,
            &format!("Unable to get '{}' function in global frame.", name),
        );

        unreachable!()
    }
}
