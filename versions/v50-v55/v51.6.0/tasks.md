# Tasks: v51.6.0 — checker / compiler ホットパス最適化

Status: COMPLETE
Date: 2026-07-19

---

## T0 — 事前確認

- [x] `cargo test` 3124 passed, 0 failed を確認（ベース確認）
- [x] `cargo clippy -- -D warnings` クリーンであることを確認
- [x] `checker.rs` に `SubstRef` が**存在しない**ことを確認（新規追加対象）
- [x] `checker.rs` に `into_ref` が**存在しない**ことを確認（新規追加対象）
- [x] `compiler_fav_runner.rs` に `SourceCache` が**存在しない**ことを確認（新規追加対象）
- [x] `driver.rs` に `ProfileBuildResult` が**存在しない**ことを確認（新規追加対象）
- [x] `driver.rs` に `cmd_profile_build` が**存在しない**ことを確認（新規追加対象）
- [x] `main.rs` の `Some("profile")` アームに `--build` フラグが**存在しない**ことを確認
- [x] `benchmarks/v51.6.0.json` が**存在しない**ことを確認（新規作成対象）
- [x] `benchmarks/` ディレクトリがリポジトリルート直下（`favnir/benchmarks/`）にあることを確認（`include_str!` パスは `../../benchmarks/`）
- [x] `check_single_file` が `driver.rs` に存在することを確認（`profile_build_file` から呼び出す）
- [x] `crate::compiler_fav_runner::compile_src_str_to_bytes` が `pub` であることを確認

## T1 — `SubstRef` 追加（`checker.rs`）

- [x] `src/middle/checker.rs` に以下 2 点を追加:
  - [x] `impl Subst` ブロック**内**（`compose` の後）: `pub fn into_ref(self) -> SubstRef { std::rc::Rc::new(self) }`
  - [x] `impl Subst` ブロック**の後**（ファイル本体）: `pub type SubstRef = std::rc::Rc<Subst>;`
- [x] `cargo build` が通ることを確認

## T2 — `SourceCache` 追加（`compiler_fav_runner.rs`）

- [x] `src/compiler_fav_runner.rs` の `collect_merged_sources` 関数直前に `SourceCache` struct を追加:
  - [x] `pub struct SourceCache(pub std::collections::HashMap<String, String>);`
  - [x] `impl SourceCache { pub fn new() -> Self {...} pub fn get_or_load(&mut self, path: &str) -> Result<String, String> {...} }`
  - [x] `impl Default for SourceCache { fn default() -> Self { Self::new() } }` 追加（clippy 対応）
- [x] `cargo build` が通ることを確認

## T3 — `ProfileBuildResult` + `profile_build_file` + `cmd_profile_build` 追加（`driver.rs`）

- [x] `// ── fav profile ──` セクションの `fn render_profile_table` 直前に v51.6.0 セクションを挿入:
  - [x] `pub struct ProfileBuildResult { pub parse_ms: f64, pub check_ms: f64, pub compile_ms: f64 }`
  - [x] `pub fn profile_build_file(path: &str) -> Result<ProfileBuildResult, String>`:
    - [x] `fs::read_to_string(path)` でソース読み込み
    - [x] `Instant::now()` → `Parser::parse_str` → elapsed で `parse_ms`
    - [x] `Instant::now()` → `check_single_file(path, false, false)` → elapsed で `check_ms`
    - [x] `Instant::now()` → `compile_src_str_to_bytes(&src)` → elapsed で `compile_ms`
    - [x] `Ok(ProfileBuildResult { parse_ms, check_ms, compile_ms })` を返す
  - [x] `pub fn cmd_profile_build(path: &str)`:
    - [x] `profile_build_file` を呼び出し、`Err` なら `eprintln!` + `process::exit(1)`
    - [x] total = parse + check + compile（0.001 下限）
    - [x] Phase / Time (ms) / % のテーブルをフォーマット出力
- [x] `cargo build` が通ることを確認

## T4 — `fav profile --build` CLI 追加（`main.rs`）

- [x] `main.rs` の `use driver::` インポートに `cmd_profile_build` を追加
- [x] `Some("profile")` アームの変数定義ブロックに `let mut build = false;` を追加
- [x] ループ内に `"--build" => { build = true; i += 1; }` アームを追加
- [x] ディスパッチ部分を更新:
  - [x] `--compare` と `--build` 同時指定時は `eprintln!` + `process::exit(1)`
  - [x] `build` が `true` なら `cmd_profile_build(&path)`
- [x] `cargo build` が通ることを確認

## T5 — `benchmarks/v51.6.0.json` 作成

- [x] `benchmarks/v51.6.0.json` を新規作成
  - [x] `"version": "51.6.0"` を含む
  - [x] `"tests_passed": 3126` を含む
  - [x] `metrics` フィールドに `profile_build_*` エントリを含む

## T6 — `v51600_tests` 追加 + バージョン更新

- [x] `driver.rs` の `v51500_tests` の直前に `v51600_tests` モジュールを追加（3 件）:
  - [x] `cargo_toml_version_is_51_6_0`: Cargo.toml に `version = "51.6.0"` が含まれることを確認
  - [x] `checker_perf_hot_path_improved`:
    - [x] `include_str!("middle/checker.rs")` で checker.rs を読み込む
    - [x] `src.contains("pub type SubstRef")` を assert
    - [x] `src.contains("pub fn into_ref")` を assert
  - [x] `compiler_perf_baseline_recorded`:
    - [x] `include_str!("../../benchmarks/v51.6.0.json")` でファイルを読み込む（`fav/src/` → `fav/` → `favnir/`）
    - [x] `json.contains("\"version\": \"51.6.0\"")` を assert
    - [x] `json.contains("tests_passed")` を assert
- [x] `driver.rs` の `v51500_tests` から `cargo_toml_version_is_51_5_0` を削除（51.6.0 に更新したため）
- [x] `fav/Cargo.toml` version → `"51.6.0"`
- [x] `cargo test` 3126 passed, 0 failed（code-review 後も変わらず）
- [x] `cargo clippy -- -D warnings` クリーン（code-review 後も変わらず）
- [x] `CHANGELOG.md` に v51.6.0 エントリ追加
- [x] `versions/current.md` を v51.6.0（3126 tests）に更新
- [x] `roadmap-v51.1-v52.0.md` の v51.6.0 実績欄を更新
- [x] tasks.md を COMPLETE に更新（T0〜T6 全 `[x]`）
