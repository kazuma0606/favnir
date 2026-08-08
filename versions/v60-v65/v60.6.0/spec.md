# v60.6.0 Spec — `fav explain-error` 全コード対応 + `long_description` フィールド追加

Date: 2026-07-31

---

## 概要

`error_catalog.rs` の `ErrorEntry` 構造体に `long_description: Option<&'static str>` フィールドを追加し、
全 97 エントリ（E0101〜E0384）に初期値を設定する。
`fav explain-error <CODE>` の出力に `long_description` セクションを追加し、
`fav generate-error-docs` コマンドで MDX コンテンツを生成する。

---

## 既存実装との関係

| 機能 | 実装状況 | 今バージョンの作業 |
|---|---|---|
| `fav explain-error <CODE>` | v24.5 実装済み（`cmd_explain_error` / `cmd_explain_error_collect`） | `long_description` セクション追加のみ |
| `ErrorEntry` 構造体 | v45.6 で `suggestion` フィールド追加済み | `long_description` フィールドを新規追加 |
| `fav generate-error-docs` | 未実装 | `cmd_generate_error_docs_str` / `cmd_generate_error_docs` を新規追加 |

---

## 変更ファイル

1. `fav/src/error_catalog.rs` — `ErrorEntry` 構造体 + 97 エントリ更新
2. `fav/src/driver.rs` — `cmd_explain_error_collect` 更新 + `cmd_generate_error_docs_str` 追加 + `v60600_tests` 追加
3. `fav/src/main.rs` — `generate-error-docs` CLI ディスパッチ追加

---

## `ErrorEntry` 構造体変更（`error_catalog.rs`）

`suggestion` フィールドの直前に追加する。

```rust
pub struct ErrorEntry {
    pub code: &'static str,
    pub title: &'static str,
    pub category: &'static str,
    pub description: &'static str,
    pub example: &'static str,
    pub fix: &'static str,
    /// v60.6.0: Markdown 形式の詳細説明
    pub long_description: Option<&'static str>,
    /// v45.6.0: static suggestion text shown with `fav explain <code>`.
    pub suggestion: Option<&'static str>,
}
```

---

## 全 97 エントリへの `long_description` 追加（一括戦略）

`ERROR_CATALOG` の全 97 エントリは `suggestion:` フィールドを持つ。
`replace_all: true` で `suggestion:` の直前に `long_description:` を挿入することで一括追加する。

初期値として以下のプレースホルダーを使用する（全エントリ共通）:
```rust
long_description: Some("See `fix` field for remediation details."),
```

テスト `explain_error_all_codes_have_long_desc` が `all(|e| e.long_description.is_some())` を
検証するため、全エントリが `Some(...)` である必要がある。

**実態確認**: 97 エントリすべてが `suggestion: Some(...)` パターンであり `suggestion: None,` のエントリは 0 件。
したがって一括追加は `suggestion: Some(` の 1 パターンのみの置換で完了する。

---

## `cmd_explain_error_collect` 更新（`driver.rs`）

`long_description` が `Some` の場合に "Long Description" セクションを追加する。

```rust
if let Some(ld) = e.long_description {
    out.push('\n');
    out.push_str("  Long Description\n");
    for line in ld.lines() {
        out.push_str(&format!("    {}\n", line));
    }
}
```

挿入位置: `suggestion` セクションの直前（または直後でも可）。

---

## `cmd_generate_error_docs_str` 追加（`driver.rs`）

MDX コンテンツを String として返す（テスト容易性のため）。
MDX の出力形式:

```
# E0101: undefined seq step / stage

A stage referenced in a seq definition does not exist.

See `fix` field for remediation details.

...
```

```rust
pub fn cmd_generate_error_docs_str() -> String {
    let mut out = String::new();
    for e in crate::error_catalog::list_all() {
        out.push_str(&format!("# {}: {}\n\n", e.code, e.title));
        out.push_str(&format!("{}\n\n", e.description));
        if let Some(ld) = e.long_description {
            out.push_str(&format!("{}\n\n", ld));
        }
        out.push_str(&format!("**Fix**: {}\n\n", e.fix));
    }
    out
}

pub fn cmd_generate_error_docs(out_dir: &str) {
    let content = cmd_generate_error_docs_str();
    let path = format!("{}/errors-all.mdx", out_dir.trim_end_matches('/'));
    match std::fs::write(&path, &content) {
        Ok(_) => println!("generated: {}", path),
        Err(e) => eprintln!("error: {}", e),
    }
}
```

追加位置: `cmd_explain_error_list_json` の直後。

---

## `main.rs` CLI ディスパッチ追加

`Some("explain-error")` ブロックの直後に追加:

```rust
Some("generate-error-docs") => {
    let out_dir = args.get(2).map(|s| s.as_str()).unwrap_or("site/content/docs/errors");
    cmd_generate_error_docs(out_dir);   // use driver::{..., cmd_generate_error_docs} 経由
}
```

---

## テスト

対象ファイル: `fav/src/driver.rs`

テスト数: ベース **3340** + 2 = **3342** tests passed, 0 failed

テストモジュール名: `v60600_tests`（`v60500_tests` の直前に挿入）

### `explain_error_all_codes_have_long_desc`

全エントリが `long_description.is_some()` であることを確認。

```rust
#[test]
fn explain_error_all_codes_have_long_desc() {
    // long_description が全エントリに設定されていることを確認
    let all = crate::error_catalog::list_all();
    assert!(!all.is_empty(), "ERROR_CATALOG must not be empty");
    for entry in all {
        assert!(
            entry.long_description.is_some(),
            "entry {} has no long_description", entry.code
        );
        let ld = entry.long_description.unwrap();
        assert!(!ld.is_empty(), "long_description for {} is empty", entry.code);
    }
}
```

### `generate_error_docs_contains_all_codes`

`cmd_generate_error_docs_str` が全エラーコードを含む MDX コンテンツを生成することを確認。
（テスト関数名を `cmd_generate_error_docs` と区別するため `generate_error_docs_contains_all_codes` を使用）

```rust
#[test]
fn generate_error_docs_contains_all_codes() {
    let out = cmd_generate_error_docs_str();
    assert!(!out.is_empty(), "generate-error-docs output should not be empty");
    // 全エントリのコードが出力に含まれる
    for entry in crate::error_catalog::list_all() {
        assert!(
            out.contains(entry.code),
            "output should contain error code {}", entry.code
        );
    }
}
```

---

## 注意事項

- `Cargo.toml` version は `"60.0.0"` のまま変更しない
- `long_description` の初期値は `Some("See \`fix\` field for remediation details.")` — テストが `is_some()` を要求するため
- `v60600_tests` は `v60500_tests` の直前（上側）に挿入する
- `cmd_generate_error_docs_str` は `pub fn` とする（テストモジュールから `use super::*` でアクセスするため）
- テスト実行: `cargo test -j 8 -- --test-threads=8`
- 将来注記: v62.8.0 の E0427 / v63.3.0 の E0428 を登録する際も `long_description` フィールドを必ず含めること（ロードマップ記載）
