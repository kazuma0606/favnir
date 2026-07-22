# Tasks: v50.7.0 — `fav run --trace` / `fav run --watch` 強化

Status: COMPLETE
Date: 2026-07-19

---

## T0 — 事前確認

- [x] `cargo test` 3103 passed, 0 failed を確認（ベース確認）
- [x] `SeqStageCheck` ハンドラが `vm.rs` に存在し、`[TRACE] stage X: exit Ok(...)` を emit することを確認
- [x] `VERBOSE_LEVEL: Cell<u8>` の定義箇所を確認（`WATCH_FIELDS: RefCell<Vec<String>>` 追加位置の特定）
- [x] `truncate_for_trace` 関数が `vm.rs` に存在することを確認
- [x] `run_verbose` テストヘルパーが `driver.rs` に存在することを確認（`run_with_watch` の実装参考）
- [x] `VM::run_with_trace` の戻り型 `Ok((Value, Vec<EmitVal>, Vec<String>))` を確認（traces が第3要素）
- [x] `cargo clippy -- -D warnings` クリーンであることを確認（ベース）

## T1 — `vm.rs` — 構造化 trace + WATCH_FIELDS

- [x] `SeqStageCheck` Ok 分岐の `[TRACE]` emit を `[trace] stage=NAME  out=VALUE` に変更
- [x] `WATCH_FIELDS: RefCell<Vec<String>>` スレッドローカル追加（`VERBOSE_LEVEL` 近傍）
- [x] `pub fn set_watch_fields(fields: Vec<String>)` 追加
- [x] `fn watch_fields() -> Vec<String>` 追加（crate プライベート）
- [x] `SeqStageCheck` Ok 分岐の `uvm` 生成条件に `|| !watch_fields().is_empty()` を追加（watch が verbose と独立して動作するため）
- [x] `SeqStageCheck` Ok 分岐に watch フック追加（`has_watch` チェック → `VMValue::Record` パターンマッチ → フィールド存在確認 → `[watch] target: — → value  (stage: name)` emit）

## T2 — `driver.rs` — `run_with_watch` + `v507000_tests`

- [x] `run_with_watch(source, watch_targets)` テストヘルパー追加（`run_verbose` と同パターン）
- [x] `v507000_tests` モジュールを `v506000_tests` の直前に追加（3 件）:
  - [x] `cargo_toml_version_is_50_7_0`: version = "50.7.0" を assert
  - [x] `run_trace_structured_output`: 2-stage seq（WrapA/WrapB）で verbose=1 実行 → `[trace] stage=WrapA` + `out=` を含む行を assert、`exit Ok` が含まれないことを assert
  - [x] `run_watch_tracks_variable`: 2-stage seq (Parse/Pass) で `run_with_watch(source, &["amount"])` → `[watch] amount:` を含む行を assert
- [x] `v506000_tests::cargo_toml_version_is_50_6_0` を削除（`lsp_hover_builtin_fn` / `lsp_hover_rune_method` は保持）

**実装上の注意点（コードレビュー対応）:**
- `run_verbose` / `run_with_watch` は `v12500_tests` のスコープ外（crate レベル非公開）のため、`v507000_tests` 内に自己完結型として再定義
- single-stage seq では SeqStageCheck が発火しないため、テストは 2-stage seq 構成を使用
- 型注釈 `|l: &String|` を追加して E0282 型推論エラーを解消
- `verbose_logs_bind_result` テストが `exit Ok` フォーマット変更で壊れたため、assertion を `out=` に更新

## T3 — バージョン更新・完了

- [x] `fav/Cargo.toml` version → `"50.7.0"`
- [x] `cargo test` 3105 passed, 0 failed
- [x] `cargo clippy -- -D warnings` クリーン
- [x] `site/` の既存 trace/debug ドキュメントに `[trace]` フォーマット変更の影響がないか確認（既存 trace 関連ドキュメントなし — スキップ）
- [x] `CHANGELOG.md` に v50.7.0 エントリ追加
- [x] `versions/current.md` を v50.7.0（3105 tests）に更新
- [x] `versions/roadmap/roadmap-v50.1-v51.0.md` の v50.7.0 実績欄に制約事項を記録
- [x] tasks.md を COMPLETE に更新（T0〜T3 全 `[x]`）
