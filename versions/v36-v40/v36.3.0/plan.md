# v36.3.0 実装計画 — W025 `schema_mismatch` lint ルール

## 実装順序

| ステップ | 対象 | 内容 |
|---|---|---|
| S1 | `CHANGELOG.md` | `## [v36.3.0]` エントリを追加（`## [v36.2.0]` の直後） |
| S2 | `fav/src/lint.rs` | `collect_schema_fields` / `collect_field_accesses_*` / `check_w025_schema_mismatch` 追加 |
| S3 | `fav/src/lint.rs` | `lint_program` から `check_w025_schema_mismatch` を呼び出す行を追加 |
| S4 | `fav/src/driver.rs` | `v36200_tests::cargo_toml_version_is_36_2_0` をスタブ化 |
| S5 | `fav/src/driver.rs` | `v36300_tests` モジュール（5 件）を追加 |
| S6 | `fav/Cargo.toml` | バージョンを `36.2.0` → `36.3.0` に更新（必ず **S2・S3・S4・S5 すべて完了後**） |
| S7 | `cargo test` | 全通過確認（≥ 2671 件） |

## 各ステップの詳細

### S1: CHANGELOG.md

`## [v36.2.0]` の `---` セパレータの直後に挿入（実装当日の日付を記入）:

```markdown
## [v36.3.0] — 2026-07-08

### Added
- W025 `schema_mismatch` lint ルール — スキーマ定義に存在しないフィールドアクセスを警告
- `check_w025_schema_mismatch` — `lint.rs` に追加
- `collect_schema_fields` / `collect_field_accesses_*` ヘルパー関数群

---
```

### S2: lint.rs — W025 実装

`lint.rs` ファイル末尾（W021 の後）に追加するコードは spec.md §1 を参照。

**注意: `LintError` の生成方法の確認**

他の W コード（例: W020）の `errors.push(...)` 呼び出しを参照し、
`LintError` の正確なコンストラクタ / フィールド名に合わせること。

例（W020 の場合）:
```rust
errors.push(LintError {
    code: "W020".to_string(),
    message: format!("..."),
    span: span.clone(),
});
```

フィールドが `code`/`message`/`span` でない場合、`LintError::new()` を使用する。

**注意: `Expr::FieldAccess` の正確な variant 名**

`ast.rs` で `FieldAccess` variant を確認してから実装すること:
```rust
Expr::FieldAccess(obj, field, span) => { ... }
```

**注意: `Expr::Lambda` の正確な variant 名**

`Lambda` の body フィールドが `Box<Expr>` か `Block` かを `ast.rs` で確認すること。

### S3: lint_program への呼び出し追加

`lint.rs` の `lint_program` 関数内、`check_w021_pure_fn_calls_effectful` の呼び出し行の後:

```rust
// v24.6.0: W021
check_w021_pure_fn_calls_effectful(program, &mut errors);
// v36.3.0: W025
check_w025_schema_mismatch(program, &mut errors);
```

### S4: driver.rs — スタブ化

`v36200_tests::cargo_toml_version_is_36_2_0` のアサーションを空実装に置き換え。
`#[test]` アノテーションは**残す**こと（テスト関数として登録されたまま中身のみ無効化）:

```rust
#[test]
fn cargo_toml_version_is_36_2_0() {
    // stubbed: version bumped to 36.3.0
}
```

### S5: driver.rs — v36300_tests モジュール追加

`v36200_tests` の閉じ `}` の後に追加。spec.md §3 のコードを参照。

### S6: Cargo.toml バージョン更新

**必ず S2・S3・S4・S5 すべて完了後に実行すること**。

`version = "36.2.0"` → `version = "36.3.0"`

### S7: cargo test

期待値: 2666（現在）+ 5（v36300_tests）= **2671 件** pass、0 failures

## 実装上の重要チェックポイント

### `LintError` 構造体の確認方法

```bash
grep -n "struct LintError\|pub code\|pub message\|pub span\|fn new" fav/src/lint.rs | head -20
```

### `Expr::FieldAccess` の確認方法

```bash
grep -n "FieldAccess" fav/src/ast.rs | head -10
```

### `Expr::Closure` の確認方法（v36.1.0 確認済: `Lambda` ではなく `Closure`）

```bash
grep -n "Closure\|Lambda" fav/src/ast.rs | head -10
```

> **注意**: Favnir AST の lambda は `Expr::Closure(Vec<String>, Box<Expr>, Span)` — `Lambda` variant は存在しない。

### `Expr::Block` の確認方法

```bash
grep -n "Expr::Block\b" fav/src/ast.rs | head -5
```

## W025 が発行されるケース・されないケース

| ケース | W025 発行 |
|---|---|
| `fn f(row: Orders) -> Int { row.nonexistent }` + `schema Orders { id: Int }` | ✅ 発行 |
| `fn f(row: Orders) -> Int { row.id }` + `schema Orders { id: Int }` | ❌ 発行しない |
| `fn f(x: Int) -> Int { x }` （schema パラメータなし） | ❌ 発行しない |
| `schema` 定義が存在しないプログラム | ❌ 発行しない |
| `fn f(rows: List<Orders>) -> Int { 0 }` （`List<T>` 型のパラメータ） | ❌ 発行しない（v36.3 スコープ外） |
