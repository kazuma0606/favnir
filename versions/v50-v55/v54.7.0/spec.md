# Spec: v54.7.0 — ドキュメントサイト Production 3.0 overview ページ

Status: COMPLETE
Date: 2026-07-23

---

## 概要

`site/content/docs/production3-overview.mdx` を新規作成する。
v51〜v55 の全機能を統合した概要ページとして、Production 3.0 への道のりを説明する。

---

## 実装スコープ

### 1. `site/content/docs/production3-overview.mdx` — 新規作成

以下のセクション構成で作成:

```
# Production 3.0 — Favnir v55 への道のり

## v51 — Developer Experience 3.0
  - 全エラーコード fav explain --error 対応完備
  - LSP インレイヒント
  - fav run --trace
  - fav run --watch 自動再実行

## v52 — Performance & Scale
  - par 並列 stage 実行（Tokio スレッドプール）
  - バックプレッシャー制御
  - fav bench --compare（CI 統合）
  - インクリメンタルコンパイル・WASM 最適化

## v53 — Data Quality & Observability 2.0
  - assert_schema ランタイム型検証
  - fav explain --lineage --with-schema
  - fav run --audit-log JSONL 監査ログ
  - OTel span 属性強化・SLA 監視 Rune

## v54 — Integration Sprint
  - fav explain --error 全コード完備（v54.1）
  - fav run --watch-diff / --watch-summary（v54.2）
  - パフォーマンスリグレッション CI 統合（v54.3）
  - fav dq-report（v54.4）
  - fav doctor（v54.5）
  ※ ロードマップの「lineage × LSP / bench × par / E2E デモ」は v54.0（Integration Sprint 宣言）
     の観点。MDX 内の v54 セクションはサブバージョン別列挙（v54.1〜v54.5）を正とする。

## v55 — Production 3.0 宣言
  > 「型安全なガード節、...Production 3.0 の姿である。」

## 関連ドキュメント
  - dx3-overview.mdx
  - integration-overview.mdx
  - data-quality-overview.mdx
```

帰属バージョンの注意:
- `--watch-diff` / `--watch-summary` は **v54.2.0** の成果物。v51 セクションには含めない。
- `MILESTONE.md` へのリンクはサイト構造から到達不可のため含めない。

### 2. `driver.rs` — `v54700_tests` 追加

`v54600_tests` の直前に追加（2 テスト）:

```rust
#[cfg(test)]
mod v54700_tests {
    use super::*;

    #[test]
    fn docs_production3_overview_exists() {
        let doc = include_str!("../../site/content/docs/production3-overview.mdx");
        assert!(!doc.is_empty(), "production3-overview.mdx should not be empty");
        assert!(doc.contains("Production 3.0"), "...");
    }

    #[test]
    fn docs_production3_has_v55() {
        let doc = include_str!("../../site/content/docs/production3-overview.mdx");
        assert!(doc.contains("v55"), "production3-overview.mdx should mention v55");
    }
}
```

`include_str!` パス: `fav/src/driver.rs` から `../../site/content/docs/production3-overview.mdx`
（`fav/` → `favnir/` → `favnir/site/content/docs/production3-overview.mdx`）

---

## テスト仕様

| テスト名 | 検証内容 |
|---|---|
| `docs_production3_overview_exists` | `production3-overview.mdx` が非空かつ `"Production 3.0"` を含む |
| `docs_production3_has_v55` | `production3-overview.mdx` が `"v55"` を含む |

注意: アサーションは最小限（ロードマップ指定の完了条件のみ）。
v54.9.0 の `production3_overview_doc_complete` テストで複数アサーション（各 `##` 見出し + 宣言文）を追加予定。

---

## バージョン更新

- `fav/Cargo.toml`: `"54.6.0"` → `"54.7.0"`

---

## 完了条件

1. `cargo test -j 8 -- --test-threads=8` → 3199 passed, 0 failed（ベース 3197 + 2 件追加）
2. `v54700_tests` 2 件 pass:
   - `docs_production3_overview_exists`
   - `docs_production3_has_v55`
3. `cargo test` 全通過後に `cargo clippy -- -D warnings` → 警告なし確認

---

## 影響範囲

| ファイル | 変更種別 |
|---|---|
| `site/content/docs/production3-overview.mdx` | 新規作成 |
| `fav/src/driver.rs` | `v54700_tests` 追加 |
| `fav/Cargo.toml` | version 更新 |
| `fav/Cargo.lock` | version 更新に伴い自動更新 |
| `CHANGELOG.md` | v54.7.0 エントリ追加 |
| `versions/current.md` | v54.7.0 / 3199 tests に更新 |
| `versions/roadmap/roadmap-v54.1-v55.0.md` | v54.7.0 実績欄を COMPLETE に更新 |

---

## 設計上の注意

- `--watch-diff` / `--watch-summary` は v54.2.0 の成果物のため、MDX の v51 セクションには含めない（v54 セクションに正しく記載）。
- `MILESTONE.md` は `site/content/docs/` からの相対パスで到達不可のためリンクしない。
- `.mdx` 拡張子付きリンクは既存 MDX ファイルと同形式（`dx3-overview.mdx` 等も同様）。
- テストのアサーションはロードマップ完了条件（`docs_production3_overview_exists` / `docs_production3_has_v55`）の最低限に準拠。構造的検証は v54.9.0 に委ねる。
- `use super::*` は `v54700_tests` 内で実質不要（`include_str!` のみ使用）だが、他テストモジュールとの慣習統一のため明示する。
