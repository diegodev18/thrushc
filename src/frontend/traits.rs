use {
    super::super::error::ThrushError,
    super::objects::{Function, Local},
};

pub trait TokenLexeme {
    fn to_str(&self) -> &str;
    fn to_string(&self) -> String;
    fn parse_scapes(&self, line: usize, span: (usize, usize)) -> Result<String, ThrushError>;
}

pub trait ParserErrorsBasics {
    fn add_error(&self) -> &str;
}

pub trait FoundObjectEither {
    fn expected_local(&self, line: usize, span: (usize, usize)) -> Result<&Local, ThrushError>;
    fn expected_function(
        &self,
        line: usize,
        span: (usize, usize),
    ) -> Result<&Function, ThrushError>;
}
