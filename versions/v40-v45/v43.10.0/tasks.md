# v43.10.0 タスク — `fav check --explain`

## ステータス: COMPLETE（2026-07-13）— 2929 tests

---

## T0 — 事前確認

- [x] `cargo test` 2927 / 0 確認
- [x] `Cargo.toml` version = `43.9.0` 確認
- [x] `v431000_tests` が `fav/src/driver.rs` に存在しないことを確認（`v43100_tests` は v43.1.0 の既存モジュール）
- [x] `get_explain_text(code: &str) -> Option<&'static str>` シグネチャが `fav/src/driver.rs` に存在することを確認
- [x] `msgs_to_type_errors` が `checker_fav_runner.rs` に存在することを確認
- [x] `cmd_check` シグネチャが 11 パラメータ（`show_inference: bool` 末尾）であることを確認
- [x] `main.rs` の `check` サブコマンド引数パースに `--show-inference` が存在することを確認
- [x] `main.rs` の `check` サブコマンド引数パースに `--explain` が存在しないことを確認

---

## T1 — driver.rs — `collect_explain_output` 追加

- [x] `collect_explain_output(src: &str, filename: &str) -> Vec<String>` を追加
  - `collect_inference_annotations` の直後、`cmd_check` の前に配置
  - `run_checker_fav` は `Result<(), Vec<String>>` を返すため `msgs_to_type_errors(msgs)` で `Vec<TypeError>` に変換してから `e.code` にアクセスする
  - 正常コードでは空 Vec を返す
  - 型エラーがある場合、各 `TypeError` に対して `get_explain_text(e.code)` を呼び出し
  - `Some(text)` の場合のみ `"  Explain: {text}"` を収集して返す

---

## T2 — driver.rs — `cmd_check` 更新

- [x] `cmd_check` シグネチャ末尾に `explain: bool` を追加（12 番目）
- [x] エラー出力ループ内に `explain` 出力ブロックを追加
  - `eprintln!("{}", format_diagnostic(&source, e))` の直後
  - `if explain && !json { if let Some(text) = get_explain_text(e.code) { println!("  Explain: {}", text); } }`
  - プロジェクトモード（`file = None`）の分岐には追加しない

---

## T3 — main.rs — `--explain` フラグ追加

- [x] `let mut explain = false;` を追加
- [x] `"--explain" => { explain = true; i += 1; }` を追加（`--show-inference` の直後）
- [x] `cmd_check` 呼び出しに `explain` を末尾に追加

---

## T4 — driver.rs — `v431000_tests` 追加 / Cargo.toml / スタブ化

- [x] `v43900_tests` モジュールの直前に `v431000_tests` を挿入
- [x] `cargo_toml_version_is_43_10_0` テスト追加（`Cargo.toml` に `"43.10.0"` を含む）
- [x] `explain_output_empty_for_well_typed_code` テスト追加
  - `fn add(a: Int, b: Int) -> Int { a + b }` → `collect_explain_output` が空 Vec を返す
- [x] `v43900_tests::cargo_toml_version_is_43_9_0` をスタブ化
  - `// Stubbed: version bumped to 43.10.0 in v43.10.0.`
- [x] `fav/Cargo.toml` version を `43.9.0` → `43.10.0` に更新

---

## T5 — CHANGELOG.md

- [x] v43.10.0 エントリ追加
  - Added: `collect_explain_output` / `fav check --explain` / `v431000_tests` 2 件
  - Changed: `cmd_check` シグネチャ更新（12 パラメータ）/ `cargo_toml_version_is_43_9_0` スタブ化

---

## T6 — テスト実行・確認

- [x] `cargo test -j 8 -- --test-threads=8` 実行
- [x] 2929 passed; 0 failed 確認
- [x] `v431000_tests` 2 件 pass 確認

---

## T7 — バージョン管理ドキュメント更新

- [x] `versions/current.md` → v43.10.0 最新安定版（2929 tests）、次版 v43.11.0
- [x] `versions/roadmap/roadmap-v43.1-v44.0.md` → v43.10.0 を `✅ COMPLETE（2026-07-13）`、推定 2929 → 実績 2929 に修正。「v39 の Llm Rune を活用」を「静的解説テキストベース MVP」に修正
- [x] `versions/v40-v45/v43.10.0/tasks.md` → COMPLETE、全チェックボックス `[x]`

---

## 実装時の知見

- **モジュール名衝突**: `v43100_tests` は v43.1.0 の既存モジュール → `v431000_tests` を使用
- **型変換必須**: `run_checker_fav` の `Err(msgs)` は `Vec<String>` → `msgs_to_type_errors(msgs)` で `Vec<TypeError>` に変換してから `e.code` にアクセス
- **`--json` との共存**: `if explain && !json` で `--json` 指定時は explain を無効化
