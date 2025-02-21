use {
    super::{
        super::{
            backend::{compiler::misc::ThrushFile, instruction::Instruction},
            diagnostic::Diagnostic,
            error::ThrushError,
            logging::LogType,
        },
        lexer::{DataTypes, Token, TokenKind},
        objects::ParserObjects,
    },
    ahash::AHashMap as HashMap,
    std::{mem, process},
};

pub struct Import<'instr> {
    tokens: Vec<Token>,
    stmts: Vec<Instruction<'instr>>,
    errors: Vec<ThrushError>,
    current: usize,
    diagnostic: Diagnostic,
    parser_objects: ParserObjects<'instr>,
}

impl<'instr> Import<'instr> {
    pub fn generate(
        tokens: Vec<Token>,
        file: &ThrushFile,
    ) -> (Vec<Instruction<'instr>>, ParserObjects<'instr>) {
        let mut imports: Import = Self {
            tokens,
            stmts: Vec::with_capacity(50_000),
            errors: Vec::with_capacity(100),
            current: 0,
            diagnostic: Diagnostic::new(file),
            parser_objects: ParserObjects::new(HashMap::new()),
        };

        imports._parse()
    }

    fn _parse(&mut self) -> (Vec<Instruction<'instr>>, ParserObjects<'instr>) {
        while !self.end() {
            if let Ok(true) = self.match_token(TokenKind::Public) {
                if let Err(e) = self.public() {
                    self.errors.push(e);
                }
            }

            if self.only_advance().is_err() {
                break;
            }
        }

        if !self.errors.is_empty() {
            self.errors.iter().for_each(|error: &ThrushError| {
                self.diagnostic.report(error, LogType::ERROR);
            });

            process::exit(1);
        }

        (
            mem::take(&mut self.stmts),
            mem::take(&mut self.parser_objects),
        )
    }

    fn public(&mut self) -> Result<(), ThrushError> {
        if self.peek().kind == TokenKind::Extern {
            while self.peek().kind != TokenKind::Fn || self.peek().kind != TokenKind::Struct {
                self.only_advance()?;
            }
        }

        if self.match_token(TokenKind::Fn)? {
            self.build_function()?;
            return Ok(());
        }

        if self.match_token(TokenKind::Struct)? {
            self.build_struct()?;
            return Ok(());
        }

        Ok(())
    }

    fn build_struct(&mut self) -> Result<(), ThrushError> {
        let name: String = self
            .consume(
                TokenKind::Identifier,
                String::from("Expected struct name"),
                String::from("Write the struct name: \"struct --> name <-- { ... };\"."),
                self.previous().line,
            )?
            .lexeme
            .clone()
            .unwrap();

        let line: usize = self.previous().line;

        self.consume(
            TokenKind::LBrace,
            String::from("Syntax Error"),
            String::from("Expected '{'."),
            line,
        )?;

        let mut fields_types: HashMap<String, DataTypes> = HashMap::new();

        while self.peek().kind != TokenKind::RBrace {
            if self.match_token(TokenKind::Comma)? {
                continue;
            }

            if self.match_token(TokenKind::Identifier)? {
                let field_name: String = self.previous().lexeme.clone().unwrap();
                let line: usize = self.previous().line;

                let field_type: DataTypes = match self.peek().kind {
                    TokenKind::DataType(kind) => {
                        self.only_advance()?;
                        kind
                    }
                    ident
                        if ident == TokenKind::Identifier
                            && self
                                .parser_objects
                                .get_struct(self.peek().lexeme.as_ref().unwrap(), line)
                                .is_ok()
                            || &name == self.peek().lexeme.as_ref().unwrap() =>
                    {
                        self.only_advance()?;
                        DataTypes::Struct
                    }
                    _ => {
                        return Err(ThrushError::Error(
                            String::from("Expected type of field"),
                            format!("Write the field type: \"{} --> i64 <--\".", field_name),
                            line,
                        ));
                    }
                };

                fields_types.insert(field_name, field_type);
                continue;
            }

            self.only_advance()?;
        }

        self.consume(
            TokenKind::RBrace,
            String::from("Syntax Error"),
            String::from("Expected '}'."),
            line,
        )?;

        self.consume(
            TokenKind::SemiColon,
            String::from("Syntax Error"),
            String::from("Expected ';'."),
            line,
        )?;

        self.add_struct(name, fields_types);

        Ok(())
    }

    fn build_function(&mut self) -> Result<(), ThrushError> {
        let line: usize = self.previous().line;

        let name: String = self
            .consume(
                TokenKind::Identifier,
                String::from("Expected function name"),
                String::from("Expected a name to the function."),
                self.previous().line,
            )?
            .lexeme
            .clone()
            .unwrap();

        self.consume(
            TokenKind::LParen,
            String::from("Syntax Error"),
            String::from("Expected '('."),
            line,
        )?;

        let mut params: Vec<Instruction> = Vec::new();
        let mut params_types: Vec<DataTypes> = Vec::new();

        let mut param_position: u32 = 0;

        while !self.match_token(TokenKind::RParen)? {
            if self.match_token(TokenKind::Comma)? {
                continue;
            }

            if self.match_token(TokenKind::Pass)? {
                continue;
            }

            if !self.match_token(TokenKind::Identifier)? {
                self.errors.push(ThrushError::Error(
                    String::from("Syntax Error"),
                    String::from("Expected argument name."),
                    line,
                ));
            }

            let ident: String = self.previous().lexeme.clone().unwrap();
            let line: usize = self.previous().line;

            if !self.match_token(TokenKind::ColonColon)? {
                self.errors.push(ThrushError::Error(
                    String::from("Syntax Error"),
                    String::from("Expected '::'."),
                    line,
                ));
            }

            let kind: DataTypes = match self.peek().kind {
                TokenKind::DataType(kind) => {
                    self.only_advance()?;
                    kind
                }
                ident
                    if ident == TokenKind::Identifier
                        && self
                            .parser_objects
                            .get_struct(self.peek().lexeme.as_ref().unwrap(), line)
                            .is_ok() =>
                {
                    self.only_advance()?;
                    DataTypes::Struct
                }
                what => {
                    return Err(ThrushError::Error(
                        String::from("Undeterminated type"),
                        format!("Type \"{}\" not exist.", what),
                        line,
                    ));
                }
            };

            params.push(Instruction::Param {
                name: ident,
                kind,
                line,
                position: param_position,
            });

            params_types.push(kind);
            param_position += 1;
        }

        self.consume(
            TokenKind::Colon,
            String::from("Syntax Error"),
            String::from("Missing return type. Expected ':' followed by return type."),
            line,
        )?;

        let return_type: (DataTypes, String) = match self.peek().kind {
            TokenKind::DataType(kind) => {
                self.only_advance()?;
                (kind, String::new())
            }
            ident
                if ident == TokenKind::Identifier
                    && self
                        .parser_objects
                        .get_struct(self.peek().lexeme.as_ref().unwrap(), line)
                        .is_ok() =>
            {
                self.only_advance()?;
                (DataTypes::Struct, self.previous().lexeme.clone().unwrap())
            }
            what => {
                return Err(ThrushError::Error(
                    String::from("Undeterminated type"),
                    format!("Type \"{}\" not exist.", what),
                    line,
                ))
            }
        };

        self.add_build_function(name.clone(), return_type.0, params_types);

        self.stmts.push(Instruction::Extern {
            name: name.clone(),
            instr: Box::new(Instruction::Function {
                name,
                params,
                body: None,
                return_type: return_type.0,
                is_public: true,
            }),
            kind: TokenKind::Fn,
        });

        Ok(())
    }

    fn add_struct(&mut self, name: String, fields: HashMap<String, DataTypes>) {
        self.parser_objects.insert_new_struct(name, fields);
    }

    fn add_build_function(&mut self, name: String, kind: DataTypes, datatypes: Vec<DataTypes>) {
        self.parser_objects.insert_new_global(
            name,
            (kind, datatypes, Vec::new(), true, false, String::new()),
        );
    }

    fn consume(
        &mut self,
        kind: TokenKind,
        error_title: String,
        help: String,
        line: usize,
    ) -> Result<&Token, ThrushError> {
        if self.peek().kind == kind {
            return self.advance();
        }

        Err(ThrushError::Error(error_title, help, line))
    }

    fn match_token(&mut self, kind: TokenKind) -> Result<bool, ThrushError> {
        if self.peek().kind == kind {
            self.only_advance()?;
            return Ok(true);
        }

        Ok(false)
    }

    fn only_advance(&mut self) -> Result<(), ThrushError> {
        if !self.end() {
            self.current += 1;
        }

        Err(ThrushError::Error(
            String::from("Syntax error"),
            String::from("EOF has been reached."),
            self.previous().line,
        ))
    }

    fn advance(&mut self) -> Result<&Token, ThrushError> {
        if !self.end() {
            self.current += 1;
            return Ok(self.previous());
        }

        Err(ThrushError::Error(
            String::from("Syntax error"),
            String::from("EOF has been reached."),
            self.previous().line,
        ))
    }

    fn peek(&self) -> &Token {
        &self.tokens[self.current]
    }

    fn end(&self) -> bool {
        self.tokens[self.current].kind == TokenKind::Eof
    }

    fn previous(&self) -> &Token {
        &self.tokens[self.current - 1]
    }
}
