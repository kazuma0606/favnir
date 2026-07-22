# Tasks: v51.9.0 — 安定化・コードフリーズ（Performance & Scale 前調整）

Status: COMPLETE
Date: 2026-07-20

---

## T0 — 事前確認

- [x] `cargo test` 3131 passed, 0 failed を確認（ベース確認）
- [x] `cargo clippy -- -D warnings` クリーンであることを確認
- [x] `site/content/docs/performance-overview.mdx` が**存在しない**ことを確認（新規作成対象）
- [x] `include_str!` パスの確認:
  - [x] `fav/src/driver.rs` から `../Cargo.toml` → `fav/Cargo.toml` を確認
  - [x] `fav/src/driver.rs` から `../../site/content/docs/performance-overview.mdx` → `favnir/site/content/docs/performance-overview.mdx` を確認
- [x] v51.8.0 の `v51800_tests` に `cargo_toml_version_is_51_8_0` が**存在しない**ことを確認（削除不要）
- [x] `driver.rs` の最新テストモジュール追加順を確認（新版が旧版の直前に来るパターン: v51800 → v51700 の順）、`v51900_tests` は `v51800_tests` の**直前**に追加することを確認

## T1 — `site/content/docs/performance-overview.mdx` 作成

- [x] `site/content/docs/performance-overview.mdx` を新規作成:
  - [x] frontmatter（`title` / `description`）を含む
  - [x] `par` キーワードを含む（並列ステージ実行への言及）
  - [x] `fav bench` キーワードを含む（ベンチマーク・回帰検出への言及）
  - [x] `Performance & Scale` キーワードを含む（マイルストーン名）
  - [x] v51.1〜v51.8 の機能群を俯瞰的にまとめた骨子構成
  - [x] 詳細ページ（`runtime/parallel.mdx` / `tools/bench-regression.mdx` 等）への参照を含む

## T2 — `v51900_tests` 追加 + バージョン更新

- [x] `driver.rs` の `v51800_tests` 直前に `v51900_tests` モジュールを追加（2 件）:
  - [x] `cargo_toml_version_is_51_9_0`:
    - [x] `include_str!("../Cargo.toml")` で Cargo.toml を読み込む
    - [x] `content.contains("version = \"51.9.0\"")` を assert
  - [x] `perf_overview_doc_exists`:
    - [x] `include_str!("../../site/content/docs/performance-overview.mdx")` で読み込む
    - [x] `src.contains("par")` を assert
    - [x] `src.contains("fav bench")` を assert
    - [x] `src.contains("Performance & Scale")` を assert
- [x] `fav/Cargo.toml` version → `"51.9.0"`（v51.8.0 のバージョンテストは存在しないため削除なし）
- [x] `cargo test` 3133 passed, 0 failed（3131 + 2 = 3133）
- [x] `cargo clippy -- -D warnings` クリーン

## T3 — 後処理

- [x] `CHANGELOG.md` に v51.9.0 エントリ追加
- [x] `versions/current.md` を v51.9.0（3133 tests）に更新
- [x] `roadmap-v51.1-v52.0.md` の v51.9.0 実績欄を更新（推定 3131 → 実績 3133 に修正）
- [x] `roadmap-v51.1-v52.0.md` の v52.0.0 テスト数推定（≥3135）が実態（3133 - 1 + 4 = 3136）と合うことを確認し注記追加
- [x] tasks.md を COMPLETE に更新（T0〜T3 全 `[x]`）

## code-review 対応（2026-07-20）

- [x] [MED] `performance-overview.mdx` L8 の `v52.0` 表記 → `Performance & Scale スプリント（v51.1〜v51.9）` に修正
- [x] [LOW] MDX 末尾改行を確認（Read で line 94 が空行 = 既存の改行あり、誤検知）
- [x] [LOW] 関連ページ4件（bench / profiling / incremental / wasm-opt）の存在を確認済み（ls で全件 EXIST）
