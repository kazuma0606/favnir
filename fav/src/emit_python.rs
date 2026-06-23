//! Fav AST → Python ソースコード生成 (v11.3.0)
//!
//! `fav transpile --target python <file.fav>` の出力バックエンド。
//! v11.1.0: 型定義 / fn / bind 脱糖 / match(Option+Result) / 基本 stdlib
//! v11.2.0: stage(TrfDef) / seq(FlwDef) / par / fn main() ガード / IO.argv()
//! v11.3.0: IO.read_file_raw / Csv.parse_raw / Schema.adapt / Json.encode_raw 実変換

use crate::frontend::parser::Parser;
use crate::ast::{
    self, BinOp, Block, Effect, Expr, FnDef, FStringPart, Lit, MatchArm, Pattern, Program, Stmt,
    TypeBody, TypeDef, TypeExpr,
};
use std::collections::HashMap;

// ── Emitter ────────────────────────────────────────────────────────────────────

pub struct Emitter {
    indent: usize,
    buf: String,
    ctr: usize, // helper 関数の連番
    // v11.3.0: import / helper フラグ
    needs_csv:            bool,
    needs_json:           bool,
    needs_io_helpers:     bool,
    needs_csv_helpers:    bool,
    needs_json_helpers:   bool,
    needs_schema_helpers: bool,
    // v11.4.0: AWS / boto3 フラグ
    needs_boto3:          bool,
    needs_base64:         bool,
    needs_aws_s3:         bool,
    needs_aws_dynamo:     bool,
    needs_aws_sqs:        bool,
    // v11.6.0: Postgres / psycopg2 フラグ
    needs_psycopg2:       bool,
    needs_pg_helpers:     bool,
    // v12.0.0: Gen namespace (secrets モジュール)
    needs_secrets:        bool,
    // v11.8.0: lineage コメント（fn/stage 名 → コメント文字列）
    lineage_comments: HashMap<String, String>,
    // 型名レジストリ（_SCHEMA_REGISTRY 生成用）
    type_names: Vec<String>,
}

impl Emitter {
    pub fn new() -> Self {
        Self {
            indent: 0,
            buf: String::new(),
            ctr: 0,
            needs_csv:            false,
            needs_json:           false,
            needs_io_helpers:     false,
            needs_csv_helpers:    false,
            needs_json_helpers:   false,
            needs_schema_helpers: false,
            needs_boto3:          false,
            needs_base64:         false,
            needs_aws_s3:         false,
            needs_aws_dynamo:     false,
            needs_aws_sqs:        false,
            needs_psycopg2:       false,
            needs_pg_helpers:     false,
            needs_secrets:        false,
            lineage_comments:     HashMap::new(),
            type_names:           Vec::new(),
        }
    }

    fn next_id(&mut self) -> usize {
        let id = self.ctr;
        self.ctr += 1;
        id
    }

    fn ind(&self) -> String {
        "    ".repeat(self.indent)
    }

    fn line(&mut self, s: &str) {
        let ind = self.ind();
        self.buf.push_str(&format!("{}{}\n", ind, s));
    }

    fn blank(&mut self) {
        self.buf.push('\n');
    }
}

// ── Public API ─────────────────────────────────────────────────────────────────

/// Program → Python ソース文字列
pub fn emit_python(prog: &Program, source_path: &str) -> String {
    let mut e = Emitter::new();
    e.emit_program(prog, source_path)
}

/// テスト用: Fav ソース文字列から直接 Python を生成
pub fn emit_python_str(fav_src: &str) -> String {
    let prog = Parser::parse_str(fav_src, "<test>").expect("parse failed in emit_python_str");
    emit_python(&prog, "<test>")
}

/// --lineage フラグ用: lineage コメント付きで Python を生成 (v11.8.0)
pub fn emit_python_with_lineage(
    prog: &Program,
    source_path: &str,
    comments: HashMap<String, String>,
) -> String {
    let mut e = Emitter::new();
    e.lineage_comments = comments;
    e.emit_program(prog, source_path)
}

// ── Program ────────────────────────────────────────────────────────────────────

impl Emitter {
    fn emit_program(&mut self, prog: &Program, source_path: &str) -> String {
        // Phase 1: 型名を収集（_SCHEMA_REGISTRY 生成用）
        self.type_names = prog.items.iter()
            .filter_map(|i| if let ast::Item::TypeDef(td) = i { Some(td.name.clone()) } else { None })
            .collect();

        // Phase 2: fn / stage / seq をサブエミッターで先行処理してフラグ検出
        let mut sub = Emitter::new();
        sub.type_names = self.type_names.clone();
        sub.lineage_comments = self.lineage_comments.clone();
        let mut has_main = false;
        for item in &prog.items {
            match item {
                ast::Item::FnDef(fd) => {
                    if fd.name == "main" { has_main = true; }
                    sub.emit_fn_def(fd);
                }
                ast::Item::TrfDef(td) => sub.emit_trf_def(td),
                ast::Item::FlwDef(fd) => sub.emit_flw_def(fd),
                _ => {}
            }
        }

        // Phase 3: フラグをコピー
        self.needs_csv            = sub.needs_csv;
        self.needs_json           = sub.needs_json;
        self.needs_io_helpers     = sub.needs_io_helpers;
        self.needs_csv_helpers    = sub.needs_csv_helpers;
        self.needs_json_helpers   = sub.needs_json_helpers;
        self.needs_schema_helpers = sub.needs_schema_helpers;
        self.needs_boto3          = sub.needs_boto3;
        self.needs_base64         = sub.needs_base64;
        self.needs_aws_s3         = sub.needs_aws_s3;
        self.needs_aws_dynamo     = sub.needs_aws_dynamo;
        self.needs_aws_sqs        = sub.needs_aws_sqs;
        self.needs_psycopg2       = sub.needs_psycopg2;
        self.needs_pg_helpers     = sub.needs_pg_helpers;
        self.needs_secrets        = sub.needs_secrets;

        // Phase 4: プレリュード（conditional imports 含む）
        self.emit_prelude(source_path);

        // Phase 5: 型定義
        for item in &prog.items {
            if let ast::Item::TypeDef(td) = item {
                self.emit_type_def(td);
            }
        }

        // Phase 6: _SCHEMA_REGISTRY（型定義が 1 件以上あれば常に emit）
        if !self.type_names.is_empty() {
            self.emit_schema_registry();
        }

        // Phase 7: ヘルパー関数
        if self.needs_io_helpers     { self.emit_io_helpers(); }
        if self.needs_csv_helpers    { self.emit_csv_helpers(); }
        if self.needs_schema_helpers { self.emit_schema_helpers(); }
        if self.needs_json_helpers   { self.emit_json_helpers(); }
        if self.needs_aws_s3         { self.emit_aws_s3_helpers(); }
        if self.needs_aws_dynamo     { self.emit_aws_dynamo_helpers(); }
        if self.needs_aws_sqs        { self.emit_aws_sqs_helpers(); }
        if self.needs_pg_helpers     { self.emit_pg_helpers(); }

        // Phase 8: fn / stage / seq（サブエミッター出力を追加）
        self.buf.push_str(&sub.buf);

        // Phase 9: __main__ ガード
        if has_main {
            self.line("if __name__ == \"__main__\":");
            self.indent += 1;
            self.line("main()");
            self.indent -= 1;
            self.blank();
        }

        self.buf.clone()
    }

    fn emit_prelude(&mut self, source_path: &str) {
        self.line("# Generated by fav transpile --target python");
        self.line(&format!("# Source: {}", source_path));
        self.line("from __future__ import annotations");
        self.line("from dataclasses import dataclass, asdict");
        self.line("from typing import List, Optional, Any");
        self.line("import sys");
        if self.needs_csv {
            self.line("import csv as _csv_mod");
            self.line("import io as _io_mod");
        }
        if self.needs_json {
            self.line("import json as _json_mod");
        }
        if self.needs_boto3  { self.line("import boto3"); }
        if self.needs_base64 { self.line("import base64 as _base64_mod"); }
        if self.needs_psycopg2 {
            self.line("import psycopg2");
            self.line("import psycopg2.extras");
            self.line("import os as _os");
        }
        if self.needs_secrets { self.line("import secrets"); }
        self.blank();
        self.line("class Ok:");
        self.indent += 1;
        self.line("def __init__(self, value): self.value = value");
        self.indent -= 1;
        self.line("class Err:");
        self.indent += 1;
        self.line("def __init__(self, error): self.error = error");
        self.indent -= 1;
        self.blank();
    }
}

// ── TypeDef ─────────────────────────────────────────────────────────────────────

impl Emitter {
    fn emit_type_def(&mut self, td: &TypeDef) {
        match &td.body {
            TypeBody::Record(fields) => {
                self.line("@dataclass");
                self.line(&format!("class {}:", td.name));
                self.indent += 1;
                if fields.is_empty() {
                    self.line("pass");
                } else {
                    for f in fields {
                        let py_ty = map_type(&f.ty);
                        self.line(&format!("{}: {}", f.name, py_ty));
                    }
                }
                self.indent -= 1;
                self.blank();
            }
            TypeBody::Sum(_) | TypeBody::Alias(_) | TypeBody::Wrapper(_) => {
                self.line(&format!(
                    "# TODO: type {} (sum/alias/wrapper) — not yet supported",
                    td.name
                ));
                self.blank();
            }
        }
    }
}

// ── 型マッピング ───────────────────────────────────────────────────────────────

fn map_type(ty: &TypeExpr) -> String {
    match ty {
        TypeExpr::Named(name, args, _) => match name.as_str() {
            "String" => "str".to_string(),
            "Int" => "int".to_string(),
            "Float" => "float".to_string(),
            "Bool" => "bool".to_string(),
            "Unit" => "None".to_string(),
            "List" => {
                let inner = args.first().map(map_type).unwrap_or_else(|| "Any".to_string());
                format!("List[{}]", inner)
            }
            "Option" => {
                let inner = args.first().map(map_type).unwrap_or_else(|| "Any".to_string());
                format!("Optional[{}]", inner)
            }
            "Result" => "Any".to_string(), // Ok/Err ヘルパークラスを使用
            other => other.to_string(),
        },
        TypeExpr::Optional(inner, _) => format!("Optional[{}]", map_type(inner)),
        TypeExpr::Fallible(inner, _) => format!("Any  # fallible {}", map_type(inner)),
        TypeExpr::Arrow(_, _, _) | TypeExpr::TrfFn { .. } => "Any".to_string(),
        TypeExpr::Intersection(_, _, _) | TypeExpr::RecordType(_, _) | TypeExpr::Schema(_, _) | TypeExpr::LinearArrow(_, _, _) => "Any".to_string(),
        TypeExpr::ConstInt(_, _) => "int".to_string(),
    }
}

fn map_effect(e: &Effect) -> &'static str {
    match e {
        Effect::Io => "IO",
        Effect::Db | Effect::DbRead | Effect::DbWrite | Effect::DbAdmin => "DB",
        Effect::Network => "Network",
        Effect::Http => "Http",
        Effect::Llm => "Llm",
        Effect::Snowflake => "Snowflake",
        Effect::Postgres => "Postgres",
        Effect::Rpc => "Rpc",
        Effect::File => "File",
        Effect::PipelineState => "PipelineState",
        Effect::Unknown(_) => "Unknown",
        _ => "Effect",
    }
}

// ── FnDef ─────────────────────────────────────────────────────────────────────

impl Emitter {
    fn emit_fn_def(&mut self, fd: &FnDef) {
        // lineage コメント（--lineage 指定時のみ）
        if let Some(comment) = self.lineage_comments.get(&fd.name).cloned() {
            self.line(&comment);
        }
        // エフェクトコメント
        if !fd.effects.is_empty() {
            let eff_strs: Vec<&str> = fd.effects.iter().map(|e| {
                if let Effect::Unknown(s) = e { return s.as_str(); }
                map_effect(e)
            }).collect();
            self.line(&format!("# effects: {}", eff_strs.join(", ")));
        }
        // def シグネチャ
        let params: Vec<String> = fd
            .params
            .iter()
            .map(|p| format!("{}: {}", p.name, map_type(&p.ty)))
            .collect();
        let ret_ty = fd
            .return_ty
            .as_ref()
            .map(map_type)
            .unwrap_or_else(|| "None".to_string());
        self.line(&format!(
            "def {}({}) -> {}:",
            fd.name,
            params.join(", "),
            ret_ty
        ));
        self.indent += 1;
        self.emit_block_body(&fd.body);
        self.indent -= 1;
        self.blank();
    }

    /// ブロックをステートメント列 + return で出力
    fn emit_block_body(&mut self, block: &Block) {
        if block.stmts.is_empty() {
            // 末尾式のみ
            let ret = self.emit_expr(&block.expr);
            self.line(&format!("return {}", ret));
            return;
        }
        for stmt in &block.stmts {
            self.emit_stmt(stmt);
        }
        // ブロック末尾の式
        match block.expr.as_ref() {
            Expr::Lit(Lit::Unit, _) => {
                // return None は省略
            }
            expr => {
                let ret = self.emit_expr(expr);
                self.line(&format!("return {}", ret));
            }
        }
    }
}

// ── Stmt ──────────────────────────────────────────────────────────────────────

impl Emitter {
    fn emit_stmt(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::Bind(b) => {
                let val = self.emit_expr(&b.expr);
                match &b.pattern {
                    Pattern::Bind(name, _) if name == "_" => {
                        self.line(&val);
                    }
                    Pattern::Bind(name, _) => {
                        self.line(&format!("{} = {}", name, val));
                    }
                    Pattern::Wildcard(_) => {
                        self.line(&val);
                    }
                    _ => {
                        // 複雑なパターンは _ に落とす
                        self.line(&format!("_ = {}", val));
                    }
                }
            }
            Stmt::Chain(c) => {
                let val = self.emit_expr(&c.expr);
                if c.name == "_" {
                    self.line(&val);
                } else {
                    self.line(&format!("{} = {}", c.name, val));
                }
            }
            Stmt::Expr(e) => {
                let val = self.emit_expr(e);
                self.line(&val);
            }
            Stmt::Yield(y) => {
                let val = self.emit_expr(&y.expr);
                self.line(&format!("yield {}", val));
            }
            Stmt::ForIn(f) => {
                let iter = self.emit_expr(&f.iter);
                self.line(&format!("for {} in {}:", f.var, iter));
                self.indent += 1;
                self.emit_block_body(&f.body);
                self.indent -= 1;
            }
            Stmt::Forall(_f) => {
                // forall not directly supported in Python output
                self.line("# forall (property test) - not emitted");
            }
        }
    }
}

// ── Expr ──────────────────────────────────────────────────────────────────────

impl Emitter {
    /// 式を Python 式文字列として返す。
    /// 複雑な変換（helper 関数）が必要な場合は self.buf に先書きして関数呼び出し式を返す。
    fn emit_expr(&mut self, expr: &Expr) -> String {
        match expr {
            Expr::Lit(lit, _) => emit_lit(lit),

            Expr::Ident(name, _) => name.clone(),

            Expr::BinOp(op, lhs, rhs, _) => {
                let l = self.emit_expr(lhs);
                let r = self.emit_expr(rhs);
                let py_op = match op {
                    BinOp::Add => "+",
                    BinOp::Sub => "-",
                    BinOp::Mul => "*",
                    BinOp::Div => "/",
                    BinOp::Eq => "==",
                    BinOp::NotEq => "!=",
                    BinOp::Lt => "<",
                    BinOp::Gt => ">",
                    BinOp::LtEq => "<=",
                    BinOp::GtEq => ">=",
                    BinOp::And => "and",
                    BinOp::Or => "or",
                    BinOp::NullCoalesce => {
                        // `a ?? b` → `(a if a is not None else b)`
                        return format!("({} if {} is not None else {})", l, l, r);
                    }
                };
                format!("({} {} {})", l, py_op, r)
            }

            Expr::Apply(func, args, _) => self.emit_apply(func, args),

            Expr::TypeApply(func, _, _) => self.emit_expr(func),

            Expr::FieldAccess(obj, field, _) => {
                let o = self.emit_expr(obj);
                format!("{}.{}", o, field)
            }

            Expr::If(cond, then_b, else_b, _) => {
                self.emit_if_expr(cond, then_b, else_b.as_deref())
            }

            Expr::Match(scrutinee, arms, _) => self.emit_match_expr(scrutinee, arms),

            Expr::Block(b) => {
                if b.stmts.is_empty() {
                    self.emit_expr(&b.expr)
                } else {
                    self.emit_block_as_helper(b)
                }
            }

            Expr::Closure(params, body, _) => self.emit_closure(params, body),

            Expr::RecordConstruct(ty_name, fields, _) => {
                let args: Vec<String> = fields
                    .iter()
                    .map(|(_, v)| self.emit_expr(v))
                    .collect();
                format!("{}({})", ty_name, args.join(", "))
            }

            Expr::FString(parts, _) => {
                let mut segments: Vec<String> = Vec::new();
                let mut has_expr = false;
                for part in parts {
                    match part {
                        FStringPart::Lit(s) => {
                            segments.push(format!("{:?}", s));
                        }
                        FStringPart::Expr(e) => {
                            has_expr = true;
                            let val = self.emit_expr(e);
                            segments.push(format!("str({})", val));
                        }
                    }
                }
                if has_expr {
                    // f-string 風に + で連結
                    segments.join(" + ")
                } else {
                    segments.join(" + ")
                }
            }

            Expr::Pipeline(exprs, _) => {
                // a |> b |> c → c(b(a))
                if exprs.is_empty() {
                    return "None".to_string();
                }
                let mut parts: Vec<String> =
                    exprs.iter().map(|e| self.emit_expr(e)).collect();
                let mut result = parts.remove(0);
                for func in parts {
                    result = format!("{}({})", func, result);
                }
                result
            }

            Expr::Question(inner, _) => {
                // `expr?` → result unwrap（簡易: そのまま通す）
                let val = self.emit_expr(inner);
                format!("{}_unwrapped", val)
            }

            Expr::Collect(block, _) => {
                // collect { yield x; ... } → list comprehension / helper
                let id = self.next_id();
                let helper = format!("_collect_{}", id);
                let saved_indent = self.indent;
                self.line(&format!("def {}():", helper));
                self.indent += 1;
                self.line("_items = []");
                for stmt in &block.stmts {
                    match stmt {
                        Stmt::Yield(y) => {
                            let val = self.emit_expr(&y.expr);
                            self.line(&format!("_items.append({})", val));
                        }
                        other => self.emit_stmt(other),
                    }
                }
                let tail = self.emit_expr(&block.expr);
                match block.expr.as_ref() {
                    Expr::Lit(Lit::Unit, _) => {}
                    _ => {
                        self.line(&format!("_items.append({})", tail));
                    }
                }
                self.line("return _items");
                self.indent = saved_indent;
                self.blank();
                format!("{}()", helper)
            }

            Expr::RecordSpread(base, updates, _) => {
                // { ...base, key: val } → {**base, "key": val}
                let b = self.emit_expr(base);
                let mut parts = vec![format!("**{}", b)];
                for (k, v) in updates {
                    let val = self.emit_expr(v);
                    parts.push(format!("\"{}\": {}", k, val));
                }
                format!("{{{}}}", parts.join(", "))
            }

            Expr::AssertMatches(_, _, _) | Expr::EmitExpr(_, _) => {
                "None  # assert/emit not supported in transpile".to_string()
            }

            Expr::ListComp { expr, clauses, .. } => {
                // Emit as Python list comprehension
                let body = self.emit_expr(expr);
                let clauses_str = clauses
                    .iter()
                    .map(|c| match c {
                        crate::ast::CompClause::For { var, src, .. } => {
                            format!("for {} in {}", var, self.emit_expr(src))
                        }
                        crate::ast::CompClause::Guard(g) => {
                            format!("if {}", self.emit_expr(g))
                        }
                    })
                    .collect::<Vec<_>>()
                    .join(" ");
                format!("[{} {}]", body, clauses_str)
            }

            Expr::ResultComp { expr, clauses, .. } => {
                // Emit as a comment — Result semantics not supported in Python transpile
                let body = self.emit_expr(expr);
                let src = clauses.iter().find_map(|c| {
                    if let crate::ast::CompClause::For { var, src, .. } = c {
                        Some(format!("for {} in {}", var, self.emit_expr(src)))
                    } else {
                        None
                    }
                }).unwrap_or_default();
                format!("[{} {}]  # result comp — errors not propagated", body, src)
            }
        }
    }

    // ── apply / stdlib 変換 ──────────────────────────────────────────────────

    fn emit_apply(&mut self, func: &Expr, args: &[Expr]) -> String {
        // stdlib 変換: FieldAccess(Ident(ns), method)
        if let Expr::FieldAccess(obj, method, _) = func {
            if let Expr::Ident(ns, _) = obj.as_ref() {
                let a: Vec<String> =
                    args.iter().map(|a| self.emit_expr(a)).collect();
                match (ns.as_str(), method.as_str()) {
                    // ── List ──────────────────────────────────────────────
                    ("List", "empty") => return "[]".to_string(),
                    ("List", "concat") if a.len() == 2 => {
                        return format!("({} + {})", a[0], a[1])
                    }
                    ("List", "length") if a.len() == 1 => {
                        return format!("len({})", a[0])
                    }
                    ("List", "filter") if a.len() == 2 => {
                        let pred = wrap_callable(&a[1]);
                        return format!("[_x for _x in {} if {}(_x)]", a[0], pred)
                    }
                    ("List", "map") if a.len() == 2 => {
                        let f = wrap_callable(&a[1]);
                        return format!("[{}(_x) for _x in {}]", f, a[0])
                    }
                    ("List", "first") if a.len() == 1 => {
                        return format!("({}[0] if {} else None)", a[0], a[0])
                    }
                    ("List", "last") if a.len() == 1 => {
                        return format!("({}[-1] if {} else None)", a[0], a[0])
                    }
                    ("List", "drop") if a.len() == 2 => {
                        return format!("{}[{}:]", a[0], a[1])
                    }
                    ("List", "take") if a.len() == 2 => {
                        return format!("{}[:{}]", a[0], a[1])
                    }
                    ("List", "nth") if a.len() == 2 => {
                        return format!(
                            "({}[{}] if len({}) > {} else None)",
                            a[0], a[1], a[0], a[1]
                        )
                    }
                    ("List", "reverse") if a.len() == 1 => {
                        return format!("list(reversed({}))", a[0])
                    }
                    ("List", "sort") if a.len() == 1 => {
                        return format!("sorted({})", a[0])
                    }
                    ("List", "contains") if a.len() == 2 => {
                        return format!("({} in {})", a[1], a[0])
                    }
                    ("List", "flatten") if a.len() == 1 => {
                        return format!(
                            "[_i for _sub in {} for _i in _sub]",
                            a[0]
                        )
                    }
                    ("List", "zip") if a.len() == 2 => {
                        return format!("list(zip({}, {}))", a[0], a[1])
                    }
                    ("List", "partition") if a.len() == 2 => {
                        return format!(
                            "([_x for _x in {} if {}(_x)], [_x for _x in {} if not {}(_x)])",
                            a[0], a[1], a[0], a[1]
                        )
                    }
                    ("List", "intersperse") if a.len() == 2 => {
                        // [sep].join(lst) — str 前提の近似
                        return format!("{}.join(str(_x) for _x in {})", a[1], a[0]);
                    }
                    ("List", name) => {
                        return format!("_list_{}({})", name, a.join(", "))
                    }

                    // ── String ────────────────────────────────────────────
                    ("String", "concat") if a.len() == 2 => {
                        return format!("({} + {})", a[0], a[1])
                    }
                    ("String", "length") if a.len() == 1 => {
                        return format!("len({})", a[0])
                    }
                    ("String", "trim") if a.len() == 1 => {
                        return format!("{}.strip()", a[0])
                    }
                    ("String", "to_upper") if a.len() == 1 => {
                        return format!("{}.upper()", a[0])
                    }
                    ("String", "to_lower") if a.len() == 1 => {
                        return format!("{}.lower()", a[0])
                    }
                    ("String", "contains") if a.len() == 2 => {
                        return format!("({} in {})", a[1], a[0])
                    }
                    ("String", "starts_with") if a.len() == 2 => {
                        return format!("{}.startswith({})", a[0], a[1])
                    }
                    ("String", "ends_with") if a.len() == 2 => {
                        return format!("{}.endswith({})", a[0], a[1])
                    }
                    ("String", "split") if a.len() == 2 => {
                        return format!("{}.split({})", a[0], a[1])
                    }
                    ("String", "replace") if a.len() == 3 => {
                        return format!("{}.replace({}, {})", a[0], a[1], a[2])
                    }
                    ("String", "join") if a.len() == 2 => {
                        return format!("{}.join({})", a[0], a[1])
                    }
                    ("String", name) => {
                        return format!("_str_{}({})", name, a.join(", "))
                    }

                    // ── Int / Float / Bool 変換 ───────────────────────────
                    ("Int", "to_string") if a.len() == 1 => {
                        return format!("str({})", a[0])
                    }
                    ("Int", "parse") if a.len() == 1 => {
                        return format!("int({})", a[0])
                    }
                    ("Float", "to_string") if a.len() == 1 => {
                        return format!("str({})", a[0])
                    }
                    ("Float", "parse") if a.len() == 1 => {
                        return format!("float({})", a[0])
                    }
                    ("Bool", "to_string") if a.len() == 1 => {
                        return format!("str({}).lower()", a[0])
                    }

                    // ── IO ────────────────────────────────────────────────
                    ("IO", "println") if a.len() == 1 => {
                        return format!("print({})", a[0])
                    }
                    ("IO", "print") if a.len() == 1 => {
                        return format!("print({}, end='')", a[0])
                    }
                    ("IO", "argv") if a.is_empty() => {
                        return "sys.argv[1:]".to_string()
                    }
                    ("IO", "argv_all") if a.is_empty() => {
                        return "sys.argv".to_string()
                    }
                    ("IO", "read_file_raw") if a.len() == 1 => {
                        self.needs_io_helpers = true;
                        return format!("_io_read_file_raw({})", a[0])
                    }
                    ("IO", "write_file_raw") if a.len() == 2 => {
                        self.needs_io_helpers = true;
                        return format!("_io_write_file_raw({}, {})", a[0], a[1])
                    }
                    ("IO", name) => {
                        return format!("_io_{}({})", name, a.join(", "))
                    }

                    // ── AWS S3 ───────────────────────────────────────────
                    ("AWS", "s3_put_object_raw") if a.len() == 3 => {
                        self.needs_boto3 = true; self.needs_aws_s3 = true;
                        return format!("_aws_s3_put_object_raw({}, {}, {})", a[0], a[1], a[2])
                    }
                    ("AWS", "s3_get_object_raw") if a.len() == 2 => {
                        self.needs_boto3 = true; self.needs_aws_s3 = true;
                        return format!("_aws_s3_get_object_raw({}, {})", a[0], a[1])
                    }
                    ("AWS", "s3_list_objects_raw") if a.len() == 2 => {
                        self.needs_boto3 = true; self.needs_aws_s3 = true;
                        return format!("_aws_s3_list_objects_raw({}, {})", a[0], a[1])
                    }
                    ("AWS", "s3_delete_object_raw") if a.len() == 2 => {
                        self.needs_boto3 = true; self.needs_aws_s3 = true;
                        return format!("_aws_s3_delete_object_raw({}, {})", a[0], a[1])
                    }
                    ("AWS", "s3_get_object_base64_raw") if a.len() == 2 => {
                        self.needs_boto3 = true; self.needs_aws_s3 = true; self.needs_base64 = true;
                        return format!("_aws_s3_get_object_base64_raw({}, {})", a[0], a[1])
                    }
                    ("AWS", "s3_put_bytes_raw") if a.len() == 3 => {
                        self.needs_boto3 = true; self.needs_aws_s3 = true;
                        return format!("_aws_s3_put_bytes_raw({}, {}, {})", a[0], a[1], a[2])
                    }
                    ("AWS", "s3_head_bucket_raw") if a.len() == 1 => {
                        self.needs_boto3 = true; self.needs_aws_s3 = true;
                        return format!("_aws_s3_head_bucket_raw({})", a[0])
                    }

                    // ── AWS DynamoDB ──────────────────────────────────────
                    ("AWS", "dynamo_scan_raw") if a.len() == 1 => {
                        self.needs_boto3 = true; self.needs_aws_dynamo = true;
                        return format!("_aws_dynamo_scan_raw({})", a[0])
                    }
                    ("AWS", "dynamo_get_item_raw") if a.len() == 2 => {
                        self.needs_boto3 = true; self.needs_aws_dynamo = true;
                        return format!("_aws_dynamo_get_item_raw({}, {})", a[0], a[1])
                    }
                    ("AWS", "dynamo_put_item_raw") if a.len() == 2 => {
                        self.needs_boto3 = true; self.needs_aws_dynamo = true;
                        return format!("_aws_dynamo_put_item_raw({}, {})", a[0], a[1])
                    }
                    ("AWS", "dynamo_delete_item_raw") if a.len() == 2 => {
                        self.needs_boto3 = true; self.needs_aws_dynamo = true;
                        return format!("_aws_dynamo_delete_item_raw({}, {})", a[0], a[1])
                    }
                    ("AWS", "dynamo_query_raw") if a.len() == 2 => {
                        self.needs_boto3 = true; self.needs_aws_dynamo = true;
                        return format!("_aws_dynamo_query_raw({}, {})", a[0], a[1])
                    }

                    // ── AWS SQS ───────────────────────────────────────────
                    ("AWS", "sqs_send_message_raw") if a.len() == 2 => {
                        self.needs_boto3 = true; self.needs_aws_sqs = true;
                        return format!("_aws_sqs_send_message_raw({}, {})", a[0], a[1])
                    }
                    ("AWS", "sqs_receive_messages_raw") if a.len() == 2 => {
                        self.needs_boto3 = true; self.needs_aws_sqs = true;
                        return format!("_aws_sqs_receive_messages_raw({}, {})", a[0], a[1])
                    }
                    ("AWS", "sqs_delete_message_raw") if a.len() == 2 => {
                        self.needs_boto3 = true; self.needs_aws_sqs = true;
                        return format!("_aws_sqs_delete_message_raw({}, {})", a[0], a[1])
                    }
                    ("AWS", "sqs_get_queue_url_raw") if a.len() == 1 => {
                        self.needs_boto3 = true; self.needs_aws_sqs = true;
                        return format!("_aws_sqs_get_queue_url_raw({})", a[0])
                    }

                    // AWS フォールバック
                    ("AWS", name) => {
                        self.needs_boto3 = true;
                        return format!("_aws_{}({})", name, a.join(", "))
                    }

                    // ── Snowflake — プレースホルダー ─────────────────────
                    ("Snowflake", name) => {
                        return format!("_snowflake_{}({})", name, a.join(", "))
                    }

                    // ── Postgres → psycopg2 (v11.6.0) ────────────────────
                    ("Postgres", "execute_raw") if a.len() == 2 => {
                        self.needs_psycopg2 = true;
                        self.needs_pg_helpers = true;
                        self.needs_json = true;
                        return format!("_pg_execute({}, {})", a[0], a[1])
                    }
                    ("Postgres", "query_raw") if a.len() == 2 => {
                        self.needs_psycopg2 = true;
                        self.needs_pg_helpers = true;
                        self.needs_json = true;
                        return format!("_pg_query({}, {})", a[0], a[1])
                    }
                    ("Postgres", name) => {
                        self.needs_psycopg2 = true;
                        self.needs_pg_helpers = true;
                        return format!("_pg_{}({})", to_snake(name), a.join(", "))
                    }

                    // ── Csv ──────────────────────────────────────────────
                    ("Csv", "parse_raw") => {
                        self.needs_csv = true;
                        self.needs_csv_helpers = true;
                        return format!("_csv_parse_raw({})", a.join(", "))
                    }

                    // ── Schema ───────────────────────────────────────────
                    ("Schema", "adapt") => {
                        self.needs_schema_helpers = true;
                        self.needs_json = true;
                        return format!("_schema_adapt({})", a.join(", "))
                    }
                    ("Schema", "to_json_array") => {
                        self.needs_schema_helpers = true;
                        self.needs_json = true;
                        return format!("_schema_to_json_array({})", a.join(", "))
                    }

                    // ── Gen (v12.0.0) ─────────────────────────────────────
                    ("Gen", "nano_id_raw") if a.is_empty() => {
                        self.needs_secrets = true;
                        return "secrets.token_hex(10)".to_string()
                    }
                    ("Gen", "uuid_raw") | ("Gen", "uuid_v7_raw") if a.is_empty() => {
                        self.needs_secrets = true;
                        return "secrets.token_hex(16)".to_string()
                    }
                    ("Gen", name) => {
                        self.needs_secrets = true;
                        return format!("_gen_{}({})", to_snake(name), a.join(", "))
                    }

                    // ── Json ─────────────────────────────────────────────
                    ("Json", "encode_raw") | ("Json", "write_raw") | ("Json", "write_array_raw")
                        if a.len() == 1 =>
                    {
                        self.needs_json = true;
                        return format!("_json_mod.dumps({})", a[0])
                    }
                    ("Json", "decode_raw") | ("Json", "parse_raw") if a.len() == 1 => {
                        self.needs_json = true;
                        self.needs_json_helpers = true;
                        return format!("_json_decode_raw({})", a[0])
                    }
                    ("Json", name) => {
                        self.needs_json = true;
                        return format!("_json_mod_{}({})", name, a.join(", "))
                    }

                    _ => {}
                }
            }
        }

        // 通常の関数呼び出し
        // PascalCase 識別子（stage/seq 名）は snake_case に変換
        let f = match func {
            Expr::Ident(name, _) => to_snake(name),
            other => self.emit_expr(other),
        };
        let a: Vec<String> = args.iter().map(|a| self.emit_expr(a)).collect();
        format!("{}({})", f, a.join(", "))
    }

    // ── if 式 ──────────────────────────────────────────────────────────────────

    fn emit_if_expr(
        &mut self,
        cond: &Expr,
        then_b: &Block,
        else_b: Option<&Block>,
    ) -> String {
        let cond_str = self.emit_expr(cond);

        // then / else が単一式のみなら三項式
        let then_simple = then_b.stmts.is_empty();
        let else_simple = else_b.map(|b| b.stmts.is_empty()).unwrap_or(true);

        if then_simple && else_simple {
            let then_val = self.emit_expr(&then_b.expr);
            let else_val = else_b
                .map(|b| self.emit_expr(&b.expr))
                .unwrap_or_else(|| "None".to_string());
            return format!("({} if {} else {})", then_val, cond_str, else_val);
        }

        // 複雑な場合は helper 関数に切り出し
        let id = self.next_id();
        let helper = format!("_cond_{}", id);
        let saved_indent = self.indent;

        self.line(&format!("def {}():", helper));
        self.indent += 1;
        self.line(&format!("if {}:", cond_str));
        self.indent += 1;
        self.emit_block_body(then_b);
        self.indent -= 1;
        if let Some(else_block) = else_b {
            self.line("else:");
            self.indent += 1;
            self.emit_block_body(else_block);
            self.indent -= 1;
        } else {
            self.line("else:");
            self.indent += 1;
            self.line("return None");
            self.indent -= 1;
        }
        self.indent = saved_indent;
        self.blank();

        format!("{}()", helper)
    }

    // ── match 式 ──────────────────────────────────────────────────────────────

    fn emit_match_expr(&mut self, scrutinee: &Expr, arms: &[MatchArm]) -> String {
        let sc = self.emit_expr(scrutinee);

        // Option パターン判定（Some / None のみで構成）
        let is_option_match = arms.iter().all(|a| {
            matches!(
                &a.pattern,
                Pattern::Variant(n, _, _) if n == "Some" || n == "None"
            ) || matches!(&a.pattern, Pattern::Wildcard(_) | Pattern::Bind(_, _))
        }) && arms
            .iter()
            .any(|a| matches!(&a.pattern, Pattern::Variant(n, _, _) if n == "Some" || n == "None"));

        // 全アームが単純式かチェック（stmts なし）
        // match 本体は helper 関数として生成（確実に動作する）
        let id = self.next_id();
        let helper = format!("_match_{}", id);
        let var = format!("_m{}", id);
        let saved_indent = self.indent;

        self.line(&format!("def {}():", helper));
        self.indent += 1;
        self.line(&format!("{} = {}", var, sc));

        let _ = is_option_match; // 将来の最適化用に保持

        for (i, arm) in arms.iter().enumerate() {
            let kw = if i == 0 { "if" } else { "elif" };
            let (cond, bindings) = self.arm_condition(&arm.pattern, &var);
            self.line(&format!("{} {}:", kw, cond));
            self.indent += 1;
            for b in bindings {
                self.line(&b);
            }
            // guard
            if let Some(guard) = &arm.guard {
                let g = self.emit_expr(guard);
                self.line(&format!("if not ({}):", g));
                self.indent += 1;
                // guard 不一致は次アームへスキップ（return None で近似）
                self.line("pass  # guard failed — fall through");
                self.indent -= 1;
            }
            let body = self.emit_expr(&arm.body);
            self.line(&format!("return {}", body));
            self.indent -= 1;
        }
        // デフォルト
        self.line("else:");
        self.indent += 1;
        self.line("return None");
        self.indent -= 1;
        self.indent = saved_indent;
        self.blank();

        format!("{}()", helper)
    }

    fn arm_condition(&self, pat: &Pattern, var: &str) -> (String, Vec<String>) {
        match pat {
            Pattern::Variant(name, inner, _) => match name.as_str() {
                "Some" => {
                    let bind = inner
                        .as_ref()
                        .and_then(|p| extract_bind_name(p))
                        .map(|n| format!("{} = {}", n, var))
                        .into_iter()
                        .collect();
                    (format!("{} is not None", var), bind)
                }
                "None" => (format!("{} is None", var), vec![]),
                "Ok" => {
                    let bind = inner
                        .as_ref()
                        .and_then(|p| extract_bind_name(p))
                        .map(|n| format!("{} = {}.value", n, var))
                        .into_iter()
                        .collect();
                    (format!("isinstance({}, Ok)", var), bind)
                }
                "Err" => {
                    let bind = inner
                        .as_ref()
                        .and_then(|p| extract_bind_name(p))
                        .map(|n| format!("{} = {}.error", n, var))
                        .into_iter()
                        .collect();
                    (format!("isinstance({}, Err)", var), bind)
                }
                other => {
                    let bind: Vec<String> = inner
                        .as_ref()
                        .and_then(|p| extract_bind_name(p))
                        .map(|n| format!("{} = {}.value", n, var))
                        .into_iter()
                        .collect();
                    (format!("isinstance({}, {})", var, other), bind)
                }
            },
            Pattern::Bind(name, _) => {
                // ワイルドカード的バインド — True で束縛
                let bind = if name == "_" {
                    vec![]
                } else {
                    vec![format!("{} = {}", name, var)]
                };
                ("True".to_string(), bind)
            }
            Pattern::Wildcard(_) => ("True".to_string(), vec![]),
            Pattern::Lit(lit, _) => {
                (format!("{} == {}", var, emit_lit(lit)), vec![])
            }
            Pattern::Record(_, _) => {
                ("True  # record pattern".to_string(), vec![])
            }
            Pattern::Or(pats, _) => {
                // emit as first matching alternative
                if let Some(first) = pats.first() {
                    self.arm_condition(first, var)
                } else {
                    ("False".to_string(), vec![])
                }
            }
            Pattern::List { .. } => ("True  # list pattern".to_string(), vec![]),
        }
    }

    // ── Block → helper 関数 ──────────────────────────────────────────────────

    fn emit_block_as_helper(&mut self, block: &Block) -> String {
        let id = self.next_id();
        let helper = format!("_blk_{}", id);
        let saved_indent = self.indent;
        self.line(&format!("def {}():", helper));
        self.indent += 1;
        self.emit_block_body(block);
        self.indent = saved_indent;
        self.blank();
        format!("{}()", helper)
    }

    // ── Closure ──────────────────────────────────────────────────────────────

    fn emit_closure(&mut self, params: &[String], body: &Expr) -> String {
        // 単純な式なら lambda に変換
        let can_lambda = is_simple_expr(body);
        if can_lambda {
            let body_str = self.emit_expr(body);
            let ps = params.join(", ");
            return format!("lambda {}: {}", ps, body_str);
        }
        // 複雑な場合は helper 関数
        let id = self.next_id();
        let helper = format!("_fn_{}", id);
        let saved_indent = self.indent;
        let ps = params.join(", ");
        self.line(&format!("def {}({}):", helper, ps));
        self.indent += 1;
        match body {
            Expr::Block(b) => self.emit_block_body(b),
            other => {
                let val = self.emit_expr(other);
                self.line(&format!("return {}", val));
            }
        }
        self.indent = saved_indent;
        self.blank();
        helper
    }
}

// ── TrfDef (stage) ────────────────────────────────────────────────────────────

impl Emitter {
    fn emit_trf_def(&mut self, td: &ast::TrfDef) {
        // lineage コメント（--lineage 指定時のみ）
        if let Some(comment) = self.lineage_comments.get(&td.name).cloned() {
            self.line(&comment);
        }
        // エフェクトコメント
        if !td.effects.is_empty() {
            let eff_strs: Vec<&str> = td.effects.iter().map(|e| {
                if let Effect::Unknown(s) = e { return s.as_str(); }
                map_effect(e)
            }).collect();
            self.line(&format!("# effects: {}", eff_strs.join(", ")));
        }
        // stage の closure パラメータ名を取得（TrfDef.params: Vec<Param>）
        let param_name = td.params.first()
            .map(|p| p.name.as_str())
            .unwrap_or("x");
        let input_ty  = map_type(&td.input_ty);
        let output_ty = map_type(&td.output_ty);
        let fn_name   = to_snake(&td.name);

        self.line(&format!(
            "def {}({}: {}) -> {}:",
            fn_name, param_name, input_ty, output_ty
        ));
        self.indent += 1;
        self.emit_block_body(&td.body);
        self.indent -= 1;
        self.blank();
    }
}

// ── FlwDef (seq) ──────────────────────────────────────────────────────────────

impl Emitter {
    fn emit_flw_def(&mut self, fd: &ast::FlwDef) {
        let fn_name = to_snake(&fd.name);

        if fd.steps.is_empty() {
            self.line(&format!("def {}(x):", fn_name));
            self.indent += 1;
            self.line("return x");
            self.indent -= 1;
            self.blank();
            return;
        }

        let has_par = fd.steps.iter().any(|s| matches!(s, ast::FlwStep::Par(_) | ast::FlwStep::ParDistributed(_)));
        if has_par {
            self.emit_flw_with_par(&fn_name, &fd.steps);
        } else {
            let chain = self.build_chain_expr("x", &fd.steps);
            self.line(&format!("def {}(x):", fn_name));
            self.indent += 1;
            self.line(&format!("return {}", chain));
            self.indent -= 1;
            self.blank();
        }
    }

    /// "x" → step0(x) → step1(step0(x)) → … のネスト式を構築
    fn build_chain_expr(&self, input: &str, steps: &[ast::FlwStep]) -> String {
        let mut expr = input.to_string();
        for step in steps {
            match step {
                ast::FlwStep::Stage(name) => {
                    expr = format!("{}({})", to_snake(name), expr);
                }
                ast::FlwStep::Par(names) | ast::FlwStep::ParDistributed(names) => {
                    // par/par_distributed はシンプルチェーンでは来ないが念のため
                    let calls: Vec<String> = names.iter()
                        .map(|n| format!("{}({})", to_snake(n), expr))
                        .collect();
                    expr = format!("[{}]", calls.join(", "));
                }
                ast::FlwStep::Tap(_) | ast::FlwStep::Inspect => {
                    // tap/inspect: pass through unchanged in Python emit
                }
            }
        }
        expr
    }

    /// par ステップを含む seq の変換（ThreadPoolExecutor 使用）
    fn emit_flw_with_par(&mut self, fn_name: &str, steps: &[ast::FlwStep]) {
        self.line(&format!("def {}(x):", fn_name));
        self.indent += 1;

        let mut cur = "x".to_string();
        let mut var_ctr = 0usize;
        let last_idx = steps.len().saturating_sub(1);

        for (i, step) in steps.iter().enumerate() {
            match step {
                ast::FlwStep::Stage(name) => {
                    if i == last_idx {
                        self.line(&format!("return {}({})", to_snake(name), cur));
                    } else {
                        let v = format!("_step{}", var_ctr);
                        var_ctr += 1;
                        self.line(&format!("{} = {}({})", v, to_snake(name), cur));
                        cur = v;
                    }
                }
                ast::FlwStep::Par(names) | ast::FlwStep::ParDistributed(names) => {
                    // par_distributed: stub in Python emit — uses local threads (gRPC dispatch not implemented)
                    self.line("from concurrent.futures import ThreadPoolExecutor");
                    self.line("with ThreadPoolExecutor() as _pool:");
                    self.indent += 1;
                    let submits: Vec<String> = names.iter()
                        .map(|n| format!("_pool.submit({}, {})", to_snake(n), cur))
                        .collect();
                    self.line(&format!("_futures = [{}]", submits.join(", ")));
                    self.line("_par_results = [_f.result() for _f in _futures]");
                    self.indent -= 1;
                    cur = "_par_results".to_string();
                }
                ast::FlwStep::Tap(_) | ast::FlwStep::Inspect => {
                    // tap/inspect: pass through unchanged in Python emit
                }
            }
        }

        self.indent -= 1;
        self.blank();
    }
}

// ── v11.3.0 ヘルパー emit ──────────────────────────────────────────────────────

impl Emitter {
    fn emit_io_helpers(&mut self) {
        self.line("def _io_read_file_raw(path: str):");
        self.indent += 1;
        self.line("try:");
        self.indent += 1;
        self.line("with open(path, encoding='utf-8') as _f:");
        self.indent += 1;
        self.line("return Ok(_f.read())");
        self.indent -= 1;
        self.indent -= 1;
        self.line("except Exception as _e:");
        self.indent += 1;
        self.line("return Err(str(_e))");
        self.indent -= 1;
        self.indent -= 1;
        self.blank();
        self.line("def _io_write_file_raw(path: str, text: str):");
        self.indent += 1;
        self.line("try:");
        self.indent += 1;
        self.line("with open(path, 'w', encoding='utf-8') as _f:");
        self.indent += 1;
        self.line("_f.write(text)");
        self.indent -= 1;
        self.line("return Ok(None)");
        self.indent -= 1;
        self.line("except Exception as _e:");
        self.indent += 1;
        self.line("return Err(str(_e))");
        self.indent -= 1;
        self.indent -= 1;
        self.blank();
    }

    fn emit_csv_helpers(&mut self) {
        self.line("def _csv_parse_raw(text: str, sep: str, has_header: bool):");
        self.indent += 1;
        self.line("try:");
        self.indent += 1;
        self.line("_r = _csv_mod.DictReader(_io_mod.StringIO(text), delimiter=sep)");
        self.line("import json as _j; return Ok(_j.dumps([dict(_row) for _row in _r]))");
        self.indent -= 1;
        self.line("except Exception as _e:");
        self.indent += 1;
        self.line("return Err(str(_e))");
        self.indent -= 1;
        self.indent -= 1;
        self.blank();
    }

    fn emit_schema_registry(&mut self) {
        self.line("_SCHEMA_REGISTRY = {");
        self.indent += 1;
        for name in self.type_names.clone() {
            self.line(&format!("\"{}\": {},", name, name));
        }
        self.indent -= 1;
        self.line("}");
        self.blank();
    }

    fn emit_schema_helpers(&mut self) {
        self.line("def _schema_adapt(raw, type_name: str):");
        self.indent += 1;
        self.line("_TYPE_CAST = {'str': str, 'int': int, 'float': float, 'bool': lambda v: str(v).lower() == 'true'}");
        self.line("try:");
        self.indent += 1;
        self.line("_cls = _SCHEMA_REGISTRY[type_name]");
        self.line("_fields = _cls.__dataclass_fields__");
        self.line("def _cast(k, v):");
        self.indent += 1;
        self.line("_ann = getattr(_fields[k], 'type', 'str')");
        self.line("_ann_s = _ann if isinstance(_ann, str) else getattr(_ann, '__name__', 'str')");
        self.line("return _TYPE_CAST.get(_ann_s, str)(v)");
        self.indent -= 1;
        self.line("return Ok([_cls(**{k: _cast(k, v) for k, v in _row.items() if k in _fields}) for _row in raw])");
        self.indent -= 1;
        self.line("except Exception as _e:");
        self.indent += 1;
        self.line("return Err(str(_e))");
        self.indent -= 1;
        self.indent -= 1;
        self.blank();
        self.line("def _schema_to_json_array(rows, type_name: str) -> str:");
        self.indent += 1;
        self.line("return _json_mod.dumps([asdict(_r) for _r in rows])");
        self.indent -= 1;
        self.blank();
    }

    fn emit_json_helpers(&mut self) {
        self.line("def _json_decode_raw(s: str):");
        self.indent += 1;
        self.line("try:");
        self.indent += 1;
        self.line("return Ok(_json_mod.loads(s))");
        self.indent -= 1;
        self.line("except Exception as _e:");
        self.indent += 1;
        self.line("return Err(str(_e))");
        self.indent -= 1;
        self.indent -= 1;
        self.blank();
    }

    // ── v11.4.0: AWS ヘルパー ────────────────────────────────────────────────

    fn emit_aws_s3_helpers(&mut self) {
        self.line("def _aws_s3_put_object_raw(bucket: str, key: str, body: str):");
        self.indent += 1;
        self.line("try:");
        self.indent += 1;
        self.line("boto3.client(\"s3\").put_object(Bucket=bucket, Key=key, Body=body.encode(\"utf-8\"))");
        self.line("return Ok(None)");
        self.indent -= 1;
        self.line("except Exception as _e:");
        self.indent += 1;
        self.line("return Err(str(_e))");
        self.indent -= 1;
        self.indent -= 1;
        self.blank();
        self.line("def _aws_s3_get_object_raw(bucket: str, key: str):");
        self.indent += 1;
        self.line("try:");
        self.indent += 1;
        self.line("_body = boto3.client(\"s3\").get_object(Bucket=bucket, Key=key)[\"Body\"].read().decode(\"utf-8\")");
        self.line("return Ok(_body)");
        self.indent -= 1;
        self.line("except Exception as _e:");
        self.indent += 1;
        self.line("return Err(str(_e))");
        self.indent -= 1;
        self.indent -= 1;
        self.blank();
        self.line("def _aws_s3_list_objects_raw(bucket: str, prefix: str):");
        self.indent += 1;
        self.line("try:");
        self.indent += 1;
        self.line("_resp = boto3.client(\"s3\").list_objects_v2(Bucket=bucket, Prefix=prefix)");
        self.line("return Ok([_o[\"Key\"] for _o in _resp.get(\"Contents\", [])])");
        self.indent -= 1;
        self.line("except Exception as _e:");
        self.indent += 1;
        self.line("return Err(str(_e))");
        self.indent -= 1;
        self.indent -= 1;
        self.blank();
        self.line("def _aws_s3_delete_object_raw(bucket: str, key: str):");
        self.indent += 1;
        self.line("try:");
        self.indent += 1;
        self.line("boto3.client(\"s3\").delete_object(Bucket=bucket, Key=key)");
        self.line("return Ok(None)");
        self.indent -= 1;
        self.line("except Exception as _e:");
        self.indent += 1;
        self.line("return Err(str(_e))");
        self.indent -= 1;
        self.indent -= 1;
        self.blank();
        self.line("def _aws_s3_head_bucket_raw(bucket: str):");
        self.indent += 1;
        self.line("try:");
        self.indent += 1;
        self.line("boto3.client(\"s3\").head_bucket(Bucket=bucket)");
        self.line("return Ok(None)");
        self.indent -= 1;
        self.line("except Exception as _e:");
        self.indent += 1;
        self.line("return Err(str(_e))");
        self.indent -= 1;
        self.indent -= 1;
        self.blank();
        self.line("def _aws_s3_get_object_base64_raw(bucket: str, key: str):");
        self.indent += 1;
        self.line("try:");
        self.indent += 1;
        self.line("_data = boto3.client(\"s3\").get_object(Bucket=bucket, Key=key)[\"Body\"].read()");
        self.line("return Ok(_base64_mod.b64encode(_data).decode(\"utf-8\"))");
        self.indent -= 1;
        self.line("except Exception as _e:");
        self.indent += 1;
        self.line("return Err(str(_e))");
        self.indent -= 1;
        self.indent -= 1;
        self.blank();
        self.line("def _aws_s3_put_bytes_raw(bucket: str, key: str, body_list):");
        self.indent += 1;
        self.line("try:");
        self.indent += 1;
        self.line("boto3.client(\"s3\").put_object(Bucket=bucket, Key=key, Body=bytes(body_list))");
        self.line("return Ok(None)");
        self.indent -= 1;
        self.line("except Exception as _e:");
        self.indent += 1;
        self.line("return Err(str(_e))");
        self.indent -= 1;
        self.indent -= 1;
        self.blank();
    }

    fn emit_aws_dynamo_helpers(&mut self) {
        // 型変換ユーティリティ
        self.line("def _dynamo_serialize(d: dict) -> dict:");
        self.indent += 1;
        self.line("_result = {}");
        self.line("for k, v in d.items():");
        self.indent += 1;
        self.line("if isinstance(v, bool):            _result[k] = {\"BOOL\": v}");
        self.line("elif isinstance(v, (int, float)):  _result[k] = {\"N\": str(v)}");
        self.line("else:                              _result[k] = {\"S\": str(v)}");
        self.indent -= 1;
        self.line("return _result");
        self.indent -= 1;
        self.blank();
        self.line("def _dynamo_deserialize(item: dict) -> dict:");
        self.indent += 1;
        self.line("_result = {}");
        self.line("for k, v in item.items():");
        self.indent += 1;
        self.line("if \"S\" in v:     _result[k] = v[\"S\"]");
        self.line("elif \"N\" in v:   _result[k] = float(v[\"N\"]) if \".\" in v[\"N\"] else int(v[\"N\"])");
        self.line("elif \"BOOL\" in v: _result[k] = v[\"BOOL\"]");
        self.line("else:            _result[k] = str(v)");
        self.indent -= 1;
        self.line("return _result");
        self.indent -= 1;
        self.blank();
        self.line("def _aws_dynamo_scan_raw(table: str):");
        self.indent += 1;
        self.line("try:");
        self.indent += 1;
        self.line("_items = boto3.client(\"dynamodb\").scan(TableName=table)[\"Items\"]");
        self.line("return Ok([_dynamo_deserialize(_i) for _i in _items])");
        self.indent -= 1;
        self.line("except Exception as _e:");
        self.indent += 1;
        self.line("return Err(str(_e))");
        self.indent -= 1;
        self.indent -= 1;
        self.blank();
        self.line("def _aws_dynamo_get_item_raw(table: str, key_dict: dict):");
        self.indent += 1;
        self.line("try:");
        self.indent += 1;
        self.line("_resp = boto3.client(\"dynamodb\").get_item(TableName=table, Key=_dynamo_serialize(key_dict))");
        self.line("_item = _resp.get(\"Item\")");
        self.line("return Ok(_dynamo_deserialize(_item) if _item else None)");
        self.indent -= 1;
        self.line("except Exception as _e:");
        self.indent += 1;
        self.line("return Err(str(_e))");
        self.indent -= 1;
        self.indent -= 1;
        self.blank();
        self.line("def _aws_dynamo_put_item_raw(table: str, item_dict: dict):");
        self.indent += 1;
        self.line("try:");
        self.indent += 1;
        self.line("boto3.client(\"dynamodb\").put_item(TableName=table, Item=_dynamo_serialize(item_dict))");
        self.line("return Ok(None)");
        self.indent -= 1;
        self.line("except Exception as _e:");
        self.indent += 1;
        self.line("return Err(str(_e))");
        self.indent -= 1;
        self.indent -= 1;
        self.blank();
        self.line("def _aws_dynamo_delete_item_raw(table: str, key_dict: dict):");
        self.indent += 1;
        self.line("try:");
        self.indent += 1;
        self.line("boto3.client(\"dynamodb\").delete_item(TableName=table, Key=_dynamo_serialize(key_dict))");
        self.line("return Ok(None)");
        self.indent -= 1;
        self.line("except Exception as _e:");
        self.indent += 1;
        self.line("return Err(str(_e))");
        self.indent -= 1;
        self.indent -= 1;
        self.blank();
        self.line("def _aws_dynamo_query_raw(table: str, filter_expr: str):");
        self.indent += 1;
        self.line("try:");
        self.indent += 1;
        self.line("_items = boto3.client(\"dynamodb\").query(TableName=table, FilterExpression=filter_expr)[\"Items\"]");
        self.line("return Ok([_dynamo_deserialize(_i) for _i in _items])");
        self.indent -= 1;
        self.line("except Exception as _e:");
        self.indent += 1;
        self.line("return Err(str(_e))");
        self.indent -= 1;
        self.indent -= 1;
        self.blank();
    }

    fn emit_aws_sqs_helpers(&mut self) {
        self.line("def _aws_sqs_send_message_raw(url: str, body: str):");
        self.indent += 1;
        self.line("try:");
        self.indent += 1;
        self.line("_resp = boto3.client(\"sqs\").send_message(QueueUrl=url, MessageBody=body)");
        self.line("return Ok(_resp.get(\"MessageId\", \"\"))");
        self.indent -= 1;
        self.line("except Exception as _e:");
        self.indent += 1;
        self.line("return Err(str(_e))");
        self.indent -= 1;
        self.indent -= 1;
        self.blank();
        self.line("def _aws_sqs_receive_messages_raw(url: str, max_count: int):");
        self.indent += 1;
        self.line("try:");
        self.indent += 1;
        self.line("_resp = boto3.client(\"sqs\").receive_message(QueueUrl=url, MaxNumberOfMessages=max_count)");
        self.line("return Ok(_resp.get(\"Messages\", []))");
        self.indent -= 1;
        self.line("except Exception as _e:");
        self.indent += 1;
        self.line("return Err(str(_e))");
        self.indent -= 1;
        self.indent -= 1;
        self.blank();
        self.line("def _aws_sqs_delete_message_raw(url: str, receipt: str):");
        self.indent += 1;
        self.line("try:");
        self.indent += 1;
        self.line("boto3.client(\"sqs\").delete_message(QueueUrl=url, ReceiptHandle=receipt)");
        self.line("return Ok(None)");
        self.indent -= 1;
        self.line("except Exception as _e:");
        self.indent += 1;
        self.line("return Err(str(_e))");
        self.indent -= 1;
        self.indent -= 1;
        self.blank();
        self.line("def _aws_sqs_get_queue_url_raw(name: str):");
        self.indent += 1;
        self.line("try:");
        self.indent += 1;
        self.line("return Ok(boto3.client(\"sqs\").get_queue_url(QueueName=name)[\"QueueUrl\"])");
        self.indent -= 1;
        self.line("except Exception as _e:");
        self.indent += 1;
        self.line("return Err(str(_e))");
        self.indent -= 1;
        self.indent -= 1;
        self.blank();
    }

    // ── Postgres / psycopg2 helpers (v11.6.0) ─────────────────────────────
    fn emit_pg_helpers(&mut self) {
        // _pg_connect
        self.line("def _pg_connect():");
        self.indent += 1;
        self.line("_url = _os.environ.get(\"DATABASE_URL\")");
        self.line("if _url:");
        self.indent += 1;
        self.line("return psycopg2.connect(_url)");
        self.indent -= 1;
        self.line("return psycopg2.connect(");
        self.indent += 1;
        self.line("host=_os.environ.get(\"PGHOST\", \"localhost\"),");
        self.line("port=int(_os.environ.get(\"PGPORT\", \"5432\")),");
        self.line("dbname=_os.environ.get(\"PGDATABASE\", \"postgres\"),");
        self.line("user=_os.environ.get(\"PGUSER\", \"postgres\"),");
        self.line("password=_os.environ.get(\"PGPASSWORD\", \"\"),");
        self.indent -= 1;
        self.line(")");
        self.indent -= 1;
        self.blank();
        // _pg_adapt_params: $N → %s, complex params → JSON strings
        self.line("def _pg_adapt(_sql, _params_json):");
        self.indent += 1;
        self.line("import re as _re");
        self.line("_sql_py = _re.sub(r'\\$\\d+', '%s', _sql)");
        self.line("_raw = _json_mod.loads(_params_json) if isinstance(_params_json, str) else _params_json");
        self.line("_ps = [_json_mod.dumps(_p) if isinstance(_p, (list, dict)) else _p for _p in _raw]");
        self.line("return _sql_py, _ps");
        self.indent -= 1;
        self.blank();
        // _pg_execute
        self.line("def _pg_execute(_sql, _params_json):");
        self.indent += 1;
        self.line("_sql_py, _ps = _pg_adapt(_sql, _params_json)");
        self.line("_conn = _pg_connect()");
        self.line("try:");
        self.indent += 1;
        self.line("with _conn.cursor() as _cur:");
        self.indent += 1;
        self.line("_cur.execute(_sql_py, _ps)");
        self.indent -= 1;
        self.line("_conn.commit()");
        self.line("return Ok(None)");
        self.indent -= 1;
        self.line("except Exception as _e:");
        self.indent += 1;
        self.line("_conn.rollback()");
        self.line("return Err(str(_e))");
        self.indent -= 1;
        self.line("finally:");
        self.indent += 1;
        self.line("_conn.close()");
        self.indent -= 1;
        self.indent -= 1;
        self.blank();
        // _pg_query
        self.line("def _pg_query(_sql, _params_json):");
        self.indent += 1;
        self.line("_sql_py, _ps = _pg_adapt(_sql, _params_json)");
        self.line("_conn = _pg_connect()");
        self.line("try:");
        self.indent += 1;
        self.line("with _conn.cursor(cursor_factory=psycopg2.extras.RealDictCursor) as _cur:");
        self.indent += 1;
        self.line("_cur.execute(_sql_py, _ps)");
        self.line("_rows = [dict(_r) for _r in _cur.fetchall()]");
        self.indent -= 1;
        self.line("return Ok(_json_mod.dumps(_rows, default=float))");
        self.indent -= 1;
        self.line("except Exception as _e:");
        self.indent += 1;
        self.line("return Err(str(_e))");
        self.indent -= 1;
        self.line("finally:");
        self.indent += 1;
        self.line("_conn.close()");
        self.indent -= 1;
        self.indent -= 1;
        self.blank();
    }
}

// ── Util ──────────────────────────────────────────────────────────────────────

/// PascalCase / camelCase → snake_case
/// LoadAll → load_all, ValidateTxn → validate_txn, IOHelper → io_helper
pub fn to_snake(name: &str) -> String {
    let mut out = String::new();
    let chars: Vec<char> = name.chars().collect();
    for (i, &ch) in chars.iter().enumerate() {
        if ch.is_uppercase() && i > 0 {
            let prev_lower = chars[i - 1].is_lowercase() || chars[i - 1].is_ascii_digit();
            let next_lower = chars.get(i + 1).map(|c| c.is_lowercase()).unwrap_or(false);
            if prev_lower || next_lower {
                out.push('_');
            }
        }
        for c in ch.to_lowercase() {
            out.push(c);
        }
    }
    out
}

fn emit_lit(lit: &Lit) -> String {
    match lit {
        Lit::Int(n) => n.to_string(),
        Lit::Float(f) => {
            let s = format!("{}", f);
            if s.contains('.') { s } else { format!("{}.0", s) }
        }
        Lit::Str(s) => format!("{:?}", s),
        Lit::Bool(b) => if *b { "True".to_string() } else { "False".to_string() },
        Lit::Unit => "None".to_string(),
    }
}

/// lambda 式の場合は括弧で囲む（`lambda x: x` → `(lambda x: x)`）
/// 既に括弧付きや通常の識別子はそのまま返す
fn wrap_callable(s: &str) -> String {
    if s.starts_with("lambda ") {
        format!("({})", s)
    } else {
        s.to_string()
    }
}

fn extract_bind_name(pat: &Pattern) -> Option<String> {
    match pat {
        Pattern::Bind(name, _) if name != "_" => Some(name.clone()),
        _ => None,
    }
}

/// 式が lambda に変換可能な単純式かどうか
fn is_simple_expr(expr: &Expr) -> bool {
    match expr {
        Expr::Lit(_, _) | Expr::Ident(_, _) => true,
        Expr::BinOp(_, l, r, _) => is_simple_expr(l) && is_simple_expr(r),
        Expr::FieldAccess(obj, _, _) => is_simple_expr(obj),
        Expr::Apply(func, args, _) => {
            is_simple_expr(func) && args.iter().all(is_simple_expr)
        }
        _ => false,
    }
}

// ── Tests ──────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod v11200_tests {
    use super::*;

    #[test]
    fn transpile_stage_basic() {
        let src = r#"stage Foo: Int -> Int = |x| { x }"#;
        let out = emit_python_str(src);
        assert!(out.contains("def foo(x: int) -> int:"), "stage def:\n{}", out);
        assert!(out.contains("return x"), "stage body:\n{}", out);
    }

    #[test]
    fn transpile_stage_effects_comment() {
        let src = r#"stage Bar: String -> String !IO = |s| { s }"#;
        let out = emit_python_str(src);
        assert!(out.contains("# effects: IO"), "effect comment:\n{}", out);
    }

    #[test]
    fn transpile_stage_multiline_body() {
        let src = r#"
stage Validate: List<Int> -> List<Int> !IO = |rows| {
  bind valid <- List.filter(rows, |x| x > 0)
  bind _ <- IO.println("done")
  valid
}"#;
        let out = emit_python_str(src);
        assert!(out.contains("def validate(rows: List[int]) -> List[int]:"), "sig:\n{}", out);
        assert!(out.contains("[_x for _x in rows"), "filter:\n{}", out);
        assert!(out.contains("print("), "println→print:\n{}", out);
        assert!(out.contains("return valid"), "return:\n{}", out);
    }

    #[test]
    fn transpile_seq_two_stages() {
        let src = r#"
stage Load: Int -> String = |x| { Int.to_string(x) }
stage Upper: String -> String = |s| { s }
seq Pipe = Load |> Upper
"#;
        let out = emit_python_str(src);
        assert!(out.contains("def pipe(x):"), "seq def:\n{}", out);
        assert!(out.contains("upper(load(x))"), "chain:\n{}", out);
    }

    #[test]
    fn transpile_seq_three_stages() {
        let src = r#"
stage A: Int -> Int = |x| { x }
stage B: Int -> Int = |x| { x }
stage C: Int -> Int = |x| { x }
seq Pipeline = A |> B |> C
"#;
        let out = emit_python_str(src);
        assert!(out.contains("def pipeline(x):"), "def:\n{}", out);
        assert!(out.contains("c(b(a(x)))"), "3-chain:\n{}", out);
    }

    #[test]
    fn transpile_seq_snake_case() {
        let src = r#"
stage LoadAll: Int -> Int = |x| { x }
stage WriteOutput: Int -> Int = |x| { x }
seq AnalyzePipeline = LoadAll |> WriteOutput
"#;
        let out = emit_python_str(src);
        assert!(out.contains("def load_all("), "load_all:\n{}", out);
        assert!(out.contains("def write_output("), "write_output:\n{}", out);
        assert!(out.contains("def analyze_pipeline(x):"), "analyze_pipeline:\n{}", out);
        assert!(out.contains("write_output(load_all(x))"), "chain:\n{}", out);
    }

    #[test]
    fn transpile_main_guard() {
        let src = r#"fn main() -> Unit !IO { IO.println("hi") }"#;
        let out = emit_python_str(src);
        assert!(out.contains("def main()"), "main def:\n{}", out);
        assert!(
            out.contains("if __name__ == \"__main__\":"),
            "__main__ guard:\n{}",
            out
        );
        assert!(out.contains("    main()"), "main() call:\n{}", out);
    }

    #[test]
    fn transpile_io_argv() {
        let src = r#"fn f() -> List<String> !IO { IO.argv() }"#;
        let out = emit_python_str(src);
        assert!(out.contains("sys.argv[1:]"), "argv:\n{}", out);
    }
}

#[cfg(test)]
mod v11100_tests {
    use super::*;

    #[test]
    fn transpile_dataclass_simple() {
        let src = "type Point = {\n  x: Int\n  y: Int\n}";
        let out = emit_python_str(src);
        assert!(out.contains("@dataclass"), "missing @dataclass:\n{}", out);
        assert!(out.contains("class Point:"), "missing class Point:\n{}", out);
        assert!(out.contains("x: int"), "missing x: int:\n{}", out);
        assert!(out.contains("y: int"), "missing y: int:\n{}", out);
    }

    #[test]
    fn transpile_fn_basic() {
        let src = "fn add(a: Int, b: Int) -> Int { a + b }";
        let out = emit_python_str(src);
        assert!(
            out.contains("def add(a: int, b: int) -> int:"),
            "missing def:\n{}",
            out
        );
        assert!(out.contains("return (a + b)"), "missing return:\n{}", out);
    }

    #[test]
    fn transpile_bind_desugars() {
        let src = "fn f(x: Int) -> Int {\n  bind a <- double(x)\n  a\n}";
        let out = emit_python_str(src);
        assert!(out.contains("a = double(x)"), "bind desugaring:\n{}", out);
    }

    #[test]
    fn transpile_match_option() {
        let src = r#"fn f(lst: List<String>) -> String {
  match List.first(lst) {
    Some(v) => v
    None    => ""
  }
}"#;
        let out = emit_python_str(src);
        assert!(
            out.contains("is not None"),
            "Option match Some missing:\n{}",
            out
        );
        assert!(out.contains("is None"), "Option match None missing:\n{}", out);
    }

    #[test]
    fn transpile_match_result() {
        let src = r#"fn f(path: String) -> String {
  match _io_read_file_raw(path) {
    Ok(text) => text
    Err(_)   => ""
  }
}"#;
        let out = emit_python_str(src);
        assert!(
            out.contains("isinstance") && out.contains("Ok"),
            "Result match missing:\n{}",
            out
        );
    }

    #[test]
    fn transpile_list_ops() {
        let src = r#"fn f(lst: List<Int>) -> Int { List.length(lst) }"#;
        let out = emit_python_str(src);
        assert!(out.contains("len(lst)"), "List.length → len():\n{}", out);
    }

    #[test]
    fn transpile_string_concat() {
        let src = r#"fn f(a: String, b: String) -> String { String.concat(a, b) }"#;
        let out = emit_python_str(src);
        assert!(out.contains("(a + b)"), "String.concat → +:\n{}", out);
    }
}

#[cfg(test)]
mod v11300_tests {
    use super::*;

    #[test]
    fn transpile_io_read_file() {
        let src = r#"fn load(path: String) -> String !IO {
  match IO.read_file_raw(path) {
    Ok(t) => t
    Err(_) => ""
  }
}"#;
        let out = emit_python_str(src);
        assert!(out.contains("_io_read_file_raw(path)"), "call site:\n{}", out);
        assert!(out.contains("def _io_read_file_raw"), "helper def:\n{}", out);
        assert!(out.contains("with open(path"), "open call:\n{}", out);
        assert!(out.contains("return Ok("), "Ok wrap:\n{}", out);
        assert!(out.contains("return Err("), "Err wrap:\n{}", out);
    }

    #[test]
    fn transpile_io_write_file() {
        let src = r#"fn save(path: String, text: String) -> Unit !IO {
  IO.write_file_raw(path, text)
}"#;
        let out = emit_python_str(src);
        assert!(out.contains("_io_write_file_raw(path, text)"), "call site:\n{}", out);
        assert!(out.contains("def _io_write_file_raw"), "helper def:\n{}", out);
        assert!(out.contains("open(path, 'w'"), "write open:\n{}", out);
    }

    #[test]
    fn transpile_csv_parse_raw() {
        let src = r#"fn parse(text: String) -> Unit !IO {
  Csv.parse_raw(text, ",", true)
}"#;
        let out = emit_python_str(src);
        assert!(out.contains("_csv_parse_raw(text"), "call site:\n{}", out);
        assert!(out.contains("import csv as _csv_mod"), "csv import:\n{}", out);
        assert!(out.contains("def _csv_parse_raw"), "helper def:\n{}", out);
        assert!(out.contains("DictReader"), "DictReader:\n{}", out);
    }

    #[test]
    fn transpile_schema_registry() {
        let src = r#"type TxnRow = { amount: Float  region: String }
fn dummy() -> Unit { () }"#;
        let out = emit_python_str(src);
        assert!(out.contains("_SCHEMA_REGISTRY"), "registry dict:\n{}", out);
        assert!(out.contains("\"TxnRow\": TxnRow"), "registry entry:\n{}", out);
    }

    #[test]
    fn transpile_schema_adapt() {
        let src = r#"type TxnRow = { amount: Float  region: String }
fn adapt(raw: List<String>) -> Unit !IO {
  Schema.adapt(raw, "TxnRow")
}"#;
        let out = emit_python_str(src);
        assert!(out.contains("_schema_adapt(raw"), "call site:\n{}", out);
        assert!(out.contains("def _schema_adapt"), "helper def:\n{}", out);
        assert!(out.contains("_SCHEMA_REGISTRY"), "registry used:\n{}", out);
    }

    #[test]
    fn transpile_schema_to_json_array() {
        let src = r#"type TxnRow = { amount: Float }
fn serialize(rows: List<TxnRow>) -> String !IO {
  Schema.to_json_array(rows, "TxnRow")
}"#;
        let out = emit_python_str(src);
        assert!(out.contains("_schema_to_json_array(rows"), "call site:\n{}", out);
        assert!(out.contains("def _schema_to_json_array"), "helper def:\n{}", out);
        assert!(out.contains("asdict"), "asdict:\n{}", out);
        assert!(out.contains("_json_mod.dumps"), "json dumps:\n{}", out);
    }

    #[test]
    fn transpile_json_encode() {
        let src = r#"fn encode(v: String) -> String { Json.encode_raw(v) }"#;
        let out = emit_python_str(src);
        assert!(out.contains("_json_mod.dumps(v)"), "encode_raw:\n{}", out);
        assert!(out.contains("import json as _json_mod"), "json import:\n{}", out);
    }

    #[test]
    fn transpile_analyze_fav_smoke() {
        // analyze.fav の主要パターンをインラインで検証
        let src = r#"
type TxnRow = {
  transaction_id: String
  amount: Float
  region: String
}

fn read_txn_csv(path: String) -> List<TxnRow> !IO {
  match IO.read_file_raw(path) {
    Err(_) => List.empty()
    Ok(text) =>
      match Csv.parse_raw(text, ",", true) {
        Err(_) => List.empty()
        Ok(raw) =>
          match Schema.adapt(raw, "TxnRow") {
            Err(_) => List.empty()
            Ok(rows) => rows
          }
      }
  }
}

fn serialize(rows: List<TxnRow>) -> String !IO {
  Schema.to_json_array(rows, "TxnRow")
}
"#;
        let out = emit_python_str(src);
        // 全ヘルパーが生成されていること
        assert!(out.contains("def _io_read_file_raw"),  "io helper:\n{}", out);
        assert!(out.contains("def _csv_parse_raw"),     "csv helper:\n{}", out);
        assert!(out.contains("def _schema_adapt"),      "schema_adapt helper:\n{}", out);
        assert!(out.contains("def _schema_to_json_array"), "to_json_array helper:\n{}", out);
        assert!(out.contains("_SCHEMA_REGISTRY"),       "registry:\n{}", out);
        // import が生成されていること
        assert!(out.contains("import csv as _csv_mod"), "csv import:\n{}", out);
        assert!(out.contains("import json as _json_mod"), "json import:\n{}", out);
    }
}

// ── v11400_tests (v11.4.0) ────────────────────────────────────────────────────

#[cfg(test)]
mod v11400_tests {
    use super::*;

    #[test]
    fn transpile_aws_s3_put() {
        let src = r#"fn store(bucket: String, key: String, body: String) -> Unit !AWS {
  AWS.s3_put_object_raw(bucket, key, body)
}"#;
        let out = emit_python_str(src);
        assert!(out.contains("_aws_s3_put_object_raw(bucket, key, body)"), "call site:\n{}", out);
        assert!(out.contains("import boto3"), "boto3 import:\n{}", out);
        assert!(out.contains("def _aws_s3_put_object_raw"), "helper def:\n{}", out);
        assert!(out.contains("put_object("), "put_object call:\n{}", out);
    }

    #[test]
    fn transpile_aws_s3_get() {
        let src = r#"fn load(bucket: String, key: String) -> Unit !AWS {
  AWS.s3_get_object_raw(bucket, key)
}"#;
        let out = emit_python_str(src);
        assert!(out.contains("_aws_s3_get_object_raw(bucket, key)"), "call site:\n{}", out);
        assert!(out.contains("def _aws_s3_get_object_raw"), "helper def:\n{}", out);
        assert!(out.contains("get_object("), "get_object call:\n{}", out);
        assert!(out.contains("decode(\"utf-8\")"), "decode:\n{}", out);
    }

    #[test]
    fn transpile_aws_s3_list() {
        let src = r#"fn list_keys(bucket: String, prefix: String) -> Unit !AWS {
  AWS.s3_list_objects_raw(bucket, prefix)
}"#;
        let out = emit_python_str(src);
        assert!(out.contains("_aws_s3_list_objects_raw(bucket, prefix)"), "call site:\n{}", out);
        assert!(out.contains("def _aws_s3_list_objects_raw"), "helper def:\n{}", out);
        assert!(out.contains("list_objects_v2("), "list_objects_v2 call:\n{}", out);
    }

    #[test]
    fn transpile_aws_dynamo_scan() {
        let src = r#"fn scan_table(table: String) -> Unit !AWS {
  AWS.dynamo_scan_raw(table)
}"#;
        let out = emit_python_str(src);
        assert!(out.contains("_aws_dynamo_scan_raw(table)"), "call site:\n{}", out);
        assert!(out.contains("def _aws_dynamo_scan_raw"), "helper def:\n{}", out);
        assert!(out.contains("def _dynamo_deserialize"), "deserialize helper:\n{}", out);
        assert!(out.contains("scan(TableName="), "scan call:\n{}", out);
    }

    #[test]
    fn transpile_aws_sqs_send() {
        let src = r#"fn send(url: String, body: String) -> Unit !AWS {
  AWS.sqs_send_message_raw(url, body)
}"#;
        let out = emit_python_str(src);
        assert!(out.contains("_aws_sqs_send_message_raw(url, body)"), "call site:\n{}", out);
        assert!(out.contains("def _aws_sqs_send_message_raw"), "helper def:\n{}", out);
        assert!(out.contains("send_message("), "send_message call:\n{}", out);
    }

    #[test]
    fn transpile_analyze_fav_aws_smoke() {
        let src = r#"
type TxnRow = {
  transaction_id: String
  amount: Float
  region: String
}

fn write_output(bucket: String, key: String, rows: List<TxnRow>) -> Unit !AWS {
  match AWS.s3_put_object_raw(bucket, key, Schema.to_json_array(rows, "TxnRow")) {
    Ok(_) => ()
    Err(_) => ()
  }
}
"#;
        let out = emit_python_str(src);
        assert!(out.contains("import boto3"), "boto3 import:\n{}", out);
        assert!(out.contains("def _aws_s3_put_object_raw"), "s3 helper:\n{}", out);
        assert!(out.contains("def _schema_to_json_array"), "schema helper:\n{}", out);
        assert!(out.contains("_SCHEMA_REGISTRY"), "schema registry:\n{}", out);
        assert!(out.contains("put_object(Bucket="), "put_object call:\n{}", out);
        assert!(out.contains("boto3.client(\"s3\")"), "boto3 s3 client:\n{}", out);
    }
}
