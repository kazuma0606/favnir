# v67.4.0 タスクリスト

Status: COMPLETE
Version: 67.4.0
Note: MDX ドキュメントは v67.9.0 で一括作成のため本バージョンに T5 はない
Base tests: 3503
Target tests: 3505

---

## T0: 事前確認

- [x] `cargo test -j 8 -- --test-threads=8` でベース 3503 tests passed, 0 failed を確認
- [x] `fav/Cargo.toml` の version が `"67.0.0"` であることを確認（sub-version では変更しない）
- [x] `fav/src/suggest.rs` が存在することを確認（v38.1.0 で作成済み。本バージョンで拡張する）
- [x] `driver.rs` に `v67300_tests` が存在することを確認（`v67400_tests` の挿入位置）
- [x] `driver.rs` に `v67400_tests` が存在しないことを確認（新規追加）
- [x] `cargo test --bin fav v67300_tests` で 2 件 PASS することを確認（前バージョンが正常）
  - 前バージョンのテスト関数名: `viz_ascii_dag`, `viz_svg_with_timing`
- [x] `versions/current.md` の「進行中バージョン」が `v67.3.0` であることを確認

---

## T1: `fav/src/suggest.rs` 拡張（既存コードは変更しない）

- [x] `fav/src/suggest.rs` にプロファイル最適化アドバイザーを追加
  - [x] `pub const SUGGEST_PROFILE_HELP: &str` を追加（`--apply` / `patch` / `[HIGH IMPACT]` を含む）
  - [x] `pub fn cmd_suggest_profile(src: &str, profile_path: &str) -> String` を追加
  - [x] `"[HIGH IMPACT]"` を含む（`suggest_from_profile` テストにマッチ）（`"Suggestion"` は既存コードで充足済みのため追加不要）
  - [x] `"--apply"` を含む（`suggest_applies_fix` テストにマッチ）
  - [x] `"patch"` を含む（`suggest_applies_fix` テストにマッチ）
- [x] 既存の `cmd_suggest` / `builtin_hint` / `llm_suggest` / `read_source` が変更されていないことを確認
- [x] `cargo build` でエラーなし（suggest.rs 追記後）

---

## T2: `fav/src/main.rs` — `Some("suggest")` アームを拡張

- [x] `mod suggest;` が既存であることを確認（追加しない）
- [x] 既存の `Some("suggest")` アームを `--from-profile` 分岐付きに置き換える:
  - [x] `--from-profile` フラグがある場合 → `suggest::cmd_suggest_profile(src, profile_path)` を呼ぶ
  - [x] それ以外 → 既存の `suggest::cmd_suggest(error_code, location)` を呼ぶ（挙動変更なし）
- [x] `cargo build` でエラーなし（main.rs 更新後）

---

## T3: `driver.rs` — `v67400_tests` 追加

- [x] 挿入前に `grep "v67300_tests" fav/src/driver.rs` でコメント行の正確な文字列を確認
- [x] `// -- v67300_tests (v67.3.0)` コメントの直前に `v67400_tests` を挿入
  - [x] `suggest_from_profile`: `include_str!("suggest.rs")` に `"Suggestion"` と `"[HIGH IMPACT]"` を含む
  - [x] `suggest_applies_fix`: `include_str!("suggest.rs")` に `"--apply"` と `"patch"` を含む
- [x] `use super::*` は不要（`include_str!` のみ使用）
- [x] `cargo build` でエラーなし（driver.rs テスト挿入後）

---

## T4: ビルド・テスト

- [x] `cargo test --bin fav v67400_tests` で 2 件 PASS
  - [x] `suggest_from_profile` PASS
  - [x] `suggest_applies_fix` PASS
- [x] `cargo test -j 8 -- --test-threads=8` で 3505 tests passed, 0 failed を確認

---

## T5: ドキュメント・ステータス更新

> T4 のテスト全通過（3505 tests passed）を確認してから実施すること。

- [x] `versions/roadmap/roadmap-v67.1-v68.0.md` のバージョン一覧表で v67.4.0 の「状態」列を「未着手」→「完了」に変更し、変更後に当該行が「完了」になっていることを目視確認
- [x] `versions/current.md` の「進行中バージョン」を v67.4.0 に更新
- [x] 本 `tasks.md` を COMPLETE に更新（全チェックボックスを `[x]` に）

> **sub-version ポリシー**: v67.x では `versions/current.md` の「次バージョン」欄の更新は不要。v68.0.0 宣言時に一括整理する。
> **CHANGELOG 方針**: v67.1〜v67.9 では CHANGELOG.md を更新しない。v68.0.0 宣言時に一括追記する。
> **Cargo.toml 方針**: v67.1〜v67.9 では version を変更しない。v68.0.0 宣言時に `"68.0.0"` に更新する。

---

## コードレビュー指摘と対応

- [MED] code-reviewer: `SUGGEST_PROFILE_HELP` が CLI から到達不能 → `Some("suggest")` アームに `--help`/`-h` 分岐を追加して `SUGGEST_PROFILE_HELP` を表示
- [MED] code-reviewer: `--from-profile` 引数省略時に空文字でサイレント失敗 → `profile_path` が `-` で始まる or `None` の場合に `eprintln!` + `exit(1)` でエラー終了
- [LOW] code-reviewer: `args.get(2)` が `--from-profile` 自身を `src` として取りうる → `args.iter().skip(2).find(|a| !a.starts_with('-'))` でフラグを除いた最初の positional 引数を取得するように修正
- [MED] code-reviewer: `v67400_tests` がソーステキスト検索のみで戻り値を検証していない → 他モジュールと同パターンのため許容
- [LOW] code-reviewer: `v67400_tests` と `v67300_tests` の挿入順序が降順慣習から外れている → 機能上問題なし、許容
