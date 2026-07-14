# v43.13.0 Plan — Language Expressiveness cookbook + 安定化

## 前提

- 現行バージョン: `43.12.0`（2935 tests）
- 追加テスト数: 2 件（MDX 存在確認のみ — `cargo_toml_version_is_43_13_0` は追加しない）
- 目標テスト数: 2937

---

## ステップ

### Step 1: `site/content/cookbook/type-inference-guide.mdx` 作成

```mdx
---
title: "Type Inference Guide"
description: "Favnir の型推論を活用して冗長な型注釈を省略する方法"
---

# Type Inference Guide

Favnir v43 では 6 カテゴリの型推論をサポートしています。
...
```

コード例を含む 40〜60 行程度の MDX。

### Step 2: `site/content/docs/language/type-inference.mdx` 作成

`docs/language/` 配下の他 MDX（generics.mdx 等）に倣う形式で作成。
型推論の 6 カテゴリすべてを解説するリファレンス文書。

**テスト対象外**（`include_str!` テストは追加しない）。

### Step 3: `site/content/docs/language-expressiveness.mdx` 作成

v43.1〜v43.13 スプリントのサマリー文書。v44.0.0 の宣言文の下準備。

### Step 4: driver.rs に `v431300_tests` 追加 / スタブ化 / Cargo.toml

`v431200_tests` の直前に挿入（2 件のみ — `cargo_toml_version_is_43_13_0` は追加しない）:

```rust
// -- v431300_tests (v43.13.0) -- Language Expressiveness cookbook + 安定化 --
#[cfg(test)]
mod v431300_tests {
    #[test]
    fn type_inference_guide_mdx_exists() {
        let content = include_str!("../../site/content/cookbook/type-inference-guide.mdx");
        assert!(!content.is_empty(), "type-inference-guide.mdx must not be empty");
    }
    #[test]
    fn language_expressiveness_doc_exists() {
        let content = include_str!("../../site/content/docs/language-expressiveness.mdx");
        assert!(!content.is_empty(), "language-expressiveness.mdx must not be empty");
    }
}
```

スタブ化: `v431200_tests::cargo_toml_version_is_43_12_0` を空ボディに置き換え、コメント `// Stubbed: version bumped to 43.13.0 in v43.13.0.` を追加。

### Step 5: Cargo.toml version bump 43.12.0 → 43.13.0

### Step 6: CHANGELOG.md に v43.13.0 エントリ追加

### Step 7: テスト実行（2937 passed; 0 failed）

### Step 8: バージョン管理ドキュメント更新

---

## include_str! パス

`fav/src/driver.rs` から `../../` = リポジトリルート（`favnir/`）を経由して MDX を参照:
- `../../site/content/cookbook/type-inference-guide.mdx`
- `../../site/content/docs/language-expressiveness.mdx`

（`site/content/docs/language/type-inference.mdx` は include_str! テスト対象外）

## 注意事項

- コードフリーズ: Rust ソースへの変更は driver.rs（テスト 2 件追加・スタブ化）と Cargo.toml のみ
- `cargo_toml_version_is_43_13_0` は**追加しない**（テスト数 2935 + 2 = 2937 を維持するため）
- MDX の内容は実質的に任意（ただし空でないこと）
