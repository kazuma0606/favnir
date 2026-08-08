# Plan — v56.3.0 — 行多相レコード活用拡張

## ゴール

- `{ field: Type | r }` インライン行変数型構文を parser で受理
- `TypeExpr::RecordType` に `Option<String>` row_var を追加
- `TypeExpr::display()` ヘルパーで `{ name: String | r }` 形式を表示
- 3231 → 3233 tests

---

## 実装ステップ

### Phase 1: Cargo.toml バージョン更新

`56.2.0` → `56.3.0`

---

### Phase 2: AST 変更（`ast.rs`）

1. `TypeExpr::RecordType` の定義を変更:
   ```rust
   // Before: RecordType(Vec<(String, TypeExpr)>, Span)
   // After:  RecordType(Vec<(String, TypeExpr)>, Option<String>, Span)
   ```

2. `span` メソッド match arm: `TypeExpr::RecordType(_, s)` → `TypeExpr::RecordType(_, _, s)`

3. `TypeExpr::display()` メソッドを `impl TypeExpr` に追加:
   - `RecordType(fields, Some(r), _)` → `"{ field: Type | r }"`
   - `RecordType(fields, None, _)` → `"{ field: Type }"`
   - `Named`, `Arrow` など基本型も対応
   - その他は `"..."` にフォールバック

---

### Phase 3: パーサー変更（`parser.rs`）

`parse_base_type` の RecordType 解析ループに `| ident` 検出を追加:

```
while peek != RBrace && !at_end:
    if peek == Pipe:
        advance   # consume `|`
        row_var = Some(expect_ident())
        break
    # … 通常フィールド解析 …
```

作成時: `TypeExpr::RecordType(fields, row_var, span)`

---

### Phase 4: 全 RecordType match arm 更新

`cargo build` のコンパイルエラーを頼りに全箇所を修正する（推定 24〜25 箇所）:

| ファイル | 変更方針 |
|---------|---------|
| `ast.rs` span | `(_, s)` → `(_, _, s)` |
| `emit_python.rs` | `(_, _)` → `(_, _, _)` |
| `driver.rs`（8 箇所） | `(fields, _)` → `(fields, _, _)` / ワイルドカード更新 |
| `fmt.rs`（2 箇所） | `(fields, row_var, _)` で row_var も `| r` 形式で出力 |
| `lint.rs` | `(fields, _)` → `(fields, _, _)` |
| `lsp/references.rs` | `(fields, _)` → `(fields, _, _)` |
| `middle/ast_lower_checker.rs`（2 箇所） | `(_, _)` → `(_, _, _)` |
| `middle/compiler.rs`（3 箇所） | `substitute_self_in_type_expr` を含む — row_var を保持して再構築 |
| `middle/checker.rs`（4 箇所） | `_row_var` で束縛（clippy 対応）、`type_expr_contains` は row_var 無視 |

**`substitute_self_in_type_expr`（compiler.rs L1680）は特に注意**:
```rust
// After:
TypeExpr::RecordType(fields, row_var, span) => TypeExpr::RecordType(
    fields.iter().map(|(n, t)| (n.clone(), substitute_self_in_type_expr(t, type_name))).collect(),
    row_var.clone(),  // ← row_var を保持
    span.clone(),
),
```

---

### Phase 5: driver.rs 更新

1. `v56200_tests::cargo_toml_version_is_56_2_0` を削除
2. `v56300_tests` モジュールを `v56200_tests` の直前に挿入:
   - `cargo_toml_version_is_56_3_0`
   - `row_poly_generic_fn`（`errors.is_empty()` assert）
   - `row_poly_lsp_hover`（`TypeExpr::display()` 直接呼出し + `Span` ダミー構築）

---

### Phase 6: ロードマップのテスト数修正

`roadmap-v56.1-v57.0.md` と `roadmap-v55.1-v60.0.md` の v56.3.0 セクションの
テスト数 `3232 + 2 = 3234` → `3231 + 2 = 3233` に修正。

---

### Phase 7: ポスト処理

- `CHANGELOG.md` に v56.3.0 エントリを追加（version: `56.2.0 → 56.3.0`）
- `versions/current.md` を v56.3.0 / 3233 tests に更新
- 両ロードマップを COMPLETE に更新

---

## テスト戦略

| テスト | 内容 |
|--------|------|
| `cargo_toml_version_is_56_3_0` | Cargo.toml バージョン確認 |
| `row_poly_generic_fn` | `{ name: String | r }` パース + 型チェック通過（Unknown 互換） |
| `row_poly_lsp_hover` | `TypeExpr::display()` が `{ name: String | r }` を返す |
| 既存 3231 件全通過 | RecordType match arm 更新で既存テストが壊れないことを確認 |

---

## リスク管理

| リスク | 対策 |
|--------|------|
| match arm 更新漏れ | `cargo build` でコンパイルエラー全件を確認してから `cargo test` |
| `substitute_self_in_type_expr` で row_var を落とす | spec に明示し、再構築時に `row_var.clone()` を渡す |
| `row_poly_generic_fn` で型エラーが出る | Unknown 互換を確認済み（checker.rs L71-74、L5628） |
| `Pipe` トークンが `|>` と衝突 | Pipe と Pipeline は別トークン（既存 OR パターンで実績あり） |
| clippy unused variable 警告 | row_var を `_row_var` として束縛 |
| `Span` ダミー構築のフィールド名不明 | `ast.rs` の `Span` 定義を確認（`file/start/end/line/col`）|
