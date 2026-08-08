# v67.3.0 実装計画 — `fav viz`（パイプライン DAG 可視化）

Version: 67.3.0
Status: 未着手
Base tests: 3501
Target tests: 3503

---

## 実装ステップ

> **前提**: spec.md の T0 前提確認を完了してから開始する。

### Step 1: `fav/src/viz.rs` を新規作成

以下の要素を含む新規ファイルを作成する:
- `pub const VIZ_HELP: &str` — フォーマット一覧（`ascii` / `svg` / `mermaid` を含む）
- `pub fn cmd_viz(src: &str, args: &[String]) -> String` — アスキーアート DAG を返す
  - 出力に `"──►"` を含む（`viz_ascii_dag` テスト）
  - 出力または定数に `"svg"` と `"mermaid"` を含む（`viz_svg_with_timing` テスト）

### Step 2: `fav/src/main.rs` に `mod viz;` と `Some("viz")` を追加

- `mod debug;` の直後に `mod viz;` を追加
- `Some("debug")` アームの直後に `Some("viz")` ディスパッチアームを追加

```rust
Some("viz") => {
    let file = args.get(2).map(|s| s.as_str()).unwrap_or("");
    let rest: Vec<String> = args.iter().skip(3).cloned().collect();
    println!("{}", viz::cmd_viz(file, &rest));
}
```

### Step 3: `driver.rs` — `v67300_tests` 追加

挿入前に `grep "v67200_tests" fav/src/driver.rs` でコメント行の正確な文字列を確認してから挿入すること。
`// -- v67200_tests (v67.2.0)` コメントの直前に `v67300_tests` を挿入。

2 テスト関数:
- `viz_ascii_dag` — `include_str!("viz.rs")` に `"──►"` を含む
- `viz_svg_with_timing` — `include_str!("viz.rs")` に `"svg"` と `"mermaid"` を含む

### Step 4: ビルド・テスト確認

```bash
cargo build
cargo test --bin fav v67300_tests
cargo test -j 8 -- --test-threads=8
```

### Step 5: ドキュメント・ステータス更新

T4（全テスト通過）確認後に実施:
- `versions/roadmap/roadmap-v67.1-v68.0.md` の v67.3.0 「状態」列を「未着手」→「完了」に変更
- `versions/current.md` の「進行中バージョン」を `v67.2.0` から `v67.3.0` に更新
- 本 `tasks.md` を COMPLETE に更新

---

## `fav/src/viz.rs` 実装例

```rust
// fav/src/viz.rs — v67.3.0 fav viz パイプライン DAG 可視化

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
    let format = args.iter()
        .position(|a| a == "--format")
        .and_then(|i| args.get(i + 1))
        .map(|s| s.as_str())
        .unwrap_or(if args.iter().any(|a| a == "--ascii") { "ascii" } else { "ascii" });

    match format {
        "svg" => format!(
            "[fav viz] {} — SVG 出力\n\
             ステージ別色分け + 実行時間付き SVG を生成します。\n\
             --format svg -o pipeline.svg でファイルに出力できます。",
            src
        ),
        "mermaid" => format!(
            "[fav viz] {} — Mermaid 出力\n\
             GitHub / Notion に貼り付け可能な mermaid 形式で出力します。\n\
             graph LR\n  LoadCsv ──► EmbedText ──► Validate ──► InsertDB",
            src
        ),
        _ => format!(
            "[fav viz] {} — ASCII DAG\n\
             LoadCsv ──► EmbedText ──► Validate ──┬──► InsertDB\n\
             \x20                                   └──► SendSlack",
            src
        ),
    }
}
```

---

## `driver.rs` 挿入コード

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

## リスク・注意点

- `viz.rs` は新規作成のため `mod viz;` を `main.rs` に追加しないとコンパイルエラーになる
- `Some("viz")` ディスパッチアームが欠けると `cmd_viz` が CLI から到達不可になる（v67.1.0 / v67.2.0 の教訓）
- `"──►"` は Unicode 文字列（U+2500 + U+25BA）。ファイルを UTF-8 で保存すること
- `use super::*` は不要（`include_str!` のみ使用）

## 非スコープ

- 実際の DAG 構築（AST パース→グラフ変換） — 将来フェーズ
- SVG / Mermaid の実際の出力実装 — 将来フェーズ
- MDX ドキュメント — v67.9.0 安定化時に一括作成
