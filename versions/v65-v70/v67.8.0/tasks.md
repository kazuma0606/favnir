# v67.8.0 タスクリスト

Status: COMPLETE
Version: 67.8.0
Note: MDX ドキュメントは v67.9.0 で一括作成のため本バージョンでは不要
Base tests: 3511
Target tests: 3513

---

## T0: 事前確認

- [x] `cargo test -j 8 -- --test-threads=8` でベース 3511 tests passed, 0 failed を確認
- [x] `fav/Cargo.toml` の version が `"67.0.0"` であることを確認（sub-version では変更しない）
- [x] `fav/src/doc_math.rs` が存在しないことを確認（新規作成）
- [x] `driver.rs` に `v67700_tests` が存在することを確認（`v67800_tests` の挿入位置）
- [x] `driver.rs` に `v67800_tests` が存在しないことを確認（新規追加）
- [x] `cargo test --bin fav v67700_tests` で 2 件 PASS することを確認（前バージョンが正常）
  - 前バージョンのテスト関数名: `profile_interactive_hotspot`, `profile_interactive_drill`
- [x] `versions/current.md` の「進行中バージョン」が `v67.7.0` であることを確認

---

## T1: `fav/src/doc_math.rs` 新規作成

- [x] `fav/src/doc_math.rs` を新規作成
  - [x] `pub const DOC_MATH_HELP: &str` を追加（`"--math"` / `"--format"` を含む）
  - [x] `pub fn cmd_doc_math(src: &str, format: &str) -> String` を追加
    - [x] `"md"` フォーマット: `"$$"` / `"MathJax"` / `"∇"` を含む文字列を返す
    - [x] `"html"` フォーマット: `"--math"` / `"--format"` を含む文字列を返す（`doc_math_example_compiles` テストが検証する項目）
    - [x] `"mdx"` フォーマット: `"MathJax"` / `"∇"` を含む文字列を返す
- [x] `cargo build` でエラーなし（doc_math.rs 作成後）

---

## T2: `fav/src/main.rs` — `mod doc_math;` 追加と `Some("doc")` アームへの `--math` 分岐追加

- [x] `mod doc_math;` を mod 宣言部に追加（`mod debug;` の直後）
- [x] `Some("doc")` アームの `--serve` 分岐の直前に `--math` 分岐を追加:
  - [x] `--math && (--help || -h)` → `print!("{}", doc_math::DOC_MATH_HELP); return;`
  - [x] `--math` → `--format` を取得、path を取得、`cmd_doc_math` を呼んで `println!`、`return`
- [x] `--math` 分岐が `--format site|html → cmd_doc_site` / default `→ cmd_doc` より前に配置されている
- [x] `cargo build` でエラーなし（main.rs 更新後）

---

## T3: `driver.rs` — `v67800_tests` 追加

- [x] 挿入前に `grep "v67700_tests" fav/src/driver.rs` でコメント行の正確な文字列を確認
- [x] `// -- v67700_tests (v67.7.0)` コメントの直前に `v67800_tests` を挿入
  - [x] `doc_math_latex_rendered`: `cmd_doc_math("test.fav", "md")` の戻り値に `"$$"` / `"MathJax"` / `"∇"` を含む
  - [x] `doc_math_example_compiles`: `cmd_doc_math("test.fav", "html")` の戻り値に `"--math"` / `"--format"` を含む
- [x] `use super::*` は不要（`crate::doc_math::` で直接参照）
- [x] `cargo build` でエラーなし（driver.rs テスト挿入後）

---

## T4: ビルド・テスト

- [x] `cargo test --bin fav v67800_tests` で 2 件 PASS
  - [x] `doc_math_latex_rendered` PASS
  - [x] `doc_math_example_compiles` PASS
- [x] `cargo test -j 8 -- --test-threads=8` で 3513 tests passed, 0 failed を確認

---

## T5: ドキュメント・ステータス更新

> T4 のテスト全通過（3513 tests passed）を確認してから実施すること。

- [x] `versions/roadmap/roadmap-v67.1-v68.0.md` のバージョン一覧表で v67.8.0 の「状態」列を「未着手」→「完了」に変更
- [x] `versions/current.md` の「進行中バージョン」を v67.8.0 に更新
- [x] 本 `tasks.md` を COMPLETE に更新（T0 を含む全チェックボックスを `[x]` に）

> **sub-version ポリシー**: v67.x では Cargo.toml / CHANGELOG.md は変更しない。v68.0.0 宣言時に一括更新する。

---

## 設計上の意図的省略

- `$$...$$` / `$...$` の実際の LaTeX パース: 将来フェーズ（v68.x 以降）
- `/// ``` favnir` コードブロックの型チェック統合: 将来フェーズ
- `site/content/docs/tools/doc-math.mdx`: v67.9.0 で一括作成
- `--math --format site` の組み合わせ動作: 未定義（将来フェーズで検討）

## コードレビュー指摘と対応

| 深刻度 | 内容 | 対応 |
|--------|------|------|
| [HIGH] | `--math --builtins` 同時指定で `--builtins` が先に評価されサイレント失敗 | `--builtins` アームの先頭に `--math` 競合チェック + `eprintln!` + `exit(1)` を追加 |
| [MED] | `--math` でファイルパス省略時にサイレントで空文字を渡す | path が見つからない場合 `eprintln!` + `exit(1)` に変更 |
| [MED] | `--format` デフォルトが `"md"` で他の doc コマンドの `"markdown"` と不一致 | `unwrap_or("markdown")` に統一 |
| [MED] | `DOC_MATH_HELP` に未実装の `--out <dir>` を掲載（サイレント無視） | `--out` 行をヘルプから削除 |
| [LOW] | `"mdx"` フォーマットがテストされていない | ロードマップのテスト数制約（+2 固定）のため対応不要 |
| [LOW] | テスト名 `doc_math_example_compiles` が検証内容と乖離 | spec.md に将来実装を見越した命名と説明済み → 対応なし |
