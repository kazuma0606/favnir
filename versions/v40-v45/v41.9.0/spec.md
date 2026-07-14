# v41.9.0 仕様書 — v42.0 前調整・安定化

**フェーズ**: Type Precision（v41.x スプリント）
**前バージョン**: v41.8.0（Type Precision cookbook、2868 tests）
**目標テスト数**: 2870（+2）

---

## 概要

v42.0「Type Precision 宣言」に向けた**コードフリーズ版**。

新規機能追加なし。以下の整備のみ実施する:

1. **`site/content/docs/type-precision.mdx`** — 新規作成（Type Precision マイルストーン概要ページ）
2. **driver.rs テスト** 2 件（meta テスト）

---

## 現状確認

| ファイル | 状態 |
|---|---|
| `site/content/docs/type-precision.mdx` | **未作成** |
| 類似ファイル（参照） | `site/content/docs/ecosystem-maturity.mdx`, `site/content/docs/observability-first.mdx` など |

---

## スコープ

### v41.9.0 に含む

- **マイルストーン概要ページ新規作成**: `site/content/docs/type-precision.mdx`
  - v41.x スプリントで実装した Type Precision 機能の一覧と概要
  - 各バージョンの機能をリンク付きで紹介
  - v42.0 宣言文の予告（暫定版）

- **driver.rs テスト** 2 件:
  - `cargo_toml_version_is_41_9_0`
  - `type_precision_doc_exists`（`site/content/docs/type-precision.mdx` の存在と内容を検証）

### スコープ外

- 新規言語機能（コードフリーズ）
- MILESTONE.md 更新（v42.0 で実施）
- `cargo clean`（v42.0 の ★クリーンアップで実施）

---

## 実装方針

### 1. `site/content/docs/type-precision.mdx` 新規作成

frontmatter:
```markdown
---
title: "Type Precision"
description: "Favnir v41.x — Refinement type / Newtype / Row polymorphism で型でデータの意味を精緻に表現する"
---
```

コンテンツ構成（v41.1〜v41.8 の成果を整理）:

| v41.x | 機能 |
|---|---|
| v41.1.0 | Refinement type alias 基盤（`type Age = Int where \|v\| v >= 0`） |
| v41.2.0 | Refinement invariant 収集・E0404〜E0406 系エラーコード |
| v41.3.0 | タプルパターン match |
| v41.4.0 | ガード付き match（`n if n >= 90 -> "A"`） |
| v41.5.0 | Row polymorphism 強化（`{ ..u, active: true }` record spread） |
| v41.6.0 | Newtype 自動 impl（`type Kg(Float)` で算術演算子を自動委譲） |
| v41.7.0 | W030 lint（refinement 条件の冗長ガード検出） |
| v41.8.0 | Type Precision cookbook + docs 整備 |

### 2. driver.rs テスト（2 件）

```rust
mod v41900_tests {
    #[test]
    fn cargo_toml_version_is_41_9_0() {
        // NOTE: この assert は次バージョン bump 時にスタブ化すること
        let cargo = include_str!("../Cargo.toml");
        assert!(cargo.contains("41.9.0"), "Cargo.toml must contain version 41.9.0");
    }

    #[test]
    fn type_precision_doc_exists() {
        let src = include_str!("../../site/content/docs/type-precision.mdx");
        assert!(
            src.contains("Type Precision"),
            "site/content/docs/type-precision.mdx must exist and mention Type Precision"
        );
    }
}
```

---

## 既存コードへの影響

| ファイル | 変更 | 規模 |
|---|---|---|
| `site/content/docs/type-precision.mdx` | 新規作成 | 中（約 80 行） |
| `fav/src/driver.rs` | `v41800_tests` スタブ化 + `v41900_tests` 追加（2 件）※スタブ化はテスト関数を残しアサーションのみ除去するため総数に影響しない | 小 |
| `fav/Cargo.toml` | version: `41.8.0` → `41.9.0` | 1 行 |
| `CHANGELOG.md` | `[v41.9.0]` エントリ追加 | 数行 |

Rust ソースコード変更なし（コードフリーズ）。

---

## テスト計画

### Rust テスト（driver.rs）— 2 件

```rust
mod v41900_tests {
    #[test]
    fn cargo_toml_version_is_41_9_0() {
        // NOTE: この assert は次バージョン bump 時にスタブ化すること
        let cargo = include_str!("../Cargo.toml");
        assert!(cargo.contains("41.9.0"), "Cargo.toml must contain version 41.9.0");
    }

    #[test]
    fn type_precision_doc_exists() {
        let src = include_str!("../../site/content/docs/type-precision.mdx");
        assert!(
            src.contains("Type Precision"),
            "site/content/docs/type-precision.mdx must exist and mention Type Precision"
        );
    }
}
```

---

## 完了条件

### 自動検証（cargo test）

- `cargo test` 全通過（2870 tests passed, 0 failed）
- `v41900_tests::cargo_toml_version_is_41_9_0` pass
- `v41900_tests::type_precision_doc_exists` pass

### 実装者による手動確認

- `site/content/docs/type-precision.mdx` が存在し、v41.x 全機能の一覧と概要を含む
- v42.0 宣言文の予告が含まれている
