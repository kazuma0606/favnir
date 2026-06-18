# v16.3.0 Spec — レコード更新構文（Record Spread / Update）

Date: 2026-06-14
Branch: master

---

## テーマ

データ変換パイプラインの核心操作「1 フィールドだけ変えたい」を自然に書けるようにする。
全フィールドを書き直す必要を排除し、`stage` 内の変換ロジックを簡潔にする。

**Before（v16.2 まで）:**
```fav
stage Enrich(row: RawRow) -> EnrichedRow {
  Result.ok({
    id:           row.id,
    name:         row.name,
    email:        row.email,
    created_at:   row.created_at,
    score:        compute_score(row),     // ← これだけが新規
    enriched_at:  DateTime.now_unix(),    // ← これだけが新規
  })
}
```

**After（v16.3 以降）:**
```fav
stage Enrich(row: RawRow) -> EnrichedRow {
  Result.ok({ ...row, score: compute_score(row), enriched_at: DateTime.now_unix() })
}
```

---

## 設計上の重要な制約

スプレッド構文は**値の組み立て方**であり、型の宣言ではない。

```fav
// ✅ 正しい使い方: 戻り型を明示した stage / fn 内でスプレッドを使用
stage Enrich(row: RawRow) -> EnrichedRow {
  Result.ok({ ...row, score: compute_score(row) })
}

// ❌ 禁止: 戻り型を省略してスプレッドを直接返す
fn enrich(row: UserRow) {
  { ...row, score: 1.0 }  // E0327: 戻り型の宣言が必要
}
```

---

## 現状分析

| コンポーネント | 状態 | 備考 |
|---|---|---|
| `...` トークン | **未実装** | lexer に DotDotDot 追加が必要 |
| `Expr::RecordSpread` | **未実装** | ast.rs に追加が必要 |
| Parser の `{ ...e, k: v }` 認識 | **未実装** | parse_record_spread 追加が必要 |
| IR `IRExpr::RecordSpread` | **未実装** | ir.rs に追加が必要 |
| `MergeRecord` opcode | **未実装** | codegen.rs / vm.rs に追加が必要 |
| E0323（未存在フィールド） | **未実装** | checker.rs に追加が必要 |
| E0327（戻り型なしスプレッド） | **未実装** | checker.rs に追加が必要 |
| E0328（Unknown base） | **未実装** | checker.rs に追加が必要 |

---

## スコープ

### A: バージョン更新

```toml
version = "16.3.0"
```

### B: `...` トークン追加（lexer.rs）

```rust
'.' if self.peek2() == Some('.') && self.peek3() == Some('.') => {
    self.advance(); // '.'
    self.advance(); // '.'
    self.advance(); // '.'
    TokenKind::DotDotDot
}
```

`TokenKind::DotDotDot` を enum に追加。

### C: AST 拡張（ast.rs）

`Expr::RecordSpread` を追加:

```rust
/// `{ ...base, key: expr, ... }` — record spread / update (v16.3.0)
RecordSpread(Box<Expr>, Vec<(String, Expr)>, Span),
```

`Expr::span()` の match に `Expr::RecordSpread(_, _, s) => s` を追加。

### D: Parser 拡張（parser.rs）

現在の式パーサー（`parse_primary` / `parse_atom`）内で `{` が来た際:
- 次のトークンが `DotDotDot` → `parse_record_spread()` を呼ぶ
- それ以外 → 既存ブロック構文へ

`parse_record_spread`:
```rust
// 構文: { ...base_expr, key1: val1, key2: val2, ... }
// '{'  は消費済み
// DotDotDot を消費 → base_expr をパース → ',' があれば field: expr を繰り返す → '}'
fn parse_record_spread(&mut self) -> Result<Expr, ParseError> {
    // DotDotDot を消費
    // base_expr をパース（parse_expr）
    // ',' を消費
    // while { 'ident' ':' expr ',' } をパース
    // '}' を消費
    // Expr::RecordSpread(base, updates, span)
}
```

### E: IR 拡張（ir.rs）

```rust
/// `{ ...base, field: val }` — runtime merge (v16.3.0)
RecordSpread(Box<IRExpr>, Vec<(String, IRExpr)>, Type),
```

`IRExpr::ty()` に `IRExpr::RecordSpread(_, _, ty) => ty` を追加。

### F: Compiler 拡張（compiler.rs / middle）

`Expr::RecordSpread(base, updates, _)` をコンパイル:

```rust
Expr::RecordSpread(base, updates, _) => {
    let base_ir = compile_expr(base, ctx);
    let updates_ir: Vec<(String, IRExpr)> = updates
        .iter()
        .map(|(k, v)| (k.clone(), compile_expr(v, ctx)))
        .collect();
    IRExpr::RecordSpread(Box::new(base_ir), updates_ir, Type::Unknown)
}
```

### G: Codegen 拡張（codegen.rs）

新 opcode `MergeRecord = 0x5C` を追加。

```rust
MergeRecord = 0x5C,
// Layout: opcode(1) + n_overrides(2) + names_idx(2) = 5 bytes
// Stack (bottom→top): base_record, val_0, val_1, ..., val_{n-1}
// constants[names_idx..names_idx+n]: override field names (CVName)
```

`IRExpr::RecordSpread(base, updates, _)` のコード生成:
1. `base` を emit（base record を stack へ）
2. `updates` の各 val を emit（スタックへ push）
3. override field 名を constants pool に追加
4. `MergeRecord(n_overrides, names_idx)` を emit

### H: VM 実装（vm.rs）

`MergeRecord` opcode の実行:
1. `n_overrides` 個の値をスタックから pop（逆順）
2. `base_record` を pop
3. base の全フィールドを HashMap にコピー
4. overrides で上書き
5. 新 Record を push

```rust
x if x == Opcode::MergeRecord as u8 => {
    let n_overrides = Self::read_u16(function, frame)? as usize;
    let names_idx = Self::read_u16(function, frame)? as usize;
    // override field names from constants
    let mut field_names = Vec::with_capacity(n_overrides);
    for i in 0..n_overrides {
        match function.constants.get(names_idx + i) {
            Some(Constant::Name(name)) => field_names.push(name.clone()),
            _ => return Err(vm.error(artifact, "MergeRecord: invalid constant")),
        }
    }
    // pop override values (pushed left-to-right, so pop right-to-left)
    let mut override_vals: Vec<VMValue> = (0..n_overrides)
        .map(|_| vm.stack.pop().unwrap())
        .collect();
    override_vals.reverse();
    // pop base record
    let base = vm.stack.pop().unwrap();
    let mut fields = match base {
        VMValue::Record(map) => map,
        _ => return Err(vm.error(artifact, "MergeRecord: base is not a record")),
    };
    // apply overrides
    for (name, val) in field_names.into_iter().zip(override_vals) {
        fields.insert(name, val);
    }
    vm.stack.push(VMValue::Record(fields));
}
```

### I: 型チェック拡張（checker.rs）

**E0323**: override フィールドが base 型に存在しない

```rust
// check_expr の RecordSpread ケース
Expr::RecordSpread(base, updates, span) => {
    let base_ty = self.check_expr(base);
    // base の型が既知のレコード型の場合、フィールドを検証
    if let Some(fields) = self.resolve_record_fields(&base_ty) {
        for (fname, _) in updates {
            if !fields.contains_key(fname) {
                self.error(TypeError::new(
                    "E0323",
                    format!("field `{}` does not exist in `{}`", fname, base_ty),
                    span.clone(),
                ));
            }
        }
    }
    // updates の各値をチェック
    for (_, expr) in updates {
        self.check_expr(expr);
    }
    Type::Unknown  // 結果型は宣言された戻り型から推論
}
```

**E0327**: 戻り型なし関数でスプレッドを返す

関数定義のチェック時（`check_fn_def`）:
- 戻り型が宣言されていない（`Type::Unknown`）かつ
- ボディの末尾式が `Expr::RecordSpread` の場合
- → E0327 を報告

**E0328**: Unknown base への spread（将来の E0323 拡張として実装）

### J: ast_lower_checker.rs 拡張

```rust
ast::Expr::RecordSpread(base, updates, _) => {
    // { ...base, k: v } → ECall("Record", "merge", [lower_expr(base), overrides])
    // checker.fav でサポートするまで _unsupported_ にフォールバック可
    v1("EVar", sv("_unsupported_spread_"))
}
```

> **注意**: checker.fav への完全対応は v16.4.0 に延期可。
> Rust パイプライン（`build_artifact` / `exec_artifact_main`）が動作すれば v163000_tests は PASS する。

### K: get_help_text 更新（driver.rs）

```rust
"E0323" => &[
    "スプレッドで更新できるのは base 型に存在するフィールドのみです",
    "新しいフィールドは宣言された戻り型で型チェックされます",
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

### L: v163000_tests 追加（driver.rs）

5 件のテスト（`run_source` ヘルパーを使用）:

1. `version_is_16_3_0`
2. `record_spread_basic` — `{ ...row, status: "ok" }` が正しいレコードを返す
3. `record_spread_multiple_fields` — 複数フィールドの上書きが動作する
4. `record_spread_field_override` — 既存フィールドの上書きが動作する
5. `record_spread_nested` — `{ ...outer, inner: { ...inner, x: 1 } }` が動作する

> **注意**: E0327 / E0323 のテストは `check_source_to_string` ヘルパーを使用する
> （v161000_tests で定義済み）。テスト実装は v163000_tests 内で `use super::check_source_to_string` するか、
> `check_source_to_string` を `pub(crate)` に変更する。

### M: wasm_codegen.rs 拡張

`IRExpr::RecordSpread` のケースを追加（`walk_closures_in_expr` / `collect_local_types`）:

```rust
IRExpr::RecordSpread(base, updates, _) => {
    walk_closures_in_expr(base, ir, map);
    for (_, v) in updates {
        walk_closures_in_expr(v, ir, map);
    }
}
```

### N: lineage.rs 拡張

`Expr::RecordSpread` のケースを追加（各 `collect_*_inner` 関数）:

```rust
ast::Expr::RecordSpread(base, fields, _) => {
    collect_XXX_inner(base, ...);
    for (_, v) in fields {
        collect_XXX_inner(v, ...);
    }
}
```

### O: サイトドキュメント

`site/content/docs/language/record-update.mdx` 新規作成:
- 基本構文（`{ ...base, key: val }`）
- Before/After 比較
- 設計上の制約（戻り型宣言の必要性）
- よくある間違い（E0327 / E0323）

---

## 完了条件

| 確認項目 | 状態 |
|---|---|
| `Cargo.toml version == "16.3.0"` | [ ] |
| `{ ...row, field: val }` がコンパイル・実行される | [ ] |
| 複数フィールドのスプレッドが動作する | [ ] |
| ネストしたスプレッドが動作する | [ ] |
| 戻り型なし関数でのスプレッド返しが E0327 を出す | [ ] |
| `cargo test v163000` 全テストパス（5/5） | [ ] |
| `cargo test` 全件パス（リグレッションなし） | [ ] |
| `site/content/docs/language/record-update.mdx` が存在する | [ ] |

---

## 既知の制約・スコープ外

- `{ ...row }` （フィールド指定なしのスプレッド） — v16.3.0 では不要（全コピーは意味がない）
- E0328（Unknown base）— 型推論が強化される v16.5.0 以降で完全対応
- `checker.fav` / `compiler.fav` への対応 — v16.4.0 に延期
- `wasm_codegen.rs` の `MergeRecord` 対応 — WASM エクスポートが必要な場合は追加実装
- `$\"...\"` / f-string との組み合わせ — 自然に動作する（RecordSpread 内に f-string が書ける）

---

## 参照

- `versions/roadmap-v16.1-v17.0.md` — v16.3.0 セクション
- `fav/src/frontend/lexer.rs` — `peek3()` 実装済み（v16.2.0）
- `fav/src/ast.rs` — `Expr::RecordConstruct` 実装済み（参考）
- `fav/src/middle/ir.rs` — `IRExpr` enum（`RecordSpread` 追加対象）
- `fav/src/backend/codegen.rs` — `Opcode::MergeRecord = 0x5C`（次の空き番号）
- `fav/src/backend/vm.rs` — `BuildRecord` / `GetField` opcode（参考）
- `fav/src/middle/checker.rs` — `check_expr` / `check_fn_def`（E0323/E0327 追加対象）
