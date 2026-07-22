# Spec: v50.9.0 — 安定化・コードフリーズ（DX 3.0 前調整）

## 概要

v50.8.0 で DX 3.0 個別機能のドキュメントが揃った。本バージョンでは
**全 lint / clippy クリーンを確認**し、v51.0.0 の宣言に向けた統合概要ページ
`site/content/docs/dx3-overview.mdx` を作成する。

---

## 背景・目的

DX 3.0 スプリント（v50.1〜v50.8）で実装した機能の全体像を一枚のページにまとめる。

| バージョン | 実装内容 |
|---|---|
| v50.1.0 | 全エラーコードに `suggestion` 追加 |
| v50.2.0 | JSON / LSP / CLI の `suggestion` 一貫出力 |
| v50.3.0 | `fav explain --error` 正式導線 |
| v50.4.0 | LSP インレイヒント — 変数・関数戻り型 |
| v50.5.0 | LSP インレイヒント — パイプライン stage 型 |
| v50.6.0 | LSP ホバー — Rune メソッドシグネチャ |
| v50.7.0 | `fav run --trace` 構造化ログ / `--watch` フィールド追跡 |
| v50.8.0 | ドキュメントサイト DX 3.0 記事（diagnostics / trace-watch） |

---

## 成果物仕様

### `site/content/docs/dx3-overview.mdx`

**ファイルパス**: `site/content/docs/dx3-overview.mdx`

**必須コンテンツ:**
- H1: `# Developer Experience 3.0 — 概要`
- DX 3.0 の目標（「診断を統一し、エディタ統合を深め、開発者の思考を止めない体験」）
- 実装済み機能の概要テーブル（バージョン × 機能 × 詳細リンク）
- 各機能へのリンク（diagnostics.mdx / trace-watch.mdx / lsp.mdx）
- キーワード: `DX 3.0` または `Developer Experience` または `dx3`（テスト用）

**最小文字数**: 300 文字以上

---

## テスト仕様

### `cargo_toml_version_is_50_9_0`

```rust
let content = include_str!("../Cargo.toml");
assert!(content.contains("version = \"50.9.0\""),
    "Cargo.toml version should be 50.9.0");
```

### `dx3_overview_doc_exists`

```rust
let content = include_str!("../../site/content/docs/dx3-overview.mdx");
assert!(content.len() >= 300, "dx3-overview.mdx is too short: {} bytes", content.len());
assert!(
    content.contains("DX 3.0") || content.contains("Developer Experience") || content.contains("dx3"),
    "dx3-overview.mdx must mention DX 3.0"
);
```

---

## バージョン要件

- `fav/Cargo.toml` version: `50.9.0`
- テスト数: 3107 → **3109**（純増 +2）
  - `v509000_tests` 3 件追加（`cargo_toml_version_is_50_9_0` + `dx3_overview_doc_exists` + `code_freeze_v50_9_0`）
  - `v508000_tests::cargo_toml_version_is_50_8_0` 1 件削除（前バージョン version assertion 削除の慣例）
  - 純増: +3 - 1 = **+2**

> **補足**: ロードマップ記載「Rust テスト 2 件」は `cargo_toml_version_is_50_9_0` + `dx3_overview_doc_exists` のカウント。慣例どおりバージョン assertion のペア（追加1・削除1）を含む合計 3 件を実装し純増 +2 を達成する。

---

## 完了条件

- `site/content/docs/dx3-overview.mdx` が存在し、300 文字以上
- `cargo test` 3109 tests passed, 0 failed
- `cargo clippy -- -D warnings` クリーン（ゼロ警告）
- `v509000_tests` 3 件 pass:
  - `cargo_toml_version_is_50_9_0`
  - `dx3_overview_doc_exists`
  - `code_freeze_v50_9_0`（Cargo.toml に `"50.9.0"` が含まれることをコードフリーズ宣言として assert）
- `CHANGELOG.md` に v50.9.0 エントリ追加
- `versions/current.md` を v50.9.0（3109 tests）に更新

---

## ロードマップ対応

roadmap-v50.1-v51.0.md v50.9.0 より:

> 全 lint / clippy クリーン確認。`site/content/docs/dx3-overview.mdx` 骨子作成
> （統一診断・インレイヒント・trace/watch の概要）。
