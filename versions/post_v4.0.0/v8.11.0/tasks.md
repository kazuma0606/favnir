# Favnir v8.11.0 Tasks

Date: 2026-05-30
Theme: fav.toml プロジェクトモードを Favnir pipeline 化

---

## Phase A: `compile_src_str_to_bytes` 抽出（compiler_fav_runner.rs）

- [x] A-1: `compile_src_str_to_bytes(merged: &str) -> Result<Vec<u8>, String>` を新規追加
  — `compile_file_to_bytes_rune` の compile 部分（VM::run → extract_bytes_from_result）を共通化
  — 変更ファイル: `fav/src/compiler_fav_runner.rs`
- [x] A-2: `compile_file_to_bytes_rune` を内部で `compile_src_str_to_bytes` を使うようリファクタリング

---

## Phase B: `collect_project_sources` 追加（compiler_fav_runner.rs）

- [x] B-1: `collect_project_sources(path, root, toml, visited, out)` を追加
  — `import "name"`（裸名 = rune として扱われる）→ `root/rune_modules/name/` を処理
  — `import "path/with/slash"` → `toml.src_dir/path/with/slash.fav`（ローカルファイル）
  — `import` / `namespace` 行を strip してから `out.push`
  — canonicalize で重複防止
  — 注: パーサーが裸名を `is_rune: true` として扱うため、`import "utils"` は rune として解決

---

## Phase C: `compile_project_to_bytes` 追加（compiler_fav_runner.rs）

- [x] C-1: `pub fn compile_project_to_bytes(entry, root, toml) -> Result<Vec<u8>, String>` を追加
  — `collect_project_sources` でソース収集 → `compile_src_str_to_bytes` でコンパイル
- [x] C-2: `pub fn collect_project_merged(entry, root, toml) -> Result<String, String>` を追加
  — 結合済みソース文字列を返す（型チェックに利用可能）

---

## Phase D: driver.rs 更新

- [x] D-1: `run_fvc_bytes(bytes: &[u8], db_url: Option<&str>, source_path: Option<&str>)` を切り出し
  — `run_with_favnir_pipeline` のバイトコード実行部分（from_bytes → VM::run → 出力）を抽出
  — `run_with_favnir_pipeline` と `run_with_favnir_pipeline_project` で共有
- [x] D-2: `check_source_str(src: &str) -> Vec<TypeError>` を追加
  — `Parser::parse_str` → `lower_program` → `run_checker_fav` のパイプライン
  — ソース文字列から直接型チェック
- [x] D-3: `run_with_favnir_pipeline_project(source_path, root, toml, db_url)` を追加
  — `collect_project_merged` でソース収集・結合
  — `check_source_str` で型チェック（エラーあり → exit）
  — `compile_src_str_to_bytes` でコンパイル
  — `run_fvc_bytes` で実行
- [x] D-4: `cmd_run` の dispatch 変更
  — `let use_favnir = !legacy && proj.is_none()` → `let use_favnir = !legacy`
  — `proj.is_some()` の場合は `run_with_favnir_pipeline_project` を呼ぶ

---

## Phase E: テスト追加（driver.rs）

- [x] E-1: `dispatch_project_uses_favnir_pipeline` テスト追加
  — `tempdir` に `fav.toml` + `rune_modules/utils/utils.fav` + `src/main.fav` を作成
  — `compile_project_to_bytes` → VM 実行 → 結果が 42 であることを確認
  — 注: `import "utils"`（裸名）が rune として解決されるため rune_modules を使用

---

## Phase F: 確認・ドキュメント

- [x] F-1: `cargo test dispatch_project` — 新規テスト通ること ✓
- [x] F-2: `cargo test checker_fav` — self-check 通ること（17 件）✓
- [x] F-3: `cargo test` — 全件通ること（1135 tests）✓
- [x] F-4: tasks.md 完了状態に更新・MEMORY.md 更新・commit

---

## 完了条件

- `fav run` の `fav.toml` プロジェクトモードが Favnir pipeline で動作する
- `import "name"`（ローカルモジュール）の再帰収集が機能する
- 型チェック・コンパイル共に Favnir 実装経由
- 既存テスト全件通る
