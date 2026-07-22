# Tasks: v50.3.0 — `explain-error` と `explain` の統合

Status: COMPLETE
Date: 2026-07-19

---

## T0 — 事前確認

- [x] `cargo test` 3095 passed, 0 failed を確認（ベース確認）
- [x] `main.rs` に `fav explain --error` ルートがないことを確認（`grep "\"--error\"" fav/src/main.rs`）
- [x] `driver.rs` に `cmd_explain_error_collect` がないことを確認
- [x] `cargo clippy -- -D warnings` クリーンであることを確認（ベース）

## T1 — `driver.rs` — `cmd_explain_error_collect` ヘルパー追加

- [x] `cmd_explain_error` 直前に `pub(crate) fn cmd_explain_error_collect(code: &str) -> Option<String>` を追加
  - [x] description / example / fix / suggestion の各フィールドを整形して String に収める
  - [x] suggestion が `Some` の場合のみ "Suggestion" セクションを追加
- [x] `cmd_explain_error` を `cmd_explain_error_collect` の print ラッパーに変更
  - [x] `Some(text)` → `print!("{}", text)`
  - [x] `None` → 既存の `eprintln!` + `process::exit(1)`
  - [x] error メッセージの `explain-error --list` を `explain --error --list` に更新

## T2 — `main.rs` — `fav explain --error` フラグ追加

- [x] `Some("explain")` アームの `compiler` チェック直後（行 755 付近）に `--error` ガードを挿入
  - [x] `args.iter().any(|a| a == "--error")` で検出
  - [x] `--list`・`--format`・`<code>` の 3 パターンをパース
  - [x] `list=true` → `cmd_explain_error_list` / `cmd_explain_error_list_json` 呼び出し
  - [x] `code=Some(c)` → `cmd_explain_error(c)` 呼び出し
  - [x] それ以外 → eprintln + exit(1)
  - [x] ブロック末尾に `return;`

## T3 — `v503000_tests` モジュール追加

- [x] `v503000_tests` モジュールを `driver.rs` の `v502000_tests` 直前に追加（3 件）
  - [x] `cargo_toml_version_is_50_3_0`: version = "50.3.0" を assert
  - [x] `explain_error_flag_works`: `cmd_explain_error_collect("E0213")` の出力に "E0213" / "Fix" / "Suggestion" が含まれることを assert
  - [x] `explain_error_all_codes_have_text`: `error_catalog::list_all()` の全エントリの `description`・`fix` が非空であることを assert

## T4 — バージョン更新・完了

- [x] `fav/Cargo.toml` version → `"50.3.0"`
- [x] `v502000_tests::cargo_toml_version_is_50_2_0` を削除
- [x] `cargo test` 3097 passed, 0 failed
- [x] `cargo clippy -- -D warnings` クリーン
- [x] `CHANGELOG.md` に v50.3.0 エントリ追加
- [x] `versions/current.md` を v50.3.0（3097 tests）に更新
- [x] `versions/roadmap/roadmap-v50.1-v51.0.md` の v50.3.0 実績を記入
- [x] tasks.md を COMPLETE に更新（T0〜T4 全 `[x]`）
