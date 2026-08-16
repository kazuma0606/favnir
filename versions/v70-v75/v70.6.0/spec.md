# v70.6.0 Spec — `bind` 分割束縛拡張 / Named Destructuring

Date: 2026-08-09
Status: 計画中

---

## Background

**本バージョンは新規 Rust パーサー実装ではなく、既存実装の E2E 検証テスト追加と compiler.fav のギャップ修正が主軸。**

### Rust パイプライン（実装済み）

`parse_bind_stmt`（parser.rs line 2680）は `parse_pattern()` を呼ぶため、すでに以下をサポート:

| 機能 | 構文 | 実装 |
|---|---|---|
| Record 分割束縛 | `bind {field} <- expr` | parser.rs: `Pattern::Record` |
| Record エイリアス | `bind {age: user_age, _} <- expr` | parser.rs: `PatternField::Alias` |
| List スプレッド | `bind [head, ..tail] <- expr` | parser.rs: `Pattern::List` (DotDot トークン) |
| Variant 分割束縛 | `bind ok(v) <- result` | parser.rs: `Pattern::Variant` |

**注**: ロードマップの `...` 表記は実際には `..`（`TokenKind::DotDot`）。

**未検証**: parse + typecheck + compile の E2E パスが未テスト（既存テストはパースのみ）。

### compiler.fav（`bind` ハンドラ）のギャップ

`compiler.fav` line 1494 の `TkBind` ハンドラは `TkIdent(vname)` のみ処理。`{` / `[` による分割束縛は未対応。

| 機能 | 状態 |
|---|---|
| `bind varname <- expr` | 実装済み（`TkIdent`）|
| `bind {field} <- expr` | **未実装**（`TkLBrace` 未対応）|
| `bind [h, ..t] <- expr` | **未実装**（`TkLBracket` 未対応）|

### checker.rs の確認事項

`compiler.rs`（AST → IR）は `BindStmt` の `pattern` を `IRPattern` に変換するパスを持つ。
checker.rs も `Pattern::Record` / `Pattern::List` を `BindStmt` コンテキストで処理できるか要確認。

---

## Goals

1. `bind_destructure_record` テスト: Record 分割束縛（`bind {field} <- expr`）を parse + typecheck + compile して成功することを確認
2. `bind_destructure_list_spread` テスト: List スプレッド束縛（`bind [head, ..tail] <- items`）を parse + typecheck + compile して成功することを確認
3. compiler.fav の `TkBind` ハンドラに `TkLBrace`（Record）と `TkLBracket`（List）分岐を追加
4. テスト 2 件追加 → 3572 tests

**スコープ外（v70.7.0 以降）:**
- ネスト分割束縛（`bind {customer: {name, email}} <- order`）の E2E 検証

---

## Syntax / API Examples

```favnir
// Record 分割束縛
fn process_user(u: User) -> String {
    bind {name, email} <- u
    f"Hello {name} ({email})"
}

// List スプレッド束縛（.. は DotDot トークン）
fn head_of(items: List<Int>) -> Int {
    bind [first, ..rest] <- items
    first
}

// 既存の bind との混在
fn process_order(ctx: AppCtx, row: OrderRow) -> Result<Unit, String> {
    bind {order_id, amount} <- row
    bind result             <- Postgres.insert(ctx, order_id, amount)
    ctx.io.println(f"Inserted {order_id}: {result} rows")
}
```

---

## テスト仕様

### `bind_destructure_record`

- Record 型 `{ name: String, score: Int }` に対して `bind {name, score} <- user` を使うソースを定義
- `Parser::parse_str` → parse 成功を assert
- `Checker::check_program` → errors が空であることを assert
- `build_artifact` → パニックなし

### `bind_destructure_list_spread`

- `List<Int>` に対して `bind [head, ..tail] <- items` を使うソースを定義
- `Parser::parse_str` → parse 成功を assert
- `Checker::check_program` → errors が空であることを assert
- `build_artifact` → パニックなし

---

## Success Criteria

- [ ] `bind_destructure_record` テスト: parse + check + build がすべて成功
- [ ] `bind_destructure_list_spread` テスト: parse + check + build がすべて成功
- [ ] compiler.fav の `TkBind` ハンドラが `TkLBrace` / `TkLBracket` を認識する
- [ ] `cargo test v706000` で 2 件 pass
- [ ] `cargo test` 全体で 3572 tests pass（0 failures）

---

## Error Codes

新規エラーコードなし

---

## Files to Modify

| ファイル | 変更内容 |
|---|---|
| `fav/src/driver.rs` | `v706000_tests` モジュール追加（2 テスト）|
| `fav/self/compiler.fav` | `TkBind` ハンドラに `TkLBrace` / `TkLBracket` 分岐追加 |
| `fav/Cargo.toml` | `version` を `"70.5.0"` → `"70.6.0"` に更新 |
| `CHANGELOG.md` | v70.6.0 エントリ追加 |
| `versions/current.md` | 進行中バージョンを v70.6.0 に更新 |
| `site/content/docs/language/` | 既存の bind/destructure MDX で十分（変更不要）|
