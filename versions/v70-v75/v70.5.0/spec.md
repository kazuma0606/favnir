# v70.5.0 Spec — パターンマッチ強化

Date: 2026-08-09
Status: 計画中

---

## Background

**本バージョンは新規構文実装ではなく、既存 Rust パイプラインの E2E 検証テスト追加と compiler.fav のギャップ修正が主軸。**

### Rust パイプライン（実装済み）

| 機能 | 実装バージョン | 場所 |
|---|---|---|
| Or-パターン `A \| B` | v17.2.0 | parser.rs・compiler.rs・codegen.rs |
| ガード `if cond` | v61.3.0 | parser.rs・codegen.rs（IRPattern::Or に統合）|
| Record パターン `{field: pat}` | v41.3.0 | parser.rs・compiler.rs（PatternField::Alias）|
| checker: 各パターン型伝播 | — | checker.rs（Pattern::Or / Record / Variant 処理済み）|

**未検証**: 既存テストは `check_src`（型チェックのみ）で、parse + compile（`build_artifact`）のパスが未確認。

### compiler.fav（自己ホスト型コンパイラ）のギャップ

`fav/self/compiler.fav` の `parse_arm_guard` / `parse_arms` / `parse_pat` を調査した結果:

| 機能 | 状態 |
|---|---|
| ガード `where cond` | 実装済み（`parse_arm_guard` が `TkWhere` を処理）|
| ガード **`if cond`** | **未実装**（`TkIf` を処理しない）|
| Or-パターン **`A \| B`** | **未実装**（`parse_arms` に `\|` 処理がない）|
| Record パターン **`{field: pat}`** | **未実装**（`parse_pat` に `TkLBrace` 処理がない）|
| Variant+Payload パターン `Ok(pat)` | 実装済み（`PVariantP`）|

---

## Goals

1. `pattern_match_nested_record` テスト: Record フィールドをパターンに使う Favnir ソースを parse + typecheck + compile して成功することを確認
2. `pattern_match_or_pattern` テスト: Or-パターン（`"a" | "b"`）を parse + typecheck + compile して成功することを確認
3. compiler.fav の `parse_arm_guard` に `TkIf` ガード対応を追加（`where` → `if` の互換）
4. テスト 2 件追加 → 3569 tests

**スコープ外（v70.6.0 以降）:**
- compiler.fav への Or-パターン（`|`）追加 — parser.fav の状態管理が複雑なため次版に先送り
- compiler.fav への Record パターン（`{field: pat}`）追加 — 同上

---

## Syntax / API Examples

```favnir
// Record パターン（フィールドリテラルマッチ）— Rust パイプラインで動作確認
type Response = { code: Int body: String }
fn classify(r: Response) -> String {
    match r {
        { code: 200, body } => body
        { code: 404, _ }    => "not found"
        _                   => "error"
    }
}

// Or-パターン（文字列リテラル複数アーム）— Rust パイプラインで動作確認
fn classify_event(kind: String) -> String {
    match kind {
        "created" | "updated" => "write"
        "deleted" | "expired" => "delete"
        _                     => "unknown"
    }
}

// ガード（if キーワード）— compiler.fav 修正後に動作
fn classify_amount(x: Float) -> String {
    match x {
        n if n > 10000.0 => "large"
        n if n > 1000.0  => "medium"
        _                => "small"
    }
}
```

---

## テスト仕様

### `pattern_match_nested_record`

- Record 型 `{ code: Int, body: String }` に対して `{ code: 200, body }` パターンを使うソースを定義
- `Parser::parse_str` → parse 成功を assert
- `Checker::check_program` → errors が空であることを assert
- `build_artifact` → `is_ok()` を assert

### `pattern_match_or_pattern`

- String に対して `"created" | "updated"` の Or-パターンを使うソースを定義
- `Parser::parse_str` → parse 成功を assert
- `Checker::check_program` → errors が空であることを assert
- `build_artifact` → `is_ok()` を assert

---

## Success Criteria

- [ ] `pattern_match_nested_record` テスト: parse + check + build がすべて成功
- [ ] `pattern_match_or_pattern` テスト: parse + check + build がすべて成功
- [ ] compiler.fav の `parse_arm_guard` が `TkIf` ガードを認識する
- [ ] `cargo test v705000` で 2 件 pass
- [ ] `cargo test` 全体で 3569 tests pass（0 failures）

---

## Error Codes

新規エラーコードなし

---

## Files to Modify

| ファイル | 変更内容 |
|---|---|
| `fav/src/driver.rs` | `v705000_tests` モジュール追加（2 テスト）|
| `fav/self/compiler.fav` | `parse_arm_guard` に `TkIf` 対応追加 |
| `fav/Cargo.toml` | `version` を `"70.4.0"` → `"70.5.0"` に更新 |
| `CHANGELOG.md` | v70.5.0 エントリ追加 |
| `versions/current.md` | 進行中バージョンを v70.5.0 に更新 |
| `site/content/docs/language/` | 既存のパターンマッチ MDX で十分（変更不要）|
