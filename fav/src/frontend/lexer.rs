// Favnir Lexer
// Tasks: 1-1 (Token), 1-2 (Span), 1-3..1-12 (tokenization), 1-13 (tests)

// ── Span ─────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Span {
    pub file: String,
    pub start: usize, // byte offset (start)
    pub end: usize,   // byte offset (end, exclusive)
    pub line: u32,    // 1-based
    pub col: u32,     // 1-based
}

impl Span {
    pub fn new(file: impl Into<String>, start: usize, end: usize, line: u32, col: u32) -> Self {
        Span { file: file.into(), start, end, line, col }
    }

    /// A zero-position span used when the source location is not available.
    pub fn dummy() -> Self {
        Span { file: String::new(), start: 0, end: 0, line: 0, col: 0 }
    }
}

// ── TokenKind ────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    // Keywords
    Type,
    Fn,
    Trf,
    Flw,
    Bind,
    Match,
    If,
    Else,
    Public,
    Internal,
    Private,
    Namespace,
    Use,
    Interface,
    With,
    Cap,
    Impl,
    For,
    Chain,
    Yield,
    Collect,
    Where,
    Test,

    // Effect keywords
    Pure,
    Io,
    Emit,   // lowercase `emit` expression keyword

    // Symbols
    LArrow,     // <-
    PipeGt,     // |>
    Pipe,       // |
    Arrow,      // ->
    Bang,       // !
    Question,   // ?
    Colon,      // :
    Comma,      // ,
    Dot,        // .
    LBrace,     // {
    RBrace,     // }
    LParen,     // (
    RParen,     // )
    LAngle,     // <
    RAngle,     // >
    Eq,         // =
    FatArrow,   // =>
    Underscore, // _

    // Arithmetic / comparison operators
    Plus,       // +
    Minus,      // -
    Star,       // *
    Slash,      // /
    EqEq,       // ==
    BangEq,     // !=
    LtEq,       // <=
    GtEq,       // >=
    Semicolon,  // ;

    // Literals
    Int(i64),
    Float(f64),
    Str(String),
    Bool(bool),

    // Identifier
    Ident(String),

    // EOF
    Eof,
}

// ── Token ────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
}

impl Token {
    pub fn new(kind: TokenKind, span: Span) -> Self {
        Token { kind, span }
    }
}

// ── LexError ─────────────────────────────────────────────────────────────────

#[derive(Debug)]
pub struct LexError {
    pub message: String,
    pub span: Span,
}

impl LexError {
    pub fn new(message: impl Into<String>, span: Span) -> Self {
        LexError { message: message.into(), span }
    }
}

impl std::fmt::Display for LexError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "error: {}\n  --> {}:{}:{}",
            self.message, self.span.file, self.span.line, self.span.col
        )
    }
}

// ── Lexer ────────────────────────────────────────────────────────────────────

pub struct Lexer {
    source: Vec<char>,
    file: String,
    pos: usize,
    line: u32,
    col: u32,
}

impl Lexer {
    pub fn new(source: &str, file: impl Into<String>) -> Self {
        Lexer {
            source: source.chars().collect(),
            file: file.into(),
            pos: 0,
            line: 1,
            col: 1,
        }
    }

    pub fn tokenize(&mut self) -> Result<Vec<Token>, LexError> {
        let mut tokens = Vec::new();
        loop {
            self.skip_whitespace_and_comments();
            if self.is_eof() {
                tokens.push(Token::new(TokenKind::Eof, self.span_here()));
                break;
            }
            tokens.push(self.next_token()?);
        }
        Ok(tokens)
    }

    // ── helpers ──────────────────────────────────────────────────────────────

    fn is_eof(&self) -> bool {
        self.pos >= self.source.len()
    }

    fn peek(&self) -> Option<char> {
        self.source.get(self.pos).copied()
    }

    fn peek2(&self) -> Option<char> {
        self.source.get(self.pos + 1).copied()
    }

    fn advance(&mut self) -> char {
        let c = self.source[self.pos];
        self.pos += 1;
        if c == '\n' {
            self.line += 1;
            self.col = 1;
        } else {
            self.col += 1;
        }
        c
    }

    fn span_here(&self) -> Span {
        Span::new(&self.file, self.pos, self.pos, self.line, self.col)
    }

    fn span_from(&self, start_pos: usize, start_line: u32, start_col: u32) -> Span {
        Span::new(&self.file, start_pos, self.pos, start_line, start_col)
    }

    // ── whitespace / comments ─────────────────────────────────────────────────

    fn skip_whitespace_and_comments(&mut self) {
        loop {
            match self.peek() {
                Some(' ') | Some('\t') | Some('\r') | Some('\n') => {
                    self.advance();
                }
                Some('/') if self.peek2() == Some('/') => {
                    while !self.is_eof() && self.peek() != Some('\n') {
                        self.advance();
                    }
                }
                _ => break,
            }
        }
    }

    // ── main dispatch ─────────────────────────────────────────────────────────

    fn next_token(&mut self) -> Result<Token, LexError> {
        let sp = self.pos;
        let sl = self.line;
        let sc = self.col;

        let c = self.peek().unwrap();

        let kind = match c {
            '{' => { self.advance(); TokenKind::LBrace }
            '}' => { self.advance(); TokenKind::RBrace }
            '(' => { self.advance(); TokenKind::LParen }
            ')' => { self.advance(); TokenKind::RParen }
            '?' => { self.advance(); TokenKind::Question }
            ':' => { self.advance(); TokenKind::Colon }
            ',' => { self.advance(); TokenKind::Comma }
            '.' => { self.advance(); TokenKind::Dot }
            '+' => { self.advance(); TokenKind::Plus }
            '*' => { self.advance(); TokenKind::Star }
            ';' => { self.advance(); TokenKind::Semicolon }

            '=' => {
                self.advance();
                if self.peek() == Some('=') {
                    self.advance();
                    TokenKind::EqEq
                } else if self.peek() == Some('>') {
                    self.advance();
                    TokenKind::FatArrow
                } else {
                    TokenKind::Eq
                }
            }

            '!' => {
                self.advance();
                if self.peek() == Some('=') {
                    self.advance();
                    TokenKind::BangEq
                } else {
                    TokenKind::Bang
                }
            }

            '<' => {
                self.advance();
                match self.peek() {
                    Some('-') => { self.advance(); TokenKind::LArrow }
                    Some('=') => { self.advance(); TokenKind::LtEq }
                    _ => TokenKind::LAngle,
                }
            }

            '>' => {
                self.advance();
                if self.peek() == Some('=') {
                    self.advance();
                    TokenKind::GtEq
                } else {
                    TokenKind::RAngle
                }
            }

            '-' => {
                self.advance();
                if self.peek() == Some('>') {
                    self.advance();
                    TokenKind::Arrow
                } else {
                    TokenKind::Minus
                }
            }

            '|' => {
                self.advance();
                if self.peek() == Some('>') {
                    self.advance();
                    TokenKind::PipeGt
                } else {
                    TokenKind::Pipe
                }
            }

            '/' => {
                self.advance();
                TokenKind::Slash
            }

            '"' => self.lex_string()?,

            '0'..='9' => self.lex_number()?,

            '_' => {
                self.advance();
                if self.peek().map(|ch| ch.is_alphanumeric() || ch == '_').unwrap_or(false) {
                    let mut name = String::from('_');
                    while self.peek().map(|ch| ch.is_alphanumeric() || ch == '_').unwrap_or(false) {
                        name.push(self.advance());
                    }
                    TokenKind::Ident(name)
                } else {
                    TokenKind::Underscore
                }
            }

            c if c.is_alphabetic() => self.lex_ident_or_keyword(),

            other => {
                self.advance();
                let span = self.span_from(sp, sl, sc);
                return Err(LexError::new(
                    format!("unexpected character '{}'", other),
                    span,
                ));
            }
        };

        Ok(Token::new(kind, self.span_from(sp, sl, sc)))
    }

    // ── identifier / keyword ──────────────────────────────────────────────────

    fn lex_ident_or_keyword(&mut self) -> TokenKind {
        let mut name = String::new();
        while self.peek().map(|c| c.is_alphanumeric() || c == '_').unwrap_or(false) {
            name.push(self.advance());
        }
        match name.as_str() {
            "type"    => TokenKind::Type,
            "fn"      => TokenKind::Fn,
            "trf"     => TokenKind::Trf,
            "flw"     => TokenKind::Flw,
            "bind"    => TokenKind::Bind,
            "match"   => TokenKind::Match,
            "if"      => TokenKind::If,
            "else"    => TokenKind::Else,
            "public"    => TokenKind::Public,
            "internal"  => TokenKind::Internal,
            "private"   => TokenKind::Private,
            "namespace" => TokenKind::Namespace,
            "use"       => TokenKind::Use,
            "interface" => TokenKind::Interface,
            "with"      => TokenKind::With,
            "cap"       => TokenKind::Cap,
            "impl"      => TokenKind::Impl,
            "for"       => TokenKind::For,
            "chain"     => TokenKind::Chain,
            "yield"     => TokenKind::Yield,
            "collect"   => TokenKind::Collect,
            "where"     => TokenKind::Where,
            "test"      => TokenKind::Test,
            "Pure"    => TokenKind::Pure,
            "Io"      => TokenKind::Io,
            "emit"    => TokenKind::Emit,
            "true"    => TokenKind::Bool(true),
            "false"   => TokenKind::Bool(false),
            _         => TokenKind::Ident(name),
        }
    }

    // ── number ────────────────────────────────────────────────────────────────

    fn lex_number(&mut self) -> Result<TokenKind, LexError> {
        let mut s = String::new();
        let mut is_float = false;

        while self.peek().map(|c| c.is_ascii_digit()).unwrap_or(false) {
            s.push(self.advance());
        }

        if self.peek() == Some('.')
            && self.peek2().map(|c| c.is_ascii_digit()).unwrap_or(false)
        {
            is_float = true;
            s.push(self.advance()); // '.'
            while self.peek().map(|c| c.is_ascii_digit()).unwrap_or(false) {
                s.push(self.advance());
            }
        }

        if is_float {
            s.parse::<f64>()
                .map(TokenKind::Float)
                .map_err(|_| LexError::new(format!("invalid float '{}'", s), self.span_here()))
        } else {
            s.parse::<i64>()
                .map(TokenKind::Int)
                .map_err(|_| LexError::new(format!("invalid integer '{}'", s), self.span_here()))
        }
    }

    // ── string ────────────────────────────────────────────────────────────────

    fn lex_string(&mut self) -> Result<TokenKind, LexError> {
        self.advance(); // opening "
        let mut s = String::new();
        loop {
            match self.peek() {
                None => {
                    return Err(LexError::new("unterminated string literal", self.span_here()));
                }
                Some('"') => {
                    self.advance();
                    break;
                }
                Some('\\') => {
                    self.advance();
                    match self.peek() {
                        Some('n')  => { self.advance(); s.push('\n'); }
                        Some('t')  => { self.advance(); s.push('\t'); }
                        Some('r')  => { self.advance(); s.push('\r'); }
                        Some('"')  => { self.advance(); s.push('"'); }
                        Some('\\') => { self.advance(); s.push('\\'); }
                        Some(c) => {
                            let span = self.span_here();
                            return Err(LexError::new(
                                format!("unknown escape '\\{}'", c),
                                span,
                            ));
                        }
                        None => {
                            return Err(LexError::new(
                                "unterminated string escape",
                                self.span_here(),
                            ));
                        }
                    }
                }
                Some(_) => {
                    s.push(self.advance());
                }
            }
        }
        Ok(TokenKind::Str(s))
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn lex(src: &str) -> Vec<TokenKind> {
        Lexer::new(src, "test")
            .tokenize()
            .expect("lex error")
            .into_iter()
            .map(|t| t.kind)
            .collect()
    }

    fn lex_err(src: &str) -> String {
        Lexer::new(src, "test")
            .tokenize()
            .expect_err("expected lex error")
            .message
    }

    // keywords
    #[test]
    fn test_keywords() {
        let kinds = lex("type fn trf flw bind match if else public internal private namespace use interface with cap impl for");
        assert_eq!(kinds, vec![
            TokenKind::Type, TokenKind::Fn, TokenKind::Trf, TokenKind::Flw,
            TokenKind::Bind, TokenKind::Match, TokenKind::If, TokenKind::Else,
            TokenKind::Public, TokenKind::Internal, TokenKind::Private,
            TokenKind::Namespace, TokenKind::Use,
            TokenKind::Interface, TokenKind::With,
            TokenKind::Cap, TokenKind::Impl, TokenKind::For,
            TokenKind::Eof,
        ]);
    }

    // v0.5.0 keywords
    #[test]
    fn test_v05_keywords() {
        let kinds = lex("chain yield collect where");
        assert_eq!(kinds, vec![
            TokenKind::Chain, TokenKind::Yield, TokenKind::Collect, TokenKind::Where,
            TokenKind::Eof,
        ]);
    }

    #[test]
    fn test_effect_keywords() {
        let kinds = lex("Pure Io");
        assert_eq!(kinds, vec![TokenKind::Pure, TokenKind::Io, TokenKind::Eof]);
    }

    // symbols
    #[test]
    fn test_symbols() {
        let kinds = lex("<- |> | -> ! ? : , . { } ( ) = _");
        assert_eq!(kinds, vec![
            TokenKind::LArrow, TokenKind::PipeGt, TokenKind::Pipe, TokenKind::Arrow,
            TokenKind::Bang, TokenKind::Question, TokenKind::Colon, TokenKind::Comma,
            TokenKind::Dot, TokenKind::LBrace, TokenKind::RBrace,
            TokenKind::LParen, TokenKind::RParen,
            TokenKind::Eq, TokenKind::Underscore,
            TokenKind::Eof,
        ]);
    }

    #[test]
    fn test_angle_brackets() {
        let kinds = lex("< >");
        assert_eq!(kinds, vec![TokenKind::LAngle, TokenKind::RAngle, TokenKind::Eof]);
    }

    #[test]
    fn test_operators() {
        let kinds = lex("+ - * / == != <= >=");
        assert_eq!(kinds, vec![
            TokenKind::Plus, TokenKind::Minus, TokenKind::Star, TokenKind::Slash,
            TokenKind::EqEq, TokenKind::BangEq, TokenKind::LtEq, TokenKind::GtEq,
            TokenKind::Eof,
        ]);
    }

    // integer literals
    #[test]
    fn test_int_literal() {
        let kinds = lex("42 0 1000");
        assert_eq!(kinds, vec![
            TokenKind::Int(42), TokenKind::Int(0), TokenKind::Int(1000),
            TokenKind::Eof,
        ]);
    }

    // float literals
    #[test]
    fn test_float_literal() {
        let kinds = lex("3.14 0.5");
        assert_eq!(kinds, vec![
            TokenKind::Float(3.14), TokenKind::Float(0.5),
            TokenKind::Eof,
        ]);
    }

    // string literals
    #[test]
    fn test_string_literal() {
        let kinds = lex(r#""hello" "world""#);
        assert_eq!(kinds, vec![
            TokenKind::Str("hello".into()),
            TokenKind::Str("world".into()),
            TokenKind::Eof,
        ]);
    }

    #[test]
    fn test_string_escapes() {
        let kinds = lex(r#""\n\t\"\\""#);
        assert_eq!(kinds, vec![
            TokenKind::Str("\n\t\"\\".into()),
            TokenKind::Eof,
        ]);
    }

    // bool literals
    #[test]
    fn test_bool_literal() {
        let kinds = lex("true false");
        assert_eq!(kinds, vec![
            TokenKind::Bool(true), TokenKind::Bool(false),
            TokenKind::Eof,
        ]);
    }

    // identifiers
    #[test]
    fn test_identifiers() {
        let kinds = lex("foo bar_baz _private MyType");
        assert_eq!(kinds, vec![
            TokenKind::Ident("foo".into()),
            TokenKind::Ident("bar_baz".into()),
            TokenKind::Ident("_private".into()),
            TokenKind::Ident("MyType".into()),
            TokenKind::Eof,
        ]);
    }

    // underscore as wildcard
    #[test]
    fn test_underscore_standalone() {
        let kinds = lex("_");
        assert_eq!(kinds, vec![TokenKind::Underscore, TokenKind::Eof]);
    }

    // comment skipping
    #[test]
    fn test_comment_skip() {
        let kinds = lex("foo // this is a comment\nbar");
        assert_eq!(kinds, vec![
            TokenKind::Ident("foo".into()),
            TokenKind::Ident("bar".into()),
            TokenKind::Eof,
        ]);
    }

    // whitespace skipping
    #[test]
    fn test_whitespace_skip() {
        let kinds = lex("  foo   \t  bar  \n  baz  ");
        assert_eq!(kinds, vec![
            TokenKind::Ident("foo".into()),
            TokenKind::Ident("bar".into()),
            TokenKind::Ident("baz".into()),
            TokenKind::Eof,
        ]);
    }

    // error: unexpected character
    #[test]
    fn test_unexpected_char() {
        let msg = lex_err("foo @ bar");
        assert!(msg.contains("unexpected character '@'"));
    }

    // error: unterminated string
    #[test]
    fn test_unterminated_string() {
        let msg = lex_err("\"hello");
        assert!(msg.contains("unterminated string literal"));
    }

    // error: unknown escape
    #[test]
    fn test_unknown_escape() {
        let msg = lex_err(r#""\q""#);
        assert!(msg.contains("unknown escape"));
    }

    // span: line and column
    #[test]
    fn test_span_line_col() {
        let tokens = Lexer::new("foo\nbar", "f.fav").tokenize().unwrap();
        assert_eq!(tokens[0].span.line, 1);
        assert_eq!(tokens[0].span.col,  1);
        assert_eq!(tokens[1].span.line, 2);
        assert_eq!(tokens[1].span.col,  1);
    }

    // a realistic snippet
    #[test]
    fn test_trf_snippet() {
        let src = "trf ParseCsv: String -> List<Row> = |text| { text }";
        let kinds = lex(src);
        assert_eq!(kinds, vec![
            TokenKind::Trf,
            TokenKind::Ident("ParseCsv".into()),
            TokenKind::Colon,
            TokenKind::Ident("String".into()),
            TokenKind::Arrow,
            TokenKind::Ident("List".into()),
            TokenKind::LAngle,
            TokenKind::Ident("Row".into()),
            TokenKind::RAngle,
            TokenKind::Eq,
            TokenKind::Pipe,
            TokenKind::Ident("text".into()),
            TokenKind::Pipe,
            TokenKind::LBrace,
            TokenKind::Ident("text".into()),
            TokenKind::RBrace,
            TokenKind::Eof,
        ]);
    }
}
