// Favnir Parser
// Tasks: 3-1..3-23

use super::lexer::{LexError, Lexer, Span, Token, TokenKind};
use crate::ast::*;

// ── ParseError (3-2) ──────────────────────────────────────────────────────────

#[derive(Debug)]
pub struct ParseError {
    pub message: String,
    pub span: Span,
}

impl ParseError {
    pub fn new(message: impl Into<String>, span: Span) -> Self {
        ParseError {
            message: message.into(),
            span,
        }
    }
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "error: {}\n  --> {}:{}:{}",
            self.message, self.span.file, self.span.line, self.span.col
        )
    }
}

impl From<LexError> for ParseError {
    fn from(e: LexError) -> Self {
        ParseError::new(e.message, e.span)
    }
}

// ── Parser (3-1) ─────────────────────────────────────────────────────────────

pub struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

/// Desugar `Ctx { fields }` destructure to a concrete ctx type name (v13.10.0).
fn desugar_ctx_fields(fields: &[(String, Option<String>)]) -> &'static str {
    let has_db_read = fields.iter().any(|(_, ty)| ty.as_deref() == Some("DbRead"));
    let has_db_write = fields.iter().any(|(_, ty)| ty.as_deref() == Some("DbWrite"));
    let has_storage_write = fields.iter().any(|(_, ty)| ty.as_deref() == Some("StorageWrite"));
    let has_io_only = fields.len() == 1 && fields[0].0 == "io" && fields[0].1.is_none();
    let has_env_only = fields.len() == 1 && fields[0].0 == "env" && fields[0].1.is_none();

    if has_io_only || has_env_only {
        return "CommonCtx";
    }
    if has_db_read && !has_db_write && !has_storage_write {
        return "LoadCtx";
    }
    if has_db_write || has_storage_write {
        return "WriteCtx";
    }
    // Fallback for mixed or unrecognized combinations
    "AppCtx"
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Parser { tokens, pos: 0 }
    }

    /// Parse source text directly.
    pub fn parse_str(source: &str, file: &str) -> Result<Program, ParseError> {
        let tokens = Lexer::new(source, file).tokenize()?;
        Parser::new(tokens).parse_program()
    }

    pub fn parse_str_expr(source: &str, file: &str) -> Result<Expr, ParseError> {
        let tokens = Lexer::new(source, file).tokenize()?;
        let mut parser = Parser::new(tokens);
        let expr = parser.parse_expr()?;
        if parser.peek() != &TokenKind::Eof {
            return Err(ParseError::new(
                format!("unexpected token after expression: {:?}", parser.peek()),
                parser.peek_span().clone(),
            ));
        }
        Ok(expr)
    }

    // ── helpers ───────────────────────────────────────────────────────────────

    fn peek(&self) -> &TokenKind {
        &self.tokens[self.pos].kind
    }

    fn peek2(&self) -> Option<&TokenKind> {
        self.tokens.get(self.pos + 1).map(|t| &t.kind)
    }

    fn peek_span(&self) -> &Span {
        &self.tokens[self.pos].span
    }

    fn advance(&mut self) -> &Token {
        let tok = &self.tokens[self.pos];
        if self.pos + 1 < self.tokens.len() {
            self.pos += 1;
        }
        tok
    }

    fn expect(&mut self, expected: &TokenKind) -> Result<Span, ParseError> {
        if self.peek() == expected {
            let span = self.peek_span().clone();
            self.advance();
            Ok(span)
        } else {
            Err(ParseError::new(
                format!("expected {:?}, got {:?}", expected, self.peek()),
                self.peek_span().clone(),
            ))
        }
    }

    /// Consume the current token if it matches any of the given kinds.
    #[allow(dead_code)]
    fn expect_any(&mut self, expected: &[TokenKind]) -> Result<Span, ParseError> {
        if expected.iter().any(|k| k == self.peek()) {
            let span = self.peek_span().clone();
            self.advance();
            Ok(span)
        } else {
            Err(ParseError::new(
                format!("expected one of {:?}, got {:?}", expected, self.peek()),
                self.peek_span().clone(),
            ))
        }
    }

    fn expect_ident(&mut self) -> Result<(String, Span), ParseError> {
        let span = self.peek_span().clone();
        match self.peek().clone() {
            TokenKind::Ident(name) => {
                self.advance();
                Ok((name, span))
            }
            // v22.5.0: `pipeline` は予約語だが、識別子として使われている既存コードとの
            // 互換性のため `use pipeline.{...}` 等の文脈では識別子として扱う。
            TokenKind::Pipeline => {
                self.advance();
                Ok(("pipeline".to_string(), span))
            }
            other => Err(ParseError::new(
                format!("expected identifier, got {:?}", other),
                span,
            )),
        }
    }

    fn at_end(&self) -> bool {
        matches!(self.peek(), TokenKind::Eof)
    }

    fn span_from(&self, start: &Span) -> Span {
        Span::new(
            &start.file,
            start.start,
            self.tokens[self.pos.saturating_sub(1)].span.end,
            start.line,
            start.col,
        )
    }

    // ── program ───────────────────────────────────────────────────────────────

    pub fn parse_program(&mut self) -> Result<Program, ParseError> {
        // 1. namespace (optional, must come first)
        let namespace = if self.peek() == &TokenKind::Namespace {
            Some(self.parse_namespace_decl()?)
        } else {
            None
        };

        // 2. use declarations (zero or more)
        // Skip `use X.{ ... }` and `use X.*` patterns — those are rune-internal
        // imports parsed as items, not namespace-path uses.
        let mut uses = Vec::new();
        while self.peek() == &TokenKind::Use && !self.is_rune_use_pattern() {
            uses.push(self.parse_use_decl()?);
        }

        // 3. top-level definitions
        let mut items = Vec::new();
        while !self.at_end() {
            items.push(self.parse_item()?);
        }
        Ok(Program {
            namespace,
            uses,
            items,
        })
    }

    fn parse_module_path(&mut self) -> Result<Vec<String>, ParseError> {
        let mut parts = Vec::new();
        let (first, _) = self.expect_ident()?;
        parts.push(first);
        while self.peek() == &TokenKind::Dot {
            self.advance();
            let (seg, _) = self.expect_ident()?;
            parts.push(seg);
        }
        Ok(parts)
    }

    fn parse_namespace_decl(&mut self) -> Result<String, ParseError> {
        self.expect(&TokenKind::Namespace)?;
        let parts = self.parse_module_path()?;
        Ok(parts.join("."))
    }

    /// Returns true when the upcoming tokens match `use Ident . { ...` or `use Ident . *`,
    /// i.e. the rune-internal import syntax rather than a namespace path.
    fn is_rune_use_pattern(&self) -> bool {
        // tokens at pos: Use
        // pos+1: Ident (module name)
        // pos+2: Dot
        // pos+3: LBrace or Star
        let t2 = self.tokens.get(self.pos + 2).map(|t| &t.kind);
        let t3 = self.tokens.get(self.pos + 3).map(|t| &t.kind);
        // `use X.{ a, b }` or `use X.*` — rune-internal import
        let is_rune_use = matches!(t2, Some(TokenKind::Dot))
            && matches!(t3, Some(TokenKind::LBrace) | Some(TokenKind::Star));
        // `use X as Y` — namespace alias (v16.6.0), must also be parsed as item
        let is_alias_use = matches!(t2, Some(TokenKind::As));
        is_rune_use || is_alias_use
    }

    fn parse_use_decl(&mut self) -> Result<Vec<String>, ParseError> {
        self.expect(&TokenKind::Use)?;
        self.parse_module_path()
    }

    fn peek_ident_text(&self, expected: &str) -> bool {
        matches!(self.peek(), TokenKind::Ident(name) if name == expected)
    }

    // ── item ──────────────────────────────────────────────────────────────────

    /// Expect an identifier token with a specific name; consume and return error otherwise.
    fn expect_ident_name(&mut self, expected: &str) -> Result<(), ParseError> {
        match self.peek().clone() {
            TokenKind::Ident(n) if n == expected => {
                self.advance();
                Ok(())
            }
            other => Err(ParseError::new(
                format!("expected `{}`, got {:?}", expected, other),
                self.peek_span().clone(),
            )),
        }
    }

    /// Expect a string literal token; consume and return the string value.
    fn expect_str(&mut self) -> Result<String, ParseError> {
        match self.peek().clone() {
            TokenKind::Str(s) => {
                self.advance();
                Ok(s)
            }
            other => Err(ParseError::new(
                format!("expected string literal, got {:?}", other),
                self.peek_span().clone(),
            )),
        }
    }

    /// Try to parse `#[api(method = "...", path = "...")]` before a fn definition.
    /// Returns None if the next tokens are not `#[api(`.
    /// v24.4.0: `#[deprecated]` を認識して bool を返す。
    fn parse_deprecated_annotation(&mut self) -> Result<bool, ParseError> {
        if self.peek() == &TokenKind::Hash
            && matches!(self.tokens.get(self.pos + 1), Some(t) if t.kind == TokenKind::LBracket)
            && matches!(self.tokens.get(self.pos + 2), Some(t) if matches!(&t.kind, TokenKind::Ident(n) if n == "deprecated"))
            && matches!(self.tokens.get(self.pos + 3), Some(t) if t.kind == TokenKind::RBracket)
        {
            self.advance(); // #
            self.advance(); // [
            self.advance(); // deprecated
            self.advance(); // ]
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// v46.1.0: `#[test]` アトリビュートを認識して bool を返す。
    /// 注: `test` は TokenKind::Test（キーワード）であり Ident ではない。
    fn parse_test_annotation(&mut self) -> Result<bool, ParseError> {
        if self.peek() == &TokenKind::Hash
            && matches!(self.tokens.get(self.pos + 1), Some(t) if t.kind == TokenKind::LBracket)
            && matches!(self.tokens.get(self.pos + 2), Some(t) if t.kind == TokenKind::Test)
            && matches!(self.tokens.get(self.pos + 3), Some(t) if t.kind == TokenKind::RBracket)
        {
            self.advance(); // #
            self.advance(); // [
            self.advance(); // test
            self.advance(); // ]
            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn parse_api_annotation(&mut self) -> Result<Option<crate::ast::ApiAnnotation>, ParseError> {
        // Lookahead: #  [  api
        let is_api = self.peek() == &TokenKind::Hash
            && matches!(self.tokens.get(self.pos + 1), Some(t) if t.kind == TokenKind::LBracket)
            && matches!(self.tokens.get(self.pos + 2), Some(t) if matches!(&t.kind, TokenKind::Ident(n) if n == "api"));
        if !is_api {
            return Ok(None);
        }
        let start = self.peek_span().clone();
        self.advance(); // #
        self.expect(&TokenKind::LBracket)?;
        self.expect_ident_name("api")?;
        self.expect(&TokenKind::LParen)?;
        // method = "..."
        self.expect_ident_name("method")?;
        self.expect(&TokenKind::Eq)?;
        let method = self.expect_str()?;
        self.expect(&TokenKind::Comma)?;
        // path = "..."
        self.expect_ident_name("path")?;
        self.expect(&TokenKind::Eq)?;
        let path = self.expect_str()?;
        // optional trailing comma
        if self.peek() == &TokenKind::Comma {
            self.advance();
        }
        self.expect(&TokenKind::RParen)?;
        self.expect(&TokenKind::RBracket)?;
        Ok(Some(crate::ast::ApiAnnotation { method, path, span: self.span_from(&start) }))
    }

    fn parse_streaming_annotation(&mut self) -> Result<Option<crate::ast::StreamingAnnotation>, ParseError> {
        // Lookahead: # [ streaming
        let is_streaming = self.peek() == &TokenKind::Hash
            && matches!(self.tokens.get(self.pos + 1), Some(t) if t.kind == TokenKind::LBracket)
            && matches!(self.tokens.get(self.pos + 2), Some(t) if matches!(&t.kind, TokenKind::Ident(n) if n == "streaming"));
        if !is_streaming {
            return Ok(None);
        }
        let start = self.peek_span().clone();
        self.advance(); // #
        self.expect(&TokenKind::LBracket)?;
        self.expect_ident_name("streaming")?;
        let mut chunk_size: Option<i64> = None;
        let mut backpressure: Option<bool> = None;
        if self.peek() == &TokenKind::LParen {
            self.advance(); // (
            // v26.4.0: support multiple comma-separated key=value pairs
            loop {
                let key = match self.peek().clone() {
                    TokenKind::Ident(k) => { self.advance(); k }
                    _ => break, // RParen or end
                };
                self.expect(&TokenKind::Eq)?;
                match key.as_str() {
                    "chunk_size" => {
                        chunk_size = Some(match self.peek().clone() {
                            TokenKind::Int(n) => { self.advance(); n }
                            other => return Err(ParseError::new(
                                format!("expected integer for chunk_size, got {:?}", other),
                                self.peek_span().clone(),
                            )),
                        });
                    }
                    "backpressure" => {
                        backpressure = Some(match self.peek().clone() {
                            TokenKind::Bool(b) => { self.advance(); b }
                            other => return Err(ParseError::new(
                                format!("expected bool for backpressure, got {:?}", other),
                                self.peek_span().clone(),
                            )),
                        });
                    }
                    _ => {
                        // unknown key: skip value token for forward compatibility
                        self.advance();
                    }
                }
                if self.peek() == &TokenKind::Comma {
                    self.advance(); // ,
                } else {
                    break;
                }
            }
            self.expect(&TokenKind::RParen)?;
        }
        self.expect(&TokenKind::RBracket)?;
        Ok(Some(crate::ast::StreamingAnnotation { chunk_size, backpressure, span: self.span_from(&start) }))
    }

    fn parse_stateful_annotation(&mut self) -> Result<bool, ParseError> {
        // Lookahead: # [ stateful ]
        let is_stateful = self.peek() == &TokenKind::Hash
            && matches!(self.tokens.get(self.pos + 1), Some(t) if t.kind == TokenKind::LBracket)
            && matches!(self.tokens.get(self.pos + 2), Some(t) if matches!(&t.kind, TokenKind::Ident(n) if n == "stateful"));
        if !is_stateful {
            return Ok(false);
        }
        self.advance(); // #
        self.expect(&TokenKind::LBracket)?;
        self.expect_ident_name("stateful")?;
        self.expect(&TokenKind::RBracket)?;
        Ok(true)
    }

    fn parse_arrow_annotation(&mut self) -> Result<bool, ParseError> {
        // Lookahead: # [ arrow ]
        let is_arrow = self.peek() == &TokenKind::Hash
            && matches!(self.tokens.get(self.pos + 1), Some(t) if t.kind == TokenKind::LBracket)
            && matches!(self.tokens.get(self.pos + 2), Some(t) if matches!(&t.kind, TokenKind::Ident(n) if n == "arrow"));
        if !is_arrow {
            return Ok(false);
        }
        self.advance(); // #
        self.expect(&TokenKind::LBracket)?;
        self.expect_ident_name("arrow")?;
        self.expect(&TokenKind::RBracket)?;
        Ok(true)
    }

    /// v22.1.0: parse optional `#[checkpoint]` annotation on stage definitions.
    fn parse_checkpoint_annotation(&mut self) -> Result<bool, ParseError> {
        // Lookahead: # [ checkpoint ]
        let is_checkpoint = self.peek() == &TokenKind::Hash
            && matches!(self.tokens.get(self.pos + 1), Some(t) if t.kind == TokenKind::LBracket)
            && matches!(self.tokens.get(self.pos + 2), Some(t) if matches!(&t.kind, TokenKind::Ident(n) if n == "checkpoint"));
        if !is_checkpoint {
            return Ok(false);
        }
        self.advance(); // #
        self.expect(&TokenKind::LBracket)?;
        self.expect_ident_name("checkpoint")?;
        self.expect(&TokenKind::RBracket)?;
        Ok(true)
    }

    /// v22.4.0: parse optional `#[trigger(event = "...", bucket/topic = "...")]` annotation on seq.
    fn parse_trigger_annotation(&mut self) -> Result<Option<crate::ast::TriggerAnnotation>, ParseError> {
        // Lookahead: # [ trigger
        let is_trigger = self.peek() == &TokenKind::Hash
            && matches!(self.tokens.get(self.pos + 1), Some(t) if t.kind == TokenKind::LBracket)
            && matches!(self.tokens.get(self.pos + 2), Some(t) if matches!(&t.kind, TokenKind::Ident(n) if n == "trigger"));
        if !is_trigger {
            return Ok(None);
        }
        let start = self.peek_span().clone();
        self.advance(); // #
        self.expect(&TokenKind::LBracket)?;
        self.expect_ident_name("trigger")?;
        self.expect(&TokenKind::LParen)?;
        // event = "..."
        self.expect_ident_name("event")?;
        self.expect(&TokenKind::Eq)?;
        let event = self.expect_str()?;
        // optional: , bucket = "..." OR , topic = "..."
        let mut bucket: Option<String> = None;
        let mut topic: Option<String> = None;
        while self.peek() == &TokenKind::Comma {
            self.advance(); // ,
            if self.peek() == &TokenKind::RParen { break; } // trailing comma
            let (key, _) = self.expect_ident()?;
            self.expect(&TokenKind::Eq)?;
            let val = self.expect_str()?;
            match key.as_str() {
                "bucket" => bucket = Some(val),
                "topic"  => topic  = Some(val),
                other    => return Err(ParseError::new(
                    format!("unknown trigger key `{}`; expected `bucket` or `topic`", other),
                    self.peek_span().clone(),
                )),
            }
        }
        self.expect(&TokenKind::RParen)?;
        self.expect(&TokenKind::RBracket)?;
        Ok(Some(crate::ast::TriggerAnnotation {
            event,
            bucket,
            topic,
            span: self.span_from(&start),
        }))
    }

    /// v22.6.0: parse optional `#[timeout(seconds = N)]` annotation on stage definitions.
    fn parse_timeout_annotation(&mut self) -> Result<Option<crate::ast::TimeoutAnnotation>, ParseError> {
        let is_timeout = self.peek() == &TokenKind::Hash
            && matches!(self.tokens.get(self.pos + 1), Some(t) if t.kind == TokenKind::LBracket)
            && matches!(self.tokens.get(self.pos + 2), Some(t) if matches!(&t.kind, TokenKind::Ident(n) if n == "timeout"));
        if !is_timeout {
            return Ok(None);
        }
        let start = self.peek_span().clone();
        self.advance();                          // #
        self.expect(&TokenKind::LBracket)?;      // [
        self.expect_ident_name("timeout")?;
        self.expect(&TokenKind::LParen)?;
        self.expect_ident_name("seconds")?;
        self.expect(&TokenKind::Eq)?;
        let seconds = match self.peek().clone() {
            TokenKind::Int(n) => { self.advance(); n as f64 }
            TokenKind::Float(f) => { self.advance(); f }
            other => return Err(ParseError::new(
                format!("expected number after `seconds =`, got {:?}", other),
                self.peek_span().clone(),
            )),
        };
        self.expect(&TokenKind::RParen)?;
        self.expect(&TokenKind::RBracket)?;
        Ok(Some(crate::ast::TimeoutAnnotation { seconds, span: self.span_from(&start) }))
    }

    /// v22.6.0: parse optional `#[retry(max = N, backoff = "...")]` annotation.
    fn parse_retry_annotation(&mut self) -> Result<Option<crate::ast::RetryAnnotation>, ParseError> {
        let is_retry = self.peek() == &TokenKind::Hash
            && matches!(self.tokens.get(self.pos + 1), Some(t) if t.kind == TokenKind::LBracket)
            && matches!(self.tokens.get(self.pos + 2), Some(t) if matches!(&t.kind, TokenKind::Ident(n) if n == "retry"));
        if !is_retry {
            return Ok(None);
        }
        let start = self.peek_span().clone();
        self.advance();                          // #
        self.expect(&TokenKind::LBracket)?;      // [
        self.expect_ident_name("retry")?;
        self.expect(&TokenKind::LParen)?;
        self.expect_ident_name("max")?;
        self.expect(&TokenKind::Eq)?;
        let max = match self.peek().clone() {
            TokenKind::Int(n) => {
                let span = self.peek_span().clone();
                self.advance();
                u32::try_from(n).map_err(|_| ParseError::new(
                    format!("retry max value {} is out of range (must be 0..4294967295)", n),
                    span,
                ))?
            }
            other => return Err(ParseError::new(
                format!("expected integer after `max =`, got {:?}", other),
                self.peek_span().clone(),
            )),
        };
        self.expect(&TokenKind::Comma)?;
        self.expect_ident_name("backoff")?;
        self.expect(&TokenKind::Eq)?;
        let backoff = self.expect_str()?;
        self.expect(&TokenKind::RParen)?;
        self.expect(&TokenKind::RBracket)?;
        Ok(Some(crate::ast::RetryAnnotation { max, backoff, span: self.span_from(&start) }))
    }

    /// v22.6.0: parse optional `#[circuit_breaker(threshold = F, window = N)]` annotation.
    fn parse_circuit_breaker_annotation(&mut self) -> Result<Option<crate::ast::CircuitBreakerAnnotation>, ParseError> {
        let is_cb = self.peek() == &TokenKind::Hash
            && matches!(self.tokens.get(self.pos + 1), Some(t) if t.kind == TokenKind::LBracket)
            && matches!(self.tokens.get(self.pos + 2), Some(t) if matches!(&t.kind, TokenKind::Ident(n) if n == "circuit_breaker"));
        if !is_cb {
            return Ok(None);
        }
        let start = self.peek_span().clone();
        self.advance();                          // #
        self.expect(&TokenKind::LBracket)?;      // [
        self.expect_ident_name("circuit_breaker")?;
        self.expect(&TokenKind::LParen)?;
        self.expect_ident_name("threshold")?;
        self.expect(&TokenKind::Eq)?;
        let threshold = match self.peek().clone() {
            TokenKind::Int(n) => { self.advance(); n as f64 }
            TokenKind::Float(f) => { self.advance(); f }
            other => return Err(ParseError::new(
                format!("expected number after `threshold =`, got {:?}", other),
                self.peek_span().clone(),
            )),
        };
        self.expect(&TokenKind::Comma)?;
        self.expect_ident_name("window")?;
        self.expect(&TokenKind::Eq)?;
        let window = match self.peek().clone() {
            TokenKind::Int(n) => {
                let span = self.peek_span().clone();
                self.advance();
                u64::try_from(n).map_err(|_| ParseError::new(
                    format!("circuit_breaker window value {} is out of range (must be non-negative)", n),
                    span,
                ))?
            }
            other => return Err(ParseError::new(
                format!("expected integer after `window =`, got {:?}", other),
                self.peek_span().clone(),
            )),
        };
        self.expect(&TokenKind::RParen)?;
        self.expect(&TokenKind::RBracket)?;
        Ok(Some(crate::ast::CircuitBreakerAnnotation { threshold, window, span: self.span_from(&start) }))
    }

    /// v42.5.0: parse optional `#[max_inflight(n)]` annotation.
    /// n は位置引数（唯一の引数のため名前省略）、正の整数必須。
    fn parse_max_inflight_annotation(&mut self) -> Result<Option<crate::ast::MaxInflightAnnotation>, ParseError> {
        let is_max_inflight = self.peek() == &TokenKind::Hash
            && matches!(self.tokens.get(self.pos + 1), Some(t) if t.kind == TokenKind::LBracket)
            && matches!(self.tokens.get(self.pos + 2), Some(t) if matches!(&t.kind, TokenKind::Ident(n) if n == "max_inflight"));
        if !is_max_inflight {
            return Ok(None);
        }
        let start = self.peek_span().clone();
        self.advance();                              // #
        self.expect(&TokenKind::LBracket)?;          // [
        self.expect_ident_name("max_inflight")?;
        self.expect(&TokenKind::LParen)?;
        let n = match self.peek().clone() {
            TokenKind::Int(raw) => {
                // raw は i64。負数（-1 等）は Lexer が Minus + Int(1) に分割するため
                // ここに到達するのは 0 以上の整数のみ。`raw <= 0` は実質 `raw == 0` のガード。
                let span = self.peek_span().clone();
                self.advance();
                if raw <= 0 {
                    return Err(ParseError::new(
                        format!("max_inflight value {} must be a positive integer (>= 1)", raw),
                        span,
                    ));
                }
                raw as u64
            }
            other => return Err(ParseError::new(
                format!("expected positive integer after `max_inflight(`, got {:?}", other),
                self.peek_span().clone(),
            )),
        };
        self.expect(&TokenKind::RParen)?;
        self.expect(&TokenKind::RBracket)?;
        Ok(Some(crate::ast::MaxInflightAnnotation { n, span: self.span_from(&start) }))
    }

    fn parse_item(&mut self) -> Result<Item, ParseError> {
        // Annotation ordering: #[deprecated] must appear before #[api(...)], #[streaming], etc.
        // because each parser tries to consume #[ ... ] and they are mutually exclusive.
        // v24.4.0: parse optional #[deprecated] annotation before fn
        let deprecated_ann = self.parse_deprecated_annotation()?;
        // v46.1.0: parse optional #[test] annotation before fn
        let test_ann = self.parse_test_annotation()?;
        // v18.8.0: parse optional `#[api(...)]` annotation before visibility/fn
        let api_annotation = self.parse_api_annotation()?;
        // v19.1.0: parse optional streaming/stateful annotations
        let streaming_ann = self.parse_streaming_annotation()?;
        let stateful_ann = self.parse_stateful_annotation()?;
        // v19.5.0: parse optional #[arrow] annotation
        let arrow_ann = self.parse_arrow_annotation()?;
        // v22.1.0: parse optional #[checkpoint] annotation
        let checkpoint_ann = self.parse_checkpoint_annotation()?;
        // v22.4.0: parse optional #[trigger(...)] annotation
        let trigger_ann = self.parse_trigger_annotation()?;
        // v22.6.0: parse optional SLA annotations
        let timeout_ann         = self.parse_timeout_annotation()?;
        let retry_ann           = self.parse_retry_annotation()?;
        let circuit_breaker_ann = self.parse_circuit_breaker_annotation()?;
        let max_inflight_ann    = self.parse_max_inflight_annotation()?;  // v42.5.0

        let vis = self.parse_visibility();

        match self.peek().clone() {
            TokenKind::Import => {
                let is_public = match vis {
                    Some(Visibility::Public) => true,
                    Some(_) => {
                        return Err(ParseError::new(
                            "only `public import` is allowed; `internal import` and `private import` are not supported",
                            self.peek_span().clone(),
                        ));
                    }
                    None => false,
                };
                self.parse_import_decl(is_public)
            }
            // v43.11.0: `opaque type Token = String` — contextual keyword "opaque"
            TokenKind::Ident(name) if name == "opaque" => {
                self.advance(); // consume "opaque" identifier
                let mut td = self.parse_type_def(vis)?;
                td.is_opaque = true;
                Ok(Item::TypeDef(td))
            }
            TokenKind::Type => Ok(Item::TypeDef(self.parse_type_def(vis)?)),
            TokenKind::Fn => {
                let mut fd = self.parse_fn_def(vis, false)?;
                fd.api_annotation = api_annotation;
                fd.deprecated = deprecated_ann; // v24.4.0
                fd.is_test = test_ann;          // v46.1.0
                Ok(Item::FnDef(fd))
            }
            TokenKind::Stage => {
                let mut td = self.parse_trf_def(vis, false)?;
                td.stateful = stateful_ann;
                td.arrow = arrow_ann;
                td.checkpoint = checkpoint_ann;
                td.timeout         = timeout_ann;          // v22.6.0
                td.retry_ann       = retry_ann;           // v22.6.0
                td.circuit_breaker = circuit_breaker_ann;  // v22.6.0
                td.max_inflight    = max_inflight_ann;     // v42.5.0
                Ok(Item::TrfDef(td))
            }
            TokenKind::Trf => {
                let span = self.peek_span().clone();
                self.advance();
                Err(ParseError::new(
                    "keyword `trf` has been removed in v2.0.0; use `stage` instead (run `fav migrate` to auto-fix)",
                    span,
                ))
            }
            TokenKind::Async => {
                self.advance(); // consume 'async'
                match self.peek().clone() {
                    TokenKind::Fn => {
                        let mut fd = self.parse_fn_def(vis, true)?;
                        fd.api_annotation = api_annotation;
                        fd.deprecated = deprecated_ann; // v24.4.0
                        fd.is_test = test_ann;          // v46.1.0
                        Ok(Item::FnDef(fd))
                    }
                    TokenKind::Stage => {
                        let mut td = self.parse_trf_def(vis, true)?;
                        td.stateful = stateful_ann;
                        td.arrow = arrow_ann;
                        td.checkpoint = checkpoint_ann;
                        td.timeout         = timeout_ann;          // v22.6.0
                        td.retry_ann       = retry_ann;           // v22.6.0
                        td.circuit_breaker = circuit_breaker_ann;  // v22.6.0
                        td.max_inflight    = max_inflight_ann;     // v42.5.0
                        Ok(Item::TrfDef(td))
                    }
                    TokenKind::Trf => {
                        let span = self.peek_span().clone();
                        self.advance();
                        Err(ParseError::new(
                            "keyword `trf` has been removed in v2.0.0; use `stage` instead",
                            span,
                        ))
                    }
                    other => Err(ParseError::new(
                        format!("expected `fn` or `stage` after `async`, got {:?}", other),
                        self.peek_span().clone(),
                    )),
                }
            }
            TokenKind::Abstract => self.parse_abstract_item(vis),
            TokenKind::Interface => Ok(Item::InterfaceDecl(self.parse_interface_decl(vis)?)),
            TokenKind::Effect => Ok(Item::EffectDef(self.parse_effect_def(vis)?)),
            TokenKind::Cap => {
                let span = self.peek_span().clone();
                self.advance();
                Err(ParseError::new(
                    "keyword `cap` has been removed in v2.0.0; use `interface` instead",
                    span,
                ))
            }
            TokenKind::Impl => {
                if vis.is_some() {
                    return Err(ParseError::new(
                        "visibility modifier on `impl` is not allowed",
                        self.peek_span().clone(),
                    ));
                }
                let is_cap_style = matches!(
                    self.tokens.get(self.pos + 2).map(|t| &t.kind),
                    Some(TokenKind::LAngle)
                );
                if is_cap_style {
                    Ok(Item::ImplDef(self.parse_impl_def()?))
                } else {
                    Ok(Item::InterfaceImplDecl(self.parse_interface_impl_decl()?))
                }
            }
            TokenKind::Pipeline => Ok(Item::PipelineDef(self.parse_pipeline_def()?)), // v22.5.0
            TokenKind::Seq => {
                let item = self.parse_flw_def_or_binding(vis)?;
                Ok(match item {
                    Item::FlwDef(mut fd) => {
                        fd.streaming = streaming_ann;
                        fd.trigger = trigger_ann; // v22.4.0
                        Item::FlwDef(fd)
                    }
                    other => other,
                })
            }
            TokenKind::Flw => {
                let span = self.peek_span().clone();
                self.advance();
                Err(ParseError::new(
                    "keyword `flw` has been removed in v2.0.0; use `seq` instead (run `fav migrate` to auto-fix)",
                    span,
                ))
            }
            TokenKind::Test => {
                if vis.is_some() {
                    return Err(ParseError::new(
                        "visibility modifier on `test` is not allowed",
                        self.peek_span().clone(),
                    ));
                }
                Ok(Item::TestDef(self.parse_test_def()?))
            }
            TokenKind::TestGroup => {
                if vis.is_some() {
                    return Err(ParseError::new(
                        "visibility modifier on `test_group` is not allowed",
                        self.peek_span().clone(),
                    ));
                }
                Ok(self.parse_test_group()?)
            }
            TokenKind::Bench => {
                if vis.is_some() {
                    return Err(ParseError::new(
                        "visibility modifier on `bench` is not allowed",
                        self.peek_span().clone(),
                    ));
                }
                Ok(Item::BenchDef(self.parse_bench_def()?))
            }
            TokenKind::Namespace => Err(ParseError::new(
                "`namespace` must appear before any definitions",
                self.peek_span().clone(),
            )),
            TokenKind::Use => {
                // `use X as Y` — namespace alias (v16.6.0)
                // `use X.{ a, b }` or `use X.*` — rune-internal file import
                let span = self.peek_span().clone();
                self.advance(); // consume 'use'
                let (module, _) = self.expect_ident()?;
                // Detect `use X as Y` pattern
                if self.peek() == &TokenKind::As {
                    self.advance(); // consume 'as'
                    let (alias, _) = self.expect_ident()?;
                    return Ok(Item::UseAlias { original: module, alias, span });
                }
                self.expect(&TokenKind::Dot)?;
                let names = if self.peek() == &TokenKind::Star {
                    self.advance();
                    RuneUseNames::Wildcard
                } else {
                    self.expect(&TokenKind::LBrace)?;
                    let mut names = vec![];
                    while !matches!(self.peek(), TokenKind::RBrace | TokenKind::Eof) {
                        let (name, _) = self.expect_ident()?;
                        names.push(name);
                        if self.peek() == &TokenKind::Comma {
                            self.advance();
                        } else {
                            break;
                        }
                    }
                    self.expect(&TokenKind::RBrace)?;
                    RuneUseNames::Specific(names)
                };
                Ok(Item::RuneUse {
                    module,
                    names,
                    span,
                })
            }
            TokenKind::Alias => {
                if vis.is_some() {
                    return Err(ParseError::new(
                        "visibility modifier on `alias` is not allowed",
                        self.peek_span().clone(),
                    ));
                }
                self.parse_alias_decl()
            }
            // `schema Name { ... }` — inline schema definition (v36.1.0)
            // Distinguish from `schema "uri"` TypeExpr form (v18.4/v32.4): next token is Ident, not Str
            TokenKind::Ident(n) if n == "schema" => {
                if vis.is_some() {
                    return Err(ParseError::new(
                        "visibility modifier on `schema` is not allowed",
                        self.peek_span().clone(),
                    ));
                }
                if matches!(self.peek2(), Some(TokenKind::Ident(_))) {
                    Ok(Item::SchemaDef(self.parse_schema_def()?))
                } else {
                    Err(ParseError::new(
                        "expected schema name (identifier) after `schema` at top-level",
                        self.peek_span().clone(),
                    ))
                }
            }
            // `cep pattern Name { ... }` — CEP パターン宣言 (v42.1.0)
            TokenKind::Ident(n) if n == "cep" => {
                Ok(Item::CepPatternDef(self.parse_cep_pattern_def()?))
            }
            other => Err(ParseError::new(
                format!(
                    "expected item (type/fn/stage/seq/interface/effect/impl/test/alias/schema/cep), got {:?}",
                    other
                ),
                self.peek_span().clone(),
            )),
        }
    }

    /// Parse `schema Name { field: Type, ... }` as a top-level item (v36.1.0)
    fn parse_schema_def(&mut self) -> Result<SchemaDef, ParseError> {
        let start = self.peek_span().clone();
        self.advance(); // consume `schema`
        let (name, _) = self.expect_ident()?;
        self.expect(&TokenKind::LBrace)?;
        let mut fields = vec![];
        while self.peek() != &TokenKind::RBrace && self.peek() != &TokenKind::Eof {
            let (field_name, _) = self.expect_ident()?;
            self.expect(&TokenKind::Colon)?;
            let field_ty = self.parse_type_expr()?;
            fields.push((field_name, field_ty));
            if self.peek() == &TokenKind::Comma {
                self.advance();
            }
        }
        self.expect(&TokenKind::RBrace)?;
        Ok(SchemaDef {
            name,
            fields,
            span: self.span_from(&start),
        })
    }

    /// Parse a CEP expression: Event name, seq(...), any(...), or not(...) (v42.2.0)
    fn parse_cep_expr(&mut self) -> Result<CepExpr, ParseError> {
        // `seq` is a reserved keyword (TokenKind::Seq), not an ident
        // `any` / `not` are plain idents (not reserved)
        if self.peek() == &TokenKind::Seq {
            self.advance(); // consume `seq`
            self.expect(&TokenKind::LParen)?;
            let mut args = Vec::new();
            while self.peek() != &TokenKind::RParen && self.peek() != &TokenKind::Eof {
                args.push(self.parse_cep_expr()?);
                if self.peek() == &TokenKind::Comma {
                    self.advance();
                }
            }
            self.expect(&TokenKind::RParen)?;
            if args.is_empty() {
                return Err(ParseError::new(
                    "seq() requires at least one argument",
                    self.peek_span().clone(),
                ));
            }
            return Ok(CepExpr::Seq(args));
        }
        if self.peek_ident_text("any") {
            self.advance(); // consume `any`
            self.expect(&TokenKind::LParen)?;
            let mut args = Vec::new();
            while self.peek() != &TokenKind::RParen && self.peek() != &TokenKind::Eof {
                args.push(self.parse_cep_expr()?);
                if self.peek() == &TokenKind::Comma {
                    self.advance();
                }
            }
            self.expect(&TokenKind::RParen)?;
            if args.is_empty() {
                return Err(ParseError::new(
                    "any() requires at least one argument",
                    self.peek_span().clone(),
                ));
            }
            return Ok(CepExpr::Any(args));
        }
        if self.peek_ident_text("not") {
            self.advance(); // consume `not`
            self.expect(&TokenKind::LParen)?;
            let inner = self.parse_cep_expr()?;
            if self.peek() != &TokenKind::RParen {
                return Err(ParseError::new(
                    "not() takes exactly one argument",
                    self.peek_span().clone(),
                ));
            }
            self.expect(&TokenKind::RParen)?;
            return Ok(CepExpr::Not(Box::new(inner)));
        }
        // Simple event name
        let (name, _) = self.expect_ident()?;
        Ok(CepExpr::Event(name))
    }

    /// Parse `cep pattern Name { Event within N }` (v42.1.0); expr 拡張 v42.2.0
    fn parse_cep_pattern_def(&mut self) -> Result<CepPatternDef, ParseError> {
        let start = self.peek_span().clone();
        self.advance(); // consume `cep`
        self.expect_ident_name("pattern")?;
        let (name, _) = self.expect_ident()?;
        self.expect(&TokenKind::LBrace)?;
        let mut body = Vec::new();
        while self.peek() != &TokenKind::RBrace && self.peek() != &TokenKind::Eof {
            // clause_start は既存コードの定義位置をそのまま使用
            let clause_start = self.peek_span().clone();
            let expr = self.parse_cep_expr()?;
            let within_secs = if self.peek_ident_text("within") {
                self.advance(); // consume `within`
                match self.peek().clone() {
                    TokenKind::Int(n) => {
                        self.advance();
                        Some(n)
                    }
                    _ => {
                        return Err(ParseError::new(
                            "expected integer after `within`",
                            self.peek_span().clone(),
                        ))
                    }
                }
            } else {
                None
            };
            body.push(CepClause {
                expr,
                within_secs,
                span: self.span_from(&clause_start),
            });
        }
        self.expect(&TokenKind::RBrace)?;
        Ok(CepPatternDef { name, body, span: self.span_from(&start) })
    }

    /// Parse `expect <expr> { <rule_expr>* }` (v36.2.0)
    fn parse_expect_stmt(&mut self) -> Result<ExpectStmt, ParseError> {
        let start = self.peek_span().clone();
        self.advance(); // consume `expect`
        let target = self.parse_expr()?;
        self.expect(&TokenKind::LBrace)?;
        let mut rules = vec![];
        while self.peek() != &TokenKind::RBrace && self.peek() != &TokenKind::Eof {
            let pos_before = self.pos;
            let rule = self.parse_expr()?;
            rules.push(rule);
            if self.peek() == &TokenKind::Semicolon {
                self.advance();
            }
            // Guard: if no tokens were consumed, stop to prevent infinite loop
            if self.pos == pos_before {
                break;
            }
        }
        self.expect(&TokenKind::RBrace)?;
        Ok(ExpectStmt {
            target: Box::new(target),
            rules,
            span: self.span_from(&start),
        })
    }

    fn parse_alias_decl(&mut self) -> Result<Item, ParseError> {
        let span = self.peek_span().clone();
        self.advance(); // consume `alias`
        let (name, _) = self.expect_ident()?;
        let params = self.parse_type_params()?;
        let params: Vec<String> = params.into_iter().map(|p| p.name).collect();
        self.expect(&TokenKind::Eq)?;
        let ty = self.parse_type_expr()?;
        Ok(Item::AliasDecl { name, params, ty, span })
    }

    fn parse_import_decl(&mut self, is_public: bool) -> Result<Item, ParseError> {
        let start = self.peek_span().clone();
        self.expect(&TokenKind::Import)?;
        // `rune` keyword is accepted but no longer required — kept for backward compatibility
        let explicit_rune = if self.peek_ident_text("rune") {
            self.advance();
            true
        } else {
            false
        };
        // v48.1.0: kind を決定する (Legacy がデフォルト、bare ident のみ Package)
        let mut kind = crate::ast::ImportKind::Legacy;
        let path = match self.peek().clone() {
            TokenKind::Str(path) => {
                self.advance();
                // v48.2.0: ./ or ../ prefix → Local import
                if path.starts_with("./") || path.starts_with("../") {
                    kind = crate::ast::ImportKind::Local;
                }
                path
            }
            // Support `import runes/X` path syntax (v28.x+): parse as rune import (Legacy)
            TokenKind::Ident(first_seg) => {
                self.advance();
                if self.peek() == &TokenKind::Slash {
                    self.advance(); // consume `/`
                    let (second_seg, _) = self.expect_ident()?;
                    // `import runes/sentry` → rune name is `sentry` (Legacy)
                    if first_seg == "runes" {
                        second_seg
                    } else {
                        format!("{}/{}", first_seg, second_seg)
                    }
                } else {
                    // v48.1.0: bare ident = Package import (`import kafka`)
                    kind = crate::ast::ImportKind::Package;
                    first_seg
                }
            }
            other => {
                return Err(ParseError::new(
                    format!("expected string literal import path, got {:?}", other),
                    self.peek_span().clone(),
                ));
            }
        };
        // A bare name (no `/` or `.`) is a rune import: `import "db"` == `import rune "db"`
        // `import runes/X` is always a rune import (explicit_rune=false but path has no `.`)
        let is_rune = explicit_rune || (!path.contains('/') && !path.contains('.'));
        let alias = if self.peek() == &TokenKind::As || self.peek_ident_text("as") {
            self.advance();
            let (alias, _) = self.expect_ident()?;
            Some(alias)
        } else {
            None
        };
        Ok(Item::ImportDecl {
            path,
            alias,
            is_rune,
            is_public,
            kind,
            span: self.span_from(&start),
        })
    }

    fn parse_effect_def(
        &mut self,
        visibility: Option<Visibility>,
    ) -> Result<EffectDef, ParseError> {
        let start = self.peek_span().clone();
        self.expect(&TokenKind::Effect)?;
        let (name, _) = self.expect_ident()?;
        Ok(EffectDef {
            visibility,
            name,
            span: self.span_from(&start),
        })
    }

    fn parse_abstract_item(&mut self, visibility: Option<Visibility>) -> Result<Item, ParseError> {
        let start = self.peek_span().clone();
        self.expect(&TokenKind::Abstract)?;
        match self.peek() {
            TokenKind::Stage => Ok(Item::AbstractTrfDef(
                self.parse_abstract_trf_def(visibility)?,
            )),
            TokenKind::Seq => Ok(Item::AbstractFlwDef(
                self.parse_abstract_flw_def(visibility)?,
            )),
            TokenKind::Trf => {
                let span = self.peek_span().clone();
                self.advance();
                Err(ParseError::new(
                    "keyword `abstract trf` has been removed in v2.0.0; use `abstract stage` instead",
                    span,
                ))
            }
            TokenKind::Flw => {
                let span = self.peek_span().clone();
                self.advance();
                Err(ParseError::new(
                    "keyword `abstract flw` has been removed in v2.0.0; use `abstract seq` instead",
                    span,
                ))
            }
            _ => Err(ParseError::new(
                "expected `stage` or `seq` after `abstract`",
                start,
            )),
        }
    }

    // ── cap_def (v0.4.0) ─────────────────────────────────────────────────────

    #[allow(dead_code)]
    fn parse_cap_def(&mut self, visibility: Option<Visibility>) -> Result<CapDef, ParseError> {
        let start = self.peek_span().clone();
        self.expect(&TokenKind::Cap)?;
        let (name, _) = self.expect_ident()?;
        // cap requires type params (at least one)
        if self.peek() != &TokenKind::LAngle {
            return Err(ParseError::new(
                "cap definition requires type parameters: `cap Name<T> = { ... }`",
                self.peek_span().clone(),
            ));
        }
        let type_params = self.parse_type_params()?;
        self.expect(&TokenKind::Eq)?;
        self.expect(&TokenKind::LBrace)?;
        let mut fields = Vec::new();
        while self.peek() != &TokenKind::RBrace && !self.at_end() {
            let fs = self.peek_span().clone();
            let (fname, _) = self.expect_ident()?;
            self.expect(&TokenKind::Colon)?;
            let fty = self.parse_type_expr()?;
            fields.push(CapField {
                name: fname,
                ty: fty,
                span: self.span_from(&fs),
            });
        }
        self.expect(&TokenKind::RBrace)?;
        Ok(CapDef {
            visibility,
            name,
            type_params,
            fields,
            span: self.span_from(&start),
        })
    }

    // ── impl_def (v0.4.0) ────────────────────────────────────────────────────

    fn parse_impl_def(&mut self) -> Result<ImplDef, ParseError> {
        let start = self.peek_span().clone();
        self.expect(&TokenKind::Impl)?;
        let (cap_name, _) = self.expect_ident()?;
        self.expect(&TokenKind::LAngle)?;
        let first_arg = self.parse_type_expr()?;
        let mut type_args = vec![first_arg];
        while self.peek() == &TokenKind::Comma {
            self.advance();
            type_args.push(self.parse_type_expr()?);
        }
        self.expect(&TokenKind::RAngle)?;
        self.expect(&TokenKind::LBrace)?;
        let mut methods = Vec::new();
        while self.peek() != &TokenKind::RBrace && !self.at_end() {
            // each method: optional visibility then `fn`
            let vis = self.parse_visibility();
            if self.peek() != &TokenKind::Fn {
                return Err(ParseError::new(
                    "expected `fn` in impl block",
                    self.peek_span().clone(),
                ));
            }
            methods.push(self.parse_fn_def(vis, false)?);
        }
        self.expect(&TokenKind::RBrace)?;
        Ok(ImplDef {
            cap_name,
            type_args,
            methods,
            span: self.span_from(&start),
        })
    }

    /// Parse generic type params with optional variance annotations: `<+T, -U, V>`.
    /// `+T` → Covariant, `-T` → Contravariant, `T` → Invariant.
    ///
    /// Lexer note: `<-T` is tokenized as `LArrow` (`<-`) followed by `Ident("T")`.
    /// `<+T` is tokenized as `LAngle` + `Plus` + `Ident("T")`.
    /// So we handle both `LAngle` (normal open) and `LArrow` (contravariant first param).
    fn parse_variance_type_params(&mut self) -> Result<Vec<crate::ast::GenericParam>, ParseError> {
        use crate::ast::{GenericParam, Variance};
        // Determine whether the angle bracket opened with `<` or `<-`.
        let first_is_contravariant;
        if self.peek() == &TokenKind::LAngle {
            self.advance(); // consume `<`
            first_is_contravariant = false;
        } else if self.peek() == &TokenKind::LArrow {
            self.advance(); // consume `<-` — first param is contravariant
            first_is_contravariant = true;
        } else {
            return Ok(vec![]);
        }
        let mut params = vec![];
        let mut is_first = true;
        loop {
            let variance = if is_first && first_is_contravariant {
                // `<-` already consumed variance for the first param
                Variance::Contravariant
            } else if self.peek() == &TokenKind::Plus {
                self.advance(); // consume `+`
                Variance::Covariant
            } else if self.peek() == &TokenKind::Minus {
                self.advance(); // consume `-`
                Variance::Contravariant
            } else {
                Variance::Invariant
            };
            is_first = false;
            let (name, _) = self.expect_ident()?;
            let bounds = self.parse_type_bounds()?;
            params.push(GenericParam { name, bounds, variance, is_const: false, const_ty: None, const_constraint: None });
            if self.peek() == &TokenKind::Comma {
                self.advance();
            } else {
                break;
            }
        }
        self.expect(&TokenKind::RAngle)?;
        Ok(params)
    }

    fn parse_interface_decl(
        &mut self,
        visibility: Option<Visibility>,
    ) -> Result<InterfaceDecl, ParseError> {
        let start = self.peek_span().clone();
        self.expect(&TokenKind::Interface)?;
        let (name, _) = self.expect_ident()?;

        // v18.6.0: optional type params with variance annotations `<+T, -U>`
        let type_params = self.parse_variance_type_params()?;

        let super_interface = if self.peek() == &TokenKind::Colon {
            self.advance();
            let (sup, _) = self.expect_ident()?;
            Some(sup)
        } else {
            None
        };

        self.expect(&TokenKind::LBrace)?;
        let mut methods = Vec::new();
        while self.peek() != &TokenKind::RBrace && !self.at_end() {
            let ms = self.peek_span().clone();
            let (method_name, _) = self.expect_ident()?;
            self.expect(&TokenKind::Colon)?;
            let ty = self.parse_type_expr()?;
            methods.push(InterfaceMethod {
                name: method_name,
                ty,
                span: self.span_from(&ms),
            });
        }
        self.expect(&TokenKind::RBrace)?;

        Ok(InterfaceDecl {
            visibility,
            name,
            super_interface,
            type_params,
            methods,
            span: self.span_from(&start),
        })
    }

    fn parse_interface_impl_decl(&mut self) -> Result<InterfaceImplDecl, ParseError> {
        let start = self.peek_span().clone();
        self.expect(&TokenKind::Impl)?;
        let (first_iface, _) = self.expect_ident()?;
        let mut interface_names = vec![first_iface];
        while self.peek() == &TokenKind::Comma {
            self.advance();
            let (iface, _) = self.expect_ident()?;
            interface_names.push(iface);
        }
        self.expect(&TokenKind::For)?;
        let (type_name, _) = self.expect_ident()?;

        if self.peek() != &TokenKind::LBrace {
            return Ok(InterfaceImplDecl {
                interface_names,
                type_name,
                type_params: vec![],
                methods: vec![],
                is_auto: true,
                span: self.span_from(&start),
            });
        }

        self.expect(&TokenKind::LBrace)?;
        let mut methods = Vec::new();
        while self.peek() != &TokenKind::RBrace && !self.at_end() {
            let (method_name, _) = self.expect_ident()?;
            self.expect(&TokenKind::Eq)?;
            let body = self.parse_expr()?;
            methods.push((method_name, body));
        }
        self.expect(&TokenKind::RBrace)?;

        Ok(InterfaceImplDecl {
            interface_names,
            type_name,
            type_params: vec![],
            methods,
            is_auto: false,
            span: self.span_from(&start),
        })
    }

    // ── test_def (v0.8.0) ────────────────────────────────────────────────────

    fn parse_test_def(&mut self) -> Result<TestDef, ParseError> {
        let start = self.peek_span().clone();
        self.expect(&TokenKind::Test)?;
        let name = match self.peek().clone() {
            TokenKind::Str(s) => {
                self.advance();
                s
            }
            _ => {
                return Err(ParseError::new(
                    "expected string literal after `test`",
                    self.peek_span().clone(),
                ));
            }
        };
        let body = self.parse_block()?;
        Ok(TestDef {
            name,
            body,
            span: self.span_from(&start),
        })
    }

    // ── test_group (v16.7.0) ─────────────────────────────────────────────────

    fn parse_test_group(&mut self) -> Result<Item, ParseError> {
        let start = self.peek_span().clone();
        self.expect(&TokenKind::TestGroup)?;
        let name = match self.peek().clone() {
            TokenKind::Str(s) => {
                self.advance();
                s
            }
            _ => {
                return Err(ParseError::new(
                    "expected string literal after `test_group`",
                    self.peek_span().clone(),
                ));
            }
        };
        self.expect(&TokenKind::LBrace)?;
        let mut tests = Vec::new();
        while self.peek() != &TokenKind::RBrace && self.peek() != &TokenKind::Eof {
            tests.push(self.parse_test_def()?);
        }
        self.expect(&TokenKind::RBrace)?;
        Ok(Item::TestGroup {
            name,
            tests,
            span: self.span_from(&start),
        })
    }

    fn parse_bench_def(&mut self) -> Result<BenchDef, ParseError> {
        let start = self.peek_span().clone();
        self.expect(&TokenKind::Bench)?;
        let description = match self.peek().clone() {
            TokenKind::Str(s) => {
                self.advance();
                s
            }
            _ => {
                return Err(ParseError::new(
                    "expected string literal after `bench`",
                    self.peek_span().clone(),
                ));
            }
        };
        let body = self.parse_block()?;
        Ok(BenchDef {
            description,
            body,
            span: self.span_from(&start),
        })
    }

    fn parse_visibility(&mut self) -> Option<Visibility> {
        match self.peek() {
            TokenKind::Public => {
                self.advance();
                Some(Visibility::Public)
            }
            TokenKind::Internal => {
                self.advance();
                Some(Visibility::Internal)
            }
            TokenKind::Private => {
                self.advance();
                Some(Visibility::Private)
            }
            _ => None,
        }
    }

    // ── type_def (3-3, 3-4) ──────────────────────────────────────────────────

    fn parse_type_def(&mut self, visibility: Option<Visibility>) -> Result<TypeDef, ParseError> {
        let start = self.peek_span().clone();
        self.expect(&TokenKind::Type)?;
        let (name, _) = self.expect_ident()?;

        // Wrapper type: `type UserId(Int)` (no `=` sign)
        if self.peek() == &TokenKind::LParen {
            self.advance(); // consume `(`
            let inner_ty = self.parse_type_expr()?;
            self.expect(&TokenKind::RParen)?;

            // Optional `with Eq, Show` clause
            let with_interfaces = if self.peek() == &TokenKind::With {
                self.advance();
                let (first, _) = self.expect_ident()?;
                let mut names = vec![first];
                while self.peek() == &TokenKind::Comma {
                    self.advance();
                    let (iface, _) = self.expect_ident()?;
                    names.push(iface);
                }
                names
            } else {
                vec![]
            };

            // Optional `where |v| pred` clause
            let invariants = if self.peek() == &TokenKind::Where {
                self.advance();
                vec![self.parse_expr()?]
            } else {
                vec![]
            };

            return Ok(TypeDef {
                visibility,
                name,
                type_params: vec![],
                with_interfaces,
                invariants,
                is_opaque: false,
                body: TypeBody::Wrapper(inner_ty),
                span: self.span_from(&start),
            });
        }

        let type_params = self.parse_type_params()?;
        let with_interfaces = if self.peek() == &TokenKind::With {
            self.advance();
            let (first, _) = self.expect_ident()?;
            let mut names = vec![first];
            while self.peek() == &TokenKind::Comma {
                self.advance();
                let (iface, _) = self.expect_ident()?;
                names.push(iface);
            }
            names
        } else {
            vec![]
        };
        self.expect(&TokenKind::Eq)?;

        let body = if self.peek() == &TokenKind::LBrace {
            // record body
            let (fields, invariants) = self.parse_record_body()?;
            return Ok(TypeDef {
                visibility,
                name,
                type_params,
                with_interfaces,
                invariants,
                is_opaque: false,
                body: TypeBody::Record(fields),
                span: self.span_from(&start),
            });
        } else if self.peek() == &TokenKind::Pipe {
            // sum body
            TypeBody::Sum(self.parse_sum_variants()?)
        } else {
            // type alias: type Name = TypeExpr
            let target = self.parse_type_expr()?;
            // v41.1.0: refinement constraint `where |v| pred` for type aliases
            let invariants = if self.peek() == &TokenKind::Where {
                self.advance(); // consume `where`
                vec![self.parse_expr()?]
            } else {
                vec![]
            };
            return Ok(TypeDef {
                visibility,
                name,
                type_params,
                with_interfaces,
                invariants,
                is_opaque: false,
                body: TypeBody::Alias(target),
                span: self.span_from(&start),
            });
        };

        Ok(TypeDef {
            visibility,
            name,
            type_params,
            with_interfaces,
            invariants: vec![],
            is_opaque: false,
            body,
            span: self.span_from(&start),
        })
    }

    /// Parse optional type parameters `<T, U, V>` with optional bounds.
    /// `<T>` → `[GenericParam { name: "T", bounds: [] }]`
    /// `<T with Ord>` → `[GenericParam { name: "T", bounds: ["Ord"] }]`
    /// `<T with Ord with Serialize, U>` → two params
    /// Returns an empty Vec if no `<` is found.
    fn parse_type_params(&mut self) -> Result<Vec<crate::ast::GenericParam>, ParseError> {
        if self.peek() != &TokenKind::LAngle {
            return Ok(vec![]);
        }
        self.advance(); // consume `<`
        let mut params = vec![self.parse_one_type_param()?];
        while self.peek() == &TokenKind::Comma {
            self.advance();
            params.push(self.parse_one_type_param()?);
        }
        self.expect(&TokenKind::RAngle)?;
        Ok(params)
    }

    fn parse_one_type_param(&mut self) -> Result<crate::ast::GenericParam, ParseError> {
        // `const N: Int` const generic param
        if self.peek_ident_text("const") {
            self.advance(); // consume `const`
            let (name, _) = self.expect_ident()?;
            self.expect(&TokenKind::Colon)?;
            let const_ty = self.parse_type_expr()?;
            // optional `where { expr }` compile-time constraint
            let const_constraint = if self.peek() == &TokenKind::Where {
                self.advance(); // consume `where`
                self.expect(&TokenKind::LBrace)?;
                let expr = self.parse_expr()?;
                self.expect(&TokenKind::RBrace)?;
                Some(Box::new(expr))
            } else {
                None
            };
            return Ok(crate::ast::GenericParam {
                name,
                bounds: vec![],
                variance: crate::ast::Variance::Invariant,
                is_const: true,
                const_ty: Some(const_ty),
                const_constraint,
            });
        }
        let (name, _) = self.expect_ident()?;
        let bounds = self.parse_type_bounds()?;
        Ok(crate::ast::GenericParam { name, bounds, variance: crate::ast::Variance::Invariant, is_const: false, const_ty: None, const_constraint: None })
    }

    /// Parse zero or more `with InterfaceName` or `with { field: Type, ... }` bounds.
    fn parse_type_bounds(&mut self) -> Result<Vec<crate::ast::TypeConstraint>, ParseError> {
        use crate::ast::TypeConstraint;
        let mut bounds = vec![];
        while self.peek() == &TokenKind::With || self.peek_ident_text("with") {
            self.advance(); // consume `with`
            if self.peek() == &TokenKind::LBrace {
                // Record field constraint: `with { field: Type, ... }`
                self.advance(); // consume `{`
                loop {
                    let (name, _) = self.expect_ident()?;
                    self.expect(&TokenKind::Colon)?;
                    let ty = self.parse_type_expr()?;
                    bounds.push(TypeConstraint::HasField { name, ty });
                    if self.peek() == &TokenKind::RBrace {
                        break;
                    }
                    if self.peek() == &TokenKind::Comma {
                        self.advance();
                    }
                }
                self.expect(&TokenKind::RBrace)?;
            } else {
                // Interface bound: `with Ord`
                let (bound_name, _) = self.expect_ident()?;
                bounds.push(TypeConstraint::Interface(bound_name));
            }
        }
        Ok(bounds)
    }

    fn parse_record_body(&mut self) -> Result<(Vec<Field>, Vec<Expr>), ParseError> {
        self.expect(&TokenKind::LBrace)?;
        let mut fields = Vec::new();
        let mut invariants = Vec::new();
        while self.peek() != &TokenKind::RBrace {
            if self.peek() == &TokenKind::Invariant {
                self.advance();
                invariants.push(self.parse_expr()?);
            } else {
                fields.push(self.parse_field()?);
            }
        }
        self.expect(&TokenKind::RBrace)?;
        Ok((fields, invariants))
    }

    fn parse_record_fields(&mut self) -> Result<Vec<Field>, ParseError> {
        let (fields, invariants) = self.parse_record_body()?;
        if !invariants.is_empty() {
            return Err(ParseError::new(
                "`invariant` is only allowed in type definitions",
                Span::dummy(),
            ));
        }
        Ok(fields)
    }

    fn parse_field(&mut self) -> Result<Field, ParseError> {
        let start = self.peek_span().clone();
        let attrs = self.parse_field_attrs()?;
        let (name, _) = self.expect_ident()?;
        self.expect(&TokenKind::Colon)?;
        let ty = self.parse_type_expr()?;
        Ok(Field {
            name,
            ty,
            attrs,
            span: self.span_from(&start),
        })
    }

    fn parse_field_attrs(&mut self) -> Result<Vec<FieldAttr>, ParseError> {
        let mut attrs = Vec::new();
        while self.peek() == &TokenKind::Hash {
            let start = self.peek_span().clone();
            self.expect(&TokenKind::Hash)?;
            self.expect(&TokenKind::LBracket)?;
            let (name, _) = self.expect_ident()?;
            let arg = if self.peek() == &TokenKind::LParen {
                self.advance();
                let arg = match self.peek().clone() {
                    TokenKind::Int(n) => {
                        self.advance();
                        n.to_string()
                    }
                    TokenKind::Str(s) => {
                        self.advance();
                        s
                    }
                    TokenKind::Ident(s) => {
                        self.advance();
                        s
                    }
                    other => {
                        return Err(ParseError::new(
                            format!("expected attribute argument, got {:?}", other),
                            self.peek_span().clone(),
                        ));
                    }
                };
                self.expect(&TokenKind::RParen)?;
                Some(arg)
            } else {
                None
            };
            self.expect(&TokenKind::RBracket)?;
            attrs.push(FieldAttr {
                name,
                arg,
                span: self.span_from(&start),
            });
        }
        Ok(attrs)
    }

    fn parse_type_arg_list(&mut self) -> Result<Vec<TypeExpr>, ParseError> {
        self.expect(&TokenKind::LAngle)?;
        let mut args = vec![self.parse_type_expr()?];
        while self.peek() == &TokenKind::Comma {
            self.advance();
            args.push(self.parse_type_expr()?);
        }
        self.expect(&TokenKind::RAngle)?;
        Ok(args)
    }

    fn parse_sum_variants(&mut self) -> Result<Vec<Variant>, ParseError> {
        let mut variants = Vec::new();
        while self.peek() == &TokenKind::Pipe {
            self.advance(); // consume `|`
            variants.push(self.parse_variant()?);
        }
        Ok(variants)
    }

    fn parse_variant(&mut self) -> Result<Variant, ParseError> {
        let start = self.peek_span().clone();
        let (name, _) = self.expect_ident()?;

        if self.peek() == &TokenKind::LParen {
            // Tuple variant: ok(User) or Add(Expr, Expr)
            self.advance();
            let mut tys = vec![self.parse_type_expr()?];
            while self.peek() == &TokenKind::Comma {
                self.advance();
                tys.push(self.parse_type_expr()?);
            }
            self.expect(&TokenKind::RParen)?;
            Ok(Variant::Tuple(name, tys, self.span_from(&start)))
        } else if self.peek() == &TokenKind::LBrace {
            // Record variant: Authenticated { user: User }
            let fields = self.parse_record_fields()?;
            Ok(Variant::Record(name, fields, self.span_from(&start)))
        } else {
            // Unit variant: Guest
            Ok(Variant::Unit(name, self.span_from(&start)))
        }
    }

    // ── type_expr (3-21) ─────────────────────────────────────────────────────

    /// Parse a type expression.
    /// `allow_arrow`: if false, stop before consuming `->` (used in trf signatures
    /// where `->` is the trf-level separator, not a function-type arrow).
    fn parse_type_expr_inner(&mut self, allow_arrow: bool) -> Result<TypeExpr, ParseError> {
        let start = self.peek_span().clone();
        let mut ty = self.parse_base_type()?;

        // postfix ? and !
        // For `!`, only consume it as "fallible" when the next token is NOT
        // an effect keyword (Pure/Io). If it is, leave `!` for the effect-ann parser.
        loop {
            match self.peek() {
                TokenKind::Question => {
                    self.advance();
                    let span = self.span_from(&start);
                    ty = TypeExpr::Optional(Box::new(ty), span);
                }
                TokenKind::Bang => {
                    let bang_line = self.peek_span().line;
                    let next_is_effect = match self.peek2() {
                        Some(TokenKind::Pure) | Some(TokenKind::Io) => self
                            .tokens
                            .get(self.pos + 1)
                            .map(|t| t.span.line == bang_line)
                            .unwrap_or(false),
                        Some(TokenKind::Ident(_)) => self
                            .tokens
                            .get(self.pos + 1)
                            .map(|t| t.span.line == bang_line)
                            .unwrap_or(false),
                        _ => false,
                    };
                    if next_is_effect {
                        break;
                    }
                    self.advance();
                    let span = self.span_from(&start);
                    ty = TypeExpr::Fallible(Box::new(ty), span);
                }
                _ => break,
            }
        }

        // intersection type `T & U` (v18.2.0)
        if self.peek() == &TokenKind::Amp {
            self.advance(); // consume `&`
            let rhs = self.parse_base_type()?;
            let span = self.span_from(&start);
            ty = TypeExpr::Intersection(Box::new(ty), Box::new(rhs), span);
        }

        // arrow ->  (only when allowed)
        if allow_arrow && self.peek() == &TokenKind::Arrow {
            self.advance();
            let rhs = self.parse_type_expr_inner(true)?;
            let span = self.span_from(&start);
            ty = TypeExpr::Arrow(Box::new(ty), Box::new(rhs), span);
        }

        // linear arrow -o  (v18.5.0)
        if allow_arrow && self.peek() == &TokenKind::LinearArrow {
            self.advance();
            let rhs = self.parse_type_expr_inner(true)?;
            let span = self.span_from(&start);
            ty = TypeExpr::LinearArrow(Box::new(ty), Box::new(rhs), span);
        }

        Ok(ty)
    }

    /// Full type expression (allows `->` for function types).
    fn parse_type_expr(&mut self) -> Result<TypeExpr, ParseError> {
        self.parse_type_expr_inner(true)
    }

    /// Type expression without consuming `->` at the top level.
    /// Used in trf signatures: `trf F: A -> B` where `->` is the trf separator.
    fn parse_type_expr_no_arrow(&mut self) -> Result<TypeExpr, ParseError> {
        self.parse_type_expr_inner(false)
    }

    fn parse_base_type(&mut self) -> Result<TypeExpr, ParseError> {
        let start = self.peek_span().clone();
        // Schema type: `schema "uri"` (v18.4.0)
        if matches!(self.peek(), TokenKind::Ident(n) if n == "schema") {
            self.advance(); // consume `schema`
            if let TokenKind::Str(uri) = self.peek().clone() {
                self.advance();
                let span = self.span_from(&start);
                return Ok(TypeExpr::Schema(uri, span));
            }
            return Err(ParseError::new(
                "expected string literal after `schema`".to_string(),
                self.peek_span().clone(),
            ));
        }
        // Inline record type: `{ field: Type, ... }` (v18.2.0)
        if self.peek() == &TokenKind::LBrace {
            self.advance(); // consume `{`
            let mut fields = vec![];
            while self.peek() != &TokenKind::RBrace {
                let (field_name, _) = self.expect_ident()?;
                self.expect(&TokenKind::Colon)?;
                let field_ty = self.parse_type_expr()?;
                fields.push((field_name, field_ty));
                if self.peek() == &TokenKind::Comma {
                    self.advance();
                }
            }
            self.expect(&TokenKind::RBrace)?;
            let span = self.span_from(&start);
            return Ok(TypeExpr::RecordType(fields, span));
        }
        // Integer literal in type position (const generic argument): `Array<100>`
        if let TokenKind::Int(n) = self.peek().clone() {
            self.advance();
            let span = self.span_from(&start);
            return Ok(TypeExpr::ConstInt(n, span));
        }

        let name = match self.peek().clone() {
            TokenKind::Ident(n) => {
                self.advance();
                n
            }
            // Allow effect keywords and soft keywords as type names
            TokenKind::Pure => {
                self.advance();
                "Pure".to_string()
            }
            TokenKind::Io => {
                self.advance();
                "Io".to_string()
            }
            // `pipeline` soft keyword — allow as type name for backward compat (v22.5.0)
            TokenKind::Pipeline => {
                self.advance();
                "pipeline".to_string()
            }
            other => {
                return Err(ParseError::new(
                    format!("expected type name, got {:?}", other),
                    self.peek_span().clone(),
                ));
            }
        };

        // optional type args: List<Row>
        let args = if self.peek() == &TokenKind::LAngle {
            self.advance();
            let mut args = vec![self.parse_type_expr()?];
            while self.peek() == &TokenKind::Comma {
                self.advance();
                args.push(self.parse_type_expr()?);
            }
            self.expect(&TokenKind::RAngle)?;
            args
        } else {
            vec![]
        };

        Ok(TypeExpr::Named(name, args, self.span_from(&start)))
    }

    // ── fn_def (3-5) ─────────────────────────────────────────────────────────

    fn parse_fn_def(
        &mut self,
        visibility: Option<Visibility>,
        is_async: bool,
    ) -> Result<FnDef, ParseError> {
        let start = self.peek_span().clone();
        self.expect(&TokenKind::Fn)?;
        let (name, _) = self.expect_ident()?;
        let type_params = self.parse_type_params()?;

        self.expect(&TokenKind::LParen)?;
        let params = self.parse_params()?;
        self.expect(&TokenKind::RParen)?;

        let return_ty = if self.peek() == &TokenKind::Arrow {
            self.advance();
            Some(self.parse_type_expr()?)
        } else {
            None
        };

        // v35.4.0: !Effect annotation syntax removed (E0374); parse_effect_ann errors on `!`.
        self.parse_effect_ann()?;

        let body = if self.peek() == &TokenKind::Eq {
            self.advance();
            let expr = self.parse_expr()?;
            Block {
                stmts: vec![],
                expr: Box::new(expr),
                span: self.span_from(&start),
            }
        } else {
            self.parse_block()?
        };

        Ok(FnDef {
            visibility,
            is_async,
            name,
            type_params,
            params,
            return_ty,
            body,
            span: self.span_from(&start),
            api_annotation: None, // set by parse_item if #[api(...)] precedes the fn
            deprecated: false,    // set by parse_item if #[deprecated] precedes the fn
            is_test: false,       // set by parse_item if #[test] precedes the fn (v46.1.0)
        })
    }

    fn parse_params(&mut self) -> Result<Vec<Param>, ParseError> {
        let mut params = Vec::new();
        while self.peek() != &TokenKind::RParen {
            let start = self.peek_span().clone();
            // v13.10.0: `Ctx { db: DbRead, io }` sugar syntax → desugar to `ctx: LoadCtx` etc.
            if matches!(self.peek(), TokenKind::Ident(n) if n == "Ctx")
                && matches!(self.tokens.get(self.pos + 1).map(|t| &t.kind), Some(TokenKind::LBrace))
            {
                self.advance(); // consume "Ctx"
                self.advance(); // consume "{"
                let mut fields: Vec<(String, Option<String>)> = Vec::new();
                while !matches!(self.peek(), TokenKind::RBrace | TokenKind::Eof) {
                    let (field_name, _) = self.expect_ident()?;
                    let field_ty = if self.peek() == &TokenKind::Colon {
                        self.advance();
                        let (ty_name, _) = self.expect_ident()?;
                        Some(ty_name)
                    } else {
                        None
                    };
                    fields.push((field_name, field_ty));
                    if self.peek() == &TokenKind::Comma {
                        self.advance();
                    }
                }
                self.expect(&TokenKind::RBrace)?;
                let ctx_type = desugar_ctx_fields(&fields);
                let span = self.span_from(&start);
                params.push(Param {
                    name: "ctx".to_string(),
                    ty: crate::ast::TypeExpr::Named(ctx_type.to_string(), vec![], span.clone()),
                    constraint: None,
                    span,
                });
                if self.peek() == &TokenKind::Comma {
                    self.advance();
                }
                continue;
            }
            let (name, _) = self.expect_ident()?;
            self.expect(&TokenKind::Colon)?;
            let ty = self.parse_fn_param_type()?;
            // v18.3.0: refinement constraint `where { expr }`
            let constraint = if self.peek() == &TokenKind::Where {
                self.advance(); // consume `where`
                self.expect(&TokenKind::LBrace)?;
                let expr = self.parse_expr()?;
                self.expect(&TokenKind::RBrace)?;
                Some(Box::new(expr))
            } else {
                None
            };
            params.push(Param {
                name,
                ty,
                constraint,
                span: self.span_from(&start),
            });
            if self.peek() == &TokenKind::Comma {
                self.advance();
            }
        }
        Ok(params)
    }

    fn parse_fn_param_type(&mut self) -> Result<TypeExpr, ParseError> {
        let start = self.peek_span().clone();
        let left = self.parse_type_expr_inner(false)?;
        if self.peek() == &TokenKind::Arrow {
            self.advance();
            let output = self.parse_type_expr_inner(false)?;
            self.parse_effect_ann()?;
            return Ok(TypeExpr::TrfFn {
                input: Box::new(left),
                output: Box::new(output),
                span: self.span_from(&start),
            });
        }
        Ok(left)
    }

    // effect annotation: ("!" effect_term)+   — v35.4.0: !Effect syntax removed (E0374)
    fn parse_effect_ann(&mut self) -> Result<(), ParseError> {
        if self.peek() == &TokenKind::Bang {
            return Err(ParseError::new(
                "[E0374] `!Effect` annotation syntax was removed in v35.4.0 \
                 — declare side effects by passing `ctx: AppCtx` as the first parameter \
                 (e.g. `fn f(ctx: AppCtx, ...) -> T { ... }`)",
                self.peek_span().clone(),
            ));
        }
        Ok(())
    }

    // ── trf_def (3-6) ────────────────────────────────────────────────────────

    fn parse_trf_def(
        &mut self,
        visibility: Option<Visibility>,
        is_async: bool,
    ) -> Result<TrfDef, ParseError> {
        let start = self.peek_span().clone();
        self.expect(&TokenKind::Stage)?;
        let (name, _) = self.expect_ident()?;
        let type_params = self.parse_type_params()?;
        self.expect(&TokenKind::Colon)?;
        let input_ty = self.parse_type_expr_no_arrow()?;
        self.expect(&TokenKind::Arrow)?;
        let output_ty = self.parse_type_expr_no_arrow()?;

        // v35.4.0: !Effect annotation syntax removed (E0374); parse_effect_ann errors on `!`.
        self.parse_effect_ann()?;

        self.expect(&TokenKind::Eq)?;
        // closure params: |param, ...| or ||
        self.expect(&TokenKind::Pipe)?;
        let params = self.parse_closure_params_typed()?;
        self.expect(&TokenKind::Pipe)?;

        let body = self.parse_block()?;

        Ok(TrfDef {
            visibility,
            is_async,
            name,
            type_params,
            input_ty,
            output_ty,
            params,
            body,
            stateful: false,
            arrow: false,
            checkpoint: false,
            timeout: None,
            retry_ann: None,
            circuit_breaker: None,
            max_inflight: None,  // v42.5.0
            span: self.span_from(&start),
        })
    }

    fn parse_abstract_trf_def(
        &mut self,
        visibility: Option<Visibility>,
    ) -> Result<AbstractTrfDef, ParseError> {
        let start = self.peek_span().clone();
        self.expect(&TokenKind::Stage)?;
        let (name, _) = self.expect_ident()?;
        let type_params = self.parse_type_params()?;
        self.expect(&TokenKind::Colon)?;
        let input_ty = self.parse_type_expr_no_arrow()?;
        self.expect(&TokenKind::Arrow)?;
        let output_ty = self.parse_type_expr_no_arrow()?;
        self.parse_effect_ann()?;
        Ok(AbstractTrfDef {
            visibility,
            name,
            type_params,
            input_ty,
            output_ty,
            span: self.span_from(&start),
        })
    }

    /// Parse typed closure params: `param: Type, ...` (used inside trf)
    fn parse_closure_params_typed(&mut self) -> Result<Vec<Param>, ParseError> {
        let mut params = Vec::new();
        while self.peek() != &TokenKind::Pipe {
            let start = self.peek_span().clone();
            // Allow `_` wildcard as an anonymous parameter
            let name = if self.peek() == &TokenKind::Underscore {
                self.advance();
                "_".to_string()
            } else {
                let (n, _) = self.expect_ident()?;
                n
            };
            // Type annotation is optional for closure params
            let ty = if self.peek() == &TokenKind::Colon {
                self.advance();
                self.parse_type_expr()?
            } else {
                TypeExpr::Named("_infer".to_string(), vec![], start.clone())
            };
            params.push(Param {
                name,
                ty,
                constraint: None,
                span: self.span_from(&start),
            });
            if self.peek() == &TokenKind::Comma {
                self.advance();
            }
        }
        Ok(params)
    }

    // ── pipeline_def (v22.5.0) ───────────────────────────────────────────────

    /// v22.5.0: parse `step "<name>" = seq <SeqName> [after "<dep>", ...]`
    fn parse_pipeline_step(&mut self) -> Result<crate::ast::PipelineStep, ParseError> {
        let start = self.peek_span().clone();
        // "step" — soft keyword
        self.expect_ident_name("step")?;
        let name = self.expect_str()?;
        self.expect(&TokenKind::Eq)?;
        self.expect(&TokenKind::Seq)?;
        let (seq_name, _) = self.expect_ident()?;
        // optional: after "<dep1>", "<dep2>"
        let mut after = Vec::new();
        if self.peek_ident_text("after") {
            self.advance(); // consume "after"
            let dep = self.expect_str()?;
            after.push(dep);
            while self.peek() == &TokenKind::Comma {
                self.advance(); // ,
                if !matches!(self.peek(), TokenKind::Str(_)) {
                    // trailing comma or unexpected token — stop consuming deps
                    break;
                }
                after.push(self.expect_str()?);
            }
        }
        Ok(crate::ast::PipelineStep { name, seq_name, after, span: self.span_from(&start) })
    }

    /// v22.5.0: parse `pipeline <Name> { step ... }`
    fn parse_pipeline_def(&mut self) -> Result<crate::ast::PipelineDef, ParseError> {
        let start = self.peek_span().clone();
        self.expect(&TokenKind::Pipeline)?;
        let (name, _) = self.expect_ident()?;
        self.expect(&TokenKind::LBrace)?;
        let mut steps = Vec::new();
        while self.peek() != &TokenKind::RBrace && !self.at_end() {
            steps.push(self.parse_pipeline_step()?);
        }
        self.expect(&TokenKind::RBrace)?;
        Ok(crate::ast::PipelineDef { name, steps, span: self.span_from(&start) })
    }

    // ── flw_def (3-7) ────────────────────────────────────────────────────────

    #[allow(dead_code)]
    fn parse_flw_def(&mut self) -> Result<FlwDef, ParseError> {
        let start = self.peek_span().clone();
        self.expect(&TokenKind::Seq)?;
        let (name, _) = self.expect_ident()?;
        let ctx_param = if self.peek() == &TokenKind::LParen {
            self.advance();
            let (ident, _) = self.expect_ident()?;
            self.expect(&TokenKind::RParen)?;
            Some(ident)
        } else {
            None
        };
        self.expect(&TokenKind::Eq)?;

        let first = self.parse_flw_step()?;
        let mut steps = vec![first];
        while self.peek() == &TokenKind::PipeGt {
            self.advance();
            steps.push(self.parse_flw_step()?);
        }

        Ok(FlwDef {
            name,
            steps,
            ctx_param,
            streaming: None,
            trigger: None,
            span: self.span_from(&start),
        })
    }

    fn parse_flw_step(&mut self) -> Result<FlwStep, ParseError> {
        if self.peek() == &TokenKind::Par {
            self.advance(); // consume `par`
            self.expect(&TokenKind::LBracket)?;
            let (first, _) = self.expect_ident()?;
            let mut names = vec![first];
            while self.peek() == &TokenKind::Comma {
                self.advance();
                let (name, _) = self.expect_ident()?;
                names.push(name);
            }
            self.expect(&TokenKind::RBracket)?;
            Ok(FlwStep::Par(names))
        } else if self.peek_ident_text("par_distributed") {
            // par_distributed [...] — soft keyword (v22.2.0)
            self.advance(); // consume "par_distributed"
            self.expect(&TokenKind::LBracket)?;
            let (first, _) = self.expect_ident()?;
            let mut names = vec![first];
            while self.peek() == &TokenKind::Comma {
                self.advance();
                let (name, _) = self.expect_ident()?;
                names.push(name);
            }
            self.expect(&TokenKind::RBracket)?;
            Ok(FlwStep::ParDistributed(names))
        } else if self.peek_ident_text("tap") {
            // tap(observer_expr) — soft keyword (v16.8.0)
            self.advance(); // consume "tap"
            self.expect(&TokenKind::LParen)?;
            let observer = self.parse_expr()?;
            self.expect(&TokenKind::RParen)?;
            Ok(FlwStep::Tap(Box::new(observer)))
        } else if self.peek_ident_text("inspect") {
            // inspect — soft keyword (v16.8.0)
            self.advance(); // consume "inspect"
            Ok(FlwStep::Inspect)
        } else if self.peek_ident_text("Merge") {
            // Merge.ordered / Merge.any — soft keywords (v51.2.0)
            // Fallback to FlwStep::Stage("Merge") for backward compat (existing tests).
            // Peek at pos+1 (Dot) and pos+2 (suffix ident) before consuming anything.
            let has_dot = self.peek2() == Some(&TokenKind::Dot);
            // Clone the suffix ident so we don't hold a borrow over the advance() calls.
            let suffix: Option<String> = if has_dot {
                self.tokens.get(self.pos + 2).and_then(|t| match &t.kind {
                    TokenKind::Ident(s) => Some(s.clone()),
                    _ => None,
                })
            } else {
                None
            };
            self.advance(); // consume "Merge"
            match suffix.as_deref() {
                Some("ordered") => {
                    self.advance(); // consume "."
                    self.advance(); // consume "ordered"
                    Ok(FlwStep::Merge(crate::ast::MergeMode::Ordered))
                }
                Some("any") => {
                    self.advance(); // consume "."
                    self.advance(); // consume "any"
                    Ok(FlwStep::Merge(crate::ast::MergeMode::Any))
                }
                _ => Ok(FlwStep::Stage("Merge".to_string())),
            }
        } else {
            let (name, _) = self.expect_ident()?;
            Ok(FlwStep::Stage(name))
        }
    }

    fn parse_abstract_flw_def(
        &mut self,
        visibility: Option<Visibility>,
    ) -> Result<AbstractFlwDef, ParseError> {
        let start = self.peek_span().clone();
        self.expect(&TokenKind::Seq)?;
        let (name, _) = self.expect_ident()?;
        let type_params = self.parse_type_params()?;
        self.expect(&TokenKind::LBrace)?;
        let mut slots = Vec::new();
        while self.peek() != &TokenKind::RBrace && !self.at_end() {
            slots.push(self.parse_flw_slot()?);
            if self.peek() == &TokenKind::Semicolon {
                self.advance();
            }
        }
        self.expect(&TokenKind::RBrace)?;
        Ok(AbstractFlwDef {
            visibility,
            name,
            type_params,
            slots,
            span: self.span_from(&start),
        })
    }

    fn parse_flw_slot(&mut self) -> Result<FlwSlot, ParseError> {
        let start = self.peek_span().clone();
        let (name, _) = self.expect_ident()?;
        self.expect(&TokenKind::Colon)?;
        let first_ty = self.parse_type_expr_no_arrow()?;
        let (abstract_trf_ty, input_ty, output_ty) =
            if matches!(self.peek(), TokenKind::Arrow) {
                self.expect(&TokenKind::Arrow)?;
                // Slot outputs may be fallible (`T!`) and are followed by either
                // an effect annotation (`!Db`) or the next slot on a new line.
                // Use full type parsing here so a trailing `!` stays part of the
                // output type instead of being misread as the start of `!Effect`.
                let output_ty = self.parse_type_expr()?;
                self.parse_effect_ann()?;
                (None, first_ty, output_ty)
            } else {
                let infer_span = self.span_from(&start);
                let infer_ty = TypeExpr::Named("_infer".into(), vec![], infer_span);
                (Some(first_ty), infer_ty.clone(), infer_ty)
            };
        Ok(FlwSlot {
            name,
            abstract_trf_ty,
            input_ty,
            output_ty,
            span: self.span_from(&start),
        })
    }

    fn parse_flw_def_or_binding(
        &mut self,
        visibility: Option<Visibility>,
    ) -> Result<Item, ParseError> {
        let start = self.peek_span().clone();
        self.expect(&TokenKind::Seq)?;
        let (name, _) = self.expect_ident()?;
        let ctx_param = if self.peek() == &TokenKind::LParen {
            self.advance();
            let (ident, _) = self.expect_ident()?;
            self.expect(&TokenKind::RParen)?;
            Some(ident)
        } else {
            None
        };
        self.expect(&TokenKind::Eq)?;

        // Check if first step is `par [...]` / `par_distributed [...]` or an ident used as template for binding.
        if self.peek() == &TokenKind::Par || self.peek_ident_text("par_distributed") {
            // par [...] — must be a FlwDef, not a binding
            let first_step = self.parse_flw_step()?;
            let mut steps = vec![first_step];
            while self.peek() == &TokenKind::PipeGt {
                self.advance();
                steps.push(self.parse_flw_step()?);
            }
            return Ok(Item::FlwDef(FlwDef {
                name,
                steps,
                ctx_param,
                streaming: None,
                trigger: None,
                span: self.span_from(&start),
            }));
        }

        let (first, _) = self.expect_ident()?;

        match self.peek() {
            TokenKind::LBrace | TokenKind::LAngle => {
                self.parse_flw_binding_rest(visibility, start, name, first)
            }
            _ => {
                let mut steps = vec![FlwStep::Stage(first)];
                while self.peek() == &TokenKind::PipeGt {
                    self.advance();
                    steps.push(self.parse_flw_step()?);
                }
                Ok(Item::FlwDef(FlwDef {
                    name,
                    steps,
                    ctx_param,
                    streaming: None,
                    trigger: None,
                    span: self.span_from(&start),
                }))
            }
        }
    }

    fn parse_flw_binding_rest(
        &mut self,
        visibility: Option<Visibility>,
        start: Span,
        name: String,
        template: String,
    ) -> Result<Item, ParseError> {
        let type_args = if self.peek() == &TokenKind::LAngle {
            self.advance();
            let mut args = vec![self.parse_type_expr()?];
            while self.peek() == &TokenKind::Comma {
                self.advance();
                args.push(self.parse_type_expr()?);
            }
            self.expect(&TokenKind::RAngle)?;
            args
        } else {
            vec![]
        };

        self.expect(&TokenKind::LBrace)?;
        let mut bindings = Vec::new();
        while self.peek() != &TokenKind::RBrace && !self.at_end() {
            let (slot, _) = self.expect_ident()?;
            self.expect(&TokenKind::LArrow)?;
            let (imp, _) = self.expect_ident()?;
            bindings.push((slot, SlotImpl::Global(imp)));
            if self.peek() == &TokenKind::Semicolon {
                self.advance();
            }
        }
        self.expect(&TokenKind::RBrace)?;

        Ok(Item::FlwBindingDef(FlwBindingDef {
            visibility,
            name,
            template,
            type_args,
            bindings,
            span: self.span_from(&start),
        }))
    }

    // ── block (3-17) ─────────────────────────────────────────────────────────

    /// `{ stmt* expr }`
    /// Intermediate statements end with `;` or are `bind`.
    /// The last expression has no `;`.
    fn parse_block(&mut self) -> Result<Block, ParseError> {
        let start = self.peek_span().clone();
        self.expect(&TokenKind::LBrace)?;

        let mut stmts = Vec::new();

        loop {
            if self.peek() == &TokenKind::RBrace {
                // empty block — treat as Unit
                self.advance();
                let span = self.span_from(&start);
                return Ok(Block {
                    stmts,
                    expr: Box::new(Expr::Lit(Lit::Unit, span.clone())),
                    span,
                });
            }

            // bind statement (trailing ; is optional)
            if self.peek() == &TokenKind::Bind {
                let bind = self.parse_bind_stmt()?;
                stmts.push(Stmt::Bind(bind));
                if self.peek() == &TokenKind::Semicolon {
                    self.advance();
                }
                continue;
            }

            // chain statement (v0.5.0, trailing ; is optional like bind)
            if self.peek() == &TokenKind::Chain {
                let chain = self.parse_chain_stmt()?;
                stmts.push(Stmt::Chain(chain));
                if self.peek() == &TokenKind::Semicolon {
                    self.advance();
                }
                continue;
            }

            // yield statement (v0.5.0, requires ;)
            if self.peek() == &TokenKind::Yield {
                let y = self.parse_yield_stmt()?;
                stmts.push(Stmt::Yield(y));
                continue;
            }

            // return statement (v45.1.0, trailing ; is optional)
            if self.peek() == &TokenKind::Return {
                let r = self.parse_return_stmt()?;
                stmts.push(Stmt::Return(r));
                if self.peek() == &TokenKind::Semicolon {
                    self.advance();
                }
                continue;
            }

            // for-in statement (v1.9.0, trailing ; is optional)
            if self.peek() == &TokenKind::For {
                let f = self.parse_for_in_stmt()?;
                stmts.push(Stmt::ForIn(f));
                if self.peek() == &TokenKind::Semicolon {
                    self.advance();
                }
                continue;
            }

            // forall property-based test statement (v17.7.0)
            if self.peek() == &TokenKind::Forall {
                let f = self.parse_forall_stmt()?;
                stmts.push(Stmt::Forall(f));
                if self.peek() == &TokenKind::Semicolon {
                    self.advance();
                }
                continue;
            }

            // expect <expr> { <rules> }  (v36.2.0)
            if matches!(self.peek(), TokenKind::Ident(n) if n == "expect") {
                let e = self.parse_expect_stmt()?;
                stmts.push(Stmt::Expect(e));
                if self.peek() == &TokenKind::Semicolon {
                    self.advance();
                }
                continue;
            }

            // expression — could be a stmt (ends with `;`) or the final expr
            let expr = self.parse_expr()?;

            if self.peek() == &TokenKind::Semicolon {
                self.advance();
                stmts.push(Stmt::Expr(expr));
            } else {
                // This is the final expression of the block
                self.expect(&TokenKind::RBrace)?;
                let span = self.span_from(&start);
                return Ok(Block {
                    stmts,
                    expr: Box::new(expr),
                    span,
                });
            }
        }
    }

    // ── bind_stmt (3-8..3-11) ────────────────────────────────────────────────

    fn parse_bind_stmt(&mut self) -> Result<BindStmt, ParseError> {
        let start = self.peek_span().clone();
        self.expect(&TokenKind::Bind)?;
        let pattern = self.parse_pattern()?;
        let annotated_ty = if self.peek() == &TokenKind::Colon {
            match &pattern {
                Pattern::Bind(_, _) => {
                    self.advance();
                    Some(self.parse_type_expr_no_arrow()?)
                }
                _ => {
                    return Err(ParseError::new(
                        "typed bind is only supported for plain name patterns",
                        self.peek_span().clone(),
                    ));
                }
            }
        } else {
            None
        };
        self.expect(&TokenKind::LArrow)?;
        let expr = self.parse_expr()?;
        Ok(BindStmt {
            pattern,
            annotated_ty,
            expr,
            span: self.span_from(&start),
        })
    }

    fn parse_chain_stmt(&mut self) -> Result<ChainStmt, ParseError> {
        let start = self.peek_span().clone();
        self.expect(&TokenKind::Chain)?;
        let (name, _) = self.expect_ident()?;
        self.expect(&TokenKind::LArrow)?;
        let expr = self.parse_expr()?;
        Ok(ChainStmt {
            name,
            expr,
            span: self.span_from(&start),
        })
    }

    fn parse_yield_stmt(&mut self) -> Result<YieldStmt, ParseError> {
        let start = self.peek_span().clone();
        self.expect(&TokenKind::Yield)?;
        let expr = self.parse_expr()?;
        self.expect(&TokenKind::Semicolon)?;
        Ok(YieldStmt {
            expr,
            span: self.span_from(&start),
        })
    }

    // -- parse_return_stmt (v45.1.0) --

    fn parse_return_stmt(&mut self) -> Result<ReturnStmt, ParseError> {
        let start = self.peek_span().clone();
        self.expect(&TokenKind::Return)?;
        let expr = self.parse_expr()?;
        Ok(ReturnStmt {
            expr,
            span: self.span_from(&start),
        })
    }

    // ── for-in stmt (v1.9.0) ──────────────────────────────────────────────────

    fn parse_for_in_stmt(&mut self) -> Result<ForInStmt, ParseError> {
        let start = self.peek_span().clone();
        self.expect(&TokenKind::For)?;
        let (var, _) = self.expect_ident()?;
        self.expect(&TokenKind::In)?;
        let iter = self.parse_expr()?;
        let body = self.parse_block()?;
        Ok(ForInStmt {
            var,
            iter,
            body,
            span: self.span_from(&start),
        })
    }

    // ── forall stmt (v17.7.0) ─────────────────────────────────────────────────

    fn parse_forall_stmt(&mut self) -> Result<ForallStmt, ParseError> {
        let start = self.peek_span().clone();
        self.expect(&TokenKind::Forall)?;
        // parse single variable: name : Type
        let var_start = self.peek_span().clone();
        let (name, _) = self.expect_ident()?;
        self.expect(&TokenKind::Colon)?;
        let ty = self.parse_type_expr()?;
        let var_span = self.span_from(&var_start);
        let vars = vec![ForallVar { name, ty, span: var_span }];
        // optional where { guard_expr }
        let guard = if self.peek() == &TokenKind::Where {
            self.advance();
            self.expect(&TokenKind::LBrace)?;
            let g = self.parse_expr()?;
            self.expect(&TokenKind::RBrace)?;
            Some(g)
        } else {
            None
        };
        // body block
        let body = self.parse_block()?;
        Ok(ForallStmt {
            vars,
            guard,
            body,
            span: self.span_from(&start),
        })
    }

    // ── pattern (3-9..3-11) ──────────────────────────────────────────────────

    fn parse_pattern(&mut self) -> Result<Pattern, ParseError> {
        let start = self.peek_span().clone();
        match self.peek().clone() {
            // wildcard: _
            TokenKind::Underscore => {
                self.advance();
                Ok(Pattern::Wildcard(start))
            }

            // record decomposition: { name, email } or { name: pat }
            TokenKind::LBrace => {
                let fields = self.parse_record_field_patterns()?;
                Ok(Pattern::Record(fields, self.span_from(&start)))
            }

            // literal patterns
            TokenKind::Int(n) => {
                let n = n;
                self.advance();
                Ok(Pattern::Lit(Lit::Int(n), start))
            }
            TokenKind::Float(f) => {
                let f = f;
                self.advance();
                Ok(Pattern::Lit(Lit::Float(f), start))
            }
            TokenKind::Str(s) => {
                let s = s;
                self.advance();
                Ok(Pattern::Lit(Lit::Str(s), start))
            }
            TokenKind::Bool(b) => {
                let b = b;
                self.advance();
                Ok(Pattern::Lit(Lit::Bool(b), start))
            }

            // unit () or tuple pattern (p1, p2, ...) or grouping (pat) (v41.3.0)
            TokenKind::LParen => {
                self.advance();
                if self.peek() == &TokenKind::RParen {
                    self.advance();
                    Ok(Pattern::Lit(Lit::Unit, self.span_from(&start)))
                } else {
                    let first = self.parse_pattern()?;
                    if self.peek() == &TokenKind::Comma {
                        // v41.3.0: tuple pattern (p1, p2, ...) → Record([Alias("_0", p1), ...])
                        let mut fields = vec![PatternField::Alias(
                            "_0".to_string(), Box::new(first), self.span_from(&start),
                        )];
                        let mut i = 1usize;
                        while self.peek() == &TokenKind::Comma {
                            self.advance();
                            if self.peek() == &TokenKind::RParen { break; }
                            fields.push(PatternField::Alias(
                                format!("_{}", i),
                                Box::new(self.parse_pattern()?),
                                self.span_from(&start),
                            ));
                            i += 1;
                        }
                        self.expect(&TokenKind::RParen)?;
                        Ok(Pattern::Record(fields, self.span_from(&start)))
                    } else {
                        self.expect(&TokenKind::RParen)?;
                        Ok(first) // grouping parens: (pat) → pat
                    }
                }
            }

            // identifier: could be Bind or Variant(...)
            TokenKind::Ident(name) => {
                let name = name;
                self.advance();
                if self.peek() == &TokenKind::LParen {
                    // tuple variant with payload: ok(pat) or Add(a, b)
                    self.advance();
                    let first = self.parse_pattern()?;
                    if self.peek() == &TokenKind::Comma {
                        // multi-arg: synthesize Record pattern with positional fields _0, _1, ...
                        let mut pats = vec![first];
                        while self.peek() == &TokenKind::Comma {
                            self.advance();
                            pats.push(self.parse_pattern()?);
                        }
                        self.expect(&TokenKind::RParen)?;
                        let fields = pats
                            .into_iter()
                            .enumerate()
                            .map(|(i, p)| {
                                PatternField::Alias(
                                    format!("_{}", i),
                                    Box::new(p),
                                    self.span_from(&start),
                                )
                            })
                            .collect();
                        let inner = Pattern::Record(fields, self.span_from(&start));
                        Ok(Pattern::Variant(
                            name,
                            Some(Box::new(inner)),
                            self.span_from(&start),
                        ))
                    } else {
                        self.expect(&TokenKind::RParen)?;
                        Ok(Pattern::Variant(
                            name,
                            Some(Box::new(first)),
                            self.span_from(&start),
                        ))
                    }
                } else if self.peek() == &TokenKind::LBrace {
                    // record variant: Authenticated { user } or Authenticated { user: pat }
                    // Represented as Variant(name, Some(Record(fields)))
                    let fields = self.parse_record_field_patterns()?;
                    let inner = Pattern::Record(fields, self.span_from(&start));
                    Ok(Pattern::Variant(
                        name,
                        Some(Box::new(inner)),
                        self.span_from(&start),
                    ))
                } else if name
                    .chars()
                    .next()
                    .map(|c| c.is_uppercase())
                    .unwrap_or(false)
                {
                    // uppercase with no payload → unit variant (e.g., Guest)
                    Ok(Pattern::Variant(name, None, self.span_from(&start)))
                } else {
                    // lowercase → bind
                    Ok(Pattern::Bind(name, self.span_from(&start)))
                }
            }

            // list-pattern: [] / [x] / [head, ..tail] / [a, b, ..rest] (v17.2.0)
            TokenKind::LBracket => {
                self.advance(); // consume '['
                let mut head: Vec<Pattern> = Vec::new();
                let mut tail: Option<String> = None;
                while self.peek() != &TokenKind::RBracket {
                    if self.peek() == &TokenKind::DotDot {
                        self.advance(); // consume '..'
                        if let TokenKind::Ident(name) = self.peek().clone() {
                            self.advance();
                            tail = Some(name);
                        }
                        break; // rest binding must be last
                    }
                    head.push(self.parse_pattern()?);
                    if self.peek() == &TokenKind::Comma {
                        self.advance();
                    } else {
                        break;
                    }
                }
                self.expect(&TokenKind::RBracket)?;
                Ok(Pattern::List {
                    head,
                    tail,
                    span: self.span_from(&start),
                })
            }

            other => Err(ParseError::new(
                format!("expected pattern, got {:?}", other),
                self.peek_span().clone(),
            )),
        }
    }

    // ── expr (3-12..3-16) ────────────────────────────────────────────────────

    fn parse_expr(&mut self) -> Result<Expr, ParseError> {
        let start = self.peek_span().clone();
        let mut lhs = self.parse_logical_or()?;

        // pipeline: expr |> expr |> ...
        // `|> match { ... }` desugars to `match lhs { ... }` (v0.5.0)
        if self.peek() == &TokenKind::PipeGt {
            let mut parts = vec![lhs];
            while self.peek() == &TokenKind::PipeGt {
                self.advance();
                // `|> match { arms }` desugars immediately
                if self.peek() == &TokenKind::Match {
                    let match_start = self.peek_span().clone();
                    self.advance(); // consume `match`
                    self.expect(&TokenKind::LBrace)?;
                    let mut arms = Vec::new();
                    while self.peek() != &TokenKind::RBrace {
                        arms.push(self.parse_match_arm()?);
                    }
                    self.expect(&TokenKind::RBrace)?;
                    // Fold the accumulated pipeline into the scrutinee
                    let scrutinee = if parts.len() == 1 {
                        parts.remove(0)
                    } else {
                        Expr::Pipeline(parts, self.span_from(&start))
                    };
                    lhs = Expr::Match(Box::new(scrutinee), arms, self.span_from(&match_start));
                    // After a `|> match`, no more pipe stages are allowed
                    return Ok(lhs);
                }
                parts.push(self.parse_logical_or()?);
            }
            lhs = Expr::Pipeline(parts, self.span_from(&start));
        }

        // ?? (null-coalesce) — lowest precedence binary operator (v1.9.0)
        while self.peek() == &TokenKind::QuestionQuestion {
            self.advance();
            let rhs = self.parse_logical_or()?;
            lhs = Expr::BinOp(
                BinOp::NullCoalesce,
                Box::new(lhs),
                Box::new(rhs),
                self.span_from(&start),
            );
        }

        Ok(lhs)
    }

    fn parse_logical_or(&mut self) -> Result<Expr, ParseError> {
        let start = self.peek_span().clone();
        let mut lhs = self.parse_logical_and()?;

        while self.peek() == &TokenKind::PipePipe {
            self.advance();
            let rhs = self.parse_logical_and()?;
            lhs = Expr::BinOp(
                BinOp::Or,
                Box::new(lhs),
                Box::new(rhs),
                self.span_from(&start),
            );
        }

        Ok(lhs)
    }

    fn parse_logical_and(&mut self) -> Result<Expr, ParseError> {
        let start = self.peek_span().clone();
        let mut lhs = self.parse_comparison()?;

        while self.peek() == &TokenKind::AmpAmp {
            self.advance();
            let rhs = self.parse_comparison()?;
            lhs = Expr::BinOp(
                BinOp::And,
                Box::new(lhs),
                Box::new(rhs),
                self.span_from(&start),
            );
        }

        Ok(lhs)
    }

    fn parse_comparison(&mut self) -> Result<Expr, ParseError> {
        let start = self.peek_span().clone();
        let mut lhs = self.parse_additive()?;

        loop {
            let op = match self.peek() {
                TokenKind::EqEq => BinOp::Eq,
                TokenKind::BangEq => BinOp::NotEq,
                TokenKind::LAngle => BinOp::Lt,
                TokenKind::RAngle => BinOp::Gt,
                TokenKind::LtEq => BinOp::LtEq,
                TokenKind::GtEq => BinOp::GtEq,
                _ => break,
            };
            self.advance();
            let rhs = self.parse_additive()?;
            lhs = Expr::BinOp(op, Box::new(lhs), Box::new(rhs), self.span_from(&start));
        }
        Ok(lhs)
    }

    fn parse_additive(&mut self) -> Result<Expr, ParseError> {
        let start = self.peek_span().clone();
        let mut lhs = self.parse_multiplicative()?;

        loop {
            let op = match self.peek() {
                TokenKind::Plus => BinOp::Add,
                TokenKind::Minus => BinOp::Sub,
                _ => break,
            };
            self.advance();
            let rhs = self.parse_multiplicative()?;
            lhs = Expr::BinOp(op, Box::new(lhs), Box::new(rhs), self.span_from(&start));
        }
        Ok(lhs)
    }

    fn parse_multiplicative(&mut self) -> Result<Expr, ParseError> {
        let start = self.peek_span().clone();
        let mut lhs = self.parse_unary()?;

        loop {
            let op = match self.peek() {
                TokenKind::Star => BinOp::Mul,
                TokenKind::Slash => BinOp::Div,
                _ => break,
            };
            self.advance();
            let rhs = self.parse_unary()?;
            lhs = Expr::BinOp(op, Box::new(lhs), Box::new(rhs), self.span_from(&start));
        }
        Ok(lhs)
    }

    fn parse_unary(&mut self) -> Result<Expr, ParseError> {
        let start = self.peek_span().clone();
        if self.peek() == &TokenKind::Minus {
            self.advance();
            let operand = self.parse_call_chain()?;
            // sugar: -x → 0 - x
            Ok(Expr::BinOp(
                BinOp::Sub,
                Box::new(Expr::Lit(Lit::Int(0), start.clone())),
                Box::new(operand),
                self.span_from(&start),
            ))
        } else {
            self.parse_call_chain()
        }
    }

    /// Parse function-call and field-access chains: `f(a).field(b)`
    fn parse_call_chain(&mut self) -> Result<Expr, ParseError> {
        let start = self.peek_span().clone();
        let mut expr = self.parse_primary()?;

        loop {
            match self.peek() {
                // field access: expr.name
                TokenKind::Dot => {
                    self.advance();
                    let (field, _) = self.expect_ident()?;
                    let span = self.span_from(&start);
                    expr = Expr::FieldAccess(Box::new(expr), field, span);
                }
                TokenKind::LAngle => {
                    let saved_pos = self.pos;
                    if let Ok(type_args) = self.parse_type_arg_list() {
                        if self.peek() == &TokenKind::LParen {
                            let span = self.span_from(&start);
                            expr = Expr::TypeApply(Box::new(expr), type_args, span);
                        } else {
                            self.pos = saved_pos;
                            break;
                        }
                    } else {
                        self.pos = saved_pos;
                        break;
                    }
                }
                // function call: expr(args)
                TokenKind::LParen => {
                    self.advance();
                    let mut args = Vec::new();
                    while self.peek() != &TokenKind::RParen {
                        args.push(self.parse_expr()?);
                        if self.peek() == &TokenKind::Comma {
                            self.advance();
                        }
                    }
                    self.expect(&TokenKind::RParen)?;
                    let span = self.span_from(&start);
                    expr = Expr::Apply(Box::new(expr), args, span);
                }
                // postfix ?: expr? — error propagation
                TokenKind::Question => {
                    self.advance();
                    let span = self.span_from(&start);
                    expr = Expr::Question(Box::new(expr), span);
                }
                _ => break,
            }
        }

        Ok(expr)
    }

    // primary expression (3-12..3-16)
    fn parse_primary(&mut self) -> Result<Expr, ParseError> {
        let start = self.peek_span().clone();

        match self.peek().clone() {
            // integer literal (3-12)
            TokenKind::Int(n) => {
                self.advance();
                Ok(Expr::Lit(Lit::Int(n), start))
            }

            // float literal (3-12)
            TokenKind::Float(f) => {
                self.advance();
                Ok(Expr::Lit(Lit::Float(f), start))
            }

            // string literal (3-12)
            TokenKind::Str(s) => {
                self.advance();
                Ok(Expr::Lit(Lit::Str(s), start))
            }

            TokenKind::FStringRaw(raw) => {
                self.advance();
                self.parse_fstring_parts(&raw, start)
            }

            // bool literal (3-12)
            TokenKind::Bool(b) => {
                self.advance();
                Ok(Expr::Lit(Lit::Bool(b), start))
            }

            // unit literal () or tuple (a, b, ...) or grouping (expr) (3-12, v41.3.0)
            TokenKind::LParen => {
                self.advance();
                if self.peek() == &TokenKind::RParen {
                    self.advance();
                    Ok(Expr::Lit(Lit::Unit, self.span_from(&start)))
                } else {
                    let first = self.parse_expr()?;
                    if self.peek() == &TokenKind::Comma {
                        // v41.3.0: tuple (a, b, ...) → RecordConstruct("__tuple__", [("_0", a), ...])
                        let mut fields = vec![("_0".to_string(), first)];
                        let mut i = 1usize;
                        while self.peek() == &TokenKind::Comma {
                            self.advance();
                            if self.peek() == &TokenKind::RParen { break; } // trailing comma
                            fields.push((format!("_{}", i), self.parse_expr()?));
                            i += 1;
                        }
                        self.expect(&TokenKind::RParen)?;
                        Ok(Expr::RecordConstruct("__tuple__".to_string(), fields, self.span_from(&start)))
                    } else {
                        self.expect(&TokenKind::RParen)?;
                        Ok(first) // grouping parens: (expr) → expr
                    }
                }
            }

            // `emit expr` — event publish (1-11)
            TokenKind::Emit => {
                self.advance();
                let inner = self.parse_expr()?;
                Ok(Expr::EmitExpr(Box::new(inner), self.span_from(&start)))
            }

            // `collect { ... }` — list accumulation (v0.5.0)
            TokenKind::Collect => {
                self.advance();
                let block = self.parse_block()?;
                Ok(Expr::Collect(Box::new(block), self.span_from(&start)))
            }

            // identifier or record construction (3-13, 1-10)
            TokenKind::Ident(name) => {
                self.advance();
                // If uppercase IDENT followed by `{` → record construction
                if name == "assert_matches" && self.peek() == &TokenKind::LParen {
                    return self.parse_assert_matches(start);
                }
                if name
                    .chars()
                    .next()
                    .map(|c| c.is_uppercase())
                    .unwrap_or(false)
                    && self.peek() == &TokenKind::LBrace
                {
                    self.advance(); // consume `{`
                    let mut fields = Vec::new();
                    while self.peek() != &TokenKind::RBrace {
                        let (field_name, _) = self.expect_ident()?;
                        self.expect(&TokenKind::Colon)?;
                        let field_expr = self.parse_expr()?;
                        fields.push((field_name, field_expr));
                        if self.peek() == &TokenKind::Comma {
                            self.advance();
                        }
                    }
                    self.expect(&TokenKind::RBrace)?;
                    Ok(Expr::RecordConstruct(name, fields, self.span_from(&start)))
                } else {
                    Ok(Expr::Ident(name, start))
                }
            }

            // `pipeline` used as identifier expression (backward compat — v22.5.0 soft keyword)
            TokenKind::Pipeline => {
                self.advance();
                Ok(Expr::Ident("pipeline".to_string(), start))
            }

            // effect keywords used as namespaces (Pure, Io, Emit → identifiers)
            TokenKind::Pure => {
                self.advance();
                Ok(Expr::Ident("Pure".into(), start))
            }
            TokenKind::Io => {
                self.advance();
                Ok(Expr::Ident("Io".into(), start))
            }

            // closure: |x, y| expr  (3-16)
            TokenKind::Pipe => {
                self.advance();
                let params = self.parse_closure_params()?;
                self.expect(&TokenKind::Pipe)?;
                let body = self.parse_expr()?;
                Ok(Expr::Closure(
                    params,
                    Box::new(body),
                    self.span_from(&start),
                ))
            }

            // empty closure: || expr
            TokenKind::PipePipe => {
                self.advance();
                let body = self.parse_expr()?;
                Ok(Expr::Closure(
                    Vec::new(),
                    Box::new(body),
                    self.span_from(&start),
                ))
            }

            TokenKind::PipeGt => {
                // `|>` at start of primary is ambiguous; treat as parse error
                Err(ParseError::new("unexpected '|>'", start))
            }

            // block (3-17) or record spread { ...base, key: val } (v16.3.0)
            TokenKind::LBrace => {
                if self.peek2() == Some(&TokenKind::DotDotDot) {
                    let start_spread = self.peek_span().clone();
                    self.advance(); // consume '{'
                    self.parse_record_spread(start_spread)
                } else {
                    Ok(Expr::Block(Box::new(self.parse_block()?)))
                }
            }

            // list comprehension / result comprehension: [expr | x <- src, ...]  (v17.3.0)
            TokenKind::LBracket => {
                self.advance(); // consume '['
                let is_result = if self.peek() == &TokenKind::Question {
                    self.advance(); // consume '?'
                    true
                } else {
                    false
                };
                let expr = self.parse_expr()?;
                self.expect(&TokenKind::Pipe)?; // '|'
                let clauses = self.parse_comp_clauses()?;
                self.expect(&TokenKind::RBracket)?;
                let span = self.span_from(&start);
                if is_result {
                    Ok(Expr::ResultComp { expr: Box::new(expr), clauses, span })
                } else {
                    Ok(Expr::ListComp { expr: Box::new(expr), clauses, span })
                }
            }

            // match (3-18, 3-19)
            TokenKind::Match => self.parse_match_expr(),

            // if (3-20)
            TokenKind::If => self.parse_if_expr(),

            other => Err(ParseError::new(
                format!("expected expression, got {:?}", other),
                start,
            )),
        }
    }

    /// `{ name, email }` or `{ name: pat }` — shared by bare record and record-variant patterns.
    fn parse_assert_matches(&mut self, start: Span) -> Result<Expr, ParseError> {
        self.expect(&TokenKind::LParen)?;
        let expr = self.parse_expr()?;
        self.expect(&TokenKind::Comma)?;
        let pattern = self.parse_pattern()?;
        self.expect(&TokenKind::RParen)?;
        Ok(Expr::AssertMatches(
            Box::new(expr),
            Box::new(pattern),
            self.span_from(&start),
        ))
    }

    /// Parse `{ ...base, key: val, ... }` — record spread (v16.3.0).
    /// Called after consuming the opening `{`.
    fn parse_record_spread(&mut self, start: Span) -> Result<Expr, ParseError> {
        self.expect(&TokenKind::DotDotDot)?;
        let base = self.parse_expr()?;
        let mut updates: Vec<(String, Expr)> = Vec::new();
        while self.peek() == &TokenKind::Comma {
            self.advance(); // consume ','
            if self.peek() == &TokenKind::RBrace {
                break; // trailing comma
            }
            let (fname, _) = self.expect_ident()?;
            self.expect(&TokenKind::Colon)?;
            let val = self.parse_expr()?;
            updates.push((fname, val));
        }
        self.expect(&TokenKind::RBrace)?;
        Ok(Expr::RecordSpread(Box::new(base), updates, self.span_from(&start)))
    }

        fn parse_record_field_patterns(&mut self) -> Result<Vec<PatternField>, ParseError> {
        self.expect(&TokenKind::LBrace)?;
        let mut fields = Vec::new();
        while self.peek() != &TokenKind::RBrace {
            let fs = self.peek_span().clone();
            let field = if self.peek() == &TokenKind::Underscore {
                self.advance();
                PatternField::Wildcard(self.span_from(&fs))
            } else {
                let (name, _) = self.expect_ident()?;
                if self.peek() == &TokenKind::Colon {
                    self.advance();
                    PatternField::Alias(name, Box::new(self.parse_pattern()?), self.span_from(&fs))
                } else {
                    PatternField::Pun(name, self.span_from(&fs))
                }
            };
            fields.push(field);
            if self.peek() == &TokenKind::Comma {
                self.advance();
            }
        }
        self.expect(&TokenKind::RBrace)?;
        Ok(fields)
    }

    fn parse_fstring_parts(&mut self, raw: &str, base_span: Span) -> Result<Expr, ParseError> {
        let mut parts = Vec::new();
        let chars: Vec<char> = raw.chars().collect();
        let mut i = 0usize;
        let mut lit = String::new();

        while i < chars.len() {
            match chars[i] {
                '\\' => {
                    if i + 1 >= chars.len() {
                        lit.push('\\');
                        i += 1;
                        continue;
                    }
                    match chars[i + 1] {
                        '{' => lit.push('{'),
                        '}' => lit.push('}'),
                        'n' => lit.push('\n'),
                        't' => lit.push('\t'),
                        'r' => lit.push('\r'),
                        '"' => lit.push('"'),
                        '\\' => lit.push('\\'),
                        other => {
                            lit.push('\\');
                            lit.push(other);
                        }
                    }
                    i += 2;
                }
                '{' => {
                    if !lit.is_empty() {
                        parts.push(FStringPart::Lit(std::mem::take(&mut lit)));
                    }
                    let mut depth = 0usize;
                    let mut expr_src = String::new();
                    i += 1;
                    while i < chars.len() {
                        match chars[i] {
                            '{' => {
                                depth += 1;
                                expr_src.push('{');
                            }
                            '}' if depth == 0 => break,
                            '}' => {
                                depth -= 1;
                                expr_src.push('}');
                            }
                            ch => expr_src.push(ch),
                        }
                        i += 1;
                    }
                    if i >= chars.len() {
                        return Err(ParseError::new(
                            "unterminated string interpolation expression",
                            base_span,
                        ));
                    }
                    let inner = Parser::parse_str_expr(&expr_src, &base_span.file)?;
                    parts.push(FStringPart::Expr(Box::new(inner)));
                    i += 1;
                }
                ch => {
                    lit.push(ch);
                    i += 1;
                }
            }
        }

        if !lit.is_empty() {
            parts.push(FStringPart::Lit(lit));
        }

        Ok(Expr::FString(parts, base_span))
    }

    // closure params: untyped  |x, y|  (3-16)
    fn parse_closure_params(&mut self) -> Result<Vec<String>, ParseError> {
        let mut params = Vec::new();
        while self.peek() != &TokenKind::Pipe {
            // Allow `_` wildcard as an anonymous parameter
            let name = if self.peek() == &TokenKind::Underscore {
                self.advance();
                "_".to_string()
            } else {
                let (n, _) = self.expect_ident()?;
                n
            };
            params.push(name);
            if self.peek() == &TokenKind::Comma {
                self.advance();
            }
        }
        Ok(params)
    }

    // ── comprehension clauses (v17.3.0) ──────────────────────────────────────

    /// Parse `var <- src, guard, var2 <- src2, ...` until `]` is seen.
    /// Called after consuming the `|` separator in `[expr | clauses]`.
    fn parse_comp_clauses(&mut self) -> Result<Vec<CompClause>, ParseError> {
        let mut clauses = Vec::new();
        loop {
            let start = self.peek_span().clone();
            // Detect `ident <- src` (For clause) via 2-token lookahead
            if matches!(self.peek(), TokenKind::Ident(_))
                && self.peek2() == Some(&TokenKind::LArrow)
            {
                let (var, _) = self.expect_ident()?;
                self.expect(&TokenKind::LArrow)?;
                let src = self.parse_expr()?;
                clauses.push(CompClause::For { var, src: Box::new(src), span: self.span_from(&start) });
            } else {
                let guard = self.parse_expr()?;
                clauses.push(CompClause::Guard(Box::new(guard)));
            }
            if self.peek() == &TokenKind::Comma {
                self.advance();
            } else {
                break;
            }
        }
        Ok(clauses)
    }

    // ── match (3-18, 3-19) ───────────────────────────────────────────────────

    fn parse_match_expr(&mut self) -> Result<Expr, ParseError> {
        let start = self.peek_span().clone();
        self.expect(&TokenKind::Match)?;
        let scrutinee = self.parse_expr()?;
        self.expect(&TokenKind::LBrace)?;

        let mut arms = Vec::new();
        while self.peek() != &TokenKind::RBrace {
            arms.push(self.parse_match_arm()?);
        }
        self.expect(&TokenKind::RBrace)?;

        Ok(Expr::Match(
            Box::new(scrutinee),
            arms,
            self.span_from(&start),
        ))
    }

    fn parse_match_arm(&mut self) -> Result<MatchArm, ParseError> {
        let start = self.peek_span().clone();
        let first_pat = self.parse_pattern()?;
        // or-pattern: collect additional alternatives separated by `|` (v17.2.0)
        let pattern = if self.peek() == &TokenKind::Pipe {
            let or_start = first_pat.span().clone();
            let mut pats = vec![first_pat];
            while self.peek() == &TokenKind::Pipe {
                self.advance(); // consume '|'
                pats.push(self.parse_pattern()?);
            }
            Pattern::Or(pats, self.span_from(&or_start))
        } else {
            first_pat
        };
        // optional guard: `if expr` (v17.2.0) or legacy `where expr` (v0.5.0)
        let guard = if self.peek() == &TokenKind::If {
            self.advance();
            Some(Box::new(self.parse_expr()?))
        } else if self.peek() == &TokenKind::Where {
            self.advance();
            Some(Box::new(self.parse_expr()?))
        } else {
            None
        };
        self.expect(&TokenKind::FatArrow)?;
        let body = self.parse_expr()?;
        // optional trailing comma
        if self.peek() == &TokenKind::Comma {
            self.advance();
        }
        Ok(MatchArm {
            pattern,
            guard,
            body,
            span: self.span_from(&start),
        })
    }

    // ── if (3-20) ────────────────────────────────────────────────────────────

    fn parse_if_expr(&mut self) -> Result<Expr, ParseError> {
        let start = self.peek_span().clone();
        self.expect(&TokenKind::If)?;
        let cond = self.parse_expr()?;
        let then_block = self.parse_block()?;
        let else_block = if self.peek() == &TokenKind::Else {
            self.advance();
            if self.peek() == &TokenKind::If {
                // `else if` → desugar to `else { if ... }`
                let start_else = self.peek_span().clone();
                let if_expr = self.parse_if_expr()?;
                let span = self.span_from(&start_else);
                Some(Box::new(Block {
                    stmts: vec![],
                    expr: Box::new(if_expr),
                    span,
                }))
            } else {
                Some(Box::new(self.parse_block()?))
            }
        } else {
            None
        };
        Ok(Expr::If(
            Box::new(cond),
            Box::new(then_block),
            else_block,
            self.span_from(&start),
        ))
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(src: &str) -> Program {
        Parser::parse_str(src, "test").expect("parse error")
    }

    fn parse_err(src: &str) -> String {
        Parser::parse_str(src, "test")
            .expect_err("expected error")
            .message
    }

    fn parse_expr_ok(src: &str) -> Expr {
        Parser::parse_str_expr(src, "test").expect("parse expr")
    }

    // type_def — record (3-3)
    #[test]
    fn test_parse_record_type() {
        let p = parse("type User = { name: String email: String }");
        assert!(matches!(p.items[0], Item::TypeDef(_)));
        if let Item::TypeDef(td) = &p.items[0] {
            assert_eq!(td.name, "User");
            assert!(matches!(td.body, TypeBody::Record(_)));
        }
    }

    // type_def — sum (3-4)
    #[test]
    fn test_parse_sum_type() {
        let p = parse("type Session = | Guest | Authenticated { user: User }");
        if let Item::TypeDef(td) = &p.items[0] {
            assert!(matches!(td.body, TypeBody::Sum(_)));
            if let TypeBody::Sum(vs) = &td.body {
                assert_eq!(vs.len(), 2);
                assert!(matches!(&vs[0], Variant::Unit(n, _) if n == "Guest"));
                assert!(matches!(&vs[1], Variant::Record(n, _, _) if n == "Authenticated"));
            }
        }
    }

    // type_def — tuple variant (3-4)
    #[test]
    fn test_parse_tuple_variant() {
        let p = parse("type ParseResult = | ok(User) | err(String)");
        if let Item::TypeDef(td) = &p.items[0] {
            if let TypeBody::Sum(vs) = &td.body {
                assert!(matches!(&vs[0], Variant::Tuple(n, _, _) if n == "ok"));
                assert!(matches!(&vs[1], Variant::Tuple(n, _, _) if n == "err"));
            }
        }
    }

    // fn_def (3-5)
    #[test]
    fn test_parse_fn_def() {
        let p = parse("fn add(x: Int) -> Int { x }");
        assert!(matches!(p.items[0], Item::FnDef(_)));
        if let Item::FnDef(f) = &p.items[0] {
            assert_eq!(f.name, "add");
            assert_eq!(f.params.len(), 1);
            assert_eq!(f.params[0].name, "x");
        }
    }

    // fn_def with visibility (3-5)
    // v34.8A: !Effect is now E0374 — test that it errors, and that visibility still parses
    #[test]
    fn test_parse_public_fn() {
        // !Io now causes E0374
        let err = Parser::parse_str("public fn main() -> Unit !Io { () }", "test").unwrap_err();
        assert!(err.message.contains("E0374"), "expected E0374, got: {}", err.message);
        // Without !Effect, public fn parses fine
        let p = parse("public fn main() -> Unit { () }");
        if let Item::FnDef(f) = &p.items[0] {
            assert_eq!(f.visibility, Some(Visibility::Public));
        }
    }

    // trf_def (3-6)
    #[test]
    fn test_parse_trf_def() {
        let p = parse("stage ParseCsv: String -> List<Row> = |text| { text }");
        assert!(matches!(p.items[0], Item::TrfDef(_)));
        if let Item::TrfDef(t) = &p.items[0] {
            assert_eq!(t.name, "ParseCsv");
        }
    }

    // v2.0.0: trf keyword is a parse error
    #[test]
    fn test_parse_trf_removed_error() {
        let result = Parser::parse_str("trf ParseCsv: String -> String = |s| s", "test");
        assert!(result.is_err());
        assert!(result.unwrap_err().message.contains("trf"));
    }

    // stage_def with effect (3-6, 3-22)
    // v34.8A: !Effect is now E0374
    #[test]
    fn test_parse_trf_with_effect() {
        let err = Parser::parse_str("stage Print: String -> Unit !Io = |s| { () }", "test").unwrap_err();
        assert!(err.message.contains("E0374"), "expected E0374, got: {}", err.message);
    }

    // seq_def (3-7)
    #[test]
    fn test_parse_flw_def() {
        let p = parse("seq Import = ParseCsv |> ValidateUser |> SaveUsers");
        if let Item::FlwDef(f) = &p.items[0] {
            assert_eq!(f.name, "Import");
            assert!(matches!(&f.steps[0], FlwStep::Stage(s) if s == "ParseCsv"));
            assert!(matches!(&f.steps[1], FlwStep::Stage(s) if s == "ValidateUser"));
            assert!(matches!(&f.steps[2], FlwStep::Stage(s) if s == "SaveUsers"));
        }
    }

    #[test]
    fn test_parse_flw_def_with_par() {
        let p = parse("seq FullReport = par [FetchOrders, FetchPrices] |> Merge |> Save");
        if let Item::FlwDef(f) = &p.items[0] {
            assert_eq!(f.name, "FullReport");
            assert!(matches!(&f.steps[0], FlwStep::Par(names) if names == &vec!["FetchOrders", "FetchPrices"]));
            assert!(matches!(&f.steps[1], FlwStep::Stage(s) if s == "Merge"));
            assert!(matches!(&f.steps[2], FlwStep::Stage(s) if s == "Save"));
        }
    }

    // v2.0.0: flw keyword is a parse error
    #[test]
    fn test_parse_flw_removed_error() {
        let result = Parser::parse_str("flw Import = ParseCsv |> ValidateUser", "test");
        assert!(result.is_err());
        assert!(result.unwrap_err().message.contains("flw"));
    }

    // bind_stmt — simple (3-9)
    #[test]
    fn test_parse_bind_simple() {
        let p = parse("fn f() -> Unit { bind x <- 1; () }");
        if let Item::FnDef(f) = &p.items[0] {
            assert!(matches!(f.body.stmts[0], Stmt::Bind(_)));
        }
    }

    // bind_stmt — record decomposition (3-10)
    #[test]
    fn test_parse_bind_record() {
        let p = parse("fn f() -> Unit { bind { name } <- user; () }");
        if let Item::FnDef(f) = &p.items[0] {
            if let Stmt::Bind(b) = &f.body.stmts[0] {
                assert!(matches!(b.pattern, Pattern::Record(_, _)));
                if let Pattern::Record(fields, _) = &b.pattern {
                    assert!(matches!(fields[0], PatternField::Pun(_, _)));
                }
            }
        }
    }

    #[test]
    fn test_parse_bind_record_alias_wildcard() {
        let p = parse("fn f() -> Unit { bind { age: user_age, _ } <- user; () }");
        if let Item::FnDef(f) = &p.items[0] {
            if let Stmt::Bind(b) = &f.body.stmts[0] {
                if let Pattern::Record(fields, _) = &b.pattern {
                    assert!(matches!(fields[0], PatternField::Alias(_, _, _)));
                    assert!(matches!(fields[1], PatternField::Wildcard(_)));
                } else {
                    panic!("expected record pattern");
                }
            }
        }
    }

    // bind_stmt — variant decomposition (3-11)
    #[test]
    fn test_parse_bind_variant() {
        let p = parse("fn f() -> Unit { bind ok(v) <- result; () }");
        if let Item::FnDef(f) = &p.items[0] {
            if let Stmt::Bind(b) = &f.body.stmts[0] {
                assert!(matches!(b.pattern, Pattern::Variant(_, Some(_), _)));
            }
        }
    }

    #[test]
    fn test_parse_bind_state_annotation() {
        let p = parse("fn f() -> Unit { bind x: PosInt <- 42; () }");
        if let Item::FnDef(f) = &p.items[0] {
            if let Stmt::Bind(b) = &f.body.stmts[0] {
                match &b.annotated_ty {
                    Some(TypeExpr::Named(name, _, _)) => assert_eq!(name, "PosInt"),
                    _ => panic!("expected bind annotation"),
                }
            }
        }
    }

    #[test]
    fn test_parse_fstring_simple() {
        let expr = parse_expr_ok(r#"$"Hello {name}!""#);
        match expr {
            Expr::FString(parts, _) => {
                assert_eq!(parts.len(), 3);
                assert!(matches!(&parts[0], FStringPart::Lit(s) if s == "Hello "));
                assert!(matches!(&parts[1], FStringPart::Expr(_)));
                assert!(matches!(&parts[2], FStringPart::Lit(s) if s == "!"));
            }
            other => panic!("expected fstring, got {:?}", other),
        }
    }

    #[test]
    fn test_parse_fstring_literal_only() {
        let expr = parse_expr_ok(r#"$"literal only""#);
        assert!(matches!(expr, Expr::FString(_, _)));
    }

    #[test]
    fn test_parse_fstring_escape_brace() {
        let expr = parse_expr_ok(r#"$"\{value\}""#);
        match expr {
            Expr::FString(parts, _) => {
                assert_eq!(parts.len(), 1);
                assert!(matches!(&parts[0], FStringPart::Lit(s) if s == "{value}"));
            }
            other => panic!("expected fstring, got {:?}", other),
        }
    }

    #[test]
    fn test_parse_assert_matches_expr() {
        let expr = parse_expr_ok(r#"assert_matches(user, { name, age })"#);
        match expr {
            Expr::AssertMatches(expr, pattern, _) => {
                assert!(matches!(*expr, Expr::Ident(_, _)));
                assert!(matches!(*pattern, Pattern::Record(_, _)));
            }
            other => panic!("expected assert_matches, got {:?}", other),
        }
    }

    // literals (3-12)
    #[test]
    fn test_parse_literals() {
        let p = parse("fn f() -> Int { 42 }");
        if let Item::FnDef(f) = &p.items[0] {
            assert!(matches!(*f.body.expr, Expr::Lit(Lit::Int(42), _)));
        }
    }

    // identifier (3-13)
    #[test]
    fn test_parse_ident_expr() {
        let p = parse("fn f() -> String { name }");
        if let Item::FnDef(f) = &p.items[0] {
            assert!(matches!(*f.body.expr, Expr::Ident(_, _)));
        }
    }

    // function application (3-14)
    #[test]
    fn test_parse_apply() {
        let p = parse("fn f() -> Unit { g(1, 2) }");
        if let Item::FnDef(f) = &p.items[0] {
            assert!(matches!(*f.body.expr, Expr::Apply(_, _, _)));
        }
    }

    // pipeline |> (3-15)
    #[test]
    fn test_parse_pipeline() {
        let p = parse("fn f() -> Unit { text |> ParseCsv |> Validate }");
        if let Item::FnDef(f) = &p.items[0] {
            assert!(matches!(*f.body.expr, Expr::Pipeline(_, _)));
            if let Expr::Pipeline(parts, _) = &*f.body.expr {
                assert_eq!(parts.len(), 3);
            }
        }
    }

    // closure (3-16)
    #[test]
    fn test_parse_closure() {
        let p = parse("fn f() -> Unit { |x| x }");
        if let Item::FnDef(f) = &p.items[0] {
            assert!(matches!(*f.body.expr, Expr::Closure(_, _, _)));
        }
    }

    #[test]
    fn test_parse_empty_closure() {
        let p = parse("fn f() -> Int { bind g <- || 42; g() }");
        if let Item::FnDef(f) = &p.items[0] {
            assert!(matches!(f.body.stmts[0], Stmt::Bind(_)));
        }
    }

    // block (3-17)
    #[test]
    fn test_parse_block() {
        let p = parse("fn f() -> Int { bind x <- 1; x + 1 }");
        if let Item::FnDef(f) = &p.items[0] {
            assert_eq!(f.body.stmts.len(), 1);
            assert!(matches!(*f.body.expr, Expr::BinOp(_, _, _, _)));
        }
    }

    // match (3-18, 3-19)
    #[test]
    fn test_parse_match() {
        let p = parse("fn f() -> Unit { match x { Guest => () Authenticated { user } => () } }");
        if let Item::FnDef(f) = &p.items[0] {
            assert!(matches!(*f.body.expr, Expr::Match(_, _, _)));
            if let Expr::Match(_, arms, _) = &*f.body.expr {
                assert_eq!(arms.len(), 2);
            }
        }
    }

    // if (3-20)
    #[test]
    fn test_parse_if() {
        let p = parse("fn f() -> String { if flag { \"yes\" } else { \"no\" } }");
        if let Item::FnDef(f) = &p.items[0] {
            assert!(matches!(*f.body.expr, Expr::If(_, _, Some(_), _)));
        }
    }

    #[test]
    fn test_parse_logical_precedence() {
        let expr = parse_expr_ok("false || 1 == 1 && true");
        match expr {
            Expr::BinOp(BinOp::Or, _, rhs, _) => {
                assert!(matches!(*rhs, Expr::BinOp(BinOp::And, _, _, _)));
            }
            other => panic!("expected or binop, got {:?}", other),
        }
    }

    // type expr: T? and T! (3-21)
    #[test]
    fn test_parse_type_optional_fallible() {
        let p = parse("fn f() -> String? { () }");
        if let Item::FnDef(f) = &p.items[0] {
            assert!(matches!(f.return_ty, Some(TypeExpr::Optional(_, _))));
        }
        let p2 = parse("fn g() -> String! { () }");
        if let Item::FnDef(f) = &p2.items[0] {
            assert!(matches!(f.return_ty, Some(TypeExpr::Fallible(_, _))));
        }
    }

    #[test]
    fn test_parse_fn_def_inferred_return() {
        let p = parse("fn double(n: Int) = n * 2");
        if let Item::FnDef(f) = &p.items[0] {
            assert!(f.return_ty.is_none());
            assert!(f.body.stmts.is_empty());
        }
    }

    #[test]
    fn test_parse_fn_def_explicit_return_eq_expr() {
        let p = parse("fn id(x: Int) -> Int = x");
        if let Item::FnDef(f) = &p.items[0] {
            assert!(matches!(f.return_ty, Some(TypeExpr::Named(_, _, _))));
            assert!(f.body.stmts.is_empty());
        }
    }

    // effect annotation (3-22) — v34.8A: !Effect syntax removed, must be E0374
    #[test]
    fn test_parse_effect_annotation() {
        let err = Parser::parse_str("fn f() -> Unit !Io { () }", "test").unwrap_err();
        assert!(err.message.contains("E0374"), "expected E0374, got: {}", err.message);
    }

    #[test]
    fn test_parse_db_read_write_admin_effects() {
        let err = Parser::parse_str("fn f() -> Unit !DbRead !DbWrite !DbAdmin { () }", "test").unwrap_err();
        assert!(err.message.contains("E0374"), "expected E0374, got: {}", err.message);
    }

    #[test]
    fn effect_def_parses() {
        let p = parse("effect Payment");
        match &p.items[0] {
            Item::EffectDef(ed) => {
                assert_eq!(ed.name, "Payment");
                assert!(ed.visibility.is_none());
            }
            other => panic!("expected EffectDef, got {:?}", other),
        }
    }

    #[test]
    fn public_effect_def_parses() {
        let p = parse("public effect Notification");
        match &p.items[0] {
            Item::EffectDef(ed) => {
                assert_eq!(ed.name, "Notification");
                assert_eq!(ed.visibility, Some(Visibility::Public));
            }
            other => panic!("expected EffectDef, got {:?}", other),
        }
    }

    #[test]
    fn test_parse_fn_param_trf_type() {
        // v34.8A: !Db in type annotation also removed (E0374)
        let err = Parser::parse_str("fn f(save: String -> Int !Db) -> Unit { () }", "test").unwrap_err();
        assert!(err.message.contains("E0374"), "expected E0374, got: {}", err.message);
    }

    #[test]
    fn test_parse_file_effect_annotation() {
        // v34.8A: !File annotation removed (E0374)
        let err = Parser::parse_str("fn f() -> String !File { \"ok\" }", "test").unwrap_err();
        assert!(err.message.contains("E0374"), "expected E0374, got: {}", err.message);
    }

    // field access
    #[test]
    fn test_parse_field_access() {
        let p = parse("fn f() -> String { user.name }");
        if let Item::FnDef(f) = &p.items[0] {
            assert!(matches!(*f.body.expr, Expr::FieldAccess(_, _, _)));
        }
    }

    #[test]
    fn parse_field_with_col_attr() {
        let p = parse("type Row = { #[col(0)] id: Int }");
        let Item::TypeDef(td) = &p.items[0] else {
            panic!("expected TypeDef")
        };
        let TypeBody::Record(fields) = &td.body else {
            panic!("expected record type")
        };
        assert_eq!(fields.len(), 1);
        assert_eq!(fields[0].attrs.len(), 1);
        assert_eq!(fields[0].attrs[0].name, "col");
        assert_eq!(fields[0].attrs[0].arg.as_deref(), Some("0"));
    }

    #[test]
    fn parse_type_with_multiple_col_attrs() {
        let p = parse("type Row = { #[col(0)] id: Int #[col(1)] name: String }");
        let Item::TypeDef(td) = &p.items[0] else {
            panic!("expected TypeDef")
        };
        let TypeBody::Record(fields) = &td.body else {
            panic!("expected record type")
        };
        assert_eq!(fields.len(), 2);
        assert_eq!(fields[0].attrs[0].arg.as_deref(), Some("0"));
        assert_eq!(fields[1].attrs[0].arg.as_deref(), Some("1"));
    }

    #[test]
    fn parse_type_apply_on_namespaced_call() {
        let p = parse("fn f(text: String) -> Unit { csv.parse<User>(text) }");
        let Item::FnDef(f) = &p.items[0] else {
            panic!("expected FnDef")
        };
        match f.body.expr.as_ref() {
            Expr::Apply(callee, args, _) => {
                assert_eq!(args.len(), 1);
                match callee.as_ref() {
                    Expr::TypeApply(inner, type_args, _) => {
                        assert_eq!(type_args.len(), 1);
                        assert!(matches!(inner.as_ref(), Expr::FieldAccess(_, _, _)));
                    }
                    other => panic!("expected TypeApply callee, got {:?}", other),
                }
            }
            other => panic!("expected Apply expr, got {:?}", other),
        }
    }

    #[test]
    fn parse_type_name_of_call() {
        let p = parse("fn f() -> String { type_name_of<Row>() }");
        let Item::FnDef(f) = &p.items[0] else {
            panic!("expected FnDef")
        };
        match f.body.expr.as_ref() {
            Expr::Apply(callee, args, _) => {
                assert!(args.is_empty());
                assert!(matches!(callee.as_ref(), Expr::TypeApply(_, _, _)));
            }
            other => panic!("expected Apply expr, got {:?}", other),
        }
    }

    // error: bad item
    #[test]
    fn test_parse_error_bad_item() {
        let msg = parse_err("bind x <- 1");
        assert!(msg.contains("expected item"));
    }

    // ── v0.2.0 parser tests (1-13) ────────────────────────────────────────────

    // 1-8, 1-9: multiple effects — v34.8A: !Effect removed (E0374)
    #[test]
    fn test_parse_multi_effect() {
        let err = Parser::parse_str("stage T: Int -> Int !Db !Emit<UserCreated> = |n| { n }", "test").unwrap_err();
        assert!(err.message.contains("E0374"), "expected E0374, got: {}", err.message);
    }

    // 1-8: fn with multiple effects — v34.8A: !Effect removed (E0374)
    #[test]
    fn test_parse_fn_multi_effect() {
        let err = Parser::parse_str("fn f() -> Unit !Io !Db { () }", "test").unwrap_err();
        assert!(err.message.contains("E0374"), "expected E0374, got: {}", err.message);
    }

    // 1-10: record construction expression
    #[test]
    fn test_parse_record_construct() {
        let p = parse(r#"fn f() -> User { User { name: "Alice", age: 30 } }"#);
        if let Item::FnDef(f) = &p.items[0] {
            assert!(matches!(*f.body.expr, Expr::RecordConstruct(_, _, _)));
            if let Expr::RecordConstruct(name, fields, _) = f.body.expr.as_ref() {
                assert_eq!(name, "User");
                assert_eq!(fields.len(), 2);
                assert_eq!(fields[0].0, "name");
                assert_eq!(fields[1].0, "age");
            }
        } else {
            panic!("expected FnDef");
        }
    }

    // 1-11: emit expression — v34.8A: !Emit<E> annotation removed (E0374); emit expr still valid
    #[test]
    fn test_parse_emit_expr() {
        let err = Parser::parse_str(r#"fn f() -> Unit !Emit<E> { emit "hello" }"#, "test").unwrap_err();
        assert!(err.message.contains("E0374"), "expected E0374, got: {}", err.message);
        // emit expression parses without annotation
        let p = parse(r#"fn f() -> Unit { emit "hello" }"#);
        if let Item::FnDef(f) = &p.items[0] {
            assert!(matches!(*f.body.expr, Expr::EmitExpr(_, _)));
        }
    }

    // 1-12: emit in block stmt position — v34.8A: !Emit<E> annotation removed (E0374)
    #[test]
    fn test_parse_emit_in_block() {
        let err = Parser::parse_str(r#"fn f() -> Unit !Emit<E> { emit "ev"; () }"#, "test").unwrap_err();
        assert!(err.message.contains("E0374"), "expected E0374, got: {}", err.message);
        // emit in block still valid without annotation
        let p = parse(r#"fn f() -> Unit { emit "ev"; () }"#);
        if let Item::FnDef(f) = &p.items[0] {
            assert_eq!(f.body.stmts.len(), 1);
            assert!(matches!(f.body.stmts[0], Stmt::Expr(Expr::EmitExpr(_, _))));
        }
    }

    // 1-9: Emit<T> as only effect — v34.8A: removed (E0374)
    #[test]
    fn test_parse_emit_effect_only() {
        let err = Parser::parse_str("fn f() -> Unit !Emit<OrderPlaced> { () }", "test").unwrap_err();
        assert!(err.message.contains("E0374"), "expected E0374, got: {}", err.message);
    }

    // ── v0.3.0 parser tests (1-12) ────────────────────────────────────────────

    #[test]
    fn test_parse_namespace() {
        let p = parse("namespace data.users\nfn f() -> Unit { () }");
        assert_eq!(p.namespace, Some("data.users".to_string()));
        assert_eq!(p.items.len(), 1);
    }

    #[test]
    fn test_parse_use() {
        let p = parse("use data.users.create\nfn f() -> Unit { () }");
        assert_eq!(p.uses.len(), 1);
        assert_eq!(p.uses[0], vec!["data", "users", "create"]);
    }

    #[test]
    fn test_parse_namespace_and_use() {
        let p = parse(
            "namespace service.main\nuse data.users.create\nuse data.users.User\nfn f() -> Unit { () }",
        );
        assert_eq!(p.namespace, Some("service.main".to_string()));
        assert_eq!(p.uses.len(), 2);
        assert_eq!(p.uses[0], vec!["data", "users", "create"]);
        assert_eq!(p.uses[1], vec!["data", "users", "User"]);
        assert_eq!(p.items.len(), 1);
    }

    #[test]
    fn test_parse_namespace_no_items() {
        let p = parse("namespace data.csv");
        assert_eq!(p.namespace, Some("data.csv".to_string()));
        assert!(p.items.is_empty());
    }

    #[test]
    fn test_parse_use_single_segment() {
        let p = parse("use main\nfn f() -> Unit { () }");
        assert_eq!(p.uses[0], vec!["main"]);
    }

    #[test]
    fn test_parse_namespace_after_def_error() {
        let msg = parse_err("fn f() -> Unit { () }\nnamespace data.csv");
        assert!(msg.contains("`namespace`"));
    }

    // ── Phase 2: generics / cap / impl (v0.4.0) ───────────────────────────────

    #[test]
    fn test_parse_generic_fn() {
        let p = parse("fn identity<T>(x: T) -> T { x }");
        let Item::FnDef(fd) = &p.items[0] else {
            panic!("expected FnDef")
        };
        assert_eq!(fd.name, "identity");
        assert_eq!(crate::ast::param_names(&fd.type_params), vec!["T"]);
        assert_eq!(fd.params[0].name, "x");
    }

    #[test]
    fn test_parse_generic_type() {
        let p = parse("type Pair<A, B> = { first: A second: B }");
        let Item::TypeDef(td) = &p.items[0] else {
            panic!("expected TypeDef")
        };
        assert_eq!(td.name, "Pair");
        assert_eq!(crate::ast::param_names(&td.type_params), vec!["A", "B"]);
    }

    #[test]
    fn test_parse_type_with_interfaces() {
        let p = parse("type UserRow with Show, Eq = { name: String }");
        let Item::TypeDef(td) = &p.items[0] else {
            panic!("expected TypeDef")
        };
        assert_eq!(td.name, "UserRow");
        assert_eq!(td.with_interfaces, vec!["Show", "Eq"]);
    }

    #[test]
    fn test_parse_type_single_invariant() {
        let p = parse("type PosInt = { value: Int invariant value > 0 }");
        let Item::TypeDef(td) = &p.items[0] else {
            panic!("expected TypeDef")
        };
        assert_eq!(td.invariants.len(), 1);
        assert!(matches!(td.body, TypeBody::Record(_)));
    }

    #[test]
    fn test_parse_type_multi_invariant() {
        let p = parse("type UserAge = { value: Int invariant value >= 0 invariant value <= 150 }");
        let Item::TypeDef(td) = &p.items[0] else {
            panic!("expected TypeDef")
        };
        assert_eq!(td.invariants.len(), 2);
    }

    #[test]
    fn test_parse_type_string_invariant() {
        let p = parse("type Email = { value: String invariant String.contains(value, \"@\") }");
        let Item::TypeDef(td) = &p.items[0] else {
            panic!("expected TypeDef")
        };
        assert_eq!(td.invariants.len(), 1);
    }

    #[test]
    fn test_parse_generic_trf() {
        let p = parse("stage MapOpt<T, U>: Option<T> -> Option<U> = |x| { x }");
        let Item::TrfDef(td) = &p.items[0] else {
            panic!("expected TrfDef")
        };
        assert_eq!(crate::ast::param_names(&td.type_params), vec!["T", "U"]);
    }

    // v2.0.0: cap keyword is a parse error
    #[test]
    fn test_parse_cap_removed_error() {
        let result = Parser::parse_str("cap Eq<T> = { equals: T -> T -> Bool }", "test");
        assert!(result.is_err());
        assert!(result.unwrap_err().message.contains("cap"));
    }

    #[test]
    fn test_parse_interface_decl() {
        let p = parse("interface Show { show: Self -> String }");
        let Item::InterfaceDecl(id) = &p.items[0] else {
            panic!("expected InterfaceDecl")
        };
        assert_eq!(id.name, "Show");
        assert_eq!(id.methods.len(), 1);
        assert_eq!(id.methods[0].name, "show");
    }

    #[test]
    fn test_parse_interface_decl_with_super() {
        let p = parse("interface Ord : Eq { compare: Self -> Self -> Int }");
        let Item::InterfaceDecl(id) = &p.items[0] else {
            panic!("expected InterfaceDecl")
        };
        assert_eq!(id.name, "Ord");
        assert_eq!(id.super_interface, Some("Eq".to_string()));
        assert_eq!(id.methods.len(), 1);
        assert_eq!(id.methods[0].name, "compare");
    }

    #[test]
    fn test_parse_interface_impl_decl() {
        let p = parse("impl Show, Eq for UserRow");
        let Item::InterfaceImplDecl(id) = &p.items[0] else {
            panic!("expected InterfaceImplDecl")
        };
        assert_eq!(id.interface_names, vec!["Show", "Eq"]);
        assert_eq!(id.type_name, "UserRow");
        assert!(id.is_auto);
    }

    #[test]
    fn test_parse_impl_def() {
        // cap-style impl uses impl Name<T> { ... } — still valid in v2.0.0 (interface impls coexist)
        let src = "impl Eq<Int> { fn equals(a: Int, b: Int) -> Bool { a == b } }";
        let p = parse(src);
        let Item::ImplDef(id) = &p.items[0] else {
            panic!("expected ImplDef")
        };
        assert_eq!(id.cap_name, "Eq");
        assert_eq!(id.type_args.len(), 1);
        assert_eq!(id.methods.len(), 1);
        assert_eq!(id.methods[0].name, "equals");
    }

    #[test]
    fn test_parse_abstract_trf() {
        // v34.8A: !Db annotation removed (E0374)
        let err = Parser::parse_str("abstract stage FetchUser: UserId -> User? !Db", "test").unwrap_err();
        assert!(err.message.contains("E0374"), "expected E0374, got: {}", err.message);
        // without annotation, abstract stage parses normally
        let p = parse("abstract stage FetchUser: UserId -> User?");
        let Item::AbstractTrfDef(td) = &p.items[0] else {
            panic!("expected AbstractTrfDef")
        };
        assert_eq!(td.name, "FetchUser");
        assert!(td.type_params.is_empty());
    }

    #[test]
    fn test_parse_abstract_trf_removed_error() {
        let result = Parser::parse_str("abstract trf FetchUser: Int -> String !Db", "test");
        assert!(result.is_err());
        assert!(result.unwrap_err().message.contains("trf"));
    }

    #[test]
    fn test_parse_abstract_trf_generic() {
        // v34.8A: !Db annotation removed (E0374)
        let err = Parser::parse_str("abstract stage Fetch<T>: Int -> T? !Db", "test").unwrap_err();
        assert!(err.message.contains("E0374"), "expected E0374, got: {}", err.message);
        // without annotation, generic abstract stage parses normally
        let p = parse("abstract stage Fetch<T>: Int -> T?");
        let Item::AbstractTrfDef(td) = &p.items[0] else {
            panic!("expected AbstractTrfDef")
        };
        assert_eq!(td.name, "Fetch");
        assert_eq!(crate::ast::param_names(&td.type_params), vec!["T"]);
    }

    #[test]
    fn test_parse_abstract_flw_single_slot() {
        let p = parse("abstract seq DataPipeline<Row> { parse: String -> List<Row>! }");
        let Item::AbstractFlwDef(fd) = &p.items[0] else {
            panic!("expected AbstractFlwDef")
        };
        assert_eq!(fd.name, "DataPipeline");
        assert_eq!(crate::ast::param_names(&fd.type_params), vec!["Row"]);
        assert_eq!(fd.slots.len(), 1);
        assert_eq!(fd.slots[0].name, "parse");
        assert!(fd.slots[0].abstract_trf_ty.is_none());
    }

    #[test]
    fn test_parse_abstract_flw_removed_error() {
        let result = Parser::parse_str(
            "abstract flw DataPipeline<Row> { parse: String -> String }",
            "test",
        );
        assert!(result.is_err());
        assert!(result.unwrap_err().message.contains("flw"));
    }

    #[test]
    fn test_parse_abstract_flw_slot_abstract_trf_shorthand() {
        let p = parse("abstract seq Pipeline<Row> { fetch: Fetch<Row> }");
        let Item::AbstractFlwDef(fd) = &p.items[0] else {
            panic!("expected AbstractFlwDef")
        };
        assert_eq!(fd.slots.len(), 1);
        assert!(
            matches!(fd.slots[0].abstract_trf_ty, Some(TypeExpr::Named(ref n, _, _)) if n == "Fetch")
        );
    }

    #[test]
    fn test_parse_abstract_flw_multi_slot() {
        // v34.8A: !Db removed from slot type (E0374); slot type now has no effects
        let p = parse(
            "abstract seq DataPipeline<Row> { parse: String -> List<Row>!; save: List<Row> -> Int }",
        );
        let Item::AbstractFlwDef(fd) = &p.items[0] else {
            panic!("expected AbstractFlwDef")
        };
        assert_eq!(fd.slots.len(), 2);
        assert_eq!(fd.slots[1].name, "save");
    }

    #[test]
    fn test_parse_flw_binding_full() {
        let p = parse(
            "seq UserImport = DataPipeline<UserRow> { parse <- ParseCsv; save <- SaveUsers }",
        );
        let Item::FlwBindingDef(fd) = &p.items[0] else {
            panic!("expected FlwBindingDef")
        };
        assert_eq!(fd.name, "UserImport");
        assert_eq!(fd.template, "DataPipeline");
        assert_eq!(fd.type_args.len(), 1);
        assert_eq!(fd.bindings.len(), 2);
        assert!(matches!(&fd.bindings[0].1, SlotImpl::Global(name) if name == "ParseCsv"));
    }

    #[test]
    fn test_parse_flw_binding_partial() {
        let p = parse("seq PartialImport = DataPipeline<UserRow> { parse <- ParseCsv }");
        let Item::FlwBindingDef(fd) = &p.items[0] else {
            panic!("expected FlwBindingDef")
        };
        assert_eq!(fd.bindings.len(), 1);
    }

    // ── v0.5.0 parser tests ────────────────────────────────────────────────────

    // task 2-10: chain stmt parses correctly
    #[test]
    fn test_parse_chain_stmt() {
        let p = parse("fn f() -> Result<Int, String> { chain n <- ok(42) ok(n) }");
        let Item::FnDef(f) = &p.items[0] else {
            panic!("expected FnDef")
        };
        assert_eq!(f.body.stmts.len(), 1);
        assert!(matches!(&f.body.stmts[0], Stmt::Chain(c) if c.name == "n"));
    }

    // task 2-11: yield stmt parses correctly
    #[test]
    fn test_parse_yield_stmt() {
        let p = parse("fn f() -> Unit { collect { yield 1; yield 2; () } }");
        let Item::FnDef(f) = &p.items[0] else {
            panic!("expected FnDef")
        };
        assert!(matches!(*f.body.expr, Expr::Collect(_, _)));
        if let Expr::Collect(block, _) = f.body.expr.as_ref() {
            assert_eq!(block.stmts.len(), 2);
            assert!(matches!(&block.stmts[0], Stmt::Yield(_)));
            assert!(matches!(&block.stmts[1], Stmt::Yield(_)));
        }
    }

    #[test]
    fn parse_simple_import() {
        let p = parse("import \"models/user\"\nfn main() -> Unit { () }");
        let Item::ImportDecl {
            path,
            alias,
            is_rune,
            is_public,
            ..
        } = &p.items[0]
        else {
            panic!("expected import");
        };
        assert_eq!(path, "models/user");
        assert_eq!(alias, &None);
        assert!(!is_rune);
        assert!(!is_public);
    }

    #[test]
    fn parse_import_with_alias() {
        let p = parse("import \"models/user\" as u\nfn main() -> Unit { () }");
        let Item::ImportDecl { alias, .. } = &p.items[0] else {
            panic!("expected import");
        };
        assert_eq!(alias.as_deref(), Some("u"));
    }

    #[test]
    fn parse_rune_import_bare_name() {
        // bare name (no slash) → rune
        let p = parse("import \"validate\"\nfn main() -> Unit { () }");
        let Item::ImportDecl { is_rune, .. } = &p.items[0] else {
            panic!("expected import");
        };
        assert!(*is_rune);
    }

    #[test]
    fn parse_rune_import_explicit_keyword() {
        // explicit `rune` keyword still accepted for backward compatibility
        let p = parse("import rune \"validate\"\nfn main() -> Unit { () }");
        let Item::ImportDecl { is_rune, .. } = &p.items[0] else {
            panic!("expected import");
        };
        assert!(*is_rune);
    }

    #[test]
    fn parse_local_import_with_slash() {
        // path with slash → local file, not a rune
        let p = parse("import \"models/user\"\nfn main() -> Unit { () }");
        let Item::ImportDecl { is_rune, .. } = &p.items[0] else {
            panic!("expected import");
        };
        assert!(!*is_rune);
    }

    #[test]
    fn parse_public_import() {
        let p = parse("public import \"models/user\"\nfn main() -> Unit { () }");
        let Item::ImportDecl { is_public, .. } = &p.items[0] else {
            panic!("expected import");
        };
        assert!(*is_public);
    }

    // task 2-12: collect expr parses correctly
    #[test]
    fn test_parse_collect_expr() {
        let p = parse("fn f() -> Unit { collect { yield 1; () } }");
        let Item::FnDef(f) = &p.items[0] else {
            panic!("expected FnDef")
        };
        assert!(matches!(*f.body.expr, Expr::Collect(_, _)));
    }

    // task 2-13: match guard parses correctly
    #[test]
    fn test_parse_match_guard() {
        let p = parse("fn f(x: Int) -> Int { match x { n where n > 0 => n _ => 0 } }");
        let Item::FnDef(f) = &p.items[0] else {
            panic!("expected FnDef")
        };
        if let Expr::Match(_, arms, _) = f.body.expr.as_ref() {
            assert!(arms[0].guard.is_some(), "first arm should have a guard");
            assert!(arms[1].guard.is_none(), "wildcard arm has no guard");
        } else {
            panic!("expected Match expr");
        }
    }

    // task 2-14: pipe match desugars correctly
    #[test]
    fn test_parse_pipe_match() {
        let p = parse("fn f(n: Int) -> Int { n |> match { x => x } }");
        let Item::FnDef(f) = &p.items[0] else {
            panic!("expected FnDef")
        };
        // After desugar, body.expr is a Match, not a Pipeline
        assert!(
            matches!(*f.body.expr, Expr::Match(_, _, _)),
            "expected Match after pipe desugar, got {:?}",
            f.body.expr
        );
    }

    // ── v4.1.0 RuneUse parser tests ───────────────────────────────────────────

    #[test]
    fn parse_rune_use_specific() {
        let p = parse("use connection.{ connect, close }\nfn f() -> Unit { () }");
        let Item::RuneUse { module, names, .. } = &p.items[0] else {
            panic!("expected RuneUse, got {:?}", &p.items[0]);
        };
        assert_eq!(module, "connection");
        assert_eq!(
            names,
            &RuneUseNames::Specific(vec!["connect".into(), "close".into()])
        );
    }

    #[test]
    fn parse_rune_use_wildcard() {
        let p = parse("use query.*\nfn f() -> Unit { () }");
        let Item::RuneUse { module, names, .. } = &p.items[0] else {
            panic!("expected RuneUse, got {:?}", &p.items[0]);
        };
        assert_eq!(module, "query");
        assert_eq!(names, &RuneUseNames::Wildcard);
    }

    #[test]
    fn parse_rune_use_single_name() {
        let p = parse("use migration.{ up }\nfn f() -> Unit { () }");
        let Item::RuneUse { module, names, .. } = &p.items[0] else {
            panic!("expected RuneUse");
        };
        assert_eq!(module, "migration");
        assert_eq!(names, &RuneUseNames::Specific(vec!["up".into()]));
    }

    #[test]
    fn parse_rune_use_does_not_consume_namespace_use() {
        // Traditional `use a.b.c` (namespace path) should still land in program.uses,
        // NOT be parsed as Item::RuneUse.
        let p = parse("use data.users\nfn f() -> Unit { () }");
        assert_eq!(p.uses, vec![vec!["data".to_string(), "users".to_string()]]);
        assert!(p.items.iter().all(|i| !matches!(i, Item::RuneUse { .. })));
    }
    // ── SchemaDef (v36.1.0) ──────────────────────────────────────────────────

    #[test]
    fn parse_schema_def_basic() {
        let p = parse("schema Orders { id: Int, amount: Float }");
        assert_eq!(p.items.len(), 1);
        let Item::SchemaDef(ref sd) = p.items[0] else { panic!("expected SchemaDef") };
        assert_eq!(sd.name, "Orders");
        assert_eq!(sd.fields.len(), 2);
        assert_eq!(sd.fields[0].0, "id");
        assert_eq!(sd.fields[1].0, "amount");
    }

    #[test]
    fn parse_schema_def_single_field() {
        let p = parse("schema User { name: String }");
        let Item::SchemaDef(ref sd) = p.items[0] else { panic!("expected SchemaDef") };
        assert_eq!(sd.name, "User");
        assert_eq!(sd.fields.len(), 1);
    }

    #[test]
    fn parse_schema_uri_not_affected() {
        // `schema "uri"` form (v18.4/v32.4) must still work as a TypeExpr in fn signatures
        let p = parse(r#"fn f(x: schema "postgres:users") -> Unit { () }"#);
        assert_eq!(p.items.len(), 1);
        assert!(matches!(p.items[0], Item::FnDef(_)));
    }

    // ── ExpectStmt (v36.2.0) ─────────────────────────────────────────────────

    #[test]
    fn parse_expect_stmt_basic() {
        let src = r#"fn validate(rows: List<Row>) -> Bool {
    expect rows {
        not_empty
        all(|r| r.ok)
    }
    true
}"#;
        let p = parse(src);
        assert!(!p.items.is_empty(), "expect block should parse without error");
        let Item::FnDef(ref fd) = p.items[0] else { panic!("expected FnDef") };
        let stmts = &fd.body.stmts;
        assert!(!stmts.is_empty(), "fn body should have at least one stmt");
        assert!(
            matches!(stmts[0], Stmt::Expect(_)),
            "first stmt should be Stmt::Expect"
        );
    }

}