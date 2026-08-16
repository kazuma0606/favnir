# v78.8.0 タスクリスト — 実行計画キャッシュ

Date: 2026-08-16
Status: COMPLETE

---

## T0: 事前確認

- [x] `fav/Cargo.toml` のバージョンが `78.7.0` であることを確認
- [x] `cargo test` が全 pass（3778 tests）であることを確認
- [x] `fav/tmp/hello.fav` が存在することを確認

---

## T1: driver.rs — 型・関数追加

- [x] `fav/src/driver.rs` の末尾に `// --- v78.8.0: 実行計画キャッシュ ---` コメントを追加する
- [x] `PlanCacheEntry` 構造体を追加する（`#[derive(Debug, Clone, PartialEq)]`、Eq/Hash なし）
  - `pipeline_hash: String`, `plan: ExecutionPlan`, `created_at: i64`
- [x] `PlanCache` 構造体を追加する（`#[derive(Debug, Clone, PartialEq)]`、Eq/Hash なし）
  - `entries: Vec<PlanCacheEntry>`, `max_size: usize`
- [x] `lookup_plan<'a>(cache: &'a PlanCache, hash: &str) -> Option<&'a ExecutionPlan>` を追加する
  - `entries.iter().find(|e| e.pipeline_hash == hash).map(|e| &e.plan)`
- [x] `insert_plan(cache: &mut PlanCache, hash: &str, plan: ExecutionPlan)` を追加する
  - `max_size == 0` → 即リターン
  - 既存 hash → `plan` と `created_at` を上書き（早期リターン）
  - 新規 + `len >= max_size` → `created_at` 最小エントリを `remove` → `push`
  - 新規 + `len < max_size` → `push`
  - `created_at` は `plan.total_cost.io_ops as i64` を使用
- [x] `cargo build` でコンパイルエラーがないことを確認する
- [x] `cargo test` で既存 3778 tests が pass することを確認する

---

## T2: CHANGELOG.md 更新（テスト追加より先）

- [x] `CHANGELOG.md` の先頭に v78.8.0 エントリを追加する（形式: `## [v78.8.0] — 2026-08-16 — 実行計画キャッシュ`）
- [x] Added セクション（構造体 2 件・関数 2 件）と Tests セクション（2 件）を含める

---

## T3: driver.rs — テストモジュール追加

- [x] `fav/src/driver.rs` に `v788000_tests` モジュールを追加する（`use super::*` 必須）
- [x] `make_empty_plan(pipeline: &str)` / `make_cache(max_size: usize)` ヘルパーを実装する
- [x] `plan_cache_hit` テストを実装する
  - `insert_plan` → `lookup_plan` → `Some` を assert
  - 存在しない hash → `None` を assert
- [x] `plan_cache_evicts_oldest_on_full` テストを実装する
  - `max_size=2` で plan_a(io_ops=10) / plan_b(io_ops=20) を挿入
  - plan_c(io_ops=30) を挿入 → plan_a がエビクション
  - `lookup_plan("hash_a")` → `None` を assert
  - `lookup_plan("hash_b")` / `lookup_plan("hash_c")` → `Some` を assert
- [x] `cargo test v788000` で 2 件が pass することを確認する

---

## T4: Cargo.toml バージョン更新

- [x] `fav/Cargo.toml` の `version` を `"78.7.0"` → `"78.8.0"` に変更する
- [x] driver.rs 内の `78.7.0` バージョン文字列アサーションを `78.8.0` に一括更新（`replace_all: true`）
- [x] **replace_all 後に** `grep -c "78.7.0" /c/Users/yoshi/favnir/fav/src/driver.rs` を実行し **出力が 1** であることを確認する（Git Bash で実行すること）
  - 残るのは `// --- v78.7.0: Stream / Batch 統合実行モード ---` の 1 件のみ
- [x] `cargo build` 実行後に `fav/Cargo.lock` が自動更新されていることを確認する

---

## T5: versions/current.md 更新

- [x] `## 進行中バージョン` 欄を `**v78.8.0**（実行計画キャッシュ）` に更新する
- [x] `## 次に切る版` 欄を `**v78.9.0**（安定化・コードフリーズ）` に更新する

---

## T6: 最終確認

- [x] `cargo test` が全 pass であることを確認する（3780 tests）
- [x] `cargo test v788000` で 2 件が pass することを確認する
- [x] `fav/Cargo.toml` のバージョンが `78.8.0` であることを確認する
- [x] `CHANGELOG.md` の先頭が `[v78.8.0]` であることを確認する
- [x] `versions/current.md` の「進行中バージョン」が v78.8.0 であることを確認する

---

## 完了チェックリスト

- [x] 全タスク（T0〜T6）が完了している
- [x] `plan_cache_hit` が pass
- [x] `plan_cache_evicts_oldest_on_full` が pass
- [x] テスト総数: 3781（+3、code-reviewer 対応で境界値テスト plan_cache_boundary_cases +1）
- [x] site/ MDX 追加: 本バージョンでは対象外（型基盤のみ）
- [x] `changelog_has_v78_8_0` テストの追加: 対象外（x.0.0 宣言バージョンのみに追加する慣例）
