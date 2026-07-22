# Plan: v50.1.0 — エラー診断統一 Phase 1（全コード suggestion 補完）

Date: 2026-07-18

---

## 実装方針

### Step 1: `error_catalog.rs` の `suggestion: None` 全件を確認・分類

```bash
grep -n "suggestion: None" fav/src/error_catalog.rs
```

34 件の行番号と前後のエラーコード定義を確認し、各コードに適切な suggestion テキストを決める。
エラーコードは各エントリの直前にある `code: "E0XXX"` 行で識別する。

主な対象と提案文の方針:

| コード系統 | suggestion 方針 |
|---|---|
| E0004〜E0012 | 型・スコープ・引数関連 → 具体的な修正アクションを提示 |
| E0314〜E0319 | Snowflake / effect 系 → 接続設定・エフェクト宣言の確認を促す |
| E0370〜E0419 | モジュール・import・セキュリティ系 → 構文例・パス確認を提示 |

### Step 2: `error_catalog.rs` の全 34 件を一括更新

各 `suggestion: None` を `suggestion: Some("...")` に置き換える。
テキスト��簡潔な英語・命令調（既存 suggestion との文体統一）。

例（E0018 duplicate bind target）:
```rust
// ��更前
suggestion: None,

// 変更後
suggestion: Some("Use a different variable name, or remove the earlier binding if it is no longer needed."),
```

### Step 3: `v501000_tests` モジュール追加

`v50000_tests` の直前に挿入する。

```rust
// -- v501000_tests (v50.1.0) -- エラー診断統一 Phase 1 --
#[cfg(test)]
mod v501000_tests {
    #[test]
    fn error_suggestion_all_covered() {
        let content = include_str!("error_catalog.rs");
        assert!(
            !content.contains("suggestion: None"),
            "error_catalog.rs must have no suggestion: None — run grep to find remaining entries"
        );
    }

    #[test]
    fn error_suggestion_e0018_text() {
        let content = include_str!("error_catalog.rs");
        assert!(
            content.contains("no longer needed"),
            "E0018 suggestion should contain 'no longer needed'"
        );
    }
}
```

> `error_suggestion_e0018_text` の assert 文字列は Step 2 で E0018 に設定した suggestion テキストに合わせ��調整する。

### Step 4: バージョン更新・完了

順序を守ること:
1. `fav/Cargo.toml` version → `"50.1.0"`
2. `cargo test` 3093 passed 確認
3. `cargo clippy -- -D warnings` クリーン確認
4. `CHANGELOG.md` に v50.1.0 エントリ追加
5. `versions/current.md` 更新
6. `versions/roadmap/roadmap-v50.1-v51.0.md` の v50.1.0 実績を記入

---

## 注意事項

- `include_str!("error_catalog.rs")`: `driver.rs` と同一ディレクトリ（`fav/src/`）への参照。
  `fav/src/driver.rs` と `fav/src/error_catalog.rs` は同階層のため、ファイル名のみで解決できる。
- `error_suggestion_all_covered` は文��列マッチ方式（`"suggestion: None"` が含まれないこと）で実装する。
  `suggestion: None` という文字列がコメントや文字列リテラル内に含まれていないことを前提とする。
- suggestion テキストは既存エントリ（`"Check that the type is defined before it is used."` 等）の
  文体（英語・三人称禁止・命令調）に揃える。
- `error_suggestion_e0018_text` の assert 文字列は実装後に E0018 の実際の suggestion テキストに合わせて確定する。
