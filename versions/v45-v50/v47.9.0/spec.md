# Spec: v47.9.0 — stdlib ドキュメント + v48.0 前調整

## 概要

v47.1〜v47.8 で追加した標準ライブラリ関数のドキュメントを `site/content/docs/stdlib/` に追加・更新する。
新規ファイル 2 件（`float.mdx`・`v2.mdx`）を作成し、既存ファイル 3 件（`list.mdx`・`string.mdx`・`map.mdx`）を更新。
Rust テストを 2 件追加して v48.0 前のコードフリーズを行う。

---

## 成果物

### 新規作成

| ファイル | 内容 |
|---|---|
| `site/content/docs/stdlib/float.mdx` | `Float.round` / `Float.clamp` / `Float.abs` / `Int.to_hex` / `Int.abs` のリファレンス |
| `site/content/docs/stdlib/v2.mdx` | Standard Library 2.0 概要ページ（全追加関数の索引）|

### 更新

| ファイル | 追加内容 |
|---|---|
| `site/content/docs/stdlib/list.mdx` | `zip` / `chunk` / `flat_map` / `group_by` / `dedupe` / `scan` / `take_while` / `drop_while` |
| `site/content/docs/stdlib/string.mdx` | `pad_left` / `trim_start` / `repeat` |
| `site/content/docs/stdlib/option.mdx` | `map` / `unwrap_or` / `and_then` / `is_some` / `is_none` |
| `site/content/docs/stdlib/result.mdx` | `map` / `map_err` / `and_then` / `is_ok` / `is_err` |
| `site/content/docs/stdlib/map.mdx` | `merge` / `filter_values` / `map_values` |
| `site/content/cookbook/stdlib-v2.mdx` | v47 シリーズ新関数を使ったサンプルパイプライン（cookbook 更新） |

> **検証方針**: `float.mdx` と `v2.mdx` の内容は Rust テストで確認。
> list / string / option / result / map / cookbook の更新内容は目視確認（テスト対象外）。
> frontmatter（`title` / `order` / `category`）の必須フィールドは目視確認。

---

## テスト（+2）

| テスト名 | 内容 |
|---|---|
| `stdlib_v2_doc_exists` | `site/content/docs/stdlib/float.mdx` に `"Float.round"` が含まれるか確認 |
| `stdlib_v2_overview_exists` | `site/content/docs/stdlib/v2.mdx` に `"Standard Library 2.0"` が含まれるか確認 |

### テストコード

```rust
#[test]
fn stdlib_v2_doc_exists() {
    let content = include_str!("../../site/content/docs/stdlib/float.mdx");
    assert!(content.contains("Float.round"), "float.mdx should document Float.round");
}

#[test]
fn stdlib_v2_overview_exists() {
    let content = include_str!("../../site/content/docs/stdlib/v2.mdx");
    assert!(content.contains("Standard Library 2.0"), "v2.mdx should mention Standard Library 2.0");
}
```

---

## 完了条件

- `site/content/docs/stdlib/float.mdx` に `Float.round` / `Float.clamp` / `Float.abs` / `Int.to_hex` / `Int.abs` が記載されている
- `site/content/docs/stdlib/v2.mdx` に `"Standard Library 2.0"` が含まれている
- `list.mdx` / `string.mdx` / `map.mdx` に v47 シリーズで追加した関数が追記されている
- `cargo test` 3041 passed, 0 failed（3039 + 2 件）
- `cargo clippy -- -D warnings` クリーン
- `fav/Cargo.toml` version → `"47.9.0"`
- `CHANGELOG.md` に v47.9.0 エントリ追加
- `versions/current.md` を v47.9.0（3041 tests）に更新、進行中バージョンを `v48.0.0` に更新
- `tasks.md` を COMPLETE に更新（T0〜T2 全 `[x]`）
