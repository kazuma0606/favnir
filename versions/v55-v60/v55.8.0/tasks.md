# Tasks — v55.8.0 — ドキュメントサイト Streaming 2.0 記事

## ステータス: COMPLETE（2026-07-24）

---

## 事前確認（T0）

- [x] `versions/roadmap/roadmap-v55.1-v56.0.md` の v55.8.0 セクションを確認
- [x] ベーステスト数 3219（v55.7.0 完了時点の実績値）を確認
- [x] `fav/Cargo.toml` が現在 `55.7.0` であることを確認（更新前）
- [x] `site/content/docs/runtime/` に `streaming-v2.mdx` が存在しないことを確認（新規追加）
- [x] `site/content/cookbook/` に `stateful-pipeline.mdx` が存在しないことを確認（新規追加）
- [x] `site/content/cookbook/` に `cep-patterns.mdx` が存在しないことを確認（新規追加）
- [x] `fav/src/driver.rs` の `v55700_tests` モジュール位置を確認（直前に `v55800_tests` を挿入）
- [x] `include_str!` パス `../../site/content/...` が正しいことを確認（`fav/src/` から 2 階層上が `favnir/`）
- [x] CI self-lint 対象（`self/compiler.fav` / `self/checker.fav`）に今回の変更が影響しないことを確認（MDX / driver.rs テスト追加のみ、Favnir ソース非依存）

---

## 実装タスク

- [x] T1: `fav/Cargo.toml` version を `55.8.0` に更新
- [x] T2: `site/content/docs/runtime/streaming-v2.mdx` を新規作成
  - [x] `"Streaming Native 2.0"` キーワードを含む
  - [x] ウィンドウ・ウォーターマーク・Exactly-once・Stateful・CEP・Stream Join セクションを含む
  - [x] `Window.tumbling` の stage 戻り型を `Stream<List<Event>>` に設定（`Stream<Int>` は不正確なため禁止）
- [x] T3: `site/content/cookbook/stateful-pipeline.mdx` を新規作成
  - [x] `State.get` / `State.set` / `State.get_or_default` のレシピを含む
  - [x] fav.toml `[stream]` 設定例を含む
- [x] T4: `site/content/cookbook/cep-patterns.mdx` を新規作成
  - [x] `CEP.sequence` / `CEP.skip_until` のレシピを含む
  - [x] 境界ケース一覧表を含む
- [x] T5: `fav/src/driver.rs` に `v55800_tests` モジュールを追加（`v55700_tests` の直前）
  - [x] `docs_streaming_v2_page_exists`（streaming-v2.mdx のキーワード 4 件検証）
  - [x] `cookbook_stateful_pipeline_exists`（stateful-pipeline.mdx のキーワード 3 件検証）
  - [x] `cookbook_cep_patterns_exists`（cep-patterns.mdx のキーワード 2 件検証）— コードレビュー対応で追加

---

## テスト・検証

- [x] T6: `cargo build` でコンパイルエラーがないことを確認
- [x] T7: `cargo test` 全通過（3222 tests passed, 0 failed）
- [x] T8: `cargo clippy -- -D warnings` クリーン

---

## ポスト処理

- [x] T9: `CHANGELOG.md` に v55.8.0 エントリ追加
- [x] T10: `versions/current.md` を v55.8.0 / 3222 tests に更新
- [x] T11: `versions/roadmap/roadmap-v55.1-v56.0.md` の v55.8.0 実績を COMPLETE に更新
- [x] T12: `versions/roadmap/roadmap-v55.1-v60.0.md` の v55.8.0 実績欄も COMPLETE に更新

---

## コードレビュー

- [x] コードレビュー実施（`/review code`）
- [x] 指摘事項対応
  - [MED] `streaming-v2.mdx` の `TumblingCount` 型注釈 `Stream<Int>` → `Stream<List<Event>>` に修正
  - [LOW] `cep-patterns.mdx` の存在確認テスト `cookbook_cep_patterns_exists` を追加（3222 tests に +1）

---

## 完了確認

- [x] `docs_streaming_v2_page_exists` pass
- [x] `cookbook_stateful_pipeline_exists` pass
- [x] `cookbook_cep_patterns_exists` pass（コードレビュー対応で追加）
- [x] 3222 tests passed, 0 failed
- [x] `cargo clippy -- -D warnings` クリーン
- [x] `streaming-v2.mdx` の `Window.tumbling` 戻り型が `Stream<List<Event>>`
- [x] `CHANGELOG.md` に v55.8.0 エントリが追加されている
- [x] `versions/current.md` が v55.8.0 / 3222 tests を反映
- [x] T11 / T12 のロードマップ更新が完了している
