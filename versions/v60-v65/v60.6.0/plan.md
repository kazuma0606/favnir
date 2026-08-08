# v60.6.0 Plan — `fav explain-error` 全コード対応 + `long_description` フィールド追加

Date: 2026-07-31

---

## 実装方針

4 ファイル変更（struct 追加・97 エントリ一括更新・`driver.rs` 関数追加・`main.rs` ディスパッチ）と
`v60600_tests` モジュール追加を行う。

---

## ステップ詳細

### Step 1: `ErrorEntry` 構造体に `long_description` フィールド追加（`error_catalog.rs`）

`suggestion` フィールドの直前に挿入する。

```rust
    /// v60.6.0: Markdown 形式の詳細説明
    pub long_description: Option<&'static str>,
    /// v45.6.0: static suggestion text shown with `fav explain <code>`.
    pub suggestion: Option<&'static str>,
```

**注意**: フィールド追加後は `ERROR_CATALOG` の全 97 エントリが `long_description` を持たないため
コンパイルエラーになる。Step 2 で一括追加するまでコンパイルは通らない。

### Step 2: 全 97 エントリへ `long_description` を一括追加（`error_catalog.rs`）

`ERROR_CATALOG` の全エントリは `suggestion:` フィールドを持つ（`Some(...)` または `None,`）。
`replace_all: true` を使い `        suggestion:` の直前に `long_description:` を挿入する。

**実態**: 全 97 エントリがすべて `suggestion: Some(...)` パターンであり `suggestion: None,` は 0 件。
（`grep -c 'suggestion: Some' fav/src/error_catalog.rs` = 97 件で確認済み）

**置換パターン（1 種類のみ）**:

```
old:  "        suggestion: Some("
new:  "        long_description: Some(\"See `fix` field for remediation details.\"),\n        suggestion: Some("
```

**確認**: `grep -c 'long_description' fav/src/error_catalog.rs` = 98 件（struct 定義 1 + エントリ 97）

### Step 3: `cmd_explain_error_collect` 更新（`driver.rs`）

`suggestion` セクションの直前に `long_description` セクションを挿入する。

対象: `driver.rs` L17951 付近（`if let Some(suggestion) = e.suggestion {` の直前）

```rust
        if let Some(ld) = e.long_description {
            out.push('\n');
            out.push_str("  Long Description\n");
            for line in ld.lines() {
                out.push_str(&format!("    {}\n", line));
            }
        }
        if let Some(suggestion) = e.suggestion {
```

### Step 4: `cmd_generate_error_docs_str` / `cmd_generate_error_docs` 追加（`driver.rs`）

`cmd_explain_error_list_json` 関数（L17979 付近）の直後に追加する。

```rust
/// v60.6.0: 全エラーコードの MDX コンテンツを String として返す（テスト容易性のため）
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

/// v60.6.0: `fav generate-error-docs [<out_dir>]` — MDX コンテンツをファイルに書き出す
pub fn cmd_generate_error_docs(out_dir: &str) {
    let content = cmd_generate_error_docs_str();
    let path = format!("{}/errors-all.mdx", out_dir.trim_end_matches('/'));
    match std::fs::write(&path, &content) {
        Ok(_) => println!("generated: {}", path),
        Err(e) => eprintln!("error: {}", e),
    }
}
```

### Step 5: `generate-error-docs` CLI ディスパッチ追加（`main.rs`）

`Some("explain-error")` ブロック（L2275 付近）の直後に追加する。

```rust
        Some("generate-error-docs") => {
            let out_dir = args.get(2).map(|s| s.as_str()).unwrap_or("site/content/docs/errors");
            cmd_generate_error_docs(out_dir);   // use driver::{..., cmd_generate_error_docs} 経由
        }
```

また `main.rs` の `use` 宣言に `cmd_generate_error_docs` を追加（driver.rs import リストの末尾）。

### Step 6: `v60600_tests` モジュール追加（`driver.rs`）

`v60500_tests` の直前（上側）に挿入する。

```rust
// -- v60600_tests (v60.6.0) -- fav explain-error 全コード対応 --
#[cfg(test)]
mod v60600_tests {
    use super::*;

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

    #[test]
    fn generate_error_docs_contains_all_codes() {
        // cmd_generate_error_docs_str が全エラーコードを含む MDX コンテンツを生成する
        // （関数名 cmd_generate_error_docs との衝突を避けるためテスト名を区別）
        let out = cmd_generate_error_docs_str();
        assert!(!out.is_empty(), "generate-error-docs output should not be empty");
        for entry in crate::error_catalog::list_all() {
            assert!(
                out.contains(entry.code),
                "output should contain error code {}", entry.code
            );
        }
    }
}
```

---

## 注意事項

- Step 1（struct 変更）とStep 2（エントリ一括追加）はセットで実行しないとコンパイルエラーになる
- `replace_all: true` の対象パターンに注意: `suggestion: Some(` と `suggestion: None,` の 2 パターンが存在する
- `cmd_generate_error_docs_str` は `pub fn` とすること（テストから `use super::*` でアクセスするため）
- `main.rs` の `use driver::{...}` インポートリストに `cmd_generate_error_docs` を追加するのを忘れずに
- テスト実行: `cargo test -j 8 -- --test-threads=8`

---

## テスト数推移

| バージョン | テスト数 | 増加 |
|---|---|---|
| v60.5.0（ベース） | 3340 | — |
| v60.6.0 | 3342 | +2 |
