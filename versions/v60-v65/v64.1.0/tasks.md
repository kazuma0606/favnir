# v64.1.0 タスクリスト

Status: COMPLETE
Version: 64.1.0
Base tests: 3431
Target tests: 3433

---

## T0: 事前確認

- [x] `cargo test -j 8 -- --test-threads=8` でベース 3431 tests passed, 0 failed を確認
- [x] `driver.rs` に `cmd_build_basic` が存在し、`lower_to_object_pub` を呼んでいることを確認（`cmd_build_ci` で同様に使用するため）
- [x] `driver.rs` に `cmd_build_ci` が存在しないことを確認（新規追加）
- [x] `driver.rs` の `try_cmd_new` match に `"rag-pipeline"` アームが存在することを確認
- [x] `driver.rs` に `v64000_tests` が存在することを確認（`v64100_tests` の挿入位置）
- [x] `driver.rs` に `v64100_tests` が存在しないことを確認

---

## T1: `driver.rs` — `cmd_build_ci` 追加

- [x] `cmd_build_link_target` の直後に `cmd_build_ci(src: &str, out: &str) -> String` を追加
- [x] 出力フォーマット: `"ci: ok — ..."` / `"ci: error: parse error: ..."` / `"ci: error: build error: ..."`
- [x] `cargo build` でエラーなし

---

## T2: `driver.rs` — `create_ci_workflow_project` + `try_cmd_new` 更新

- [x] `create_rag_pipeline_project` の直後に `create_ci_workflow_project(root, name)` を追加
  - `pipeline.fav`、`fav.toml`、`.github/workflows/build.yml` を生成
  - ワークフローに `fav build pipeline.fav --link --ci` ステップを含める
- [x] `pub(crate) fn create_ci_workflow_project_pub` ラッパーを追加（テスト用）
- [x] `try_cmd_new` の `"rag-pipeline"` の直後に `"ci-workflow"` アームを追加
- [x] エラーメッセージ末尾に `ci-workflow` を追記
- [x] `cargo build` でエラーなし

---

## T3: `driver.rs` — `v64100_tests` 追加

- [x] `v64000_tests` の直前に `v64100_tests` を挿入
  - `build_ci_flag_output_format`
  - `new_template_has_ci_workflow`
- [x] `cargo build` でエラーなし

---

## T4: ビルド・テスト

- [x] `cargo test --bin fav v64100_tests` で 2 件 PASS
  - `build_ci_flag_output_format` PASS
  - `new_template_has_ci_workflow` PASS
- [x] `cargo test -j 8 -- --test-threads=8` で 3433 tests passed, 0 failed を確認

---

## T5: ドキュメント更新

- [x] `CHANGELOG.md` 先頭に v64.1.0 エントリを追加
- [x] `versions/roadmap/roadmap-v64.1-v65.0.md` v64.1.0 セクションに実績追記（完了条件テスト数 3420 → 3433 に修正）
- [x] `versions/current.md` の「進行中」を v64.1.0（3433 tests）に更新
- [x] `site/` MDX 追加は非スコープ（`--ci` フラグのサイトドキュメント化は後送り）
- [x] tasks.md を COMPLETE に更新（本ファイル）
