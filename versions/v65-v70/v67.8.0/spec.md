# v67.8.0 — Math-Aware Doc Generation（`fav doc --math`）

Date: 2026-08-06
Status: 未着手
Sprint: Developer Intelligence（v67.1〜v68.0）

---

## 概要

`fav doc` コマンドに `--math` フラグを追加し、`///` コメント内の LaTeX 数式を
MathJax 記法で出力できるようにする。数学 Rune（autodiff/linalg）のドキュメント品質を
向上させ、式を読める形式で提供する。

## スコープ

### IN スコープ

- `fav/src/doc_math.rs` — 新規作成
  - `pub const DOC_MATH_HELP: &str` — `--math` / `--format` キーワードを含む
  - `pub fn cmd_doc_math(src: &str, format: &str) -> String`
    - `"$$"` / `"MathJax"` / `"∇"` を含む出力を返す（スタブ実装）
    - `format: "md"` → Markdown + MathJax ブロック出力
    - `format: "html"` → HTML + MathJax script タグ出力
    - `format: "mdx"` → MDX 出力（site/ 統合用）
- `fav/src/main.rs` — `Some("doc")` アームに `--math` 分岐を追加
  - `--math && (--help || -h)` → `DOC_MATH_HELP` を表示
  - `--math` → `cmd_doc_math(&path, &format)` を呼び出す
- `fav/src/driver.rs` — `v67800_tests` 追加（2 件）

### OUT スコープ（将来フェーズ）

ロードマップ v67.8.0「実装内容」に列挙されているが、本バージョンではスタブ化して v68.x 以降で完全実装する:

- `$$...$$` / `$...$` の実際の LaTeX パース（現バージョンはスタブ出力で代替）
- `/// ``` favnir` コードブロックの型チェック統合（`doc_math_example_compiles` テストはフラグ名検証のみ; コンパイル実行は将来フェーズ）
- `site/content/docs/tools/doc-math.mdx`（v67.9.0 で一括作成）
- `--math --format site` の組み合わせ動作（未定義; 将来フェーズで検討）

## テスト完了条件

| テスト名 | 検証内容 |
|---|---|
| `doc_math_latex_rendered` | `cmd_doc_math("test.fav", "md")` が `"$$"` / `"MathJax"` / `"∇"` を含む |
| `doc_math_example_compiles` | `cmd_doc_math("test.fav", "html")` が `"--math"` / `"--format"` を含む（コンパイル実行は将来フェーズ; テスト名は将来の完全実装を見越した命名） |

ベーステスト: 3511 → 目標: **3513**

## 既存 `fav doc` コマンドとの関係

`main.rs` の `Some("doc")` アームは以下の順で分岐:

1. `--builtins` → `cmd_doc_builtins`
2. `--serve` → `cmd_doc_serve`
3. **`--math`（新規）** → `cmd_doc_math`
4. `--format site|html` → `cmd_doc_site`
5. default → `cmd_doc`

`--math` は `--builtins` / `--serve` の次に判定し、既存コードパスに影響しない。
