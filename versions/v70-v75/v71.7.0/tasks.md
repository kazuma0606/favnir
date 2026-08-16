# v71.7.0 タスクリスト — WebAssembly ターゲット テストカバレッジ確立

Date: 2026-08-11
Status: 完了

---

## T0: 事前確認

- [x] `fav/Cargo.toml` のバージョンが `71.6.0` であることを確認
- [x] `cargo test` が 3602 tests pass（0 failures）であることを確認
- [x] `driver.rs` に `v716000_tests` モジュールが存在することを確認（line 58463）
- [x] `driver.rs` に `v717000_tests` が未存在であることを確認
- [x] `build_wasm_artifact` のシグネチャを確認（`driver.rs` line ~2626: 非 pub）
- [x] `build_wasm_artifact_with_config` のシグネチャを確認（`driver.rs` line ~2661: pub）
- [x] `wasm_exec_main` が `wasm_exec` モジュールに存在することを確認

---

## T1: `v717000_tests` モジュール追加（`driver.rs`）

- [x] `v716000_tests` モジュールの直後に `v717000_tests` モジュールを追加した
- [x] `use super::{build_wasm_artifact, build_wasm_artifact_with_config, WasmBuildConfig}` を使った（`crate::driver::` ではなく `super::` — 非 pub 関数のため）
- [x] `wasm_target_compiles` テストを実装した
  - `WasmBuildConfig::default()` で `build_wasm_artifact_with_config` を呼ぶ（dce はデフォルト true）
  - `!bytes.is_empty()` を assert
  - `bytes[..4] == b"\0asm"` を assert
- [x] `wasm_target_runs_simple_pipeline` テストを実装した
  - `build_wasm_artifact` で WASM バイト列を生成
  - `crate::backend::wasm_exec::wasm_exec_main(&bytes)` で実行し `Ok` を assert
- [x] `#[cfg(not(target_arch = "wasm32"))]` ガードを付与した
- [x] `cargo build` でエラーがないことを確認

---

## T2: `cargo_toml_version` テスト文字列を更新

- [x] `driver.rs` 内の `"71.6.0"` バージョンアサーション文字列を `"71.7.0"` に更新した（replace_all）

---

## T3: `fav/Cargo.toml` バージョン更新

- [x] `version = "71.6.0"` → `version = "71.7.0"` に変更した
- [x] `fav/Cargo.lock` が自動更新されることを確認

---

## T4: 部分テスト確認（新規テストのみ）

- [x] `cargo test v717000` で 2 件 pass することを確認（全体テストは T7）

---

## T5: CHANGELOG.md 更新

- [x] `## [v71.7.0]` エントリを先頭に追加した

---

## T6: versions/current.md 更新

- [x] 「進行中バージョン」を `v71.7.0`（WebAssembly ターゲット テストカバレッジ確立）に更新した
- [x] 「次に切る版」を `v71.8.0` に更新した

---

## T7: 最終確認

- [x] `cargo test v717000` で 2 件 pass することを確認
- [x] `cargo test` 全体で 3604 tests pass（0 failures）であることを確認
- [x] `fav/Cargo.toml` のバージョンが `71.7.0` であることを確認
- [x] `versions/current.md` が正しく更新されていることを確認

---

## スコープ外（明示的除外）

- `@favnir/wasm` npm パッケージの実際のパブリッシュ: CI/CD スコープ外
- WASM stdio ブリッジの新規実装: v51.7.0〜v64.7.0 で実装済み、本バージョンはテスト確認のみ
- `wasm-bindgen` 統合: 将来バージョン
- `site/` MDX 追加: 機能追加なし（テストカバレッジ追加のみ）のため対象外

---

## コードレビュー指摘対応

| 優先度 | 指摘 | 対応 |
|---|---|---|
| [HIGH] | `use crate::driver::build_wasm_artifact` は非 pub で使えない | `use super::{...}` に修正 |
| [HIGH] | `wasm_exec_main` の use 宣言漏れ | 完全パスで呼ぶ旨をコメントで明示し、テストコードも完全パス使用 |
| [HIGH] | ロードマップテスト数 3595 → 3604 不一致 | ロードマップを 3602+2=3604 に更新 |
| [HIGH] | ロードマップ 3 項目がスコープ外（乖離） | ロードマップの実装内容記述を実態に合わせて更新 |
| [MED] | `dce: true` の冗長明示 | `WasmBuildConfig::default()` のみに変更（dce はデフォルト true） |
| [MED] | T4/T7 テスト確認の役割不明瞭 | T4 を「部分確認（v717000 のみ）」、T7 を「全体確認」として明確化 |
| [LOW] | tasks.md に site/ MDX 除外理由が未明示 | スコープ外セクションに理由を追記 |
| [MED] | `#[cfg(not(target_arch = "wasm32"))]` の意図が未コメント | 「wasm_exec_main はネイティブ専用のため除外」コメントを追加 |
| [MED] | `wasm_target_compiles` が `wasm_output_correct` との差分不明 | doc コメントに「マジックバイト検証に特化（実行なし）」を明記 |
| [MED] | `wasm_target_runs_simple_pipeline` が `wasm_build_compat_check` との差分不明 | doc コメントに「`build_wasm_artifact` 直接パスを検証」を明記 |
| [LOW] | `assert!(!bytes.is_empty())` は `bytes.len() >= 4` を保証しない | `assert!(bytes.len() >= 4, ...)` に変更し境界チェックを明示 |

---

## 完了チェックリスト

- [x] 全タスク（T0〜T7）が完了している
- [x] `wasm_target_compiles` が pass
- [x] `wasm_target_runs_simple_pipeline` が pass
- [x] テスト総数: 3604（+2、実績ベース: 3602 + 2）
