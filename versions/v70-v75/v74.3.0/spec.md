# v74.3.0 仕様書 — Documentation Site 2.0

Date: 2026-08-13

---

## Background

Favnir の公式ドキュメントサイトを v2.0 構造に刷新する。
Getting Started（5 分チュートリアル）・Language Reference（全構文）・Migration Guide（v35→v75）を
`site/content/docs/v2/` 以下に新規追加し、ユーザーの学習コストを大幅に下げる。

本バージョンでは 3 つの MDX ファイルを作成し、`driver.rs` の `include_str!` テストで
それらが存在・内容を持つことを確認する。

---

## Goals

1. `site/content/docs/v2/getting-started.mdx` — 5 分チュートリアルを作成する
2. `site/content/docs/v2/migration-v35-v75.mdx` — v35→v75 移行ガイドを作成する
3. `site/content/docs/v2/language-reference.mdx` — 全構文一覧を作成する
4. `v743000_tests` モジュール（2 件）を追加する
   - `docs_site2_getting_started_exists`
   - `docs_site2_migration_guide_v35_to_v75`

---

## ドキュメント構造

```
site/content/docs/v2/
├── getting-started.mdx     # 5分チュートリアル: インストール→hello world→pipeline
├── migration-v35-v75.mdx   # v35→v75 移行手順（構文変更・廃止フラグ一覧）
└── language-reference.mdx  # 全構文一覧（bind / stage / par / interface / ctx / Rune）
```

### `getting-started.mdx` 最小構成

```mdx
---
title: Getting Started
---

# Getting Started

Favnir を 5 分で試す。

## インストール

...

## Hello World パイプライン

...
```

### `migration-v35-v75.mdx` 最小構成

```mdx
---
title: Migration Guide v35 to v75
---

# Migration Guide — v35 → v75

...
```

### `language-reference.mdx` 最小構成

```mdx
---
title: Language Reference
---

# Language Reference

全構文一覧...
```

---

## Rust テスト（`driver.rs` の `v743000_tests`）

```rust
fn docs_site2_getting_started_exists() {
    let src = include_str!("../../site/content/docs/v2/getting-started.mdx");
    assert!(src.contains("Getting Started"), "getting-started title missing");
    assert!(src.contains("Favnir"), "Favnir mention missing");
}

fn docs_site2_migration_guide_v35_to_v75() {
    let src = include_str!("../../site/content/docs/v2/migration-v35-v75.mdx");
    assert!(src.contains("Migration"), "migration guide title missing");
    assert!(src.contains("v35"), "v35 reference missing");
    assert!(src.contains("v75"), "v75 reference missing");
}
```

---

## Success Criteria

1. `docs_site2_getting_started_exists` テストが pass する
   - `site/content/docs/v2/getting-started.mdx` が存在する
   - `"Getting Started"` と `"Favnir"` を含む
2. `docs_site2_migration_guide_v35_to_v75` テストが pass する
   - `site/content/docs/v2/migration-v35-v75.mdx` が存在する
   - `"Migration"` / `"v35"` / `"v75"` を含む
3. `cargo test` で 3675 tests pass（0 failures）

---

## スコープ外（明示的除外）

- Rune Catalog（後続バージョンで対応）
- Cookbook（10+ レシピ）（後続バージョンで対応）
- API Reference（fav CLI の全フラグ）（後続バージョンで対応）
- Video Transcripts（後続バージョンで対応）
- サイトのビルド・デプロイ

## Error Codes

新規エラーコードなし

---

## Files to Modify / Create

| ファイル | 変更内容 |
|---|---|
| `site/content/docs/v2/getting-started.mdx` | 新規作成（5 分チュートリアル） |
| `site/content/docs/v2/migration-v35-v75.mdx` | 新規作成（移行ガイド） |
| `site/content/docs/v2/language-reference.mdx` | 新規作成（全構文一覧） |
| `fav/src/driver.rs` | `v743000_tests` 追加（`include_str!` パス: `../../site/...`） |
| `fav/Cargo.toml` | `version = "74.3.0"` に更新 |
| `CHANGELOG.md` | v74.3.0 エントリを先頭に追加 |
| `versions/current.md` | 進行中バージョン・次に切る版を更新 |

### `include_str!` パス解説

`fav/src/driver.rs` からの相対パス:
- `../` = `fav/`
- `../../` = `favnir/`（リポジトリルート）
- `../../site/` = `favnir/site/`

→ `include_str!("../../site/content/docs/v2/getting-started.mdx")`
