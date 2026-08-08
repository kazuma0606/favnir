# v63.6.0 タスクリスト

Status: COMPLETE
Version: 63.6.0
Base tests: 3416
Target tests: 3418

---

## T0: 事前確認

- [x] `cargo test -j 8 -- --test-threads=8` でベース 3416 tests passed, 0 failed を確認
- [x] `lint.rs` で W040 が v61.7.0「type hole `_` inferred」として実装済みで W041 が空きであることを確認
- [x] `lint.rs` の `LintConfig` に `perf: bool` フィールドが存在することを確認
- [x] `lint.rs` の `lint_program` 末尾が `check_w040_type_holes` で終わることを確認（W041 挿入位置確認）
- [x] `toml.rs` に `BackpressureConfig` が存在しないことを確認（新規追加）
- [x] `toml.rs` に `ParallelConfig` が存在することを確認（追加パターンの参照）
- [x] `driver.rs` に `v63500_tests` が存在することを確認（`v63600_tests` の挿入位置確認）

**非スコープ注意**: `vm.rs` への backpressure 実行時統合・W042 実装・`[backpressure]` の VM 注入は
本バージョンの非スコープ（後送り）。実装しないこと。spec.md §非スコープ を参照。

---

## T1: `lint.rs` — W041 関数群追加

- [x] ファイル末尾に `check_w041_perf_hint_large_collect` + 関連ヘルパー 5 関数を追加:
  - `check_w041_perf_hint_large_collect(program, errors)`
  - `check_w041_in_block(block, errors)`
  - `check_w041_in_expr(expr, errors)` — `Expr::Collect` を検出し W041 発火
  - `block_mentions_filter(block) -> bool` — "filter" の言及を検索
  - `stmt_mentions_name_w041(stmt, name) -> bool`
  - `expr_mentions_name_w041(expr, name) -> bool` — `Expr::Ident` / `Expr::FieldAccess` / `Expr::Apply` を再帰検索
- [x] `cargo build` でエラーなし

---

## T2: `lint.rs` — `lint_program_with_config` に W041 呼び出し追加 + `#[allow(dead_code)]` 削除

- [x] `LintConfig.perf` フィールドの `#[allow(dead_code)]` アトリビュートと「将来用」コメントを削除し、W041 用の doc コメントに更新する
- [x] `lint_program_with_config` の末尾（`errors` を返す直前）に追加:
  ```rust
  if config.perf || config.strict {
      check_w041_perf_hint_large_collect(program, &mut errors);
  }
  ```
- [x] `cargo build` でエラーなし

---

## T3: `toml.rs` — `BackpressureConfig` 構造体追加

- [x] `ParallelConfig` の直後（`// ── Build config` の直前）に追加:
  ```rust
  // ── Backpressure config (v63.6.0)
  pub struct BackpressureConfig {
      pub strategy: String,        // default: "block"
      pub max_queue_depth: usize,  // default: 500
      pub warn_threshold: usize,   // default: 400
  }
  impl Default for BackpressureConfig { ... }
  ```
- [x] `cargo build` でエラーなし

---

## T4: `toml.rs` — `FavToml` フィールド追加 + `parse_fav_toml` 4箇所更新

- [x] `FavToml` 構造体 — `parallel: Option<ParallelConfig>` の直後に追加:
  ```rust
  pub backpressure: Option<BackpressureConfig>,
  ```
- [x] `parse_fav_toml` ローカル変数 — `let mut parallel_cfg` の直後に追加:
  ```rust
  let mut backpressure_cfg: Option<BackpressureConfig> = None;
  ```
- [x] セクション検出 — `[parallel]` の直後に追加:
  ```rust
  if trimmed == "[backpressure]" { section = "backpressure"; continue; }
  ```
- [x] セクション処理 — `"parallel" => { ... }` ブロックの直後に追加:
  ```rust
  "backpressure" => { let mut current = backpressure_cfg.take()...; ... }
  ```
- [x] `FavToml { ... }` リテラル — `parallel: parallel_cfg,` の直後に追加:
  ```rust
  backpressure: backpressure_cfg,
  ```
- [x] `checker.rs`（2箇所）・`resolver.rs`（3箇所）・`driver.rs`（1箇所）の `FavToml { ... }` リテラルに `backpressure: None,` を追加する（`grep -rn "FavToml {" fav/src/` で漏れ確認）
- [x] `cargo build` でエラーなし

---

## T5: `driver.rs` — `v63600_tests` 追加

- [x] `v63500_tests` の直前（ファイル先頭方向）に以下を挿入:
  ```rust
  // -- v63600_tests (v63.6.0) -- バックプレッシャー制御 W041 lint + [backpressure] 設定 --
  #[cfg(test)]
  mod v63600_tests {
      fn lint_w041_large_collect() { ... }   // W041 が perf mode で発火
      fn backpressure_toml_parsed() { ... }  // strategy/max_queue_depth/warn_threshold を検証
  }
  ```
  （`use` 不要: フルパスで使用）
- [x] `cargo build` でエラーなし

---

## T6: ビルド・テスト

- [x] `cargo build` でコンパイルエラー 0（全ステップ完了後の最終確認）
- [x] `cargo test --bin fav v63600_tests` で 2 件 PASS
  - `lint_w041_large_collect` PASS
  - `backpressure_toml_parsed` PASS
- [x] `cargo test -j 8 -- --test-threads=8` で 3418 tests passed, 0 failed を確認

---

## T7: ドキュメント更新

- [x] `CHANGELOG.md` 先頭に v63.6.0 エントリを追加
- [x] `versions/roadmap/roadmap-v63.1-v64.0.md` v63.6.0 セクションに実績追記
- [x] `versions/current.md` の「進行中」を v63.6.0（3418 tests）に更新
- [x] tasks.md を COMPLETE に更新（本ファイル）
- ~~`site/` MDX 追加~~ — 非スコープ
- ~~`vm.rs` バックプレッシャー実行時統合~~ — 非スコープ

---

## 完了サマリー

- Status: COMPLETE
- Tests: 3418 passed, 0 failed
- 主要実装: `lint.rs`（W041 + ヘルパー + `lint_program_with_config` 更新 + `#[allow(dead_code)]` 削除）+ `toml.rs`（`BackpressureConfig`）+ `driver.rs`（`v63600_tests`）+ `checker.rs`（2箇所 `backpressure: None`）+ `resolver.rs`（3箇所 `backpressure: None`）
- 完了日: 2026-08-02
