# Spec: v53.7.0 — ドキュメントサイト全体最終チェック

Status: 計画中
Date: 2026-07-22

---

## 概要

v51〜v53 スプリントで追加した機能（`par` 並列 stage・`assert_schema`・lineage × LSP・inlay hints）を
`site/content/docs/glossary.mdx` に反映し、用語定義を最新状態に保つ。

併せて `docs_no_broken_links` テストで主要 docs ページの存在確認を行い、
リンク切れの代理指標（Rust テストで確認可能な範囲）とする。

---

## 実装スコープ

### 1. `site/content/docs/glossary.mdx` 新規作成

v51〜v53 の新語彙を含む用語集を作成する。

```mdx
---
title: "用語集"
description: "Favnir の主要な用語・概念の定義一覧"
---

# 用語集

## par

`par [A, B] |> Merge.ordered` — 複数の stage を並列実行するキーワード。
v51.x で導入。各 stage はスレッド分離で並列実行され、結果は `Merge` で結合される。

## assert_schema

`assert_schema<T>(map)` — 実行時にマップのフィールドが型 `T` に一致するか検証する組み込み関数。
v52.x で導入。型不一致は E0419 エラーを発生させる。`--strict-schema` フラグで未知フィールドもエラーにできる。

## lineage

データリネージ — stage 間のデータの流れ（upstream / downstream）とスキーマ変換を追跡する機能。
v53.1.0 で LSP ホバーへの統合が完了し、エディタ上でリネージを確認できるようになった。

## inlay hints

LSP インレイヒント — エディタに型情報・stage 名・推論結果をインラインで表示する機能。
v51.x で導入。`fav` の LSP サーバーが VS Code 等に対して hint を送信する。

## rune

外部サービスとの接続を担う Favnir のプラグイン単位。`import kafka` のように参照する。
`runes/` ディレクトリに配置される。

## stage

パイプラインの処理単位。`stage Name: InputType -> OutputType = |arg| { ... }` の形式で定義する。

## pipeline

`pipeline Name { stage ... }` ブロックで定義される stage の連鎖。
`seq` / `par` / `|>` でデータフローを記述する。
```

---

### 2. テスト仕様

`v53700_tests` モジュールを `driver.rs` に追加（`v53600_tests` の直前）:

```rust
// -- v53700_tests (v53.7.0) -- ドキュメントサイト最終チェック --
#[cfg(test)]
mod v53700_tests {
    #[test]
    fn docs_no_broken_links() {
        // 主要 docs ページの存在確認（Rust テストで確認可能なリンク切れ代理指標）
        let base = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../site/content/docs");
        assert!(base.join("introduction.mdx").exists(), "docs/introduction.mdx must exist");
        assert!(base.join("quickstart.mdx").exists(), "docs/quickstart.mdx must exist");
        assert!(base.join("glossary.mdx").exists(), "docs/glossary.mdx must exist");
        assert!(base.join("installation.mdx").exists(), "docs/installation.mdx must exist");
    }

    #[test]
    fn docs_glossary_updated() {
        let content = include_str!("../../site/content/docs/glossary.mdx");
        assert!(
            content.contains("## par"),
            "glossary.mdx must have ## par section (v51 addition)"
        );
        assert!(
            content.contains("assert_schema"),
            "glossary.mdx must define assert_schema (v52 addition)"
        );
        assert!(
            content.contains("lineage"),
            "glossary.mdx must define lineage (v53.1 addition)"
        );
        assert!(
            content.contains("inlay"),
            "glossary.mdx must define inlay hints (v51 addition)"
        );
    }
}
```

---

## バージョン更新

- `fav/Cargo.toml`: `"53.6.0"` → `"53.7.0"`

---

## 完了条件

- `cargo test` 3177 passed, 0 failed（ベース 3175 + 2 件追加）
  - 注: ロードマップ推定値 3171 との差 +6（詳細内訳は roadmap 各バージョンの実績コメントを参照）
- `v53700_tests` 2 件 pass:
  - `docs_no_broken_links`
  - `docs_glossary_updated`
- `cargo clippy -- -D warnings` クリーン
- `site/content/docs/glossary.mdx` に `par` / `assert_schema` / `lineage` / `inlay` が含まれる

---

## 影響範囲

| ファイル | 変更種別 |
|---|---|
| `site/content/docs/glossary.mdx` | 新規作成 |
| `fav/src/driver.rs` | `v53700_tests` 追加 |
| `fav/Cargo.toml` | version 更新 |
| `CHANGELOG.md` | v53.7.0 エントリ追加 |
| `versions/current.md` | v53.7.0 / 3177 tests に更新 |
| `versions/roadmap/roadmap-v53.1-v54.0.md` | v53.7.0 実績欄を COMPLETE に更新 |

---

## 設計上の注意

- `docs_no_broken_links` テストは Rust で実行可能な範囲（ファイル存在確認）での代理指標。
  実際の HTML リンク切れチェックは CI の mdx-lint 等で別途行う想定。
- `include_str!("../../site/content/docs/glossary.mdx")`:
  `fav/src/driver.rs` から `../../` = `fav/` の親ディレクトリ（`favnir/`）→ `site/content/docs/glossary.mdx` ✓
- `env!("CARGO_MANIFEST_DIR").join("../site/content/docs")`:
  `CARGO_MANIFEST_DIR` = `fav/`、`../` = `favnir/`、`site/content/docs/` に到達 ✓（`examples/v55-demo` と同パターン）
- 「全 MDX ページの用語統一」はロードマップに記載があるが、v53.7.0 では glossary.mdx 作成と
  テストによる代理確認のみ実施する。既存 MDX 全ファイルの表記統一は別途 CI mdx-lint で対応する方針とし、
  スコープを意図的に限定する。
- v53600_tests にバージョンピンテストは存在しないため、バージョン更新時の空化対象なし
- `introduction.mdx` / `quickstart.mdx` / `installation.mdx` は既存ファイルとして存在確認済み
