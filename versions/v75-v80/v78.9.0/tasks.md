# v78.9.0 タスクリスト — 安定化・コードフリーズ

Date: 2026-08-16
Status: COMPLETE

---

## T0: 事前確認

- [x] `fav/Cargo.toml` のバージョンが `78.8.0` であることを確認
- [x] `cargo test` が全 pass（3781 tests）であることを確認
- [x] `fav/tmp/hello.fav` が存在することを確認

---

## T1: CHANGELOG.md 更新（テスト追加より先）

- [x] `CHANGELOG.md` の先頭に v78.9.0 エントリを追加する（形式: `## [v78.9.0] — 2026-08-16 — 安定化・コードフリーズ`）
- [x] Added セクション:「なし（新機能追加なし）」と明記する
- [x] Tests セクション（2 件）を含める

---

## T2: driver.rs — テストモジュール追加

- [x] `fav/src/driver.rs` の末尾に `// --- v78.9.0: 安定化・コードフリーズ ---` コメントを追加する
- [x] `v789000_tests` モジュールを追加する（`use super::*` 必須）
- [x] `execution_effects_full_sprint_all_stable` テストを実装する
  - v78.1: `check_cache_valid` が TTL 内でtrue を返すことを assert
  - v78.2: `hit_rate(&simulate_lru_cache(...))` が 0.0 以上であることを assert
  - v78.3: `select_join_strategy` が小テーブルで BroadcastJoin を返すことを assert
  - v78.4: `select_min_cost_strategy` が broadcast/hash コスト比較で正しい戦略を返すことを assert
  - v78.5: `format_execution_plan` の出力が "Execution Plan:" を含むことを assert
  - v78.6: `plan_parallel_execution` の返値が partition_count と一致することを assert
  - v78.7: `select_execution_mode` が小データ+高レイテンシ許容で Adaptive を返すことを assert
  - v78.8: `insert_plan` → `lookup_plan` でキャッシュヒットすることを assert
- [x] `execution_effects_e2e_pipeline_runs` テストを実装する
  - モード選択 → コスト推定 → 戦略選択 → 計画構築 → 可視化 → キャッシュ挿入 → 取得の E2E フロー
  - `select_execution_mode(10_000, 1_000, selector)` → `Batch` を assert
  - `select_min_cost_strategy` → `BroadcastJoin` を assert
  - `format_execution_plan` の出力が "E2EPipeline" / "BroadcastJoin" を含むことを assert
  - `lookup_plan` → `Some` / pipeline 名を assert
- [x] `cargo test v789000` で 2 件が pass することを確認する

---

## T3: Cargo.toml バージョン更新

- [x] `fav/Cargo.toml` の `version` を `"78.8.0"` → `"78.9.0"` に変更する
- [x] driver.rs 内の `78.8.0` バージョン文字列アサーションを `78.9.0` に一括更新（`replace_all: true`）
- [x] **replace_all 後に** `grep -c "78.8.0" /c/Users/yoshi/favnir/fav/src/driver.rs` を実行し **出力が 1** であることを確認する（Git Bash で実行）
  - 残るのは `// --- v78.8.0: 実行計画キャッシュ ---` の 1 件のみ

---

## T4: versions/current.md 更新

- [x] `## 進行中バージョン` 欄を `**v78.9.0**（安定化・コードフリーズ）` に更新する
- [x] `## 次に切る版` 欄を `**v79.0.0**（Execution Effects 1.0 宣言）` に更新する

---

## T5: 最終確認

- [x] `cargo test` が全 pass であることを確認する（3783 tests）
- [x] `cargo test v789000` で 2 件が pass することを確認する
- [x] `fav/Cargo.toml` のバージョンが `78.9.0` であることを確認する
- [x] `CHANGELOG.md` の先頭が `[v78.9.0]` であることを確認する
- [x] `versions/current.md` の「進行中バージョン」が v78.9.0 であることを確認する

---

## 完了チェックリスト

- [x] 全タスク（T0〜T5）が完了している
- [x] `execution_effects_full_sprint_all_stable` が pass
- [x] `execution_effects_e2e_pipeline_runs` が pass
- [x] テスト総数: 3783（+2）
- [x] 新機能追加なし（統合テストのみ）
- [x] site/ MDX 追加: 対象外
- [x] `changelog_has_v78_9_0` テストの追加: 対象外（x.0.0 宣言バージョンのみに追加する慣例）
