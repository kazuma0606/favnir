# v66.8.0 タスクリスト

Status: COMPLETE
Version: 66.8.0
Base tests: 3489
Target tests: 3491
Actual tests: 3491

---

## T0: 事前確認

- [x] `cargo test -j 8 -- --test-threads=8` でベース 3489 tests passed, 0 failed を確認
- [x] `fav/Cargo.toml` の version が `"66.0.0"` であることを確認（sub-version では変更しない）
- [x] `fav/src/lint.rs` に `W054` が存在することを確認（W055〜W059 の挿入位置の目印）
- [x] `fav/src/lint.rs` に `W055` が存在しないことを確認（新規追加）
- [x] `driver.rs` に `v66700_tests` が存在することを確認（`v66800_tests` の挿入位置）
- [x] `driver.rs` に `v66800_tests` が存在しないことを確認（新規追加）
- [x] `cargo test --bin fav v66700_tests` で 2 件 PASS することを確認（前バージョンが正常）
  - 前バージョンのテスト関数名: `feature_store_define_feature`, `feature_store_versioned_retrieval`
- [x] `versions/current.md` の「進行中バージョン」が `v66.7.0` であることを確認

---

## T1: `lint.rs` — W055〜W059 スタブ追加

- [x] `run_all_checks`（lint.rs 行 164〜169 の直後）に W055〜W059 の呼び出しを追加（W050〜W054 と同形式）
  - `// v66.8.0: W055〜W059 AI Pipeline Lint Rules` コメント付き
- [x] ブロックヘッダーコメント `// ── W055〜W059: AI Pipeline Lint Rules (v66.8.0)` を追加
- [x] `check_w055_untyped_llm_output` スタブ関数を追加
- [x] `check_w056_dim_implicit_cast` スタブ関数を追加
- [x] `check_w057_query_without_upsert` スタブ関数を追加
- [x] `check_w058_unbuffered_stream_inference` スタブ関数を追加
- [x] `check_w059_llm_no_retry` スタブ関数を追加
- [x] スタブ関数は `pub` なし、引数はアンダースコアプレフィックス（W050〜W054 と同形式）
- [x] `cargo build` でエラーなし

---

## T2: `driver.rs` — `v66800_tests` 追加

- [x] `// -- v66700_tests (v66.7.0)` コメントの直前に `v66800_tests` を挿入
  - [x] `lint_w055_untyped_llm_output`: lint.rs に `"W055"` / `"W056"` を含む
  - [x] `lint_w056_dim_implicit_cast`: lint.rs に `"W057"` / `"W058"` / `"W059"` を含む
  - [x] `include_str!` パスが `"lint.rs"`（`fav/src/` 内）
- [x] `use super::*` は不要（`include_str!` のみ使用）
- [x] `cargo build` でエラーなし

---

## T3: ビルド・テスト

- [x] `cargo test --bin fav v66800_tests` で 2 件 PASS
  - [x] `lint_w055_untyped_llm_output` PASS
  - [x] `lint_w056_dim_implicit_cast` PASS
- [x] `cargo test -j 8 -- --test-threads=8` で 3491 tests passed, 0 failed を確認

---

## T4: ドキュメント・ステータス更新

> T3 のテスト全通過（3491 tests passed）を確認してから実施すること。

- [x] `versions/roadmap/roadmap-v66.1-v67.0.md` のバージョン一覧表で v66.8.0 の「状態」列を「完了」に変更し、変更後に当該行が「完了」になっていることを目視確認
- [x] `versions/current.md` の「進行中バージョン」を v66.8.0 に更新
- [x] 本 `tasks.md` を COMPLETE に更新（全チェックボックスを `[x]` に）

> **CHANGELOG 方針**: v66.1〜v66.9 では CHANGELOG.md を更新しない。v67.0.0 宣言時に一括追記する。
> **MDX 方針**: `site/` の MDX ドキュメントは v66.9.0 安定化時に一括作成するため今バージョンは省略。

---

## コードレビュー指摘と対応

- [HIGH] spec-reviewer: `"../lint.rs"` パスはコンパイルエラー → `"lint.rs"` に修正
- [HIGH] spec-reviewer: W050〜W054 は `run_all_checks` 登録済みなのに W055〜W059 は登録しない方針が矛盾 → 登録するよう方針変更、実装済み
- [HIGH] spec-reviewer: ロードマップのコメントが「検出する」とスコープ縮小と不整合 → ロードマップにスコープ縮小注記を追加
- [MED] spec-reviewer: テスト関数名とアサート対象コード範囲のずれ → ロードマップ指定の名称であることを plan.md に明記
- [LOW] spec-reviewer: テスト数 +2 の根拠が spec に未記載 → 技術ノートに追記
