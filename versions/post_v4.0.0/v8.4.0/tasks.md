# Favnir v8.4.0 Tasks

Date: 2026-05-30
Theme: `fav run --self-host` 型チェックを checker.fav へ切替

---

## Phase A: `check_file_with_fav` ヘルパー追加

- [x] A-1: 専用ヘルパーは不要 — 既存の `check_single_file(path, false)` が同等の処理を担う
  — `load_file` → `Parser::parse_str` → `ast_lower_checker::lower_program`
    → `checker_fav_runner::run_checker_fav`

---

## Phase B: `cmd_run_self_hosted` の型チェック切替

- [x] B-1: `cmd_run_self_hosted` 内の `load_and_check_program(file)` を削除
- [x] B-2: `find_entry(file)` でパス解決 → `check_single_file(&source_path, false)` で checker.fav 型チェック
  — エラーがあれば `format_diagnostic` でソースを含む出力 → `process::exit(1)`
- [x] B-3: `cargo build` — エラーなし確認

---

## Phase C: 統合テスト

- [x] C-1: `run_self_hosted_checker_fav_valid` — 有効コードが checker.fav を通過する
- [x] C-2: `run_self_hosted_checker_fav_catches_type_error` — !IO 未宣言エフェクト（E0003）を検出
  — 診断結果: wrong_args/undef_var は Ok（checker.fav 未対応）、io_no_effect は E0003 ✓
- [x] C-3: 既存 run_self_hosted_tests 5 件が引き続き通る

---

## Phase D: 最終確認・ドキュメント

- [x] D-1: `cargo test` — 1120 tests passing（+2 新規）
- [x] D-2: このファイルを完了状態に更新
- [x] D-3: commit

---

## 完了条件

- `cmd_run_self_hosted` が checker.fav で型チェックする ✓
- `fav run --self-host` が E0003 エフェクトエラーを検出して終了する ✓
- `--self-host` の型チェック・コンパイルが共に Favnir 実装経由 ✓
- 既存テスト全件通る（1118 → 1120）✓
- 新規統合テスト 2 件 ✓

---

## 実装ノート

- `check_file_with_fav` は `check_single_file`（非 legacy パス）と同じロジック。
  重複を避けるため、将来的には `check_single_file` を `check_file_with_fav` 経由にまとめられる。
- `find_entry` の戻り値: `(String, Option<(FavToml, PathBuf)>)` — `source_path` のみ使う。
- rune import を含むファイルは checker.fav での完全チェック対象外（v8.4.0 limitation）。
  rune import ありの場合は `fav run`（通常）を推奨。
- スタックオーバーフロー対策: 通常サイズのファイルでは問題なし。
  self_check 相当の巨大ファイルは別途 64MB スタックスレッドが必要。
