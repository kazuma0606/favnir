# Favnir v8.3.0 Tasks

Date: 2026-05-29
Theme: `fav run --self-host` — Favnir セルフホストコンパイルパス

---

## Phase A: compiler.fav に compile_bytes 追加

- [x] A-1: `compile_file_quiet(path)` を追加 — IO.println フェーズメッセージなし版
  — `IO.read_file_raw` → `lex` → `parse_tokens` → `compile` の純粋パイプライン
- [x] A-2: `public fn compile_bytes(path: String) -> Result<List<Int>, String>` を追加
  — `compile_file_quiet` + `serialize_artifact` でバイトコードを返す
- [x] A-3: `cargo build` — コンパイルエラーなし確認
  — `fav check self/compiler.fav` は既存の E0005（infer_arg_tys 逆順バグ）が残るが
    これは pre-existing でありこのバージョンのスコープ外

---

## Phase B: compiler_fav_runner.rs 新規作成

- [x] B-1: `src/compiler_fav_runner.rs` 新規作成
  — `OnceLock<Arc<FvcArtifact>>` で compiler.fav をキャッシュ（checker_fav_runner.rs と同パターン）
  — Rust パイプラインで compiler.fav 自体をコンパイル（bootstrap）
- [x] B-2: `pub fn compile_file_to_bytes(path: &str) -> Result<Vec<u8>, String>`
  — `compile_bytes` Favnir 関数を VM 実行
  — `Value::Variant("ok", Some(Value::List([Value::Int(n),...])))`  → `Vec<u8>` 変換
- [x] B-3: `main.rs` に `mod compiler_fav_runner;` を追加

---

## Phase C: fav run --self-host の実装

- [x] C-1: `driver.rs` に `pub fn cmd_run_self_hosted(file, db_url)` を追加
  — 型チェック: Rust checker（`load_and_check_program`）
  — コンパイル: `compiler_fav_runner::compile_file_to_bytes` (Favnir)
  — デシリアライズ: `FvcArtifact::from_bytes(&bytes)`
  — 実行: `exec_artifact_main_with_source`
- [x] C-2: `main.rs` の `Some("run")` ブランチに `--self-host` フラグを追加
  — `fav run --self-host <file>` → `cmd_run_self_hosted`
  — `fav run <file>` → 従来の `cmd_run`（後方互換）

---

## Phase D: 統合テスト（5 件）

- [x] D-1: `run_self_hosted_bool_main` — `public fn main() -> Bool { true }` → `Bool(true)`
- [x] D-2: `run_self_hosted_arithmetic` — `add(3, 4)` → `Int(7)`
- [x] D-3: `run_self_hosted_string_concat` — `String.concat("hello", " world")` → `Str("hello world")`
- [x] D-4: `run_self_hosted_if_else` — `if true { 42 } else { 0 }` → `Int(42)`
- [x] D-5: `run_self_hosted_let_bind` — `bind x <- 10; bind y <- 20; x + y` → `Int(30)`

---

## Phase E: 最終確認

- [x] E-1: `cargo test` — 1118 tests passing（+5 新規）
- [x] E-2: このファイルを完了状態に更新
- [x] E-3: commit

---

## 完了条件

- `compiler.fav` に `public fn compile_bytes` が追加されている ✓
- `compiler_fav_runner.rs` が OnceLock パターンで compiler.fav をロード ✓
- `fav run --self-host <file>` が Favnir コンパイルパスで動作 ✓
- 既存テスト全件通る（1113 → 1118）✓
- 新規統合テスト 5 件 ✓

---

## 実装ノート

- `compile_file_quiet` は `compile_file` の IO.println なし版。
  `IO.read_file_raw` は残す（ファイル読み込みに必要）。
- `compiler_fav_runner.rs` は `src/` 直下（backend を含まない lib.rs と競合しないため）。
  `main.rs` のみで `mod compiler_fav_runner;` を宣言。
- `fav check self/compiler.fav` の E0005 エラーは pre-existing（v6.2.0 以前から存在）。
  checker.fav の `infer_arg_tys` が引数型リストを逆順に構築するため、
  複雑なネスト `Result.and_then` で型推論が失敗する。bootstrap テストは別途通過済み。
- `fav run --self-host` は Rust type checker を残す（compiler.fav は型チェックを行わない）。
  将来的に checker.fav も呼び出すことで完全 Favnir パスを実現できる。
- 一時ファイル名はテストごとに一意 (`fav_sh_{name}.fav`) にして並列実行の競合を防ぐ。
