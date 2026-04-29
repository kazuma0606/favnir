// Favnir Parser
// Tasks: 3-1..3-23

use crate::lexer::{Lexer, LexError, Span, Token, TokenKind};
use crate::ast::*;

// ── ParseError (3-2) ──────────────────────────────────────────────────────────

#[derive(Debug)]
pub struct ParseError {
    pub message: String,
    pub span: Span,
}

impl ParseError {
    pub fn new(message: impl Into<String>, span: Span) -> Self {
        ParseError { message: message.into(), span }
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

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Parser { tokens, pos: 0 }
    }

    /// Parse source text directly.
    pub fn parse_str(source: &str, file: &str) -> Result<Program, ParseError> {
        let tokens = Lexer::new(source, file).tokenize()?;
        Parser::new(tokens).parse_program()
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

    fn expect_ident(&mut self) -> Result<(String, Span), ParseError> {
        let span = self.peek_span().clone();
        match self.peek().clone() {
            TokenKind::Ident(name) => {
                self.advance();
                Ok((name, span))
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
        let mut uses = Vec::new();
        while self.peek() == &TokenKind::Use {
            uses.push(self.parse_use_decl()?);
        }

        // 3. top-level definitions
        let mut items = Vec::new();
        while !self.at_end() {
            items.push(self.parse_item()?);
        }
        Ok(Program { namespace, uses, items })
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

    fn parse_use_decl(&mut self) -> Result<Vec<String>, ParseError> {
        self.expect(&TokenKind::Use)?;
        self.parse_module_path()
    }

    // ── item ──────────────────────────────────────────────────────────────────

    fn parse_item(&mut self) -> Result<Item, ParseError> {
        let vis = self.parse_visibility();

        match self.peek().clone() {
            TokenKind::Type   => Ok(Item::TypeDef(self.parse_type_def(vis)?)),
            TokenKind::Fn     => Ok(Item::FnDef(self.parse_fn_def(vis)?)),
            TokenKind::Trf    => Ok(Item::TrfDef(self.parse_trf_def(vis)?)),
            TokenKind::Cap    => Ok(Item::CapDef(self.parse_cap_def(vis)?)),
            TokenKind::Impl   => {
                if vis.is_some() {
                    return Err(ParseError::new(
                        "visibility modifier on `impl` is not allowed",
                        self.peek_span().clone(),
                    ));
                }
                Ok(Item::ImplDef(self.parse_impl_def()?))
            }
            TokenKind::Flw    => {
                if vis.is_some() {
                    return Err(ParseError::new(
                        "visibility on flw is not supported",
                        self.peek_span().clone(),
                    ));
                }
                Ok(Item::FlwDef(self.parse_flw_def()?))
            }
            TokenKind::Namespace => Err(ParseError::new(
                "`namespace` must appear before any definitions",
                self.peek_span().clone(),
            )),
            TokenKind::Use => Err(ParseError::new(
                "`use` must appear before any definitions",
                self.peek_span().clone(),
            )),
            other => Err(ParseError::new(
                format!("expected item (type/fn/trf/flw/cap/impl), got {:?}", other),
                self.peek_span().clone(),
            )),
        }
    }

    // ── cap_def (v0.4.0) ─────────────────────────────────────────────────────

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
            fields.push(CapField { name: fname, ty: fty, span: self.span_from(&fs) });
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
            methods.push(self.parse_fn_def(vis)?);
        }
        self.expect(&TokenKind::RBrace)?;
        Ok(ImplDef {
            cap_name,
            type_args,
            methods,
            span: self.span_from(&start),
        })
    }

    fn parse_visibility(&mut self) -> Option<Visibility> {
        match self.peek() {
            TokenKind::Public   => { self.advance(); Some(Visibility::Public) }
            TokenKind::Internal => { self.advance(); Some(Visibility::Internal) }
            TokenKind::Private  => { self.advance(); Some(Visibility::Private) }
            _ => None,
        }
    }

    // ── type_def (3-3, 3-4) ──────────────────────────────────────────────────

    fn parse_type_def(&mut self, visibility: Option<Visibility>) -> Result<TypeDef, ParseError> {
        let start = self.peek_span().clone();
        self.expect(&TokenKind::Type)?;
        let (name, _) = self.expect_ident()?;
        let type_params = self.parse_type_params()?;
        self.expect(&TokenKind::Eq)?;

        let body = if self.peek() == &TokenKind::LBrace {
            // record body
            TypeBody::Record(self.parse_record_fields()?)
        } else if self.peek() == &TokenKind::Pipe {
            // sum body
            TypeBody::Sum(self.parse_sum_variants()?)
        } else {
            return Err(ParseError::new(
                "expected '{' (record) or '|' (sum) in type definition",
                self.peek_span().clone(),
            ));
        };

        Ok(TypeDef { visibility, name, type_params, body, span: self.span_from(&start) })
    }

    /// Parse optional type parameters `<T, U, V>`.
    /// Returns an empty Vec if no `<` is found.
    fn parse_type_params(&mut self) -> Result<Vec<String>, ParseError> {
        if self.peek() != &TokenKind::LAngle {
            return Ok(vec![]);
        }
        self.advance(); // consume `<`
        let (first, _) = self.expect_ident()?;
        let mut params = vec![first];
        while self.peek() == &TokenKind::Comma {
            self.advance();
            let (name, _) = self.expect_ident()?;
            params.push(name);
        }
        self.expect(&TokenKind::RAngle)?;
        Ok(params)
    }

    fn parse_record_fields(&mut self) -> Result<Vec<Field>, ParseError> {
        self.expect(&TokenKind::LBrace)?;
        let mut fields = Vec::new();
        while self.peek() != &TokenKind::RBrace {
            fields.push(self.parse_field()?);
        }
        self.expect(&TokenKind::RBrace)?;
        Ok(fields)
    }

    fn parse_field(&mut self) -> Result<Field, ParseError> {
        let start = self.peek_span().clone();
        let (name, _) = self.expect_ident()?;
        self.expect(&TokenKind::Colon)?;
        let ty = self.parse_type_expr()?;
        Ok(Field { name, ty, span: self.span_from(&start) })
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
            // Tuple variant: ok(User)
            self.advance();
            let ty = self.parse_type_expr()?;
            self.expect(&TokenKind::RParen)?;
            Ok(Variant::Tuple(name, ty, self.span_from(&start)))
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
                    let next_is_effect = match self.peek2() {
                        Some(TokenKind::Pure) | Some(TokenKind::Io) => true,
                        Some(TokenKind::Ident(n)) => matches!(
                            n.as_str(), "Db" | "Network" | "Emit"
                        ),
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

        // arrow ->  (only when allowed)
        if allow_arrow && self.peek() == &TokenKind::Arrow {
            self.advance();
            let rhs = self.parse_type_expr_inner(true)?;
            let span = self.span_from(&start);
            ty = TypeExpr::Arrow(Box::new(ty), Box::new(rhs), span);
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
        let name = match self.peek().clone() {
            TokenKind::Ident(n) => { self.advance(); n }
            // Allow effect keywords as type names (e.g., "Io" as a type)
            TokenKind::Pure => { self.advance(); "Pure".to_string() }
            TokenKind::Io   => { self.advance(); "Io".to_string() }
            other => {
                return Err(ParseError::new(
                    format!("expected type name, got {:?}", other),
                    self.peek_span().clone(),
                ))
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

    fn parse_fn_def(&mut self, visibility: Option<Visibility>) -> Result<FnDef, ParseError> {
        let start = self.peek_span().clone();
        self.expect(&TokenKind::Fn)?;
        let (name, _) = self.expect_ident()?;
        let type_params = self.parse_type_params()?;

        self.expect(&TokenKind::LParen)?;
        let params = self.parse_params()?;
        self.expect(&TokenKind::RParen)?;

        self.expect(&TokenKind::Arrow)?;
        let return_ty = self.parse_type_expr()?;

        // optional effect annotation: !Io !Db ...
        let effects = self.parse_effect_ann()?;

        let body = self.parse_block()?;

        Ok(FnDef {
            visibility,
            name,
            type_params,
            params,
            return_ty,
            effects,
            body,
            span: self.span_from(&start),
        })
    }

    fn parse_params(&mut self) -> Result<Vec<Param>, ParseError> {
        let mut params = Vec::new();
        while self.peek() != &TokenKind::RParen {
            let start = self.peek_span().clone();
            let (name, _) = self.expect_ident()?;
            self.expect(&TokenKind::Colon)?;
            let ty = self.parse_type_expr()?;
            params.push(Param { name, ty, span: self.span_from(&start) });
            if self.peek() == &TokenKind::Comma {
                self.advance();
            }
        }
        Ok(params)
    }

    // effect annotation: ("!" effect_term)+   (1-8, 1-9)
    // effect_term = Pure | Io | Db | Network | Emit<IDENT>
    fn parse_effect_ann(&mut self) -> Result<Vec<Effect>, ParseError> {
        let mut effects = Vec::new();
        while self.peek() == &TokenKind::Bang {
            self.advance(); // consume !
            let effect = match self.peek().clone() {
                TokenKind::Pure => { self.advance(); Effect::Pure }
                TokenKind::Io   => { self.advance(); Effect::Io }
                TokenKind::Ident(ref name) => {
                    match name.as_str() {
                        "Db" => { self.advance(); Effect::Db }
                        "Network" => { self.advance(); Effect::Network }
                        "Trace" => { self.advance(); Effect::Trace }
                        "Emit" => {
                            self.advance();
                            self.expect(&TokenKind::LAngle)?;
                            let (event_name, _) = self.expect_ident()?;
                            self.expect(&TokenKind::RAngle)?;
                            Effect::Emit(event_name)
                        }
                        other => return Err(ParseError::new(
                            format!("expected effect name (Pure|Io|Db|Network|Emit<T>), got `{}`", other),
                            self.peek_span().clone(),
                        )),
                    }
                }
                other => return Err(ParseError::new(
                    format!("expected effect name after `!`, got {:?}", other),
                    self.peek_span().clone(),
                )),
            };
            effects.push(effect);
        }
        Ok(effects)
    }

    // ── trf_def (3-6) ────────────────────────────────────────────────────────

    fn parse_trf_def(&mut self, visibility: Option<Visibility>) -> Result<TrfDef, ParseError> {
        let start = self.peek_span().clone();
        self.expect(&TokenKind::Trf)?;
        let (name, _) = self.expect_ident()?;
        let type_params = self.parse_type_params()?;
        self.expect(&TokenKind::Colon)?;
        let input_ty = self.parse_type_expr_no_arrow()?;
        self.expect(&TokenKind::Arrow)?;
        let output_ty = self.parse_type_expr_no_arrow()?;

        // optional effect annotation: !Db !Emit<UserCreated> ...
        let effects = self.parse_effect_ann()?;

        self.expect(&TokenKind::Eq)?;
        // closure params: |param, ...| or ||
        self.expect(&TokenKind::Pipe)?;
        let params = self.parse_closure_params_typed()?;
        self.expect(&TokenKind::Pipe)?;

        let body = self.parse_block()?;

        Ok(TrfDef {
            visibility,
            name,
            type_params,
            input_ty,
            output_ty,
            effects,
            params,
            body,
            span: self.span_from(&start),
        })
    }

    /// Parse typed closure params: `param: Type, ...` (used inside trf)
    fn parse_closure_params_typed(&mut self) -> Result<Vec<Param>, ParseError> {
        let mut params = Vec::new();
        while self.peek() != &TokenKind::Pipe {
            let start = self.peek_span().clone();
            let (name, _) = self.expect_ident()?;
            // Type annotation is optional for closure params
            let ty = if self.peek() == &TokenKind::Colon {
                self.advance();
                self.parse_type_expr()?
            } else {
                TypeExpr::Named("_infer".to_string(), vec![], start.clone())
            };
            params.push(Param { name, ty, span: self.span_from(&start) });
            if self.peek() == &TokenKind::Comma {
                self.advance();
            }
        }
        Ok(params)
    }

    // ── flw_def (3-7) ────────────────────────────────────────────────────────

    fn parse_flw_def(&mut self) -> Result<FlwDef, ParseError> {
        let start = self.peek_span().clone();
        self.expect(&TokenKind::Flw)?;
        let (name, _) = self.expect_ident()?;
        self.expect(&TokenKind::Eq)?;

        let (first, _) = self.expect_ident()?;
        let mut steps = vec![first];
        while self.peek() == &TokenKind::PipeGt {
            self.advance();
            let (step, _) = self.expect_ident()?;
            steps.push(step);
        }

        Ok(FlwDef { name, steps, span: self.span_from(&start) })
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

            // expression — could be a stmt (ends with `;`) or the final expr
            let expr = self.parse_expr()?;

            if self.peek() == &TokenKind::Semicolon {
                self.advance();
                stmts.push(Stmt::Expr(expr));
            } else {
                // This is the final expression of the block
                self.expect(&TokenKind::RBrace)?;
                let span = self.span_from(&start);
                return Ok(Block { stmts, expr: Box::new(expr), span });
            }
        }
    }

    // ── bind_stmt (3-8..3-11) ────────────────────────────────────────────────

    fn parse_bind_stmt(&mut self) -> Result<BindStmt, ParseError> {
        let start = self.peek_span().clone();
        self.expect(&TokenKind::Bind)?;
        let pattern = self.parse_pattern()?;
        self.expect(&TokenKind::LArrow)?;
        let expr = self.parse_expr()?;
        Ok(BindStmt { pattern, expr, span: self.span_from(&start) })
    }

    fn parse_chain_stmt(&mut self) -> Result<ChainStmt, ParseError> {
        let start = self.peek_span().clone();
        self.expect(&TokenKind::Chain)?;
        let (name, _) = self.expect_ident()?;
        self.expect(&TokenKind::LArrow)?;
        let expr = self.parse_expr()?;
        Ok(ChainStmt { name, expr, span: self.span_from(&start) })
    }

    fn parse_yield_stmt(&mut self) -> Result<YieldStmt, ParseError> {
        let start = self.peek_span().clone();
        self.expect(&TokenKind::Yield)?;
        let expr = self.parse_expr()?;
        self.expect(&TokenKind::Semicolon)?;
        Ok(YieldStmt { expr, span: self.span_from(&start) })
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
            TokenKind::Int(n)    => { let n = n; self.advance(); Ok(Pattern::Lit(Lit::Int(n), start)) }
            TokenKind::Float(f)  => { let f = f; self.advance(); Ok(Pattern::Lit(Lit::Float(f), start)) }
            TokenKind::Str(s)    => { let s = s; self.advance(); Ok(Pattern::Lit(Lit::Str(s), start)) }
            TokenKind::Bool(b)   => { let b = b; self.advance(); Ok(Pattern::Lit(Lit::Bool(b), start)) }

            // unit: ()
            TokenKind::LParen => {
                self.advance();
                self.expect(&TokenKind::RParen)?;
                Ok(Pattern::Lit(Lit::Unit, self.span_from(&start)))
            }

            // identifier: could be Bind or Variant(...)
            TokenKind::Ident(name) => {
                let name = name;
                self.advance();
                if self.peek() == &TokenKind::LParen {
                    // tuple variant with payload: ok(pat)
                    self.advance();
                    let inner = self.parse_pattern()?;
                    self.expect(&TokenKind::RParen)?;
                    Ok(Pattern::Variant(name, Some(Box::new(inner)), self.span_from(&start)))
                } else if self.peek() == &TokenKind::LBrace {
                    // record variant: Authenticated { user } or Authenticated { user: pat }
                    // Represented as Variant(name, Some(Record(fields)))
                    let fields = self.parse_record_field_patterns()?;
                    let inner = Pattern::Record(fields, self.span_from(&start));
                    Ok(Pattern::Variant(name, Some(Box::new(inner)), self.span_from(&start)))
                } else if name.chars().next().map(|c| c.is_uppercase()).unwrap_or(false) {
                    // uppercase with no payload → unit variant (e.g., Guest)
                    Ok(Pattern::Variant(name, None, self.span_from(&start)))
                } else {
                    // lowercase → bind
                    Ok(Pattern::Bind(name, self.span_from(&start)))
                }
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
        let mut lhs = self.parse_comparison()?;

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
                    lhs = Expr::Match(
                        Box::new(scrutinee),
                        arms,
                        self.span_from(&match_start),
                    );
                    // After a `|> match`, no more pipe stages are allowed
                    return Ok(lhs);
                }
                parts.push(self.parse_comparison()?);
            }
            lhs = Expr::Pipeline(parts, self.span_from(&start));
        }

        Ok(lhs)
    }

    fn parse_comparison(&mut self) -> Result<Expr, ParseError> {
        let start = self.peek_span().clone();
        let mut lhs = self.parse_additive()?;

        loop {
            let op = match self.peek() {
                TokenKind::EqEq   => BinOp::Eq,
                TokenKind::BangEq => BinOp::NotEq,
                TokenKind::LAngle => BinOp::Lt,
                TokenKind::RAngle => BinOp::Gt,
                TokenKind::LtEq   => BinOp::LtEq,
                TokenKind::GtEq   => BinOp::GtEq,
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
                TokenKind::Plus  => BinOp::Add,
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
                TokenKind::Star  => BinOp::Mul,
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

            // bool literal (3-12)
            TokenKind::Bool(b) => {
                self.advance();
                Ok(Expr::Lit(Lit::Bool(b), start))
            }

            // unit literal () (3-12)
            TokenKind::LParen => {
                self.advance();
                if self.peek() == &TokenKind::RParen {
                    self.advance();
                    Ok(Expr::Lit(Lit::Unit, self.span_from(&start)))
                } else {
                    let inner = self.parse_expr()?;
                    self.expect(&TokenKind::RParen)?;
                    Ok(inner)
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
                if name.chars().next().map(|c| c.is_uppercase()).unwrap_or(false)
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

            // effect keywords used as namespaces (Pure, Io, Emit → identifiers)
            TokenKind::Pure => { self.advance(); Ok(Expr::Ident("Pure".into(), start)) }
            TokenKind::Io   => { self.advance(); Ok(Expr::Ident("Io".into(), start)) }

            // closure: |x, y| expr  (3-16)
            TokenKind::Pipe => {
                self.advance();
                let params = self.parse_closure_params()?;
                self.expect(&TokenKind::Pipe)?;
                let body = self.parse_expr()?;
                Ok(Expr::Closure(params, Box::new(body), self.span_from(&start)))
            }

            // empty closure: || expr
            TokenKind::PipeGt => {
                // `|>` at start of primary is ambiguous; treat as parse error
                Err(ParseError::new("unexpected '|>'", start))
            }

            // block (3-17)
            TokenKind::LBrace => {
                Ok(Expr::Block(Box::new(self.parse_block()?)))
            }

            // match (3-18, 3-19)
            TokenKind::Match => {
                self.parse_match_expr()
            }

            // if (3-20)
            TokenKind::If => {
                self.parse_if_expr()
            }

            other => Err(ParseError::new(
                format!("expected expression, got {:?}", other),
                start,
            )),
        }
    }

    /// `{ name, email }` or `{ name: pat }` — shared by bare record and record-variant patterns.
    fn parse_record_field_patterns(&mut self) -> Result<Vec<FieldPattern>, ParseError> {
        self.expect(&TokenKind::LBrace)?;
        let mut fields = Vec::new();
        while self.peek() != &TokenKind::RBrace {
            let fs = self.peek_span().clone();
            let (name, _) = self.expect_ident()?;
            let pattern = if self.peek() == &TokenKind::Colon {
                self.advance();
                Some(self.parse_pattern()?)
            } else {
                None
            };
            fields.push(FieldPattern { name, pattern, span: self.span_from(&fs) });
            if self.peek() == &TokenKind::Comma {
                self.advance();
            }
        }
        self.expect(&TokenKind::RBrace)?;
        Ok(fields)
    }

    // closure params: untyped  |x, y|  (3-16)
    fn parse_closure_params(&mut self) -> Result<Vec<String>, ParseError> {
        let mut params = Vec::new();
        while self.peek() != &TokenKind::Pipe {
            let (name, _) = self.expect_ident()?;
            params.push(name);
            if self.peek() == &TokenKind::Comma {
                self.advance();
            }
        }
        Ok(params)
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

        Ok(Expr::Match(Box::new(scrutinee), arms, self.span_from(&start)))
    }

    fn parse_match_arm(&mut self) -> Result<MatchArm, ParseError> {
        let start = self.peek_span().clone();
        let pattern = self.parse_pattern()?;
        // optional `where guard_expr` (v0.5.0)
        let guard = if self.peek() == &TokenKind::Where {
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
        Ok(MatchArm { pattern, guard, body, span: self.span_from(&start) })
    }

    // ── if (3-20) ────────────────────────────────────────────────────────────

    fn parse_if_expr(&mut self) -> Result<Expr, ParseError> {
        let start = self.peek_span().clone();
        self.expect(&TokenKind::If)?;
        let cond = self.parse_expr()?;
        let then_block = self.parse_block()?;
        let else_block = if self.peek() == &TokenKind::Else {
            self.advance();
            Some(Box::new(self.parse_block()?))
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
    #[test]
    fn test_parse_public_fn() {
        let p = parse("public fn main() -> Unit !Io { () }");
        if let Item::FnDef(f) = &p.items[0] {
            assert_eq!(f.visibility, Some(Visibility::Public));
            assert!(f.effects.contains(&Effect::Io));
        }
    }

    // trf_def (3-6)
    #[test]
    fn test_parse_trf_def() {
        let p = parse("trf ParseCsv: String -> List<Row> = |text| { text }");
        assert!(matches!(p.items[0], Item::TrfDef(_)));
        if let Item::TrfDef(t) = &p.items[0] {
            assert_eq!(t.name, "ParseCsv");
            assert!(t.effects.is_empty());
        }
    }

    // trf_def with effect (3-6, 3-22)
    #[test]
    fn test_parse_trf_with_effect() {
        let p = parse("trf Print: String -> Unit !Io = |s| { () }");
        if let Item::TrfDef(t) = &p.items[0] {
            assert!(t.effects.contains(&Effect::Io));
        }
    }

    // flw_def (3-7)
    #[test]
    fn test_parse_flw_def() {
        let p = parse("flw Import = ParseCsv |> ValidateUser |> SaveUsers");
        if let Item::FlwDef(f) = &p.items[0] {
            assert_eq!(f.name, "Import");
            assert_eq!(f.steps, vec!["ParseCsv", "ValidateUser", "SaveUsers"]);
        }
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

    // type expr: T? and T! (3-21)
    #[test]
    fn test_parse_type_optional_fallible() {
        let p = parse("fn f() -> String? { () }");
        if let Item::FnDef(f) = &p.items[0] {
            assert!(matches!(f.return_ty, TypeExpr::Optional(_, _)));
        }
        let p2 = parse("fn g() -> String! { () }");
        if let Item::FnDef(f) = &p2.items[0] {
            assert!(matches!(f.return_ty, TypeExpr::Fallible(_, _)));
        }
    }

    // effect annotation (3-22)
    #[test]
    fn test_parse_effect_annotation() {
        let p = parse("fn f() -> Unit !Io { () }");
        if let Item::FnDef(f) = &p.items[0] {
            assert!(f.effects.contains(&Effect::Io));
        }
    }

    // field access
    #[test]
    fn test_parse_field_access() {
        let p = parse("fn f() -> String { user.name }");
        if let Item::FnDef(f) = &p.items[0] {
            assert!(matches!(*f.body.expr, Expr::FieldAccess(_, _, _)));
        }
    }

    // error: bad item
    #[test]
    fn test_parse_error_bad_item() {
        let msg = parse_err("bind x <- 1");
        assert!(msg.contains("expected item"));
    }

    // ── v0.2.0 parser tests (1-13) ────────────────────────────────────────────

    // 1-8, 1-9: multiple effects including Emit<T>
    #[test]
    fn test_parse_multi_effect() {
        let p = parse("trf T: Int -> Int !Db !Emit<UserCreated> = |n| { n }");
        if let Item::TrfDef(t) = &p.items[0] {
            assert!(t.effects.contains(&Effect::Db));
            assert!(t.effects.contains(&Effect::Emit("UserCreated".into())));
            assert_eq!(t.effects.len(), 2);
        } else {
            panic!("expected TrfDef");
        }
    }

    // 1-8: fn with multiple effects
    #[test]
    fn test_parse_fn_multi_effect() {
        let p = parse("fn f() -> Unit !Io !Db { () }");
        if let Item::FnDef(f) = &p.items[0] {
            assert!(f.effects.contains(&Effect::Io));
            assert!(f.effects.contains(&Effect::Db));
            assert_eq!(f.effects.len(), 2);
        } else {
            panic!("expected FnDef");
        }
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

    // 1-11: emit expression
    #[test]
    fn test_parse_emit_expr() {
        let p = parse(r#"fn f() -> Unit !Emit<E> { emit "hello" }"#);
        if let Item::FnDef(f) = &p.items[0] {
            assert!(matches!(*f.body.expr, Expr::EmitExpr(_, _)));
        } else {
            panic!("expected FnDef");
        }
    }

    // 1-12: emit in block stmt position
    #[test]
    fn test_parse_emit_in_block() {
        let p = parse(r#"fn f() -> Unit !Emit<E> { emit "ev"; () }"#);
        if let Item::FnDef(f) = &p.items[0] {
            assert_eq!(f.body.stmts.len(), 1);
            assert!(matches!(f.body.stmts[0], Stmt::Expr(Expr::EmitExpr(_, _))));
        } else {
            panic!("expected FnDef");
        }
    }

    // 1-9: Emit<T> as only effect
    #[test]
    fn test_parse_emit_effect_only() {
        let p = parse("fn f() -> Unit !Emit<OrderPlaced> { () }");
        if let Item::FnDef(f) = &p.items[0] {
            assert!(f.effects.contains(&Effect::Emit("OrderPlaced".into())));
        } else {
            panic!("expected FnDef");
        }
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
        let p = parse("namespace service.main\nuse data.users.create\nuse data.users.User\nfn f() -> Unit { () }");
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
        let Item::FnDef(fd) = &p.items[0] else { panic!("expected FnDef") };
        assert_eq!(fd.name, "identity");
        assert_eq!(fd.type_params, vec!["T"]);
        assert_eq!(fd.params[0].name, "x");
    }

    #[test]
    fn test_parse_generic_type() {
        let p = parse("type Pair<A, B> = { first: A second: B }");
        let Item::TypeDef(td) = &p.items[0] else { panic!("expected TypeDef") };
        assert_eq!(td.name, "Pair");
        assert_eq!(td.type_params, vec!["A", "B"]);
    }

    #[test]
    fn test_parse_generic_trf() {
        let p = parse("trf MapOpt<T, U>: Option<T> -> Option<U> = || { x }");
        let Item::TrfDef(td) = &p.items[0] else { panic!("expected TrfDef") };
        assert_eq!(td.type_params, vec!["T", "U"]);
    }

    #[test]
    fn test_parse_cap_def() {
        let p = parse("cap Eq<T> = { equals: T -> T -> Bool }");
        let Item::CapDef(cd) = &p.items[0] else { panic!("expected CapDef") };
        assert_eq!(cd.name, "Eq");
        assert_eq!(cd.type_params, vec!["T"]);
        assert_eq!(cd.fields.len(), 1);
        assert_eq!(cd.fields[0].name, "equals");
    }

    #[test]
    fn test_parse_impl_def() {
        let src = "cap Eq<T> = { equals: T -> T -> Bool }\nimpl Eq<Int> { fn equals(a: Int, b: Int) -> Bool { a == b } }";
        let p = parse(src);
        let Item::ImplDef(id) = &p.items[1] else { panic!("expected ImplDef") };
        assert_eq!(id.cap_name, "Eq");
        assert_eq!(id.type_args.len(), 1);
        assert_eq!(id.methods.len(), 1);
        assert_eq!(id.methods[0].name, "equals");
    }

    // ── v0.5.0 parser tests ────────────────────────────────────────────────────

    // task 2-10: chain stmt parses correctly
    #[test]
    fn test_parse_chain_stmt() {
        let p = parse("fn f() -> Result<Int, String> { chain n <- ok(42) ok(n) }");
        let Item::FnDef(f) = &p.items[0] else { panic!("expected FnDef") };
        assert_eq!(f.body.stmts.len(), 1);
        assert!(matches!(&f.body.stmts[0], Stmt::Chain(c) if c.name == "n"));
    }

    // task 2-11: yield stmt parses correctly
    #[test]
    fn test_parse_yield_stmt() {
        let p = parse("fn f() -> Unit { collect { yield 1; yield 2; () } }");
        let Item::FnDef(f) = &p.items[0] else { panic!("expected FnDef") };
        assert!(matches!(*f.body.expr, Expr::Collect(_, _)));
        if let Expr::Collect(block, _) = f.body.expr.as_ref() {
            assert_eq!(block.stmts.len(), 2);
            assert!(matches!(&block.stmts[0], Stmt::Yield(_)));
            assert!(matches!(&block.stmts[1], Stmt::Yield(_)));
        }
    }

    // task 2-12: collect expr parses correctly
    #[test]
    fn test_parse_collect_expr() {
        let p = parse("fn f() -> Unit { collect { yield 1; () } }");
        let Item::FnDef(f) = &p.items[0] else { panic!("expected FnDef") };
        assert!(matches!(*f.body.expr, Expr::Collect(_, _)));
    }

    // task 2-13: match guard parses correctly
    #[test]
    fn test_parse_match_guard() {
        let p = parse("fn f(x: Int) -> Int { match x { n where n > 0 => n _ => 0 } }");
        let Item::FnDef(f) = &p.items[0] else { panic!("expected FnDef") };
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
        let Item::FnDef(f) = &p.items[0] else { panic!("expected FnDef") };
        // After desugar, body.expr is a Match, not a Pipeline
        assert!(matches!(*f.body.expr, Expr::Match(_, _, _)), "expected Match after pipe desugar, got {:?}", f.body.expr);
    }
}
