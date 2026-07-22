# Tasks: v50.6.0 — LSP ホバー情報強化（Rune メソッドシグネチャ）

Status: COMPLETE
Date: 2026-07-19

---

## T0 — 事前確認

- [x] `cargo test` 3101 passed, 0 failed を確認（ベース確認）
- [x] `builtin_hover_at` / `rune_hover_at` が未実装であることを確認（`grep "builtin_hover\|rune_hover" fav/src/lsp/hover.rs`）
- [x] `BUILTIN_FNS` が `completion.rs` に定義済みであることを確認
- [x] `rune.toml` に `[[exports]]` セクションが存在しないことを確認（静的テーブル代替��根拠）
- [x] `cargo clippy -- -D warnings` クリーンであることを確認（ベース）

## T1 — `lsp/hover.rs` — `RuneFn` + ヘルパー追加

- [x] `display_type` の直後に以下を追加:
  - [x] `pub(crate) struct RuneFn` — rune, name, signature, effect, doc フィールド
  - [x] `pub(crate) const RUNE_FNS: &[RuneFn]` — 4 エントリ（kafka: consume/produce、csv: read/write）
  - [x] `fn word_and_ns_at(source, offset)` — `(namespace, method)` 抽出ヘルパー（モジュールプライベート）
  - [x] `pub(crate) fn builtin_hover_at(source, offset)` — PascalCase NS → BUILTIN_FNS 検索
  - [x] `pub(crate) fn rune_hover_at(source, offset)` — lowercase NS → RUNE_FNS 検索
- [x] `handle_hover` に `builtin_hover_at` / `rune_hover_at` の優先チェックを追加���`type_at` より前）
- [x] 既存の `handle_hover` テスト（`lsp/hover.rs` 内 `tests` モジュール）が引き��き通過することを確認

## T2 — `v506000_tests` モジュール追加

- [x] `v506000_tests` モジュールを `driver.rs` の `v505000_tests` 直前に追加（3 件）
  - [x] `cargo_toml_version_is_50_6_0`: version = "50.6.0" を assert
  - [x] `lsp_hover_builtin_fn`: `builtin_hover_at("List.map(items, f)", 5)` が Some を返し、`map` �� `List` を含むことを assert
  - [x] `lsp_hover_rune_method`: `rune_hover_at("kafka.consume(topic)", 6)` が Some を返し、`consume` と `Kafka` を含むことを assert

## T3 — バージョン更新・完了

- [x] `fav/Cargo.toml` version → `"50.6.0"`
- [x] `v505000_tests::cargo_toml_version_is_50_5_0` のみを削除（モジュール内の `lsp_inlay_hint_stage_type` / `lsp_inlay_hint_pipeline_type` は保持）
- [x] `cargo test` 3103 passed, 0 failed
- [x] `cargo clippy -- -D warnings` クリーン
- [x] `CHANGELOG.md` に v50.6.0 エントリ追加
- [x] `versions/current.md` を v50.6.0（3103 tests）に更新
- [x] `versions/roadmap/roadmap-v50.1-v51.0.md` の v50.6.0 実績欄に「`rune.toml` 動的読み込みは未実装、静的 `RUNE_FNS` テーブルで代替��と���録する
- [x] tasks.md を COMPLETE に更新（T0〜T3 全 `[x]`）
