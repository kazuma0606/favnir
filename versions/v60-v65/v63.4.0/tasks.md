# v63.4.0 タスクリスト

Status: COMPLETE
Version: 63.4.0
Base tests: 3412
Target tests: 3414

---

## T0: 事前確認

- [x] `cargo test -j 8 -- --test-threads=8` でベース 3412 tests passed, 0 failed を確認
- [x] `fav/src/toml.rs` に `BuildConfig` 構造体と `build: Option<BuildConfig>` が存在することを確認（追加パターンの参照）
- [x] `fav/src/toml.rs` の `FavToml` に `parallel` フィールドがまだ存在しないことを確認
- [x] `driver.rs` に `v63300_tests` が存在することを確認（挿入位置確認）
- [x] `toml.rs` に `parse_fav_toml_pub` が存在することを確認（テストから使用）

**非スコープ注意**: ロードマップ v63.4.0 の「`vm.rs` への `ParallelConfig` 注入」および
「`fav run --parallel-stats` CLI フラグ」は本バージョンの非スコープ（後送り）。
実装しないこと。spec.md §非スコープ を参照。

---

## T1: `toml.rs` — `ParallelConfig` 構造体追加

- [x] `BuildConfig` 構造体の直前に `ParallelConfig` 構造体と `impl Default` を追加:
  ```rust
  #[derive(Debug, Clone)]
  pub struct ParallelConfig {
      pub max_threads: usize,  // 0 = CPU コア数
      pub queue_depth: usize,  // デフォルト: 256
  }
  impl Default for ParallelConfig { ... }
  ```
- [x] `cargo build` でエラーなし

---

## T2: `toml.rs` — `FavToml` フィールド追加 + `parse_fav_toml` 4箇所更新

- [x] `FavToml` 構造体 — `build: Option<BuildConfig>` の直後に追加:
  ```rust
  pub parallel: Option<ParallelConfig>,
  ```
- [x] `parse_fav_toml` ローカル変数 — `let mut build_cfg` の直後に追加:
  ```rust
  let mut parallel_cfg: Option<ParallelConfig> = None;
  ```
- [x] セクション検出 — `[build]` の直後に追加:
  ```rust
  if trimmed == "[parallel]" { section = "parallel"; continue; }
  ```
- [x] セクション処理 — `"build" => { ... }` ブロックの直後に追加:
  ```rust
  "parallel" => { let mut current = parallel_cfg.take()...; ... }
  ```
- [x] `FavToml { ... }` リテラル — `build: build_cfg,` の直後に追加:
  ```rust
  parallel: parallel_cfg,
  ```
- [x] `cargo build` でエラーなし（FavToml 定義とリテラルの両方が揃っていることを確認）

---

## T3: `driver.rs` — `cmd_parallel_stats` 追加

- [x] `cmd_run_with_cache` の直後に追加:
  ```rust
  pub fn cmd_parallel_stats(toml_content: &str) -> String { ... }
  ```
  （`parse_fav_toml_pub` → `parallel.unwrap_or_default()` → `available_parallelism()` で effective 計算）
- [x] `cargo build` でエラーなし

---

## T4: `driver.rs` — `v63400_tests` 追加

- [x] `v63300_tests` の直前（ファイル先頭方向）に以下を挿入:
  （`use` 不要: `crate::toml::parse_fav_toml_pub` / `crate::driver::cmd_parallel_stats` をフルパスで使用）
  ```rust
  // -- v63400_tests (v63.4.0) -- par 動的スレッドプール・[parallel] fav.toml 設定 --
  #[cfg(test)]
  mod v63400_tests { ... }
  ```
- [x] `cargo build` でエラーなし

---

## T5: ビルド・テスト

- [x] `cargo build` でコンパイルエラー 0（全ステップ完了後の最終確認）
- [x] `cargo test v63400` で 2 件 PASS
  - `parallel_toml_config_parsed` PASS
  - `parallel_stats_output` PASS
- [x] `cargo test -j 8 -- --test-threads=8` で 3414 tests passed, 0 failed を確認

---

## T6: ドキュメント更新

- [x] `CHANGELOG.md` 先頭に v63.4.0 エントリを追加
- [x] `versions/roadmap/roadmap-v63.1-v64.0.md` v63.4.0 セクションに実績追記
- [x] `versions/current.md` の「進行中」を v63.4.0（3414 tests）に更新
- [x] tasks.md を COMPLETE に更新（本ファイル）
- ~~`site/` MDX 追加~~ — 非スコープ（v63.x 以降）

---

## 完了サマリー

- Status: COMPLETE
- Tests: 3414 passed, 0 failed
- 主要実装: `toml.rs`（`ParallelConfig` + `FavToml` フィールド + パース）+ `driver.rs`（`cmd_parallel_stats` + `v63400_tests`）
- 完了日: 2026-08-02
