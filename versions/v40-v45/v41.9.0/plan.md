# v41.9.0 実装プラン — v42.0 前調整・安定化

**フェーズ**: Type Precision（v41.x スプリント）
**目標テスト数**: 2870（+2）

---

## ステップ概要

1. `site/content/docs/type-precision.mdx` 新規作成
2. `fav/src/driver.rs` — `v41800_tests` スタブ化 + `v41900_tests` 追加
3. `fav/Cargo.toml` バージョン bump（41.8.0 → 41.9.0）
4. `CHANGELOG.md` エントリ追加
5. `cargo test` 実行・確認

---

## Step 1: `site/content/docs/type-precision.mdx` 新規作成

frontmatter:
```markdown
---
title: "Type Precision"
description: "Favnir v41.x — Refinement type / Newtype / Row polymorphism で型でデータの意味を精緻に表現する"
---
```

コンテンツ構成（英語、約 80 行）:

- **Overview**: Type Precision フェーズ概要（v41.x スプリントの目標）
- **Feature Table**: v41.1〜v41.8 の機能一覧

| Version | Feature |
|---|---|
| v41.1.0 | Refinement type alias 基盤（`type Age = Int where \|v\| v >= 0`） |
| v41.2.0 | Refinement invariant 収集・E0404〜E0406 系エラーコード |
| v41.3.0 | タプルパターン match |
| v41.4.0 | ガード付き match（`n if n >= 90 -> "A"`） |
| v41.5.0 | Row polymorphism 強化（`{ ..u, active: true }` record spread） |
| v41.6.0 | Newtype 自動 impl（`type Kg(Float)` で算術演算子を自動委譲） |
| v41.7.0 | W030 lint（refinement 条件の冗長ガード検出） |
| v41.8.0 | Type Precision cookbook + docs 整備 |

- **v42.0 Preview**: 宣言文の予告（暫定版）

---

## Step 2: driver.rs 更新

### 2a. `v41800_tests::cargo_toml_version_is_41_8_0` スタブ化

対象行を探す:
```rust
fn cargo_toml_version_is_41_8_0() {
    // NOTE: この assert は次バージョン bump 時にスタブ化すること
    let cargo = include_str!("../Cargo.toml");
    assert!(cargo.contains("41.8.0"), ...);
}
```

スタブ化後:
```rust
fn cargo_toml_version_is_41_8_0() {
    // Stubbed: version bumped to 41.9.0 -- assertion intentionally removed
}
```

### 2b. `v41900_tests` モジュール追加（`v41800_tests` の直前）

> **挿入位置の補足**: driver.rs は「新しいバージョンが上（先頭側）」の降順配置になっている（例: v41800_tests が v41700_tests より前にある）。
> したがって v41900_tests は v41800_tests の**直前（上）**に追加する。

```rust
// -- v41900_tests (v41.9.0) -- v42.0 前調整・安定化 --
#[cfg(test)]
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

## Step 3: Cargo.toml バージョン bump

```toml
version = "41.9.0"
```

---

## Step 4: CHANGELOG.md エントリ追加

`[v41.8.0]` の直前に追加:

```markdown
## [v41.9.0] — 2026-07-12

### Added
- `site/content/docs/type-precision.mdx` — Type Precision マイルストーン概要ページ新規作成
- v42.0 宣言文の予告（暫定版）を掲載

### Changed
- `fav/Cargo.toml`: version `41.8.0` → `41.9.0`
```

---

## AST・API 確認事項

- `include_str!("../../site/content/docs/type-precision.mdx")` — `fav/src/driver.rs` から見た相対パス
- Rust ソースコード変更なし（コードフリーズ）

---

## 注意事項

- コードフリーズ: 新規 Rust 機能追加は行わない
- `type_precision_doc_exists` テストは `include_str!` でコンパイル時にファイル存在を強制するため、ファイル作成が先
