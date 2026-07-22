# Tasks: v50.5.0 — LSP インレイヒント Phase 2（パイプライン stage 型）

Status: COMPLETE
Date: 2026-07-19

---

## T0 — 事前確認

- [x] `cargo test` 3099 passed, 0 failed を確認（ベース確認）
- [x] `collect_pipeline_type_hints` が未実装であることを確認（`grep "pipeline_type" fav/src/lsp/inlay_hints.rs`）
- [x] `collect_stage_hints` / `find_stage_prefix` が実装済みであることを確認
- [x] `Type::Arrow` / `Type::Trf` が `checker.rs` に存在することを確認
- [x] `cargo clippy -- -D warnings` クリーンであることを確認（ベース）

## T1 — `lsp/inlay_hints.rs` — `collect_pipeline_type_hints` 追加

- [x] `collect_fn_return_hints` の直後（`find_type_at` の直前）に `pub(crate) fn collect_pipeline_type_hints` を追加
  - [x] `find_stage_prefix` を再利用して `stage <Name>` 行をスキャン
  - [x] `name_end == 0` ガード（stage 名が空の場合のスキップ）
  - [x] `name == "_"` ガード（ワイルドカード stage のスキップ）
  - [x] `find_type_at` で型を取得し、`Type::Arrow` / `Type::Trf` のみヒントを生成
  - [x] ラベル形式: `format!(": {}", ty.display())`
  - [x] `_ => None` で非関数型はスキップ
- [x] `handle_inlay_hints` に `hints.extend(collect_pipeline_type_hints(...))` を追加（`collect_fn_return_hints` の直後）

## T2 — `v505000_tests` モジュール追加

- [x] `v505000_tests` モジュールを `driver.rs` の `v504000_tests` 直前に追加（3 件）
  - [x] `cargo_toml_version_is_50_5_0`: version = "50.5.0" を assert
  - [x] `lsp_inlay_hint_stage_type`: `collect_stage_hints` が `Type::Arrow(RawOrder, Order)` の stage に対して `RawOrder`・`Order` を含むラベルのヒントを返すことを assert
  - [x] `lsp_inlay_hint_pipeline_type`: `collect_pipeline_type_hints` が 2 stage ソースに対して 2 件のヒントを返し、`RawOrder`・`ValidOrder` を含むことを assert

## T3 — バージョン更新・完了

- [x] `fav/Cargo.toml` version → `"50.5.0"`
- [x] `v504000_tests::cargo_toml_version_is_50_4_0` を削除
- [x] `cargo test` 3101 passed, 0 failed
- [x] `cargo clippy -- -D warnings` クリーン
- [x] `CHANGELOG.md` に v50.5.0 エントリ追加
- [x] `versions/current.md` を v50.5.0（3101 tests）に更新
- [x] `versions/roadmap/roadmap-v50.1-v51.0.md` の v50.5.0 実績欄に「`references.rs` 変更は不要と判断、`inlay_hints.rs` 内の新関数 `collect_pipeline_type_hints` で代替」と記録する
- [x] tasks.md を COMPLETE に更新（T0〜T3 全 `[x]`）
