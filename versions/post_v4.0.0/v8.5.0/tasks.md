# Favnir v8.5.0 Tasks

Date: 2026-05-30
Theme: `fav run` のデフォルト Favnir 化（単一ファイル自動 dispatch）

---

## Phase A: `cmd_run` の dispatch 化（driver.rs）

- [x] A-1: `run_with_favnir_pipeline(source_path, db_url)` 内部関数を追加
  — `check_single_file(path, false)` → `compile_file_to_bytes` → `FvcArtifact::from_bytes`
    → `exec_artifact_main_with_source`
- [x] A-2: `load_run_config(file)` ヘルパーを抽出（設定ロードを cmd_run と cmd_run_self_hosted で共有）
- [x] A-3: `cmd_run(file, db_url, legacy: bool)` に `legacy` 引数を追加
  — dispatch 判定: `!legacy && proj.is_none() && !has_rune_imports(&program)`
  — true → `run_with_favnir_pipeline`
  — false → 既存 Rust pipeline（`load_and_check_program` 経由、二重パースは許容）
- [x] A-4: `cmd_run_self_hosted` を `cmd_run(file, db_url, false)` の thin wrapper に変更
- [x] A-5: `cargo build` — コンパイルエラーなし確認

---

## Phase B: main.rs のフラグ処理更新

- [x] B-1: `cmd_run_self_hosted` import を削除（不要になったため）
- [x] B-2: `Some("run")` ブランチに `--legacy` フラグを追加
  — `--legacy` → `legacy = true`（Rust pipeline 強制）
  — `--self-host` → no-op（後方互換 alias、デフォルトと同じ動作）
  — デフォルト → `legacy = false`
- [x] B-3: `cmd_run(file, db_path.as_deref(), legacy)` に統一

---

## Phase C: 統合テスト

- [x] C-1: `dispatch_single_file_uses_favnir` — 単一ファイルが `has_rune_imports=false` になり
  Favnir pipeline でコンパイル・実行されること（`21 * 2 == 42`）
- [x] C-2: `dispatch_rune_import_uses_rust_fallback` — `import rune "sql"` で
  `has_rune_imports=true` が返ること（Rust pipeline フォールバック検出）
- [x] C-3: 既存 run_self_hosted_tests 7 件が引き続き通る

---

## Phase D: 最終確認・ドキュメント

- [x] D-1: `cargo test` — 1122 tests passing（+2 新規）
- [x] D-2: このファイルを完了状態に更新
- [x] D-3: commit

---

## 完了条件

- `fav run <file>`（単一ファイル）がデフォルトで Favnir pipeline を使う ✓
- `fav run --legacy <file>` が Rust pipeline を使う ✓
- rune import を含むファイルが自動で Rust pipeline にフォールバックする ✓
- fav.toml プロジェクトモードが自動で Rust pipeline を使う ✓
- 既存テスト全件通る（1120 → 1122）✓
- 新規統合テスト 2 件 ✓

---

## 実装ノート

- `cmd_run` のシグネチャ変更: `cmd_run(file, db_url)` → `cmd_run(file, db_url, legacy)`。
  直接呼んでいる caller は `legacy=false` を追加するだけ。
- 二重パースの許容: dispatch 判定のパース + Favnir pipeline 内の再パース（check_single_file）。
  パフォーマンス問題が出れば v8.6.0 で最適化。
- `ensure_no_partial_flw`: Favnir pipeline パスではスキップ（compiler.fav 非対応）。
  `--legacy` パスでは従来通り実行。
- Favnir pipeline の適用条件:
  1. `!legacy` — --legacy フラグなし
  2. `proj.is_none()` — fav.toml プロジェクトモードでない
  3. `!has_rune_imports(&program)` — rune import なし
  いずれか一つでも false なら Rust pipeline に自動フォールバック（エラーにはしない）。
