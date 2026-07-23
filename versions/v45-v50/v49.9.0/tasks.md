# Tasks: v49.9.0 — v50.0 前調整・安定化

Status: COMPLETE
Date: 2026-07-18

---

## T0 — 事前確認

- [x] `cargo test` 3085 passed, 0 failed を確認（ベース確認）
- [x] `language-maturity-overview.mdx` が `favnir/site/content/docs/` に存在することを確認（v49.8.0 で作成済み）
- [x] `language-maturity-overview.mdx` に `"| v46 |"` 形式のテーブル行が含まれていないことを確認（テーブル追記が必要な状態）
- [x] `v498000_tests` モジュールが `driver.rs` に存在することを確認（挿入位置の前提）
- [x] `cargo clippy -- -D warnings` クリーンであることを確認（ベース）

## T1 — `language-maturity-overview.mdx` 充実化

- [x] `language-maturity-overview.mdx` に v46〜v49 機能一覧テーブルを追加
  - [x] `"| v49 |"` という文字列を含む
  - [x] `"| v46 |"` / `"| v47 |"` / `"| v48 |"` も含む（テーブル形式）
  - [x] `## 関連ドキュメント` の前に挿入

## T2 — `v499000_tests` 追加

- [x] `v499000_tests` モジュールを `v498000_tests` の直前に追加（2 テスト）
- [x] 挿入後 `grep -n v499000_tests src/driver.rs` で存在確認
  - [x] `cargo_toml_version_is_49_9_0`:
    - [x] `include_str!("../Cargo.toml")` で読み込み（`fav/Cargo.toml`）
    - [x] `"49.9.0"` が含まれることを確認
  - [x] `language_maturity_overview_doc_exists`:
    - [x] `include_str!("../../site/content/docs/language-maturity-overview.mdx")` で読み込み
    - [x] `"| v49 |"` が含まれることを確認（テーブル行）
    - [x] `"| v46 |"` が含まれることを確認（テーブル行）

## T3 — バージョン更新・完了

- [x] `fav/Cargo.toml` version → `"49.9.0"`（先に更新）
- [x] `cargo test` 3087 passed, 0 failed（Cargo.toml 更新後に実行）
- [x] `cargo clippy -- -D warnings` クリーン
- [x] `cargo fmt -- --check` クリーン（Rust ソース対象）
- [x] `CHANGELOG.md` に v49.9.0 エントリ追加（v50.0 前調整・安定化を明記）
- [x] `versions/current.md` を v49.9.0（3087 tests）に更新、進行中バージョンを `v50.0.0` に更新
- [x] `versions/roadmap/roadmap-v49.1-v50.0.md` の v49.9.0 実績を 3087 に記入
- [x] tasks.md を COMPLETE に更新（T0〜T3 全 `[x]`）
