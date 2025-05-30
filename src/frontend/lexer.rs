use std::fmt::{self, Display};
use std::process;

use crate::middle::types::frontend::lexer::tokenkind::TokenKind;
use crate::middle::types::frontend::parser::stmts::traits::TokenExtensions;
use crate::standard::misc::CompilerFile;

use super::super::{
    logging::LoggingType,
    standard::{
        constants::MINIMAL_ERROR_CAPACITY, diagnostic::Diagnostician, error::ThrushCompilerIssue,
    },
};

use {
    ahash::{HashMap, HashMapExt},
    lazy_static::lazy_static,
    std::mem,
};

const KEYWORDS_CAPACITY: usize = 62;
const MINIMAL_TOKENS_CAPACITY: usize = 100_000;

lazy_static! {
    static ref KEYWORDS: HashMap<&'static [u8], TokenKind> = {
        let mut keywords: HashMap<&'static [u8], TokenKind> =
            HashMap::with_capacity(KEYWORDS_CAPACITY);

        keywords.insert(b"local", TokenKind::Local);
        keywords.insert(b"fn", TokenKind::Fn);
        keywords.insert(b"if", TokenKind::If);
        keywords.insert(b"elif", TokenKind::Elif);
        keywords.insert(b"else", TokenKind::Else);
        keywords.insert(b"for", TokenKind::For);
        keywords.insert(b"while", TokenKind::While);
        keywords.insert(b"loop", TokenKind::Loop);
        keywords.insert(b"true", TokenKind::True);
        keywords.insert(b"false", TokenKind::False);
        keywords.insert(b"or", TokenKind::Or);
        keywords.insert(b"and", TokenKind::And);
        keywords.insert(b"const", TokenKind::Const);
        keywords.insert(b"struct", TokenKind::Struct);
        keywords.insert(b"return", TokenKind::Return);
        keywords.insert(b"break", TokenKind::Break);
        keywords.insert(b"continue", TokenKind::Continue);
        keywords.insert(b"bindings", TokenKind::Bindings);
        keywords.insert(b"bind", TokenKind::Bind);
        keywords.insert(b"this", TokenKind::This);
        keywords.insert(b"Me", TokenKind::Me);
        keywords.insert(b"match", TokenKind::Match);
        keywords.insert(b"pattern", TokenKind::Pattern);
        keywords.insert(b"mut", TokenKind::Mut);
        keywords.insert(b"nullptr", TokenKind::NullPtr);
        keywords.insert(b"type", TokenKind::Type);
        keywords.insert(b"enum", TokenKind::Enum);
        keywords.insert(b"address", TokenKind::Address);
        keywords.insert(b"carry", TokenKind::Carry);
        keywords.insert(b"write", TokenKind::Write);
        keywords.insert(b"@import", TokenKind::Import);
        keywords.insert(b"@public", TokenKind::Public);
        keywords.insert(b"@extern", TokenKind::Extern);
        keywords.insert(b"@ignore", TokenKind::Ignore);
        keywords.insert(b"@hot", TokenKind::Hot);
        keywords.insert(b"@minsize", TokenKind::MinSize);
        keywords.insert(b"@alwaysinline", TokenKind::AlwaysInline);
        keywords.insert(b"@noinline", TokenKind::NoInline);
        keywords.insert(b"@inline", TokenKind::InlineHint);
        keywords.insert(b"@safestack", TokenKind::SafeStack);
        keywords.insert(b"@weakstack", TokenKind::WeakStack);
        keywords.insert(b"@strongstack", TokenKind::StrongStack);
        keywords.insert(b"@precisefp", TokenKind::PreciseFloats);
        keywords.insert(b"@convention", TokenKind::Convention);
        keywords.insert(b"new", TokenKind::New);

        keywords.insert(b"s8", TokenKind::S8);
        keywords.insert(b"s16", TokenKind::S16);
        keywords.insert(b"s32", TokenKind::S32);
        keywords.insert(b"s64", TokenKind::S64);
        keywords.insert(b"u8", TokenKind::U8);
        keywords.insert(b"u16", TokenKind::U16);
        keywords.insert(b"u32", TokenKind::U32);
        keywords.insert(b"u64", TokenKind::U64);
        keywords.insert(b"f32", TokenKind::F32);
        keywords.insert(b"f64", TokenKind::F64);
        keywords.insert(b"bool", TokenKind::Bool);
        keywords.insert(b"char", TokenKind::Char);
        keywords.insert(b"ptr", TokenKind::Ptr);
        keywords.insert(b"str", TokenKind::Str);
        keywords.insert(b"void", TokenKind::Void);

        keywords
    };
}

pub struct Lexer<'a> {
    tokens: Vec<Token<'a>>,
    errors: Vec<ThrushCompilerIssue>,
    code: &'a [u8],
    start: usize,
    current: usize,
    line: usize,
    span: (usize, usize),
    diagnostician: Diagnostician,
}

impl<'a> Lexer<'a> {
    pub fn lex(code: &'a [u8], file: &'a CompilerFile) -> Vec<Token<'a>> {
        Self {
            tokens: Vec::with_capacity(MINIMAL_TOKENS_CAPACITY),
            errors: Vec::with_capacity(MINIMAL_ERROR_CAPACITY),
            code,
            start: 0,
            current: 0,
            line: 1,
            span: (0, 0),
            diagnostician: Diagnostician::new(file),
        }
        .start()
    }

    fn start(&mut self) -> Vec<Token<'a>> {
        while !self.end() {
            self.start = self.current;
            self.start_span();

            if let Err(error) = self.scan() {
                self.errors.push(error)
            }
        }

        if !self.errors.is_empty() {
            self.errors.iter().for_each(|error| {
                self.diagnostician
                    .build_diagnostic(error, LoggingType::Error);
            });

            process::exit(1);
        };

        self.tokens.push(Token {
            lexeme: "",
            kind: TokenKind::Eof,
            span: Span::new(self.line, self.span),
        });

        mem::take(&mut self.tokens)
    }

    fn scan(&mut self) -> Result<(), ThrushCompilerIssue> {
        match self.advance() {
            b'[' => self.make(TokenKind::LBracket),
            b']' => self.make(TokenKind::RBracket),
            b'(' => self.make(TokenKind::LParen),
            b')' => self.make(TokenKind::RParen),
            b'{' => self.make(TokenKind::LBrace),
            b'}' => self.make(TokenKind::RBrace),
            b',' => self.make(TokenKind::Comma),
            b'.' if self.char_match(b'.') => self.make(TokenKind::Range),
            b'.' => self.make(TokenKind::Dot),
            b'%' => self.make(TokenKind::Arith),
            b'*' => self.make(TokenKind::Star),
            b'/' if self.char_match(b'/') => loop {
                if self.peek() == b'\n' || self.end() {
                    break;
                }

                self.advance();
            },
            b'/' if self.char_match(b'*') => loop {
                if self.char_match(b'*') && self.char_match(b'/') {
                    break;
                } else if self.end() {
                    self.end_span();

                    let span: Span = Span::new(self.line, self.span);

                    return Err(ThrushCompilerIssue::Error(
                        String::from("Syntax Error"),
                        String::from(
                            "Unterminated multiline comment. Did you forget to close the comment with a '*/'?",
                        ),
                        None,
                        span,
                    ));
                }

                self.advance();
            },
            b'/' => self.make(TokenKind::Slash),
            b';' => self.make(TokenKind::SemiColon),
            b'-' if self.char_match(b'-') => self.make(TokenKind::MinusMinus),
            b'-' if self.char_match(b'=') => self.make(TokenKind::MinusEq),
            b'-' if self.char_match(b'>') => self.make(TokenKind::Arrow),
            b'-' => self.make(TokenKind::Minus),
            b'+' if self.char_match(b'+') => self.make(TokenKind::PlusPlus),
            b'+' if self.char_match(b'=') => self.make(TokenKind::PlusEq),
            b'+' => self.make(TokenKind::Plus),
            b':' if self.char_match(b':') => self.make(TokenKind::ColonColon),
            b':' => self.make(TokenKind::Colon),
            b'!' if self.char_match(b'=') => self.make(TokenKind::BangEq),
            b'!' => self.make(TokenKind::Bang),
            b'=' if self.char_match(b'=') => self.make(TokenKind::EqEq),
            b'=' => self.make(TokenKind::Eq),
            b'<' if self.char_match(b'=') => self.make(TokenKind::LessEq),
            b'<' if self.char_match(b'<') => self.make(TokenKind::LShift),
            b'<' => self.make(TokenKind::Less),
            b'>' if self.char_match(b'=') => self.make(TokenKind::GreaterEq),
            b'>' if self.char_match(b'>') => self.make(TokenKind::RShift),
            b'>' => self.make(TokenKind::Greater),
            b'|' if self.char_match(b'|') => self.make(TokenKind::Or),
            b'&' if self.char_match(b'&') => self.make(TokenKind::And),
            b' ' | b'\r' | b'\t' => {}
            b'\n' => self.line += 1,
            b'\'' => self.char()?,
            b'"' => self.string()?,
            b'0'..=b'9' => self.number()?,
            b'a'..=b'z' | b'A'..=b'Z' | b'_' | b'@' => self.identifier()?,
            _ => {
                self.end_span();

                let span: Span = Span::new(self.line, self.span);

                return Err(ThrushCompilerIssue::Error(
                    String::from("Unknown character"),
                    String::from("The compiler does not know how to handle this character."),
                    None,
                    span,
                ));
            }
        }

        Ok(())
    }

    fn identifier(&mut self) -> Result<(), ThrushCompilerIssue> {
        while self.is_alpha(self.peek())
            || self.peek().is_ascii_digit()
            || self.peek() == b'!' && self.peek() != b':'
        {
            self.advance();
        }

        let code: &[u8] = &self.code[self.start..self.current];

        if let Some(token_type) = KEYWORDS.get(code) {
            self.make(*token_type);
        } else {
            self.make(TokenKind::Identifier);
        }

        Ok(())
    }

    fn number(&mut self) -> Result<(), ThrushCompilerIssue> {
        let mut is_hexadecimal: bool = false;
        let mut is_binary: bool = false;

        while self.peek().is_ascii_digit()
            || self.peek() == b'_'
            || self.peek() == b'.'
            || self.peek() == b'x'
            || self.peek() == b'b'
            || is_hexadecimal
            || is_binary
        {
            if is_hexadecimal && self.previous() == b'0' && self.peek() == b'x' {
                self.end_span();

                return Err(ThrushCompilerIssue::Error(
                    String::from("Syntax error"),
                    String::from("Hexadecimal identifier '0x' cannot be repeated."),
                    None,
                    Span::new(self.line, self.span),
                ));
            }

            if is_binary && self.previous() == b'0' && self.peek() == b'b' {
                self.end_span();

                return Err(ThrushCompilerIssue::Error(
                    String::from("Syntax error"),
                    String::from("Binary identifier '0b' cannot be repeated."),
                    None,
                    Span::new(self.line, self.span),
                ));
            }

            if is_hexadecimal && !self.peek().is_ascii_alphanumeric() {
                self.end_span();
                break;
            }

            if is_binary && !self.peek().is_ascii_digit() {
                self.end_span();
                break;
            }

            if self.peek() == b'x' && self.peek_next().is_ascii_alphanumeric() {
                is_hexadecimal = true;
            }

            if self.peek() == b'b' && self.peek_next().is_ascii_digit() {
                is_binary = true;
            }

            self.advance();
        }

        self.end_span();

        let lexeme: &str = self.lexeme();

        self.check_number(lexeme)?;

        let span: Span = Span::new(self.line, self.span);

        if lexeme.contains(".") {
            self.tokens.push(Token {
                lexeme,
                kind: TokenKind::Float,
                span,
            });

            return Ok(());
        }

        self.tokens.push(Token {
            lexeme,
            kind: TokenKind::Integer,
            span,
        });

        Ok(())
    }

    #[inline]
    fn check_number(&mut self, lexeme: &str) -> Result<(), ThrushCompilerIssue> {
        if lexeme.contains('.') {
            return self.parse_float(lexeme);
        }

        self.parse_integer(lexeme)
    }

    #[inline]
    fn parse_float(&self, lexeme: &str) -> Result<(), ThrushCompilerIssue> {
        let dot_count: usize = lexeme.bytes().filter(|&b| b == b'.').count();

        let span: Span = Span::new(self.line, self.span);

        if dot_count > 1 {
            return Err(ThrushCompilerIssue::Error(
                String::from("Syntax error"),
                String::from("Float values should only contain one dot."),
                None,
                span,
            ));
        }

        if lexeme.parse::<f32>().is_ok() {
            return Ok(());
        }

        if lexeme.parse::<f64>().is_ok() {
            return Ok(());
        }

        Err(ThrushCompilerIssue::Error(
            String::from("Syntax error"),
            String::from("Out of bounds."),
            None,
            span,
        ))
    }

    #[inline]
    fn parse_integer(&self, lexeme: &str) -> Result<(), ThrushCompilerIssue> {
        const I8_MIN: isize = -128;
        const I8_MAX: isize = 127;
        const I16_MIN: isize = -32768;
        const I16_MAX: isize = 32767;
        const I32_MIN: isize = -2147483648;
        const I32_MAX: isize = 2147483647;

        const U8_MIN: usize = 0;
        const U8_MAX: usize = 255;
        const U16_MIN: usize = 0;
        const U16_MAX: usize = 65535;
        const U32_MIN: usize = 0;
        const U32_MAX: usize = 4294967295;

        let span: Span = Span::new(self.line, self.span);

        if lexeme.starts_with("0x") {
            let cleaned_lexeme: String = lexeme
                .strip_prefix("0x")
                .unwrap_or(&lexeme.replace("0x", ""))
                .replace("_", "");

            return match isize::from_str_radix(&cleaned_lexeme, 16) {
                Ok(num) => {
                    if (I8_MIN..=I8_MAX).contains(&num)
                        || (I16_MIN..=I16_MAX).contains(&num)
                        || (I32_MIN..=I32_MAX).contains(&num)
                        || (isize::MIN..=isize::MAX).contains(&num)
                    {
                        return Ok(());
                    } else {
                        return Err(ThrushCompilerIssue::Error(
                            String::from("Syntax error"),
                            String::from("Out of bounds signed hexadecimal format."),
                            None,
                            span,
                        ));
                    }
                }

                Err(_) => match usize::from_str_radix(&cleaned_lexeme, 16) {
                    Ok(num) => {
                        if (U8_MIN..=U8_MAX).contains(&num)
                            || (U16_MIN..=U16_MAX).contains(&num)
                            || (U32_MIN..=U32_MAX).contains(&num)
                            || (usize::MIN..=usize::MAX).contains(&num)
                        {
                            return Ok(());
                        } else {
                            return Err(ThrushCompilerIssue::Error(
                                String::from("Syntax error"),
                                String::from("Out of bounds unsigned hexadecimal format."),
                                None,
                                span,
                            ));
                        }
                    }

                    Err(_) => Err(ThrushCompilerIssue::Error(
                        String::from("Syntax error"),
                        String::from("Invalid numeric hexadecimal format."),
                        None,
                        span,
                    )),
                },
            };
        }

        if lexeme.starts_with("0b") {
            let cleaned_lexeme: String = lexeme
                .strip_prefix("0b")
                .unwrap_or(&lexeme.replace("0b", ""))
                .replace("_", "");

            return match isize::from_str_radix(&cleaned_lexeme, 2) {
                Ok(num) => {
                    if (I8_MIN..=I8_MAX).contains(&num)
                        || (I16_MIN..=I16_MAX).contains(&num)
                        || (I32_MIN..=I32_MAX).contains(&num)
                        || (isize::MIN..=isize::MAX).contains(&num)
                    {
                        return Ok(());
                    } else {
                        return Err(ThrushCompilerIssue::Error(
                            String::from("Syntax error"),
                            String::from("Out of bounds signed binary format."),
                            None,
                            span,
                        ));
                    }
                }

                Err(_) => match usize::from_str_radix(&cleaned_lexeme, 2) {
                    Ok(num) => {
                        if (U8_MIN..=U8_MAX).contains(&num)
                            || (U16_MIN..=U16_MAX).contains(&num)
                            || (U32_MIN..=U32_MAX).contains(&num)
                            || (usize::MIN..=usize::MAX).contains(&num)
                        {
                            return Ok(());
                        } else {
                            return Err(ThrushCompilerIssue::Error(
                                String::from("Syntax error"),
                                String::from("Out of bounds unsigned binary format."),
                                None,
                                span,
                            ));
                        }
                    }

                    Err(_) => Err(ThrushCompilerIssue::Error(
                        String::from("Syntax error"),
                        String::from("Invalid binary format."),
                        None,
                        span,
                    )),
                },
            };
        }

        match lexeme.parse::<usize>() {
            Ok(num) => {
                if (U8_MIN..=U8_MAX).contains(&num)
                    || (U16_MIN..=U16_MAX).contains(&num)
                    || (U32_MIN..=U32_MAX).contains(&num)
                    || (usize::MIN..=usize::MAX).contains(&num)
                {
                    Ok(())
                } else {
                    Err(ThrushCompilerIssue::Error(
                        String::from("Syntax error"),
                        String::from("Out of bounds."),
                        None,
                        span,
                    ))
                }
            }

            Err(_) => match lexeme.parse::<isize>() {
                Ok(num) => {
                    if (I8_MIN..=I8_MAX).contains(&num)
                        || (I16_MIN..=I16_MAX).contains(&num)
                        || (I32_MIN..=I32_MAX).contains(&num)
                        || (isize::MIN..=isize::MAX).contains(&num)
                    {
                        Ok(())
                    } else {
                        Err(ThrushCompilerIssue::Error(
                            String::from("Syntax error"),
                            String::from("Out of bounds."),
                            None,
                            span,
                        ))
                    }
                }

                Err(_) => Err(ThrushCompilerIssue::Error(
                    String::from("Syntax error"),
                    String::from("Out of bounds."),
                    None,
                    span,
                )),
            },
        }
    }

    fn char(&mut self) -> Result<(), ThrushCompilerIssue> {
        while self.peek() != b'\'' && !self.end() {
            self.advance();
        }

        self.end_span();

        let span: Span = Span::new(self.line, self.span);

        if self.peek() != b'\'' {
            return Err(ThrushCompilerIssue::Error(
                String::from("Syntax error"),
                String::from("Unclosed char. Did you forget to close the char with a \'?"),
                None,
                span,
            ));
        }

        self.advance();

        if self.code[self.start + 1..self.current - 1].len() > 1 {
            return Err(ThrushCompilerIssue::Error(
                String::from("Syntax error"),
                String::from("A char data type only can contain one character."),
                None,
                span,
            ));
        }

        let lexeme: &str = self.shrink_lexeme();

        self.tokens.push(Token {
            kind: TokenKind::Char,
            lexeme,
            span,
        });

        Ok(())
    }

    fn string(&mut self) -> Result<(), ThrushCompilerIssue> {
        while self.is_string_boundary() {
            self.advance();
        }

        self.end_span();

        let span: Span = Span::new(self.line, self.span);

        if self.peek() != b'"' {
            return Err(ThrushCompilerIssue::Error(
                String::from("Syntax error"),
                String::from(
                    "Unclosed literal str. Did you forget to close the literal str with a '\"'?",
                ),
                None,
                span,
            ));
        }

        self.advance();

        let lexeme: &str = self.shrink_lexeme();

        self.tokens.push(Token {
            kind: TokenKind::Str,
            lexeme,
            span,
        });

        Ok(())
    }

    fn make(&mut self, kind: TokenKind) {
        self.end_span();

        self.tokens.push(Token {
            lexeme: self.lexeme(),
            kind,
            span: Span::new(self.line, self.span),
        });
    }

    fn char_match(&mut self, ch: u8) -> bool {
        if !self.end() && self.code[self.current] == ch {
            self.current += 1;
            return true;
        }

        false
    }

    fn advance(&mut self) -> u8 {
        let char: u8 = self.code[self.current];
        self.current += 1;

        char
    }

    #[must_use]
    #[inline]
    fn lexeme(&self) -> &'a str {
        core::str::from_utf8(&self.code[self.start..self.current]).unwrap_or("�")
    }

    #[must_use]
    #[inline]
    fn shrink_lexeme(&self) -> &'a str {
        core::str::from_utf8(&self.code[self.start + 1..self.current - 1]).unwrap_or("�")
    }

    #[inline]
    fn start_span(&mut self) {
        self.span.0 = self.start;
    }

    #[inline]
    fn end_span(&mut self) {
        self.span.1 = self.current;
    }

    #[inline]
    fn peek_next(&self) -> u8 {
        if self.current + 1 >= self.code.len() {
            return b'\0';
        }

        self.code[self.current + 1]
    }

    #[must_use]
    fn previous(&self) -> u8 {
        self.code[self.current - 1]
    }

    #[must_use]
    #[inline]
    fn peek(&self) -> u8 {
        if self.end() {
            return b'\0';
        }

        self.code[self.current]
    }

    #[must_use]
    #[inline]
    fn is_string_boundary(&self) -> bool {
        self.peek() != b'"' && !self.end()
    }

    #[must_use]
    #[inline]
    fn end(&self) -> bool {
        self.current >= self.code.len()
    }

    #[must_use]
    #[inline]
    fn is_alpha(&self, char: u8) -> bool {
        char.is_ascii_lowercase() || char.is_ascii_uppercase() || char == b'_'
    }
}

#[derive(Debug)]
pub struct Token<'token> {
    pub lexeme: &'token str,
    pub kind: TokenKind,
    pub span: Span,
}

impl TokenExtensions for str {
    fn parse_scapes(&self, span: Span) -> Result<Vec<u8>, ThrushCompilerIssue> {
        let source: &[u8] = self.as_bytes();

        let mut parsed_string: Vec<u8> = Vec::with_capacity(source.len());

        let mut i: usize = 0;

        while i < self.len() {
            if source[i] == b'\\' {
                i += 1;

                match source.get(i) {
                    Some(b'n') => parsed_string.push(b'\n'),
                    Some(b't') => parsed_string.push(b'\t'),
                    Some(b'r') => parsed_string.push(b'\r'),
                    Some(b'\\') => parsed_string.push(b'\\'),
                    Some(b'0') => parsed_string.push(b'\0'),
                    Some(b'\'') => parsed_string.push(b'\''),
                    Some(b'"') => parsed_string.push(b'"'),
                    _ => {
                        return Err(ThrushCompilerIssue::Error(
                            String::from("Syntax Error"),
                            String::from("Invalid escape sequence."),
                            None,
                            span,
                        ));
                    }
                }

                i += 1;

                continue;
            }

            parsed_string.push(source[i]);

            i += 1;
        }

        Ok(parsed_string)
    }

    fn get_first_byte(&self) -> u8 {
        self.as_bytes()[0]
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct Span {
    pub line: usize,
    pub span: (usize, usize),
}

impl Display for Span {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.line, self.span.0)
    }
}

impl Span {
    pub fn new(line: usize, span: (usize, usize)) -> Self {
        Self { line, span }
    }

    pub fn get_line(&self) -> usize {
        self.line
    }

    pub fn get_span_start(&self) -> usize {
        self.span.0
    }

    pub fn get_span_end(&self) -> usize {
        self.span.0
    }
}
