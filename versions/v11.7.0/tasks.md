# Favnir v11.7.0 Tasks

Date: 2026-06-06
Theme: uv 統合 — `fav transpile` が uv プロジェクトとして完結する出力を生成

---

## Phase A — CLI 引数拡張（driver.rs）

- [x] A-1: `cmd_transpile` に `out_dir: Option<String>` / `do_check: bool` / `do_run: bool` 追加
- [x] A-2: `--out-dir` / `--check` / `--run` のパース追加

---

## Phase B — `--out-dir` サポート

- [x] B-1: `basename` 決定ロジック追加（`.fav` のファイル名ステム）
- [x] B-2: `build_pyproject_content(py_src, name)` ヘルパー関数追加
- [x] B-3: `--out-dir` 指定時に `<dir>/main.py` 生成
- [x] B-4: `--out-dir` 指定時に `<dir>/pyproject.toml` 生成（既存スキップ）
- [x] B-5: 既存の `--out` / デフォルト pyproject.toml 生成を `build_pyproject_content` で統一

---

## Phase C — README.md 生成

- [x] C-1: `build_readme_content(input_path, name)` ヘルパー関数追加
- [x] C-2: `--out-dir` 指定時に `<dir>/README.md` 生成（既存スキップ）

---

## Phase D — `--check` オプション

- [x] D-1: `--check` フラグ時に `python -m py_compile <out>` を実行
- [x] D-2: 成功時に `fav transpile: syntax ok` 出力、失敗時に exit 1
- [x] D-3: `python` 未発見時は `warning: python not found, skipping --check` を表示してスキップ

---

## Phase E — `--run` オプション

- [x] E-1: `--run` フラグ時に `uv run main.py` を `<out-dir>` 内で実行
- [x] E-2: `uv` 未発見時は `error: uv not found (install: pip install uv)` で exit 1

---

## Phase F — テスト（6 件）

- [x] F-1: `v11700_tests` モジュール追加
  - [x] `transpile_out_dir_creates_main_py` — `--out-dir` で `main.py` が生成される
  - [x] `transpile_out_dir_creates_pyproject` — `pyproject.toml` が生成される
  - [x] `transpile_out_dir_creates_readme` — `README.md` が生成される
  - [x] `transpile_check_valid_python` — `--check` が valid Python に成功（python なければスキップ）
  - [x] `transpile_pyproject_project_name` — `name = "basename"` が正しい
  - [x] `transpile_pyproject_has_correct_deps` — boto3 / psycopg2 依存が含まれる
- [x] F-2: `cargo test v11700` — 6 件通過
- [x] F-3: `cargo test --lib` — 705 件以上通過

---

## Phase G — バージョン更新 + コミット

- [x] G-1: `fav/Cargo.toml` version → `"11.7.0"`
- [x] G-2: `cargo build` で `Cargo.lock` 更新
- [ ] G-3: `git commit & push` — CI 確認

---

## 完了条件サマリー

| 確認項目 | 状態 |
|---|---|
| `--out-dir` で `main.py` / `pyproject.toml` / `README.md` が生成される | |
| `pyproject.toml` に boto3 / psycopg2 依存が自動追加される | |
| `pyproject.toml` の `name` が `.fav` ファイル名から生成される | |
| `--check` で Python 構文検証が動作する | |
| `--run` で `uv run main.py` が実行される | |
| `cargo test v11700` 6 件通過 | |
| `cargo test --lib` 705 件以上通過 | |
