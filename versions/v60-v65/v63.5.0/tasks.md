# v63.5.0 タスクリスト

Status: COMPLETE
Version: 63.5.0
Base tests: 3414
Target tests: 3416

---

## T0: 事前確認

- [x] `cargo test -j 8 -- --test-threads=8` でベース 3414 tests passed, 0 failed を確認
- [x] `fav/Cargo.toml` に `sysinfo` が存在しないことを確認（新規追加予定）
- [x] `driver.rs` に `cmd_profile_compare` が存在することを確認（挿入位置確認）
- [x] `driver.rs` に `v63400_tests` が存在することを確認（`v63500_tests` の挿入位置確認）
- [x] `fav/src/ast.rs` の `Program` 構造体に `items: Vec<Item>` フィールドが存在し、`Item::TrfDef(TrfDef)` が stage 定義であり `TrfDef.name: String` が stage 名であることを確認（`cmd_profile_memory` の実装前提）

**非スコープ注意**: `main.rs` への `--memory` CLI フラグ追加・per-row 実計測（jemalloc 統合）・
複数回実行平均化は本バージョンの非スコープ。実装しないこと。spec.md §非スコープ を参照。

---

## T1: `Cargo.toml` — `sysinfo` 追加

- [x] `[target.'cfg(not(target_arch = "wasm32"))'.dependencies]` セクション末尾に追加:
  ```toml
  sysinfo = "0.30"
  ```
- [x] `cargo build` でエラーなし

---

## T2: `driver.rs` — `cmd_profile_memory` 追加

- [x] `cmd_profile_compare` の直前に追加:
  ```rust
  pub fn cmd_profile_memory(src: &str, json_mode: bool) -> String { ... }
  ```
  - `Parser::parse_str` で stage 名を取得
  - `#[cfg(not(target_arch = "wasm32"))]` ブロックで `sysinfo::System` を使い RSS 計測
  - `#[cfg(target_arch = "wasm32")]` ブロックで `peak_rss_mb = 0`
  - `json_mode` に応じて JSON 配列 / テーブル形式で返す
  - `"Total peak"` 行を末尾に追加
- [x] `cargo build` でエラーなし

---

## T3: `driver.rs` — `v63500_tests` 追加

- [x] `v63400_tests` の直前（ファイル先頭方向）に以下を挿入:
  ```rust
  // -- v63500_tests (v63.5.0) -- メモリプロファイリング fav profile --memory --
  #[cfg(test)]
  mod v63500_tests {
      #[test]
      fn profile_memory_flag_works() { ... }
      #[test]
      fn profile_memory_per_stage() { ... }
  }
  ```
  （`use` 不要: `crate::driver::cmd_profile_memory` をフルパスで使用）
- [x] `cargo build` でエラーなし

---

## T4: ビルド・テスト

- [x] `cargo build` でコンパイルエラー 0（全ステップ完了後の最終確認）
- [x] `cargo test v63500_tests` で 2 件 PASS
  - `profile_memory_flag_works` PASS
  - `profile_memory_per_stage` PASS
- [x] `cargo test -j 8 -- --test-threads=8` で 3416 tests passed, 0 failed を確認

---

## T5: ドキュメント更新

- [x] `CHANGELOG.md` 先頭に v63.5.0 エントリを追加
- [x] `versions/roadmap/roadmap-v63.1-v64.0.md` v63.5.0 セクションに実績追記
- [x] `versions/current.md` の「進行中」を v63.5.0（3416 tests）に更新
- [x] tasks.md を COMPLETE に更新（本ファイル）
- ~~`site/` MDX 追加~~ — 非スコープ（v63.x 以降）

---

## 完了サマリー

- Status: COMPLETE
- Tests: 3416 passed, 0 failed
- 主要実装: `Cargo.toml`（`sysinfo` 追加）+ `driver.rs`（`cmd_profile_memory` + `v63500_tests`）
- 完了日: 2026-08-02
