# v44.9.0 Plan — v45.0 前調整・安定化

## 前提

- 現行バージョン: `44.8.0`（2960 tests）
- 追加テスト数: 2 件（`cargo_toml_version_is_44_9_0` + `precision_and_flow_overview_doc_exists`）
- 目標テスト数: 2962
- ロードマップ推定（2950）は旧見積もり。実績 2960 を基準とする
- コードフリーズ: 新規 Rust 機能・ヘルパー関数・AST 変更なし

---

## 参考

- 既存 MDX テストパターン: `v44700_tests` モジュール内の `precision_and_flow_doc_exists`
  - `include_str!("../../site/content/docs/precision-and-flow.mdx")` + `.contains("Precision & Flow")`
- `precision-and-flow.mdx`（v44.7.0）: 機能別詳細説明（6 セクション + コードスニペット）
- `precision-and-flow-overview.mdx`（本バージョン）: スプリント俯瞰サマリー（完了一覧 + v45.0 への道筋）

---

## ステップ

### Step 1: `site/content/docs/precision-and-flow-overview.mdx` 作成

```mdx
---
title: Precision & Flow Overview
description: Favnir v44.x スプリント「Precision & Flow」の達成事項と v45.0 への道筋
---

# Precision & Flow Overview

Favnir v44.x スプリントのゴールは **「型推論・Refinement type・CEP・Opaque type・Back-pressure を統合し、最小限の注釈で安全なリアルタイムパイプラインを記述できる状態を宣言する」** ことです。

## 達成事項（v44.1〜v44.8）

| バージョン | 機能 | 状態 |
|---|---|---|
| v44.1.0 | Refinement type × Streaming 統合 | ✅ COMPLETE |
| v44.2.0 | CEP × Refinement type | ✅ COMPLETE |
| v44.3.0 | Stream join × Opaque type | ✅ COMPLETE |
| v44.4.0 | 型推論 × パイプライン lineage | ✅ COMPLETE |
| v44.5.0 | Back-pressure × `fav policy` 統合 | ✅ COMPLETE |
| v44.6.0 | Precision & Flow E2E デモ | ✅ COMPLETE |
| v44.7.0 | ドキュメントサイト Precision & Flow 概要ページ | ✅ COMPLETE |
| v44.8.0 | パフォーマンス最終調整 | ✅ COMPLETE |

## 詳細ドキュメント

- [Precision & Flow 機能詳細](./precision-and-flow) — 各機能のコードスニペット付き解説
- [E2E デモ](../../../infra/e2e-demo/precision-flow/) — 統合パイプラインの完全実装例

## v45.0 Precision & Flow 宣言へ

v44.9.0（本バージョン）はコードフリーズ版です。v45.0 で以下を宣言します。

> 「型推論がジェネリクスと戻り値型を補完し、最小限の注釈で安全なコードが書ける。
> ウィンドウ集計・CEP・Stream join が型安全に記述でき、
> refinement type と opaque type がデータの意味を型で守る。
>
> これが Favnir v45.0 — Precision & Flow の姿である。」
```

### Step 2: driver.rs — `v44900_tests` 追加 / スタブ化 / Cargo.toml

`v44800_tests` の直前（上の行）に挿入（driver.rs はバージョン降順配置）:

```rust
// -- v44900_tests (v44.9.0) -- v45.0 前調整・安定化 --
#[cfg(test)]
mod v44900_tests {
    #[test]
    fn cargo_toml_version_is_44_9_0() {
        let toml = include_str!("../Cargo.toml");
        assert!(toml.contains("version = \"44.9.0\""), "Cargo.toml version mismatch");
    }
    #[test]
    fn precision_and_flow_overview_doc_exists() {
        let src = include_str!("../../site/content/docs/precision-and-flow-overview.mdx");
        assert!(
            src.contains("Precision & Flow"),
            "site/content/docs/precision-and-flow-overview.mdx must exist and mention Precision & Flow"
        );
    }
}
```

スタブ化: `v44800_tests::cargo_toml_version_is_44_8_0` の `assert!` 行のみを削除し、以下に置き換える（`#[test]` アトリビュートと関数シグネチャは残す）:

```rust
// Stubbed: version bumped to 44.9.0 in v44.9.0.
```

`fav/Cargo.toml` version: `44.8.0` → `44.9.0`

### Step 3: CHANGELOG.md に v44.9.0 エントリ追加

### Step 4: テスト実行（2962 passed; 0 failed）

### Step 5: バージョン管理ドキュメント更新

- `versions/current.md` → v44.9.0、2962 tests、次版 v45.0.0
- `versions/roadmap/roadmap-v44.1-v45.0.md` → v44.9.0 を `✅ COMPLETE`
- `versions/v40-v45/v44.9.0/tasks.md` → COMPLETE

---

## 注意事項

- コードフリーズのため `collect_*` 系ヘルパーは追加しない
- `precision-and-flow-overview.mdx` に `"Precision & Flow"` が含まれることを確認（アサート条件）
- `v44800_tests` の `cargo_toml_version_is_44_8_0` のスタブ化対象を確認（存在するはず）
- ロードマップは `precision-and-flow-overview.mdx` を「更新」と記載しているが、ファイルが未存在のため新規作成となる（spec.md に明記済み）
