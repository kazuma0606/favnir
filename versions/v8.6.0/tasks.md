# Favnir v8.6.0 Tasks

Date: 2026-05-30
Theme: `fav run` の rune import 対応（Favnir pipeline 制限解除）

---

## Phase A: compiler.fav に `compile_bytes_from_src` 追加

- [x] A-1: `compile_bytes_from_src(src: String) -> Result<List<Int>, String>` を `compile_bytes` の直後に追加
  — `lex(src)` → `parse_tokens` → `compile` → `serialize_artifact` のチェーン
  — `IO.read_file_raw` なし（引数の文字列をそのまま使う）
- [x] A-2: `cargo build` — compiler.fav の bootstrap コンパイルが通ること確認

---

## Phase B: compiler_fav_runner.rs に rune 対応版追加

- [x] B-1: `collect_merged_sources(path, visited, out)` ヘルパーを追加
  — パースして `ImportDecl { is_rune: true }` を検出
  — `source_dir/rune_modules/<name>/` を探索（`crate::toml::rune_entry_file` 使用）
  — rune ファイルを再帰処理（deps-first）
  — `import rune "..."` 行と `namespace ...` 行を除去してから `out` に push
  — visited セットで重複防止
- [x] B-2: `compile_file_to_bytes_rune(path) -> Result<Vec<u8>, String>` を追加
  — `collect_merged_sources` で全ソース収集
  — `sources.join("\n")` で結合
  — `compile_bytes_from_src` を VM 経由で呼ぶ（fn_idx_by_name 使用）
  — `List<Int>` → `Vec<u8>` 変換（既存 `compile_file_to_bytes` と同パターン）

---

## Phase C: driver.rs の dispatch 条件更新

- [x] C-1: `run_with_favnir_pipeline` 内の `compile_file_to_bytes` を
  `compile_file_to_bytes_rune` に置換
- [x] C-2: `cmd_run` の dispatch 条件から `&& !has_rune_imports(&program)` を削除
  — `let use_favnir = !legacy && proj.is_none();`
- [x] C-3: `has_rune_imports` 関数の参照箇所を確認し、不要なら削除（他で使用なければ）

---

## Phase D: テスト追加・更新

- [x] D-1: `dispatch_rune_import_uses_favnir_pipeline` テストを追加
  — temp dir に `rune_modules/mymath/mymath.fav` と `main.fav` を作成
  — `compile_file_to_bytes_rune` でコンパイル → VM 実行 → 結果確認（42）
- [x] D-2: `dispatch_rune_import_uses_rust_fallback` テストを削除または更新
  — `has_rune_imports` が true を返すだけのテストとして残すか、完全削除
- [x] D-3: 既存 `run_self_hosted_tests` (7件) が引き続き通ること確認
- [x] D-4: checker.fav が rune import ファイルの型チェックで問題を起こさないか確認
  — 問題があれば型エラー時の Rust フォールバック処理を追加

---

## Phase E: 最終確認・ドキュメント

- [x] E-1: `cargo build` — コンパイルエラーなし
- [x] E-2: `cargo test` — 1122 tests passing（dispatch_rune_import テストを入れ替え）
- [x] E-3: このファイルを完了状態に更新
- [x] E-4: commit

---

## 完了条件

- `fav run <file_with_rune_import>` が Favnir pipeline で動く ✓
- `fav run <simple_file>` が引き続き Favnir pipeline で動く ✓
- `fav run --legacy <file>` が Rust pipeline で動く ✓
- `fav run` の fav.toml プロジェクトモードが Rust pipeline フォールバックを維持 ✓
- 既存テスト全件通る ✓
- 新規統合テスト 1 件 ✓

---

## 実装ノート

- `collect_merged_sources` は `load_all_items`（driver.rs:294）の
  Standalone モード（else ブランチ、driver.rs:378-398）を参考に実装
- `rune_entry_file` は `crate::toml::rune_entry_file(rune_dir, name)` で使用可能
- List<Int> → Vec<u8> 変換は `compile_file_to_bytes` の既存ロジックをコピー
- checker.fav が rune 関数を未知として E0001/E0002 を誤検出する可能性あり
  → v8.6.0 では `check_single_file` エラーが出たら warning のみ出して続行するか、
    または型エラー時に Rust フォールバックする（検討）
