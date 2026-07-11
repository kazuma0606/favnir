# v38.7.0 タスクリスト — Llm Rune 強化

## ステータス: COMPLETE

> ロードマップ整合: `roadmap-v38.1-v39.0.md` の v38.7.0（「Llm Rune 強化」）に沿ったバージョン。

## T0: 事前確認

- [x] `cargo test` の実測通過数を確認（目安: 2770（v38.6.0 完了時点の実績値））し、実測値をここに記録: 2770
- [x] Cargo.toml バージョンが `38.6.0` であることを確認
- [x] `v38600_tests::cargo_toml_version_is_38_6_0` がライブアサーション（`assert!(cargo.contains("38.6.0"), ...)`）であることを確認し、行番号を Grep で確認して記録: 43864
- [x] `v38600_tests` の他テスト（`changelog_has_v38_6_0` / `rag_pipeline_fn_exists` / `rag_pipeline_has_four_stages` / `fav_new_rag_pipeline_ok`）はバージョン変更後も pass することを確認（`changelog_has_v38_6_0` は CHANGELOG に v38.6.0 エントリが残るため pass、`rag_pipeline_*` は driver.rs の内容を検索するため削除しない限り pass）
- [x] `driver.rs` に `v38700_tests` モジュールが存在しないことを確認（今回新規作成）
- [x] `v38600_tests` の閉じ `}` の行番号を Grep/Read で確認し、ここに記録: 43907
- [x] `CHANGELOG.md` に `[v38.7.0]` エントリが存在しないことを確認（今回新規作成）
- [x] `vm.rs` に `Llm.stream_raw` / `Llm.function_call_raw` / `Llm.embed_raw` が存在しないことを確認（今回追加）
- [x] `runes/llm/client.fav` に `stream` / `function_call` / `embed` が存在しないことを確認（今回追加）
- [x] `versions/current.md` の最新安定版が `v38.6.0`・次バージョンが `v38.7.0` であることを確認
- [x] `versions/roadmap/roadmap-v38.1-v39.0.md` の v38.7.0 が未完了（✅ なし）であることを確認（T12 で更新）

## T1: CHANGELOG.md に [v38.7.0] エントリを追加

- [x] `## [v38.6.0]` の直前に `## [v38.7.0]` エントリを挿入
- [x] 日付を `2026-07-10` に設定
- [x] セパレータが `—`（全角ダッシュ U+2014）形式であることを確認

## T2: `vm.rs` — 3 VM primitive ディスパッチ追加

- [x] Read で `Llm.extract_raw` ブロック末尾と `// ── Snowflake` セクションコメントの行番号を確認
- [x] その間に `Llm.stream_raw` / `Llm.function_call_raw` / `Llm.embed_raw` ディスパッチを追加（spec.md §1 に従う）
- [x] `include_str!("backend/vm.rs")` パスが driver.rs から正しく解決できることを確認（コンパイルで確認）

## T3: `vm.rs` — `llm_embed` ヘルパー関数追加

- [x] Read で `llm_call_chat` の終端 `}` の行番号を確認（line 6676）
- [x] その直後（line 6677、`// ── Snowflake helpers (v10.2.0)` の前）に `llm_embed` 関数を追加（spec.md §1 に従う）
- [x] `#[cfg(not(target_arch = "wasm32"))]` を `fn llm_embed` の前に付与していることを確認
- [x] ディスパッチアームには `#[cfg]` を付与していないことを確認
- [x] `ureq::Error::Status` / `ureq::Error::Transport` の両アームを実装していることを確認

## T4: `driver.rs` — primitives テーブルに 3 エントリ追加

- [x] Read で `Llm.extract_raw` エントリの行番号を確認
- [x] その直後に `Llm.stream_raw` / `Llm.function_call_raw` / `Llm.embed_raw` の `p!()` エントリを追加（spec.md §2 に従う）

## T5: `runes/llm/client.fav` — 3 公開関数追加

- [x] Read で `extract<T>` 関数の末尾 `}` の行番号を確認
- [x] その直後に `stream` / `function_call` / `embed` 関数を追加（spec.md §3 に従う）
- [x] `Llm.stream_raw` / `Llm.function_call_raw` / `Llm.embed_raw` を呼び出していることを確認

## T6: `runes/llm/llm.fav` — use 宣言を更新

- [x] `use client.{ complete, chat, extract }` を `use client.{ complete, chat, extract, stream, function_call, embed }` に更新

## T7: `runes/llm/llm.test.fav` — 3 テスト追加

- [x] 既存 3 テストの直後に `llm_stream_no_key_is_err` / `llm_function_call_no_key_is_err` / `llm_embed_no_provider_is_err` を追加（spec.md §5 に従う）
- [x] `llm_embed_no_provider_is_err` は `LLM_PROVIDER` 未設定（デフォルト anthropic）のとき API キー不要で `Err` を返すことを確認（`ANTHROPIC_API_KEY` unset に依存しない）
- [x] `llm_stream_no_key_is_err` / `llm_function_call_no_key_is_err` は `ANTHROPIC_API_KEY` unset で `Err` を返すことを確認（既存 `llm_rune_test_file_passes` の `remove_var` 前処理で担保）

## T8: `driver.rs` — `v38600_tests::cargo_toml_version_is_38_6_0` をスタブ化

- [x] Grep で `cargo_toml_version_is_38_6_0` の行番号を確認（43864）
- [x] ライブアサーション → `// Stubbed: version bumped to 38.7.0 — assertion intentionally removed` に変更
- [x] `changelog_has_v38_6_0` / `rag_pipeline_*` / `fav_new_rag_pipeline_ok` テストはスタブ化しないことを確認
- [x] スタブ形式が前バージョンのスタブと一致していることを確認

## T9: `driver.rs` — `v38700_tests` モジュールを新規追加（T1 完了後に実施）

- [x] T1（CHANGELOG 追加）が完了していることを確認してから着手
- [x] `v38600_tests` の閉じ `}` の行番号（T0 で記録済み: 43907）を Read で特定してから Edit を実行
- [x] `v38600_tests` の閉じ `}` の後に `v38700_tests` モジュールを追加（3 テスト）
  - [x] `cargo_toml_version_is_38_7_0`
  - [x] `changelog_has_v38_7_0`
  - [x] `llm_rune_enhanced_primitives_exist`（vm.rs に 3 primitive が含まれる、`include_str!("backend/vm.rs")`）

## T10: Cargo.toml バージョン更新（T1〜T9 すべて完了後）

- [x] `fav/Cargo.toml` バージョンを `38.7.0` に更新

## T11: テスト実行

- [x] T10（Cargo.toml 更新）が完了していることを確認してから着手
- [x] `cargo test` 全通過 — ≥ 2773 passed; 0 failed — 実測: 2774 passed, 0 failed（code-reviewer 対応で +1）
- [x] `v38700_tests` の 4 テストがすべて pass（code-reviewer [MED] 対応: `llm_test_fav_has_new_functions` 追加）
- [x] `cargo_toml_version_is_38_7_0` が pass
- [x] `changelog_has_v38_7_0` が pass
- [x] `llm_rune_enhanced_primitives_exist` が pass
- [x] `llm_rune_test_file_passes`（既存テスト）も pass（新しい llm.test.fav テストを含む）
- [x] **修正**: `include_str!("../backend/vm.rs")` → `include_str!("backend/vm.rs")` に修正（spec のパス記述誤りを実装時に発見・修正）

## T12: ドキュメント更新（T11 完了後）

- [x] `versions/v36-v40/v38.7.0/tasks.md` を COMPLETE ステータスに更新（T0〜T12 全チェックボックスを `[x]` に）
- [x] `versions/current.md` を v38.7.0（最新安定版）・v38.8.0（次バージョン）に更新
- [x] `versions/roadmap/roadmap-v38.1-v39.0.md` の v38.7.0 を完了済みにマーク（✅）し、テスト件数を 3 件に更新
- [x] roadmap の v38.7.0 行を Read で確認し ✅ が含まれることをここに記録: ✅ 確認: ✅ v38.7.0 — Llm Rune 強化
- [x] roadmap の v38.7.0 行を Read で確認し「3 件」が含まれることをここに記録: テスト件数 3 件確認: Rust テスト 3 件（2773 tests passed, 0 failed）
- [x] `versions/current.md` を Read で確認し「v38.7.0」が最新安定版として含まれることをここに記録: 確認: **v38.7.0** — Llm Rune 強化（stream / function_call / embed）（2026-07-10）

---

## 完了条件チェックリスト（spec.md 対応）

| # | spec.md 完了条件 | 確認方法 |
|---|---|---|
| 1 | `vm.rs` に `Llm.stream_raw`, `Llm.function_call_raw`, `Llm.embed_raw` が含まれる | `llm_rune_enhanced_primitives_exist` テスト ✅ |
| 2 | `CHANGELOG.md` に `[v38.7.0]` が含まれる | `changelog_has_v38_7_0` テスト ✅ |
| 3 | `Cargo.toml` バージョンが `38.7.0` | `cargo_toml_version_is_38_7_0` テスト ✅ |
| 4 | `cargo test` 全通過（failures=0 かつテスト数 ≥ 2773） | 実測: 2773 passed, 0 failed ✅ |
| 5 | `roadmap-v38.1-v39.0.md` の v38.7.0 が ✅ かつテスト件数が 3 件 | 確認済み ✅ |
| 6 | `versions/current.md` が v38.7.0（最新安定版）に更新されている | 確認済み ✅ |

## 実装中に発見・修正した問題

| 問題 | 修正内容 |
|---|---|
| `include_str!("../backend/vm.rs")` のパス誤り（spec 記述ミス） | `include_str!("backend/vm.rs")` に修正。`fav/src/driver.rs` から `fav/src/backend/vm.rs` は `backend/vm.rs` が正しい相対パス。spec.md / tasks.md の注意事項も更新済み。 |
