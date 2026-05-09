// src/fmt.rs — Favnir source formatter (Phase 2, v0.8.0)
//
// Entry point: `format_program(prog: &Program) -> String`
// Produces canonical Favnir source with 4-space indentation.

use crate::ast::*;

// ── public API ────────────────────────────────────────────────────────────────

pub fn format_program(prog: &Program) -> String {
    let mut f = Formatter::new();
    f.program(prog)
}

// ── Formatter ─────────────────────────────────────────────────────────────────

struct Formatter {
    indent: usize,
}

impl Formatter {
    fn new() -> Self {
        Formatter { indent: 0 }
    }

    fn pad(&self) -> String {
        "    ".repeat(self.indent)
    }

    // ── Program ───────────────────────────────────────────────────────────────

    fn program(&mut self, prog: &Program) -> String {
        let mut parts: Vec<String> = Vec::new();

        if let Some(ns) = &prog.namespace {
            parts.push(format!("namespace {}", ns));
        }
        for use_path in &prog.uses {
            parts.push(format!("use {}", use_path.join(".")));
        }

        for item in &prog.items {
            parts.push(self.item(item));
        }

        let mut out = parts.join("\n\n");
        if !out.ends_with('\n') {
            out.push('\n');
        }
        out
    }

    // ── Item ──────────────────────────────────────────────────────────────────

    fn item(&mut self, item: &Item) -> String {
        match item {
            Item::NamespaceDecl(ns, _) => format!("namespace {}", ns),
            Item::UseDecl(path, _)     => format!("use {}", path.join(".")),
            Item::EffectDef(ed)        => self.effect_def(ed),
            Item::TypeDef(td)          => self.type_def(td),
            Item::FnDef(fd)            => self.fn_def(fd),
            Item::TrfDef(td)           => self.trf_def(td),
            Item::AbstractTrfDef(td)   => self.abstract_trf_def(td),
            Item::FlwDef(fd)           => self.flw_def(fd),
            Item::AbstractFlwDef(fd)   => self.abstract_flw_def(fd),
            Item::FlwBindingDef(fd)    => self.flw_binding_def(fd),
            Item::InterfaceDecl(id)    => self.interface_decl(id),
            Item::InterfaceImplDecl(d) => self.interface_impl_decl(d),
            Item::CapDef(cd)           => self.cap_def(cd),
            Item::ImplDef(id)          => self.impl_def(id),
            Item::TestDef(td)          => self.test_def(td),
            Item::BenchDef(bd)         => self.bench_def(bd),
        }
    }

    fn effect_def(&mut self, ed: &EffectDef) -> String {
        let vis = fmt_visibility(ed.visibility.as_ref());
        format!("{}effect {}", vis, ed.name)
    }

    // ── TypeDef ───────────────────────────────────────────────────────────────

    fn type_def(&mut self, td: &TypeDef) -> String {
        let vis = fmt_visibility(td.visibility.as_ref());
        let params = fmt_type_params(&td.type_params);
        match &td.body {
            TypeBody::Record(fields) => {
                let field_strs: Vec<String> = fields
                    .iter()
                    .map(|f| format!("    {}: {}", f.name, self.type_expr(&f.ty)))
                    .collect();
                format!("{}type {}{} = {{\n{}\n}}", vis, td.name, params, field_strs.join("\n"))
            }
            TypeBody::Sum(variants) => {
                let var_strs: Vec<String> = variants
                    .iter()
                    .map(|v| format!("    | {}", self.variant(v)))
                    .collect();
                format!("{}type {}{} =\n{}", vis, td.name, params, var_strs.join("\n"))
            }
            TypeBody::Alias(target) => {
                format!("{}type {}{} = {}", vis, td.name, params, self.type_expr(target))
            }
        }
    }

    fn variant(&mut self, v: &Variant) -> String {
        match v {
            Variant::Unit(name, _) => name.clone(),
            Variant::Tuple(name, ty, _) => format!("{}({})", name, self.type_expr(ty)),
            Variant::Record(name, fields, _) => {
                let fs: Vec<String> = fields
                    .iter()
                    .map(|f| format!("{}: {}", f.name, self.type_expr(&f.ty)))
                    .collect();
                format!("{} {{ {} }}", name, fs.join("  "))
            }
        }
    }

    // ── FnDef ─────────────────────────────────────────────────────────────────

    fn fn_def(&mut self, fd: &FnDef) -> String {
        let vis = fmt_visibility(fd.visibility.as_ref());
        let params = fmt_type_params(&fd.type_params);
        let args = self.params(&fd.params);
        let ret = self.type_expr(&fd.return_ty);
        let effects = fmt_effects(&fd.effects);
        let body = self.block(&fd.body);
        format!("{}fn {}{}{}{}{} {}", vis, fd.name, params, args, ret_arrow(&ret), effects, body)
    }

    fn params(&mut self, params: &[Param]) -> String {
        let ps: Vec<String> = params
            .iter()
            .map(|p| format!("{}: {}", p.name, self.type_expr(&p.ty)))
            .collect();
        format!("({})", ps.join(", "))
    }

    // ── TrfDef ────────────────────────────────────────────────────────────────

    fn trf_def(&mut self, td: &TrfDef) -> String {
        let vis = fmt_visibility(td.visibility.as_ref());
        let type_params = fmt_type_params(&td.type_params);
        let input = self.type_expr(&td.input_ty);
        let output = self.type_expr(&td.output_ty);
        let effects = fmt_effects(&td.effects);
        let body = self.block(&td.body);
        // stage syntax: stage Name<T>: Input -> Output !Effects = |params| { body }
        // params are lambda params (just names, types come from input_ty)
        let lambda = if td.params.is_empty() {
            String::new()
        } else {
            let names: Vec<&str> = td.params.iter().map(|p| p.name.as_str()).collect();
            format!("|{}| ", names.join(", "))
        };
        format!(
            "{}stage {}{}: {} -> {}{} = {}{}",
            vis, td.name, type_params, input, output, effects, lambda, body
        )
    }

    fn abstract_trf_def(&mut self, td: &AbstractTrfDef) -> String {
        let vis = fmt_visibility(td.visibility.as_ref());
        let input = self.type_expr(&td.input_ty);
        let output = self.type_expr(&td.output_ty);
        let effects = fmt_effects(&td.effects);
        format!("{}abstract stage {}: {} -> {}{}", vis, td.name, input, output, effects)
    }

    // ── FlwDef ────────────────────────────────────────────────────────────────

    fn flw_def(&mut self, fd: &FlwDef) -> String {
        format!("seq {} = {}", fd.name, fd.steps.join(" |> "))
    }

    fn abstract_flw_def(&mut self, fd: &AbstractFlwDef) -> String {
        let vis = fmt_visibility(fd.visibility.as_ref());
        let params = fmt_type_params(&fd.type_params);
        let slots: Vec<String> = fd.slots.iter().map(|slot| {
            format!(
                "    {}: {} -> {}{}",
                slot.name,
                self.type_expr(&slot.input_ty),
                self.type_expr(&slot.output_ty),
                fmt_effects(&slot.effects),
            )
        }).collect();
        format!("{}abstract seq {}{} {{\n{}\n}}", vis, fd.name, params, slots.join("\n"))
    }

    fn flw_binding_def(&mut self, fd: &FlwBindingDef) -> String {
        let vis = fmt_visibility(fd.visibility.as_ref());
        let type_args = if fd.type_args.is_empty() {
            String::new()
        } else {
            format!("<{}>", fd.type_args.iter().map(|t| self.type_expr(t)).collect::<Vec<_>>().join(", "))
        };
        let bindings: Vec<String> = fd.bindings.iter()
            .map(|(slot, imp)| format!("    {} <- {}", slot, fmt_slot_impl(imp)))
            .collect();
        format!("{}seq {} = {}{} {{\n{}\n}}", vis, fd.name, fd.template, type_args, bindings.join("\n"))
    }

    fn interface_decl(&mut self, id: &InterfaceDecl) -> String {
        let vis = fmt_visibility(id.visibility.as_ref());
        let super_part = id
            .super_interface
            .as_ref()
            .map(|s| format!(" : {}", s))
            .unwrap_or_default();
        let methods: Vec<String> = id
            .methods
            .iter()
            .map(|m| format!("    {}: {}", m.name, self.type_expr(&m.ty)))
            .collect();
        format!(
            "{}interface {}{} {{\n{}\n}}",
            vis,
            id.name,
            super_part,
            methods.join("\n")
        )
    }

    fn interface_impl_decl(&mut self, id: &InterfaceImplDecl) -> String {
        let ifaces = id.interface_names.join(", ");
        if id.is_auto {
            format!("impl {} for {}", ifaces, id.type_name)
        } else {
            let methods: Vec<String> = id
                .methods
                .iter()
                .map(|(name, expr)| format!("    {} = {}", name, self.expr(expr)))
                .collect();
            format!(
                "impl {} for {} {{\n{}\n}}",
                ifaces,
                id.type_name,
                methods.join("\n")
            )
        }
    }

    // ── CapDef ────────────────────────────────────────────────────────────────

    fn cap_def(&mut self, cd: &CapDef) -> String {
        let vis = fmt_visibility(cd.visibility.as_ref());
        let params = fmt_type_params(&cd.type_params);
        let fields: Vec<String> = cd
            .fields
            .iter()
            .map(|f| format!("    {}: {}", f.name, self.type_expr(&f.ty)))
            .collect();
        format!("{}cap {}{} = {{\n{}\n}}", vis, cd.name, params, fields.join("\n"))
    }

    // ── ImplDef ───────────────────────────────────────────────────────────────

    fn impl_def(&mut self, id: &ImplDef) -> String {
        let type_args: Vec<String> = id.type_args.iter().map(|t| self.type_expr(t)).collect();
        let type_args_str = if type_args.is_empty() {
            String::new()
        } else {
            format!("<{}>", type_args.join(", "))
        };
        self.indent += 1;
        let methods: Vec<String> = id.methods.iter().map(|m| {
            let s = self.fn_def(m);
            format!("{}{}", self.pad(), s)
        }).collect();
        self.indent -= 1;
        format!("impl {}{} {{\n{}\n}}", id.cap_name, type_args_str, methods.join("\n\n"))
    }

    // ── TestDef ───────────────────────────────────────────────────────────────

    fn test_def(&mut self, td: &TestDef) -> String {
        let body = self.block(&td.body);
        format!("test {:?} {}", td.name, body)
    }

    fn bench_def(&mut self, bd: &BenchDef) -> String {
        let body = self.block(&bd.body);
        format!("bench {:?} {}", bd.description, body)
    }

    // ── Block ─────────────────────────────────────────────────────────────────

    fn block(&mut self, block: &Block) -> String {
        // Empty block with just a unit literal → `{}`
        if block.stmts.is_empty() {
            if let Expr::Lit(Lit::Unit, _) = block.expr.as_ref() {
                return "{}".to_string();
            }
        }

        self.indent += 1;
        let pad = self.pad();

        let mut lines: Vec<String> = Vec::new();
        for stmt in &block.stmts {
            lines.push(format!("{}{}", pad, self.stmt(stmt)));
        }
        // Final expression
        let final_expr = self.expr(&block.expr);
        lines.push(format!("{}{}", pad, final_expr));

        self.indent -= 1;
        let close_pad = self.pad();

        format!("{{\n{}\n{}}}", lines.join("\n"), close_pad)
    }

    // ── Stmt ──────────────────────────────────────────────────────────────────

    fn stmt(&mut self, stmt: &Stmt) -> String {
        match stmt {
            Stmt::Bind(b) => {
                let pat = self.pattern(&b.pattern);
                let ann = b
                    .annotated_ty
                    .as_ref()
                    .map(|ty| format!(": {}", self.type_expr(ty)))
                    .unwrap_or_default();
                let expr = self.expr(&b.expr);
                format!("bind {}{} <- {}", pat, ann, expr)
            }
            Stmt::Expr(e) => {
                format!("{};", self.expr(e))
            }
            Stmt::Chain(c) => {
                let expr = self.expr(&c.expr);
                format!("chain {} <- {}", c.name, expr)
            }
            Stmt::Yield(y) => {
                let expr = self.expr(&y.expr);
                format!("yield {};", expr)
            }
            Stmt::ForIn(f) => {
                let iter = self.expr(&f.iter);
                let body = self.block(&f.body);
                format!("for {} in {} {}", f.var, iter, body)
            }
        }
    }

    // ── Expr ──────────────────────────────────────────────────────────────────

    fn expr(&mut self, expr: &Expr) -> String {
        match expr {
            Expr::Lit(lit, _) => fmt_lit(lit),

            Expr::Ident(name, _) => name.clone(),

            Expr::Pipeline(steps, _) => {
                let parts: Vec<String> = steps.iter().map(|e| self.expr(e)).collect();
                parts.join(" |> ")
            }

            Expr::Apply(func, args, _) => {
                let f = self.expr(func);
                let as_: Vec<String> = args.iter().map(|a| self.expr(a)).collect();
                format!("{}({})", f, as_.join(", "))
            }

            Expr::FieldAccess(obj, field, _) => {
                format!("{}.{}", self.expr(obj), field)
            }

            Expr::Block(b) => self.block(b),

            Expr::Match(scrutinee, arms, _) => {
                let s = self.expr(scrutinee);
                self.indent += 1;
                let pad = self.pad();
                let arm_strs: Vec<String> = arms
                    .iter()
                    .map(|arm| {
                        let pat = self.pattern(&arm.pattern);
                        let guard = arm.guard.as_ref()
                            .map(|g| format!(" where {}", self.expr(g)))
                            .unwrap_or_default();
                        let body = self.expr(&arm.body);
                        format!("{}{}{} => {}", pad, pat, guard, body)
                    })
                    .collect();
                self.indent -= 1;
                let close_pad = self.pad();
                format!("match {} {{\n{}\n{}}}", s, arm_strs.join("\n"), close_pad)
            }

            Expr::AssertMatches(expr, pattern, _) => {
                format!("assert_matches({}, {})", self.expr(expr), self.pattern(pattern))
            }

            Expr::Collect(block, _) => {
                let b = self.block(block);
                format!("collect {}", b)
            }

            Expr::If(cond, then_block, else_block, _) => {
                let c = self.expr(cond);
                let t = self.block(then_block);
                match else_block {
                    Some(eb) => {
                        let e = self.block(eb);
                        format!("if {} {} else {}", c, t, e)
                    }
                    None => format!("if {} {}", c, t),
                }
            }

            Expr::Closure(params, body, _) => {
                let ps = params.join(", ");
                let b = self.expr(body);
                format!("|{}| {}", ps, b)
            }

            Expr::BinOp(op, lhs, rhs, _) => {
                let l = self.expr_paren_if_needed(lhs, op);
                let r = self.expr_paren_if_needed(rhs, op);
                format!("{} {} {}", l, fmt_binop(op), r)
            }

            Expr::RecordConstruct(name, fields, _) => {
                let fs: Vec<String> = fields
                    .iter()
                    .map(|(k, v)| format!("{}: {}", k, self.expr(v)))
                    .collect();
                format!("{} {{ {} }}", name, fs.join("  "))
            }

            Expr::FString(parts, _) => {
                let mut out = String::from("$\"");
                for part in parts {
                    match part {
                        FStringPart::Lit(s) => out.push_str(&fmt_fstring_lit(s)),
                        FStringPart::Expr(expr) => {
                            out.push('{');
                            out.push_str(&self.expr(expr));
                            out.push('}');
                        }
                    }
                }
                out.push('"');
                out
            }

            Expr::EmitExpr(inner, _) => {
                format!("emit {}", self.expr(inner))
            }
        }
    }

    /// Wrap in parens if needed for correct precedence inside a binop.
    fn expr_paren_if_needed(&mut self, expr: &Expr, _parent_op: &BinOp) -> String {
        match expr {
            // Binary operations always parenthesized to be safe
            Expr::BinOp(_, _, _, _) => format!("({})", self.expr(expr)),
            _ => self.expr(expr),
        }
    }

    // ── Pattern ───────────────────────────────────────────────────────────────

    fn pattern(&mut self, pat: &Pattern) -> String {
        match pat {
            Pattern::Wildcard(_) => "_".to_string(),
            Pattern::Lit(lit, _) => fmt_lit(lit),
            Pattern::Bind(name, _) => name.clone(),
            Pattern::Variant(name, inner, _) => {
                match inner {
                    None => name.clone(),
                    Some(p) => format!("{}({})", name, self.pattern(p)),
                }
            }
            Pattern::Record(fields, _) => {
                let fs: Vec<String> = fields.iter().map(|fp| {
                    match &fp.pattern {
                        None => fp.name.clone(),
                        Some(p) => format!("{}: {}", fp.name, self.pattern(p)),
                    }
                }).collect();
                format!("{{ {} }}", fs.join(", "))
            }
        }
    }

    // ── TypeExpr ──────────────────────────────────────────────────────────────

    fn type_expr(&self, ty: &TypeExpr) -> String {
        match ty {
            TypeExpr::Named(name, args, _) => {
                if args.is_empty() {
                    name.clone()
                } else {
                    let as_: Vec<String> = args.iter().map(|a| self.type_expr(a)).collect();
                    format!("{}<{}>", name, as_.join(", "))
                }
            }
            TypeExpr::Optional(inner, _) => format!("{}?", self.type_expr(inner)),
            TypeExpr::Fallible(inner, _)  => format!("{}!", self.type_expr(inner)),
            TypeExpr::Arrow(from, to, _) => {
                format!("{} -> {}", self.type_expr(from), self.type_expr(to))
            }
            TypeExpr::TrfFn { input, output, effects, .. } => {
                format!(
                    "{} -> {}{}",
                    self.type_expr(input),
                    self.type_expr(output),
                    fmt_effects(effects),
                )
            }
        }
    }
}

// ── free helpers ──────────────────────────────────────────────────────────────

fn fmt_visibility(vis: Option<&Visibility>) -> &'static str {
    match vis {
        Some(Visibility::Public)   => "public ",
        Some(Visibility::Internal) => "internal ",
        Some(Visibility::Private)  => "private ",
        None                       => "",
    }
}

fn fmt_type_params(params: &[String]) -> String {
    if params.is_empty() {
        String::new()
    } else {
        format!("<{}>", params.join(", "))
    }
}

fn fmt_fstring_lit(s: &str) -> String {
    let mut out = String::new();
    for ch in s.chars() {
        match ch {
            '{' => out.push_str("\\{"),
            '\n' => out.push_str("\\n"),
            '\t' => out.push_str("\\t"),
            '\r' => out.push_str("\\r"),
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            _ => out.push(ch),
        }
    }
    out
}

fn fmt_effects(effects: &[Effect]) -> String {
    let strs: Vec<String> = effects.iter().filter_map(fmt_effect).collect();
    if strs.is_empty() {
        String::new()
    } else {
        format!(" {}", strs.join(" "))
    }
}

fn fmt_effect(eff: &Effect) -> Option<String> {
    match eff {
        Effect::Pure         => None,
        Effect::Io           => Some("!Io".to_string()),
        Effect::Db           => Some("!Db".to_string()),
        Effect::Network      => Some("!Network".to_string()),
        Effect::File         => Some("!File".to_string()),
        Effect::Trace        => Some("!Trace".to_string()),
        Effect::Unknown(name) => Some(format!("!{}", name)),
        Effect::Emit(t)      => Some(format!("!Emit<{}>", t)),
        Effect::EmitUnion(ts) => Some(format!("!Emit<{}>", ts.join("|"))),
    }
}

fn fmt_slot_impl(imp: &SlotImpl) -> &str {
    match imp {
        SlotImpl::Global(name) | SlotImpl::Local(name) => name.as_str(),
    }
}

fn ret_arrow(ret: &str) -> String {
    if ret == "Unit" {
        " -> Unit".to_string()
    } else {
        format!(" -> {}", ret)
    }
}

fn fmt_lit(lit: &Lit) -> String {
    match lit {
        Lit::Int(n)   => n.to_string(),
        Lit::Float(f) => {
            // Preserve decimal point
            let s = format!("{}", f);
            if s.contains('.') { s } else { format!("{}.0", s) }
        }
        Lit::Str(s)   => format!("{:?}", s),
        Lit::Bool(b)  => b.to_string(),
        Lit::Unit     => "()".to_string(),
    }
}

fn fmt_binop(op: &BinOp) -> &'static str {
    match op {
        BinOp::Add   => "+",
        BinOp::Sub   => "-",
        BinOp::Mul   => "*",
        BinOp::Div   => "/",
        BinOp::Eq    => "==",
        BinOp::NotEq => "!=",
        BinOp::Lt    => "<",
        BinOp::Gt    => ">",
        BinOp::LtEq          => "<=",
        BinOp::GtEq          => ">=",
        BinOp::NullCoalesce  => "??",
    }
}

// ── tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::format_program;
    use crate::frontend::parser::Parser;

    fn assert_idempotent(source: &str) {
        let prog1 = Parser::parse_str(source, "test.fav").expect("parse 1");
        let formatted = format_program(&prog1);
        let prog2 = Parser::parse_str(&formatted, "test.fav").expect("parse 2 (formatted)");
        let formatted2 = format_program(&prog2);
        assert_eq!(
            formatted, formatted2,
            "formatter is not idempotent.\nFirst pass:\n{}\nSecond pass:\n{}",
            formatted, formatted2
        );
    }

    #[test]
    fn fmt_simple_fn_is_idempotent() {
        assert_idempotent(r#"
public fn main() -> Unit !Io {
    IO.println("Hello, Favnir!")
}
"#);
    }

    #[test]
    fn fmt_sum_type_is_idempotent() {
        assert_idempotent(r#"
type Direction =
    | North
    | South
    | East
    | West

fn direction_name(d: Direction) -> String {
    match d {
        North => "North"
        South => "South"
        East => "East"
        West => "West"
    }
}
"#);
    }

    #[test]
    fn fmt_trf_flw_is_idempotent() {
        assert_idempotent(r#"
stage Double: Int -> Int = |x| {
    x + x
}

seq Quadruple = Double |> Double

public fn main() -> Int {
    2 |> Quadruple
}
"#);
    }

    #[test]
    fn fmt_if_else_is_idempotent() {
        assert_idempotent(r#"
fn abs(n: Int) -> Int {
    if n < 0 {
        0 - n
    } else {
        n
    }
}
"#);
    }

    #[test]
    fn fmt_closure_and_bind_is_idempotent() {
        assert_idempotent(r#"
public fn main() -> Int {
    bind f <- |x| x + 1
    f(10)
}
"#);
    }

    #[test]
    fn fmt_record_type_is_idempotent() {
        assert_idempotent(r#"
type User = {
    name: String
    age: Int
}
"#);
    }

    #[test]
    fn fmt_test_def_is_idempotent() {
        assert_idempotent(r#"
fn add(a: Int, b: Int) -> Int {
    a + b
}

test "add works" {
    assert_eq(add(1, 2), 3)
}
"#);
    }
}
