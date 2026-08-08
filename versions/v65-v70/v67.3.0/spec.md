# v67.3.0 Spec — `fav viz`（パイプライン DAG 可視化）

Version: 67.3.0
Status: 未着手
Base tests: 3501
Target tests: 3503

---

## 概要

パイプラインの依存関係を DAG（有向非巡回グラフ）として可視化する `fav viz` コマンドを実装する。
CI 環境ではアスキーアート、ブラウザでは SVG、GitHub README では Mermaid 形式を使い分ける。

ロードマップ `roadmap-v67.1-v68.0.md` の v67.3.0 セクションに準拠。

---

## 前提確認（T0 で実施）

- `cargo test -j 8 -- --test-threads=8` でベース 3501 tests passed, 0 failed を確認
- `fav/Cargo.toml` の version が `"67.0.0"` であることを確認（sub-version では変更しない）
- `fav/src/viz.rs` が存在しないことを確認（新規作成）
- `driver.rs` に `v67200_tests` が存在することを確認（`v67300_tests` の挿入位置）
- `driver.rs` に `v67300_tests` が存在しないことを確認（新規追加）
- `cargo test --bin fav v67200_tests` で 2 件 PASS することを確認（前バージョンが正常）
  - 前バージョンのテスト関数名: `debug_record_replay`, `debug_rewind_to_step`
- `versions/current.md` の「進行中バージョン」が `v67.2.0` であることを確認

---

## 実装スコープ

### 1. `fav/src/viz.rs` — 新規作成

`fav viz` コマンドのコア実装。以下のキーワードを含むこと（テストでアサートされる）:
- `"──►"` — アスキーアート DAG の矢印（`viz_ascii_dag` テスト）
- ステージ名を含む文字列（`viz_ascii_dag` テスト）
- `"svg"` — SVG フォーマット出力の説明（`viz_svg_with_timing` テスト）
- `"mermaid"` — Mermaid フォーマット出力の説明（`viz_svg_with_timing` テスト）

> **シグネチャ注記**: ロードマップは `cmd_viz(src, format, output)` と 3 引数シグネチャを定義しているが、
> v67.3.0 では `args: &[String]` で代替する。`format` / `output` の独立引数化は将来フェーズで採用する。

追加する定数・関数の例:

```rust
pub const VIZ_HELP: &str = "\
fav viz — パイプライン DAG 可視化

使用例:
  fav viz pipeline.fav --ascii
  fav viz pipeline.fav --format svg -o pipeline.svg
  fav viz pipeline.fav --format mermaid
  fav viz pipeline.fav --from-profile fav-profile.json

フォーマット:
  ascii    アスキーアート DAG（CI 向け）
  svg      SVG 出力（ブラウザで閲覧可能、実行時間付き色分け）
  mermaid  Mermaid 形式（GitHub / Notion に貼り付け可能）
";

pub fn cmd_viz(src: &str, args: &[String]) -> String {
    // アスキーアート出力（デフォルト / --ascii / --format ascii）
    // SVG / Mermaid はフォーマット文字列として出力
}
```

`cmd_viz` の出力にアスキーアート DAG（`──►` を含む）を含めること。

### 2. `main.rs` — `mod viz;` と `Some("viz")` ディスパッチを追加

- `mod viz;` を `mod debug;` の直後に追加
- `Some("viz")` ディスパッチアームを `Some("debug")` の直後に追加:

```rust
Some("viz") => {
    let file = args.get(2).map(|s| s.as_str()).unwrap_or("");
    let rest: Vec<String> = args.iter().skip(3).cloned().collect();
    println!("{}", viz::cmd_viz(file, &rest));
}
```

### 3. `driver.rs` — `v67300_tests` 追加

挿入位置: `// -- v67200_tests (v67.2.0)` コメントの直前

```rust
// -- v67300_tests (v67.3.0) -- fav viz DAG 可視化 --
#[cfg(test)]
mod v67300_tests {
    #[test]
    fn viz_ascii_dag() {
        let src = include_str!("viz.rs");
        assert!(
            src.contains("──►"),
            "viz.rs should contain '──►' for ascii DAG output"
        );
    }

    #[test]
    fn viz_svg_with_timing() {
        let src = include_str!("viz.rs");
        assert!(
            src.contains("svg") && src.contains("mermaid"),
            "viz.rs should contain 'svg' and 'mermaid' format descriptions"
        );
    }
}
```

---

## 完了条件

- `fav/src/viz.rs` が `"──►"` / `"svg"` / `"mermaid"` を含む
- `fav/src/main.rs` に `mod viz;` と `Some("viz")` ディスパッチアームが存在する
- `cargo build` でエラーなし
- `cargo test --bin fav v67300_tests` で 2 件 PASS
  - `viz_ascii_dag` PASS
  - `viz_svg_with_timing` PASS
- `cargo test -j 8 -- --test-threads=8` で 3503 tests passed, 0 failed

---

## 非スコープ

- 実際の DAG 構築（AST パース→グラフ変換） — 将来フェーズ
- SVG ファイル書き出し（`-o pipeline.svg`） — 将来フェーズ
- `par` ステージの並列分岐・合流の視覚化 — 将来フェーズ
- `--from-profile` オプション（プロファイルデータとの統合） — 将来フェーズ
- MDX ドキュメント — v67.9.0 安定化時に一括作成

---

## 技術ノート

### `include_str!` パス（`fav/src/driver.rs` 起点）

- `"viz.rs"` → `fav/src/viz.rs`（同じ `fav/src/` ディレクトリ）

### `"──►"` 文字について

U+2500（─）× 2 + U+25BA（►）× 1、計 3 文字。
spec/plan に貼り付けてある `──►` をそのままコピーして使うのが最も安全。
`VIZ_HELP` または `cmd_viz` の出力文字列リテラルに含めれば `include_str!` テストが通る。

### テスト数増加の根拠

`v67300_tests` モジュール内の `#[test]` fn 2 件（`viz_ascii_dag` / `viz_svg_with_timing`）で +2。
