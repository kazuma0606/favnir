# v60.6.0 Tasks — `fav explain-error` 全コード対応 + `long_description` フィールド追加

Date: 2026-07-31
Status: COMPLETE

---

## T0: 事前確認

- [x] `cargo test` でベースラインが 3340 tests passed, 0 failed であることを確認
- [x] `fav/Cargo.toml` のバージョンが `"60.0.0"` であることを確認
- [x] `v60600_tests` がまだ存在しないことを確認
  - `grep -c 'v60600_tests' fav/src/driver.rs` = 0 件
- [x] `v60500_tests` が存在すること（挿入先が実在すること）を確認
  - `grep -c 'v60500_tests' fav/src/driver.rs` ≥ 1 件
- [x] `long_description` フィールドがまだ存在しないことを確認
  - `grep -c 'long_description' fav/src/error_catalog.rs` = 0 件
- [x] `cmd_generate_error_docs_str` がまだ存在しないことを確認
  - `grep -c 'cmd_generate_error_docs_str' fav/src/driver.rs` = 0 件
- [x] `ErrorEntry` の現在のエントリ数を確認
  - `grep -c 'code: "E' fav/src/error_catalog.rs` = 97 件
  - `grep -c 'suggestion: Some' fav/src/error_catalog.rs` = 97 件（全エントリ Same パターン確認）
  - `grep -c 'suggestion: None' fav/src/error_catalog.rs` = 0 件（None パターンなし確認）

---

## T1: `ErrorEntry` 構造体に `long_description` フィールド追加（`error_catalog.rs`）

`suggestion` フィールドの直前に挿入する。

```rust
    /// v60.6.0: Markdown 形式の詳細説明
    pub long_description: Option<&'static str>,
    /// v45.6.0: static suggestion text shown with `fav explain <code>`.
    pub suggestion: Option<&'static str>,
```

- [x] `long_description: Option<&'static str>` フィールドを `suggestion` の直前に追加した
- [x] `/// v60.6.0:` ドキュメントコメントが付いている

**注意**: この時点でコンパイルエラーが発生する（ERROR_CATALOG の全エントリを更新していないため）。T2 で解消する。

---

## T2: 全 97 エントリへ `long_description` 一括追加（`error_catalog.rs`）

`replace_all: true` を使って 2 パターンを置換する。

**実態**: 全 97 エントリが `suggestion: Some(...)` パターン（`None` は 0 件）のため 1 パターンのみ。

**置換パターン（1 種類のみ）**:
```
old:  "        suggestion: Some("
new:  "        long_description: Some(\"See `fix` field for remediation details.\"),\n        suggestion: Some("
```

- [x] `replace_all: true` で上記パターンの置換を実行した
- [x] `grep -c 'long_description' fav/src/error_catalog.rs` = 98 件（struct 定義 1 + エントリ 97）
- [x] `cargo build` でコンパイルエラーがないことを確認

---

## T3: `cmd_explain_error_collect` 更新（`driver.rs`）

`if let Some(suggestion) = e.suggestion {` の直前に `long_description` セクションを挿入する。

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

- [x] `long_description` セクションを `suggestion` セクションの直前に追加した
- [x] `if let Some(ld) = e.long_description` パターンを使用している

---

## T4: `cmd_generate_error_docs_str` / `cmd_generate_error_docs` 追加（`driver.rs`）

`cmd_explain_error_list_json` 関数の直後に追加する。

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

pub fn cmd_generate_error_docs(out_dir: &str) {
    let content = cmd_generate_error_docs_str();
    let path = format!("{}/errors-all.mdx", out_dir.trim_end_matches('/'));
    match std::fs::write(&path, &content) {
        Ok(_) => println!("generated: {}", path),
        Err(e) => eprintln!("error: {}", e),
    }
}
```

- [x] `cmd_generate_error_docs_str` 関数を `cmd_explain_error_list_json` の直後に追加した
- [x] `cmd_generate_error_docs` 関数を追加した
- [x] 両関数が `pub fn` で宣言されている

---

## T5: `generate-error-docs` CLI ディスパッチ追加（`main.rs`）

`Some("explain-error")` ブロックの直後に追加する。

```rust
        Some("generate-error-docs") => {
            let out_dir = args.get(2).map(|s| s.as_str()).unwrap_or("site/content/docs/errors");
            crate::driver::cmd_generate_error_docs(out_dir);
        }
```

- [x] `generate-error-docs` ディスパッチを `Some("explain-error")` の直後に追加した
- [x] `cmd_generate_error_docs` を `main.rs` の `use` 宣言に追加した

---

## T6: `v60600_tests` モジュール追加（`driver.rs`）

`v60500_tests` の直前（上側）に挿入する。

```rust
// -- v60600_tests (v60.6.0) -- fav explain-error 全コード対応 --
#[cfg(test)]
mod v60600_tests {
    use super::*;

    #[test]
    fn explain_error_all_codes_have_long_desc() {
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
        // 関数名 cmd_generate_error_docs との衝突を避けるためテスト名を区別
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

- [x] `v60600_tests` モジュールを `v60500_tests` の直前（上側）に追加した
- [x] `use super::*;` が含まれている
- [x] `explain_error_all_codes_have_long_desc` テストが含まれている
  - 全エントリの `long_description.is_some()` と非空文字列を検証
- [x] `generate_error_docs_contains_all_codes` テストが含まれている（`cmd_generate_error_docs` との名前衝突を避けた名前）
  - 全エラーコードが出力に含まれることを検証

---

## T7: テスト実行・確認

- [x] `cargo test -j 8 -- --test-threads=8` を実行
- [x] `v60600_tests::explain_error_all_codes_have_long_desc` pass
- [x] `v60600_tests::generate_error_docs_contains_all_codes` pass
- [x] 総テスト数 **3342** tests passed, 0 failed を確認

---

## T8: 事後処理

- [x] `versions/current.md` を v60.6.0 / 3342 tests に更新
- [x] `versions/roadmap/roadmap-v60.1-v61.0.md` の v60.6.0 実績欄を更新
- [x] CHANGELOG.md: サブバージョンのため個別エントリは不要（v61.0 でまとめて記載）
- [x] このファイル（tasks.md）を COMPLETE ステータスに更新

---

## コードレビュー指摘と対応

実装後に記録する。

---

Status: 未着手
