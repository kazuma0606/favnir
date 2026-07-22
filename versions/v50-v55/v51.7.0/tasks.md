# Tasks: v51.7.0 — WASM ビルドサイズ最適化

Status: COMPLETE
Date: 2026-07-20

---

## T0 — 事前確認

- [x] `cargo test` 3126 passed, 0 failed を確認（ベース確認）
- [x] `cargo clippy -- -D warnings` クリーンであることを確認
- [x] `wasm_opt_pass.rs` に `WasmOptLevel::Os` が**存在しない**ことを確認（新規追加対象）
- [x] `wasm_dce.rs` に `dce_from_exports` が**存在しない**ことを確認（新規追加対象）
- [x] `cmd_build "wasm"` アームが `build_wasm_artifact` を呼んでいることを確認（変更対象）
- [x] `build_wasm_artifact_with_config` が `driver.rs` に存在することを確認（切替先）
- [x] `WasmBuildConfig` が `driver.rs` に存在することを確認（config 構築で使用）
- [x] `benchmarks/v51.7.0.json` が**存在しない**ことを確認（新規作成対象）
- [x] `benchmarks/` がリポジトリルート直下（`favnir/benchmarks/`）にあることを確認（`include_str!` パスは `../../benchmarks/`）
- [x] `grep -r "WasmOptLevel" src/` で `wasm_opt_pass.rs` 以外の match 箇所を洗い出し、影響ファイルをリストアップする（exhaustive match エラー防止）

## T1 — `WasmOptLevel::Os` 追加（`wasm_opt_pass.rs`）

- [x] T0 で洗い出した `wasm_opt_pass.rs` 以外の exhaustive match 箇所にも `Os =>` アームを追加（ある場合）
- [x] `src/backend/wasm_opt_pass.rs` の `WasmOptLevel` enum に `Os` バリアントを追加
- [x] `flag()` メソッドの match に `WasmOptLevel::Os => "-Os"` を追加
- [x] `cargo build` が通ることを確認（exhaustive match エラーがないことを確認）

## T2 — `dce_from_exports` 追加（`wasm_dce.rs`）

- [x] `src/backend/wasm_dce.rs` の `apply_dce` の後に `dce_from_exports` を追加:
  - [x] シグネチャ: `pub fn dce_from_exports(ir: &mut IRProgram, entry_names: &[&str]) -> DceReport`
  - [x] `entry_names.is_empty()` の場合は `DceReport { removed: 0, remaining: ir.fns.len() }` を早期 return
  - [x] `let mut reachable = HashSet::new()` を宣言
  - [x] `for &entry in entry_names { reachable.extend(collect_reachable_fns(ir, entry)); }` でエントリ union 収集
  - [x] `apply_dce(ir, &reachable)` を呼び出して返す
- [x] `cargo build` が通ることを確認

## T3 — `cmd_build "wasm"` 強化（`driver.rs`）

- [x] `cmd_build` の `"wasm"` アーム（行 1703 付近）を変更:
  - [x] `build_wasm_artifact(&program)` の呼び出しを削除
  - [x] `WasmBuildConfig { dce: true, opt_level: WasmOptLevel::Os, size_report: ..., ..WasmBuildConfig::default() }` を構築
  - [x] `size_report` は `std::env::var("FAV_WASM_SIZE_REPORT").map(|v| v == "1").unwrap_or(false)` で設定
  - [x] `build_wasm_artifact_with_config(&program, &wasm_config)` を呼び出す
  - [x] `println!` を `"built {} ({} bytes)"` 形式に更新
- [x] `cargo build` が通ることを確認

## T4 — `benchmarks/v51.7.0.json` 作成

- [x] `benchmarks/v51.7.0.json` を新規作成（spec.md の JSON 仕様通り）
  - [x] `"version": "51.7.0"` を含む
  - [x] `"tests_passed": 3129` を含む
  - [x] `metrics` フィールドに `wasm_*` エントリを含む

## T5 — `v51700_tests` 追加 + バージョン更新

- [x] `driver.rs` の `v51600_tests` 直前に `v51700_tests` モジュールを追加（4 件）:
  - [x] `cargo_toml_version_is_51_7_0`:
    - [x] `include_str!("../Cargo.toml")` で Cargo.toml を読み込む
    - [x] `content.contains("version = \"51.7.0\"")` を assert
  - [x] `wasm_dce_removes_unused_fns`:
    - [x] `include_str!("backend/wasm_dce.rs")` で wasm_dce.rs を読み込む
    - [x] `src.contains("pub fn dce_from_exports")` を assert
    - [x] `src.contains("entry_names.is_empty()")` を assert
  - [x] `wasm_bundle_size_reduced`:
    - [x] `include_str!("driver.rs")` で driver.rs を読み込む
    - [x] `src.contains("build_wasm_artifact_with_config")` を assert
    - [x] `src.contains("WasmOptLevel::Os")` を assert
  - [x] `benchmark_json_exists`:
    - [x] `include_str!("../../benchmarks/v51.7.0.json")` で JSON を読み込む
    - [x] `json.contains("\"version\": \"51.7.0\"")` を assert
- [x] `driver.rs` の `v51600_tests` から `cargo_toml_version_is_51_6_0` を削除
- [x] `fav/Cargo.toml` version → `"51.7.0"`
- [x] `cargo test` 3129 passed, 0 failed（3126 + 4 新規 - 1 削除 = 3129）
- [x] `cargo clippy -- -D warnings` クリーン
- [x] `CHANGELOG.md` に v51.7.0 エントリ追加
- [x] `versions/current.md` を v51.7.0（3129 tests）に更新
- [x] `roadmap-v51.1-v52.0.md` の v51.7.0 実績欄を更新
- [x] tasks.md を COMPLETE に更新（T0〜T5 全 `[x]`）

## code-review 対応（2026-07-20）

- [x] [MED] `build_wasm_artifact_with_config` の DCE 呼び出しを `dce_from_exports(&["main"])` に修正
- [x] [MED] `dce_from_exports` 内の `std::collections::HashSet::new()` → `HashSet::new()` に統一
- [x] [MED] `FAV_WASM_SIZE_REPORT=1` のみ有効である旨のコメント追加
- [x] [LOW] `wasm_opt_pass.rs` に `os_flag_is_minus_os` テスト追加
- [x] `cargo test` 3130 passed, 0 failed
- [x] `cargo clippy -- -D warnings` クリーン
