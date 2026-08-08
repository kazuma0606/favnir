# v67.8.0 実装計画

## ステップ

### Step 1: `fav/src/doc_math.rs` 新規作成

```rust
// fav/src/doc_math.rs — v67.8.0 Math-Aware Doc Generation

pub const DOC_MATH_HELP: &str = "\
fav doc --math — 数式対応ドキュメント生成

使用例:
  fav doc --math runes/autodiff/autodiff.fav
  fav doc --math runes/linalg/linalg.fav --format html
  fav doc --math pipeline.fav --format mdx --out site/content/docs/

フラグ:
  --math          LaTeX 数式を MathJax 記法で出力
  --format <fmt>  出力フォーマット: md (default) / html / mdx
  --out <dir>     出力ディレクトリ（default: docs/）
  --help, -h      このヘルプを表示

対応記法:
  $$...$$  ブロック数式（MathJax ブロック形式）
  $...$    インライン数式
";

pub fn cmd_doc_math(src: &str, format: &str) -> String {
    // スタブ実装: 将来フェーズで /// コメントの LaTeX パースを実装
    let math_block = "$$\n\\nabla f(x) = \\frac{\\partial f}{\\partial x}\n$$";
    match format {
        "html" => format!(
            "<!-- fav doc --math --format html: {} -->\n\
             <script src=\"https://cdn.jsdelivr.net/npm/mathjax@3\"></script>\n\
             <p>Gradient: ∇f(x)</p>\n\
             <p>{}</p>\n\
             <!-- --math --format html output -->",
            src, math_block
        ),
        "mdx" => format!(
            "{{/* fav doc --math --format mdx: {} */}}\n\
             import MathJax from 'better-react-mathjax';\n\n\
             ## Math Reference\n\n\
             {}\n\n\
             Gradient: ∇f(x) — 逆伝播法（backpropagation）で計算",
            src, math_block
        ),
        _ => format!(
            "<!-- fav doc --math: {} -->\n\n\
             ## Math Reference\n\n\
             {}\n\n\
             Gradient: ∇f(x)\n\n\
             MathJax 形式で出力（`--format html` で HTML+MathJax 出力）",
            src, math_block
        ),
    }
}
```

### Step 2: `fav/src/main.rs` — `Some("doc")` アームに `--math` 分岐を追加

`--serve` 分岐の直後（`let format = args.windows(2)...` の直前）に挿入:

```rust
// fav doc --math [--format md|html|mdx] [path]
if args.iter().any(|a| a == "--math") {
    if args.iter().any(|a| a == "--help" || a == "-h") {
        print!("{}", doc_math::DOC_MATH_HELP);
        return;
    }
    let format = args.windows(2)
        .find(|w| w[0] == "--format")
        .map(|w| w[1].as_str())
        .unwrap_or("md");
    let path = args.iter().skip(2)
        .find(|a| !a.starts_with('-'))
        .map(|s| s.as_str())
        .unwrap_or("");
    println!("{}", doc_math::cmd_doc_math(path, format));
    return;
}
```

また `mod doc_math;` を main.rs の mod 宣言部に追加。

### Step 3: `driver.rs` — `v67800_tests` 追加

```rust
// -- v67800_tests (v67.8.0) -- Math-Aware Doc Generation --
#[cfg(test)]
mod v67800_tests {
    #[test]
    fn doc_math_latex_rendered() {
        let result = crate::doc_math::cmd_doc_math("test.fav", "md");
        assert!(
            result.contains("$$") && result.contains("MathJax") && result.contains("∇"),
            "cmd_doc_math should output '$$', 'MathJax', and '∇'"
        );
    }

    #[test]
    fn doc_math_example_compiles() {
        let result = crate::doc_math::cmd_doc_math("test.fav", "html");
        assert!(
            result.contains("--math") && result.contains("--format"),
            "cmd_doc_math html output should mention '--math' and '--format'"
        );
    }
}
```

`// -- v67700_tests (v67.7.0)` の直前に挿入（新しいものが上）。

## 注意事項

- `mod doc_math;` を `main.rs` の mod 宣言部に追加（`mod doc_server;` / `mod docs_assets;` 付近）
- `--math` フラグは `--builtins` / `--serve` の次に判定する（既存コードに影響なし）
- sub-version ポリシー: Cargo.toml / CHANGELOG は変更しない
