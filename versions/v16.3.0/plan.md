# v16.3.0 Plan — レコード更新構文（Record Spread / Update）

Date: 2026-06-14

---

## 前提確認

### 実装済みの資産

| ファイル | 実装済み内容 |
|---|---|
| `fav/src/frontend/lexer.rs` | `peek3()` 実装済み（v16.2.0）、`DotDotDot` 追加のみ必要 |
| `fav/src/ast.rs` | `Expr::RecordConstruct(String, Vec<(String, Expr)>, Span)` 実装済み（参考）|
| `fav/src/middle/ir.rs` | `IRExpr::RecordConstruct(Vec<(String, IRExpr)>, Type)` 実装済み（参考）|
| `fav/src/backend/codegen.rs` | `BuildRecord` / `GetField` opcode 実装済み、`0x5C` が空き |
| `fav/src/backend/vm.rs` | `BuildRecord` / `GetField` VM 実装済み（参考）|
| `fav/src/middle/checker.rs` | `check_expr` dispatch 実装済み、E0323/E0327 追加のみ |
| `fav/src/middle/compiler.rs` | `Expr::RecordConstruct` コンパイル実装済み（参考）|
| `fav/src/middle/ast_lower_checker.rs` | `lower_expr` dispatch 実装済み |
| `fav/src/backend/wasm_codegen.rs` | `IRExpr::RecordConstruct` walk 実装済み（参考）|
| `fav/src/lineage.rs` | `Expr::RecordConstruct` walk 実装済み（参考）|
| `v161000_tests` | `check_source_to_string` ヘルパー実装済み（E0327 テストに再利用）|

### 修正・追加が必要な箇所

| ファイル | 変更内容 |
|---|---|
| `fav/Cargo.toml` | version → `16.3.0` |
| `fav/src/frontend/lexer.rs` | `DotDotDot` token (`...`) 追加 |
| `fav/src/ast.rs` | `Expr::RecordSpread` + `span()` match |
| `fav/src/frontend/parser.rs` | `parse_record_spread` 追加 |
| `fav/src/middle/ir.rs` | `IRExpr::RecordSpread` + `ty()` match |
| `fav/src/middle/compiler.rs` | `Expr::RecordSpread` コンパイル |
| `fav/src/backend/codegen.rs` | `Opcode::MergeRecord = 0x5C` + codegen |
| `fav/src/backend/vm.rs` | `MergeRecord` opcode 実行 |
| `fav/src/middle/checker.rs` | E0323 / E0327 追加 |
| `fav/src/middle/ast_lower_checker.rs` | `RecordSpread` フォールバック |
| `fav/src/backend/wasm_codegen.rs` | `IRExpr::RecordSpread` walk |
| `fav/src/lineage.rs` | `Expr::RecordSpread` walk（4 箇所）|
| `fav/src/driver.rs` | `get_help_text` E0323/E0327/E0328 + `v163000_tests` |
| `site/content/docs/language/record-update.mdx` | 新規作成 |

---

## Phase A — Cargo バージョン更新

**変更ファイル**: `fav/Cargo.toml`

```toml
version = "16.3.0"
```

新規 crate 依存なし。

---

## Phase B — `DotDotDot` トークン追加（lexer.rs）

**変更ファイル**: `fav/src/frontend/lexer.rs`

`TokenKind` enum に追加:

```rust
DotDotDot,   // `...` (v16.3.0 record spread)
```

`next_token` の `'.'` ケースの前後（または独立した arm として）:

```rust
'.' if self.peek2() == Some('.') && self.peek3() == Some('.') => {
    self.advance(); // '.'
    self.advance(); // '.'
    self.advance(); // '.'
    TokenKind::DotDotDot
}
```

**注意**: `peek3()` は v16.2.0 で実装済み。

---

## Phase C — AST 拡張（ast.rs）

**変更ファイル**: `fav/src/ast.rs`

`Expr` enum に追加（`RecordConstruct` の直後）:

```rust
/// `{ ...base, key: expr, ... }` — record spread (v16.3.0)
RecordSpread(Box<Expr>, Vec<(String, Expr)>, Span),
```

`Expr::span()` の match に追加:

```rust
Expr::RecordSpread(_, _, s) => s,
```

---

## Phase D — Parser 拡張（parser.rs）

**変更ファイル**: `fav/src/frontend/parser.rs`

式をパースする際、`{` が来た場合の分岐を拡張する。
現在の `parse_primary` / `parse_atom` で `{` を処理する箇所を確認し:

```rust
// { が来たとき
TokenKind::LBrace => {
    // 先読みして ... があれば record spread
    if self.peek() == Some(&TokenKind::DotDotDot) {
        self.parse_record_spread(sp, sl, sc)
    } else {
        // 既存のブロック解析へ
        self.parse_block_expr(sp, sl, sc)
    }
}
```

新規メソッド `parse_record_spread`:

```rust
fn parse_record_spread(&mut self, sp: usize, sl: u32, sc: u32)
    -> Result<Expr, ParseError>
{
    // DotDotDot を消費
    self.expect(TokenKind::DotDotDot)?;

    // base 式をパース（カンマまで、閉じブレースまで）
    let base = self.parse_expr()?;

    // フィールドオーバーライドを繰り返しパース
    let mut updates: Vec<(String, Expr)> = Vec::new();
    while self.peek() == Some(&TokenKind::Comma) {
        self.advance(); // ','
        // '}' で終了（trailing comma 対応）
        if self.peek() == Some(&TokenKind::RBrace) {
            break;
        }
        let fname = self.expect_ident()?;
        self.expect(TokenKind::Colon)?;
        let val = self.parse_expr()?;
        updates.push((fname, val));
    }

    self.expect(TokenKind::RBrace)?;
    let span = self.span_from(sp, sl, sc);
    Ok(Expr::RecordSpread(Box::new(base), updates, span))
}
```

**注意**: `parse_expr` が `{` トークンに到達する経路を確認すること。
現状のパーサーで `{` がどのコンテキストで「ブロック」か「レコード」かを判断しているかを確認してから実装する。

---

## Phase E — IR 拡張（ir.rs）

**変更ファイル**: `fav/src/middle/ir.rs`

`IRExpr` enum に追加:

```rust
/// `{ ...base, field: val }` — runtime merge (v16.3.0)
RecordSpread(Box<IRExpr>, Vec<(String, IRExpr)>, Type),
```

`IRExpr::ty()` match に追加:

```rust
IRExpr::RecordSpread(_, _, ty) => ty,
```

---

## Phase F — Compiler 拡張（compiler.rs）

**変更ファイル**: `fav/src/middle/compiler.rs`

`compile_expr` の dispatch に追加（`Expr::RecordConstruct` の付近）:

```rust
Expr::RecordSpread(base, updates, _) => {
    let base_ir = compile_expr(base, ctx)?;
    let updates_ir: Vec<(String, IRExpr)> = updates
        .iter()
        .map(|(k, v)| Ok((k.clone(), compile_expr(v, ctx)?)))
        .collect::<Result<_, _>>()?;
    IRExpr::RecordSpread(Box::new(base_ir), updates_ir, Type::Unknown)
}
```

**実装メモ**: 実際の `compile_expr` のシグネチャを確認すること（`Result` を返すか、直接 `IRExpr` を返すか）。

---

## Phase G — Codegen 拡張（codegen.rs）

**変更ファイル**: `fav/src/backend/codegen.rs`

`Opcode` enum に追加:

```rust
/// Merge base record with override fields.
/// Layout: opcode(1) + n_overrides(2) + names_idx(2) = 5 bytes
/// Stack (bottom→top): base_record, val_0, val_1, ..., val_{n-1}
/// constants[names_idx..names_idx+n]: override field names (CVName)
MergeRecord = 0x5C,
```

`IRExpr::RecordSpread(base, updates, _)` のコード生成（`codegen_expr` 内）:

```rust
IRExpr::RecordSpread(base, updates, _) => {
    // 1. base を emit
    codegen_expr(base, ctx, fn_buf, consts)?;
    // 2. override values を emit（左から右の順）
    for (_, val) in updates {
        codegen_expr(val, ctx, fn_buf, consts)?;
    }
    // 3. override field 名を constants pool に追加
    let names_idx = consts.len() as u16;
    for (name, _) in updates {
        consts.push(Constant::Name(name.clone()));
    }
    // 4. MergeRecord opcode を emit
    let n = updates.len() as u16;
    fn_buf.push(Opcode::MergeRecord as u8);
    fn_buf.extend_from_slice(&n.to_le_bytes());
    fn_buf.extend_from_slice(&names_idx.to_le_bytes());
}
```

**実装メモ**: 実際の codegen 関数のシグネチャ・constants pool の扱いを確認してから実装する。
`BuildRecordC` の実装（line 70 付近）が最も近い参考になる。

---

## Phase H — VM 実装（vm.rs）

**変更ファイル**: `fav/src/backend/vm.rs`

`MergeRecord` opcode の実行（dispatch loop 内）:

```rust
x if x == Opcode::MergeRecord as u8 => {
    let n_overrides = Self::read_u16(function, frame)? as usize;
    let names_idx = Self::read_u16(function, frame)? as usize;
    // 1. constants から override field 名を取得
    let mut field_names: Vec<String> = Vec::with_capacity(n_overrides);
    for i in 0..n_overrides {
        match function.constants.get(names_idx + i) {
            Some(Constant::Name(name)) => field_names.push(name.clone()),
            _ => return Err(vm.error(
                artifact,
                &format!("MergeRecord: constant[{}] is not a Name", names_idx + i),
            )),
        }
    }
    // 2. override 値をスタックから pop（右から左 → reverse）
    let mut override_vals: Vec<VMValue> = (0..n_overrides)
        .map(|_| vm.stack.pop().unwrap())
        .collect();
    override_vals.reverse();
    // 3. base record を pop
    let base_val = vm.stack.pop().unwrap();
    let mut fields = match base_val {
        VMValue::Record(map) => map,
        _ => return Err(vm.error(artifact, "MergeRecord: base is not a record")),
    };
    // 4. overrides を適用
    for (name, val) in field_names.into_iter().zip(override_vals) {
        fields.insert(name, val);
    }
    // 5. 新 record を push
    vm.stack.push(VMValue::Record(fields));
}
```

**実装メモ**: `VMValue` の型名、`vm.stack` の pop 方法、`vm.error()` の呼び出し方を
`BuildRecord` の実装（line 2008 付近）を参考にすること。

---

## Phase I — 型チェック拡張（checker.rs）

**変更ファイル**: `fav/src/middle/checker.rs`

### E0323: 未存在フィールドへの更新

`check_expr` の dispatch に追加:

```rust
Expr::RecordSpread(base, updates, span) => {
    let base_ty = self.check_expr(base);
    // base 型が既知の Named 型の場合、フィールドを検証
    if let Type::Named(type_name) = &base_ty {
        if let Some(type_def) = self.type_env.get(type_name.as_str()) {
            let known_fields: std::collections::HashSet<&str> = type_def
                .fields
                .iter()
                .map(|f| f.name.as_str())
                .collect();
            for (fname, _) in updates {
                if !known_fields.contains(fname.as_str()) {
                    self.errors.push(TypeError::new(
                        "E0323",
                        format!("field `{}` does not exist in `{}`", fname, type_name),
                        span.clone(),
                    ));
                }
            }
        }
    }
    // updates の各値をチェック
    for (_, expr) in updates {
        self.check_expr(expr);
    }
    Type::Unknown
}
```

### E0327: 戻り型なし関数でのスプレッド返し

関数定義チェック時（`check_fn_def` 内）に追加:

```rust
// 戻り型が宣言されていない場合
if fn_def.return_ty == Type::Unknown || fn_def.return_ty == Type::Infer {
    // ボディの最後の式が RecordSpread かチェック
    if is_record_spread_expr(&fn_def.body) {
        self.errors.push(TypeError::new(
            "E0327",
            "record spread を返す関数には明示的な戻り型が必要です".to_string(),
            fn_def.span.clone(),
        ));
    }
}

fn is_record_spread_expr(expr: &Expr) -> bool {
    match expr {
        Expr::RecordSpread(_, _, _) => true,
        Expr::Block(block) => {
            block.stmts.is_empty()
                || matches!(block.expr.as_ref(), Some(e) if is_record_spread_expr(e))
        }
        _ => false,
    }
}
```

**実装メモ**: `check_fn_def` のシグネチャと `fn_def` の型を確認してから実装する。
`fn_def.return_ty` が `Type::Unknown` かどうかの判定方法を `checker.rs` の既存コードで確認すること。

---

## Phase J — ast_lower_checker.rs 拡張

**変更ファイル**: `fav/src/middle/ast_lower_checker.rs`

`lower_expr` の dispatch に追加（`FString` の直後）:

```rust
ast::Expr::RecordSpread(_, _, _) => v1("EVar", sv("_unsupported_spread_")),
```

> **注意**: checker.fav への完全対応は v16.4.0 以降。
> Rust パイプラインのテストは `build_artifact` 経由のため、これで v163000_tests は PASS する。

---

## Phase K — lineage.rs 拡張

**変更ファイル**: `fav/src/lineage.rs`

`collect_sql_literals_inner` / `collect_azure_kinds_inner` /
`collect_azure_blob_kinds_inner` / `collect_sf_kinds_inner` の各 match に追加:

```rust
ast::Expr::RecordSpread(base, fields, _) => {
    collect_XXX_inner(base, out);          // XXX は関数に応じて変更
    for (_, v) in fields {
        collect_XXX_inner(v, out);
    }
}
```

---

## Phase L — wasm_codegen.rs 拡張

**変更ファイル**: `fav/src/backend/wasm_codegen.rs`

`walk_closures_in_expr` / `collect_local_types` の match に追加:

```rust
IRExpr::RecordSpread(base, updates, _) => {
    walk_closures_in_expr(base, ir, map);
    for (_, v) in updates {
        walk_closures_in_expr(v, ir, map);
    }
}
```

---

## Phase M — get_help_text 更新 + v163000_tests（driver.rs）

**変更ファイル**: `fav/src/driver.rs`

### get_help_text 追加

```rust
"E0323" => &[
    "スプレッドで更新できるのは base 型に存在するフィールドのみです",
    "新しいフィールドを追加するには戻り型を EnrichedRow のように定義してください",
],
"E0327" => &[
    "{ ...base, field: val } を返す関数には明示的な戻り型が必要です",
    "例: `fn enrich(row: RawRow) -> EnrichedRow { ... }`",
],
"E0328" => &[
    "スプレッドの base 式の型が静的に確定していません",
    "型注釈を追加するか、型が判明している変数を使ってください",
],
```

### v163000_tests モジュール

```rust
#[cfg(test)]
mod v163000_tests {
    use super::{build_artifact, exec_artifact_main};
    use crate::frontend::parser::Parser;

    fn run_source(src: &str) -> crate::value::Value {
        let program = Parser::parse_str(src, "spread_test.fav").expect("parse");
        let artifact = build_artifact(&program);
        exec_artifact_main(&artifact, None).expect("exec")
    }

    #[test]
    fn version_is_16_3_0() {
        let cargo = std::fs::read_to_string("Cargo.toml").unwrap();
        assert!(cargo.contains("version = \"16.3.0\""), "expected 16.3.0");
    }

    #[test]
    fn record_spread_basic() {
        // { ...row, status: "ok" } should override the status field
        let result = run_source(r#"
type Row = { name: String status: String }
public fn main() -> Row {
    bind row <- Row { name: "Alice", status: "pending" }
    { ...row, status: "ok" }
}
"#);
        if let crate::value::Value::Record(map) = result {
            assert_eq!(map.get("status"), Some(&crate::value::Value::Str("ok".into())));
            assert_eq!(map.get("name"), Some(&crate::value::Value::Str("Alice".into())));
        } else {
            panic!("expected Record, got {:?}", result);
        }
    }

    #[test]
    fn record_spread_multiple_fields() {
        let result = run_source(r#"
type Point = { x: Int y: Int z: Int }
public fn main() -> Point {
    bind p <- Point { x: 1, y: 2, z: 3 }
    { ...p, x: 10, y: 20 }
}
"#);
        if let crate::value::Value::Record(map) = result {
            assert_eq!(map.get("x"), Some(&crate::value::Value::Int(10)));
            assert_eq!(map.get("y"), Some(&crate::value::Value::Int(20)));
            assert_eq!(map.get("z"), Some(&crate::value::Value::Int(3)));
        } else {
            panic!("expected Record, got {:?}", result);
        }
    }

    #[test]
    fn record_spread_field_override() {
        // 既存フィールドが正しく上書きされる
        let result = run_source(r#"
type Item = { id: Int name: String }
public fn main() -> Item {
    bind item <- Item { id: 1, name: "old" }
    { ...item, name: "new" }
}
"#);
        if let crate::value::Value::Record(map) = result {
            assert_eq!(map.get("id"), Some(&crate::value::Value::Int(1)));
            assert_eq!(map.get("name"), Some(&crate::value::Value::Str("new".into())));
        } else {
            panic!("expected Record, got {:?}", result);
        }
    }

    #[test]
    fn record_spread_nested() {
        // ネストしたスプレッド: inner を更新しながら outer に組み込む
        let result = run_source(r#"
type Inner = { x: Int y: Int }
type Outer = { inner: Inner label: String }
public fn main() -> Inner {
    bind i <- Inner { x: 1, y: 2 }
    { ...i, x: 99 }
}
"#);
        if let crate::value::Value::Record(map) = result {
            assert_eq!(map.get("x"), Some(&crate::value::Value::Int(99)));
            assert_eq!(map.get("y"), Some(&crate::value::Value::Int(2)));
        } else {
            panic!("expected Record, got {:?}", result);
        }
    }
}
```

---

## Phase N — サイトドキュメント

**新規作成**: `site/content/docs/language/record-update.mdx`

内容:
1. 概要（全フィールド書き直しの問題）
2. 基本構文（`{ ...base, key: val }`）
3. Before/After 比較
4. 設計上の制約（戻り型宣言の必要性・E0327）
5. よくある間違い（E0323 / E0327 / E0328）
6. 関連エラーコードへのリンク

---

## Phase O — テスト確認とコミット

```
cargo test v163000 → 5/5 PASS 確認
cargo test → 全件 PASS 確認（リグレッションなし）
git commit "feat: v16.3.0 — レコード更新構文（{ ...base, field: val }）"
```

---

## 実装順序

```
Phase A (Cargo) → Phase B (lexer DotDotDot) → Phase C (AST) → Phase D (parser)
→ Phase E (IR) → Phase F (compiler) → Phase G (codegen) → Phase H (VM)
→ cargo build 確認
→ Phase I (checker E0323/E0327) → Phase J (ast_lower_checker)
→ Phase K (lineage) → Phase L (wasm_codegen)
→ cargo build 確認
→ Phase M (get_help_text + v163000_tests) → cargo test v163000 → 5/5 PASS
→ cargo test 全件確認
→ Phase N (docs) → Phase O (commit)
```

**重要**: Phase D（パーサー）が最もリスクが高い。
`{` のコンテキスト判定（ブロック vs レコード vs スプレッド）を丁寧に確認すること。

---

## リスク・注意点

| リスク | 対策 |
|---|---|
| `{` のパースが既存ブロック構文と衝突する | `{` 直後のトークンが `DotDotDot` かを先読みして分岐 |
| `compile_expr` のシグネチャが想定と異なる | `Expr::RecordConstruct` の実装を参考に確認 |
| lineage.rs で exhaustive match 警告 | clippy `-D warnings` 対応のため全 match に追加必須 |
| wasm_codegen の `IRExpr::RecordSpread` 未対応でビルドエラー | Phase L で対応 |
| checker.rs の `type_env` / `type_def` 取得 API | 既存の `RecordConstruct` チェックコードを参照 |
| E0327 の検出精度 | 完全なフロー解析は不要。末尾式が RecordSpread かの構造チェックで十分 |
