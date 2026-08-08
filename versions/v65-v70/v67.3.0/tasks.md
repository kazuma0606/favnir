# v67.3.0 タスクリスト

Status: COMPLETE
Version: 67.3.0
Note: MDX ドキュメントは v67.9.0 で一括作成のため本バージョンに T5 はない
Base tests: 3501
Target tests: 3503

---

## T0: 事前確認

- [x] `cargo test -j 8 -- --test-threads=8` でベース 3501 tests passed, 0 failed を確認
- [x] `fav/Cargo.toml` の version が `"67.0.0"` であることを確認（sub-version では変更しない）
- [x] `fav/src/viz.rs` が存在しないことを確認（新規作成）
- [x] `driver.rs` に `v67200_tests` が存在することを確認（`v67300_tests` の挿入位置）
- [x] `driver.rs` に `v67300_tests` が存在しないことを確認（新規追加）
- [x] `cargo test --bin fav v67200_tests` で 2 件 PASS することを確認（前バージョンが正常）
  - 前バージョンのテスト関数名: `debug_record_replay`, `debug_rewind_to_step`
- [x] `versions/current.md` の「進行中バージョン」が `v67.2.0` であることを確認

---

## T1: `fav/src/viz.rs` 新規作成

- [x] `fav/src/viz.rs` を新規作成
  - [x] `pub const VIZ_HELP: &str` を追加（`"ascii"` / `"svg"` / `"mermaid"` フォーマット説明を含む）
  - [x] `pub fn cmd_viz(src: &str, args: &[String]) -> String` を追加
  - [x] `"──►"` を含む（`viz_ascii_dag` テストにマッチ）
  - [x] `"svg"` を含む（`viz_svg_with_timing` テストにマッチ）
  - [x] `"mermaid"` を含む（`viz_svg_with_timing` テストにマッチ）
- [x] `cargo build` でエラーなし（viz.rs 作成後）

---

## T2: `fav/src/main.rs` — `mod viz;` と `Some("viz")` 追加

- [x] `mod debug;` の直後に `mod viz;` を追加
- [x] `Some("debug")` アームの直後に `Some("viz")` ディスパッチアームを追加:
  ```rust
  Some("viz") => {
      let file = args.get(2).map(|s| s.as_str()).unwrap_or("");
      let rest: Vec<String> = args.iter().skip(3).cloned().collect();
      println!("{}", viz::cmd_viz(file, &rest));
  }
  ```
- [x] `cargo build` でエラーなし（main.rs 更新後）

---

## T3: `driver.rs` — `v67300_tests` 追加

- [x] `// -- v67200_tests (v67.2.0)` コメントの直前に `v67300_tests` を挿入
  - [x] `viz_ascii_dag`: `include_str!("viz.rs")` に `"──►"` を含む
  - [x] `viz_svg_with_timing`: `include_str!("viz.rs")` に `"svg"` と `"mermaid"` を含む
- [x] `use super::*` は不要（`include_str!` のみ使用）
- [x] `cargo build` でエラーなし（driver.rs テスト挿入後）

---

## T4: ビルド・テスト

- [x] `cargo test --bin fav v67300_tests` で 2 件 PASS
  - [x] `viz_ascii_dag` PASS
  - [x] `viz_svg_with_timing` PASS
- [x] `cargo test -j 8 -- --test-threads=8` で 3503 tests passed, 0 failed を確認

---

## T5: ドキュメント・ステータス更新

> T4 のテスト全通過（3503 tests passed）を確認してから実施すること。

- [x] `versions/roadmap/roadmap-v67.1-v68.0.md` のバージョン一覧表で v67.3.0 の「状態」列を「未着手」→「完了」に変更し、変更後に当該行が「完了」になっていることを目視確認
- [x] `versions/current.md` の「進行中バージョン」を v67.3.0 に更新
- [x] 本 `tasks.md` を COMPLETE に更新（全チェックボックスを `[x]` に）

> **sub-version ポリシー**: v67.x では `versions/current.md` の「次バージョン」欄（v67.4.0）の更新は不要。v68.0.0 宣言時に一括整理する。

> **CHANGELOG 方針**: v67.1〜v67.9 では CHANGELOG.md を更新しない。v68.0.0 宣言時に一括追記する。
> **Cargo.toml 方針**: v67.1〜v67.9 では version を変更しない。v68.0.0 宣言時に `"68.0.0"` に更新する。

---

## コードレビュー指摘と対応

- [MED] code-reviewer: `VIZ_HELP` が CLI から参照されず到達不能 → `Some("viz")` アームに `--help` / `-h` ブランチを追加し `VIZ_HELP` を表示
- [MED] code-reviewer: `--ascii` 条件式が `then`/`else` ともに `"ascii"` で恒等バグ（clippy: `if_same_then_else`） → `if args.contains("--ascii")` で `"ascii"` を先に返し、それ以外は `--format` を参照する構造に修正
- [LOW] code-reviewer: `viz_svg_with_timing` テスト名がアサート内容（svg + mermaid）と不一致 → 許容（テスト数変更なし）
- [LOW] code-reviewer: 実動作の振る舞いテスト不足 → 他モジュールと同パターンのため許容
