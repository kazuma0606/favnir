# v38.6.0 タスクリスト — RAG パイプラインテンプレート

## ステータス: COMPLETE

> ロードマップ整合: `roadmap-v38.1-v39.0.md` の v38.6.0（「RAG パイプラインテンプレート」）に沿ったバージョン。

## T0: 事前確認

- [x] `cargo test` の実測通過数を確認（目安: 2765（v38.5.0 完了時点の実績値））し、実測値をここに記録: 2765
- [x] Cargo.toml バージョンが `38.5.0` であることを確認
- [x] `v38500_tests::cargo_toml_version_is_38_5_0` がライブアサーション（`assert!(cargo.contains("38.5.0"), ...)`）であることを確認し、行番号を Grep で確認して記録: 43812
- [x] `v38500_tests` の他 4 テスト（`changelog_has_v38_5_0` / `explain_verbose_basic` / `explain_verbose_with_location` / `explain_verbose_unknown_code`）はバージョン変更後も pass することを確認
- [x] `driver.rs` に `v38600_tests` モジュールが存在しないことを確認（今回新規作成）
- [x] `v38500_tests` の閉じ `}` の行番号を Grep/Read で確認し、ここに記録: 43849
- [x] `CHANGELOG.md` に `[v38.6.0]` エントリが存在しないことを確認（今回新規作成）
- [x] `driver.rs` に `create_rag_pipeline_project` が存在しないことを確認（今回追加）
- [x] `driver.rs` の `try_cmd_new` に `"rag-pipeline"` アームが存在しないことを確認（今回追加）
- [x] `versions/current.md` の最新安定版が `v38.5.0`・次バージョンが `v38.6.0` であることを確認
- [x] `versions/roadmap/roadmap-v38.1-v39.0.md` の v38.6.0 が未完了（✅ なし）であることを確認（T7 で更新）

## T1: CHANGELOG.md に [v38.6.0] エントリを追加

- [x] `## [v38.5.0]` の直前に `## [v38.6.0]` エントリを挿入
- [x] 日付を `2026-07-10` に設定
- [x] セパレータが `—`（全角ダッシュ U+2014）形式であることを確認

## T2: `driver.rs` — `create_rag_pipeline_project` 関数追加

- [x] Read で `create_multi_source_etl_project` の終端（`Ok(())` + `}`）の行番号を確認
- [x] その直後に `create_rag_pipeline_project` を追加（spec.md §1 のコードブロックに従う）
- [x] 生成ファイルが以下を含む:
  - [x] `src/main.fav`: Ingest / Embed / Retrieve / Generate 4 ステージ + pipeline 宣言
  - [x] `fav.toml`: `[runes] llm = "1.0.0"`, `csv = "1.0.0"`
  - [x] `data/documents.csv`: サンプルデータ
  - [x] `README.md`: 使い方説明

## T3: `driver.rs` — `try_cmd_new` に `"rag-pipeline"` アーム追加 + エラーメッセージ更新

- [x] Read で `"multi-source"` アームの行番号を確認
- [x] `"multi-source"` アームの直後、`other =>` の直前に `"rag-pipeline" => create_rag_pipeline_project(&root, name),` を追加
- [x] `other =>` のエラーメッセージの `multi-source` の直後・`)` の直前に `|rag-pipeline` を追加（`multi-source|rag-pipeline)` となること）

## T4: `driver.rs` — `cmd_new_list` に `rag-pipeline` 追加

- [x] Read で `cmd_new_list` 内の `"multi-source"` 行の行番号を確認
- [x] その直後に `rag-pipeline` の println! 行を追加（spec.md §3 のコードブロックに従う）
- [x] `v308000_tests::cmd_new_list_contains_all_templates`（または同等テスト）が `rag-pipeline` を網羅していないことを確認し、意図的に拡張対象外とする（既存テストは破壊されないため問題なし）

## T5: `driver.rs` — `v38500_tests::cargo_toml_version_is_38_5_0` をスタブ化

- [x] Grep で `cargo_toml_version_is_38_5_0` の行番号を確認
- [x] ライブアサーション → `// Stubbed: version bumped to 38.6.0 — assertion intentionally removed` に変更
- [x] **注意:** `changelog_has_v38_5_0` / `explain_verbose_*` テストはスタブ化しない
- [x] スタブ形式が前バージョンのスタブと一致していることを確認

## T6: `driver.rs` — `v38600_tests` モジュールを新規追加（T1 完了後に実施）

- [x] T1（CHANGELOG 追加）が完了していることを確認してから着手
- [x] `v38500_tests` の閉じ `}` の行番号（T0 で記録済み）を Read で特定してから Edit を実行
- [x] `v38500_tests` の閉じ `}` の後に `v38600_tests` モジュールを追加（4 テスト）
  - [x] `cargo_toml_version_is_38_6_0`
  - [x] `changelog_has_v38_6_0`
  - [x] `rag_pipeline_fn_exists`（driver.rs に `create_rag_pipeline_project` が含まれる）
  - [x] `rag_pipeline_has_four_stages`（driver.rs に `Ingest`・`Embed`・`Retrieve`・`Generate` が含まれる）

## T7: バージョン更新（T1〜T6 すべて完了後）

- [x] `fav/Cargo.toml` バージョンを `38.6.0` に更新

## T8: テスト実行

- [x] T7（Cargo.toml 更新）が完了していることを確認してから着手
- [x] `cargo test` 全通過 — ≥ 2769 passed; 0 failed — 実測: 2770 passed, 0 failed（code-reviewer 対応で +1）
- [x] `v38600_tests` の 5 テストがすべて pass（code-reviewer [MED] 対応: `fav_new_rag_pipeline_ok` 追加）
- [x] `cargo_toml_version_is_38_6_0` が pass
- [x] `changelog_has_v38_6_0` が pass
- [x] `rag_pipeline_fn_exists` が pass
- [x] `rag_pipeline_has_four_stages` が pass
- [x] `fav_new_rag_pipeline_ok` が pass

## T9: ドキュメント更新（T8 完了後）

- [x] `versions/v36-v40/v38.6.0/tasks.md` を COMPLETE ステータスに更新（T0〜T9 全チェックボックスを `[x]` に）
- [x] `versions/current.md` を v38.6.0（最新安定版）・v38.7.0（次バージョン）に更新
- [x] `versions/roadmap/roadmap-v38.1-v39.0.md` の v38.6.0 を完了済みにマーク（✅）し、テスト件数を 4 件に更新
- [x] roadmap の v38.6.0 行を Read で確認し ✅ が含まれることをここに記録: ✅ 確認: ✅ v38.6.0 — RAG パイプラインテンプレート
- [x] roadmap の v38.6.0 行を Read で確認し「4 件」が含まれることをここに記録: テスト件数 4 件確認: Rust テスト 4 件（2769 tests passed, 0 failed）
- [x] `versions/current.md` を Read で確認し「v38.6.0」が最新安定版として含まれることをここに記録: 確認: **v38.6.0** — `fav new --template rag-pipeline` RAG パイプラインテンプレート（2026-07-10）

---

## 完了条件チェックリスト（spec.md 対応）

| # | spec.md 完了条件 | 確認方法 |
|---|---|---|
| 1 | `driver.rs` に `create_rag_pipeline_project` が含まれる | `rag_pipeline_fn_exists` テスト ✅ |
| 2 | RAG テンプレートが Ingest / Embed / Retrieve / Generate 4 ステージを含む | `rag_pipeline_has_four_stages` テスト ✅ |
| 3 | `CHANGELOG.md` に `[v38.6.0]` が含まれる | `changelog_has_v38_6_0` テスト ✅ |
| 4 | `Cargo.toml` バージョンが `38.6.0` | `cargo_toml_version_is_38_6_0` テスト ✅ |
| 5 | `cargo test` 全通過（failures=0 かつテスト数 ≥ 2769） | 実測: 2769 passed, 0 failed ✅ |
| 6 | `roadmap-v38.1-v39.0.md` の v38.6.0 が ✅ かつテスト件数が 4 件 | 確認済み ✅ |
| 7 | `versions/current.md` が v38.6.0（最新安定版）に更新されている | 確認済み ✅ |
