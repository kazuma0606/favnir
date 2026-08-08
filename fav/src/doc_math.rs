// fav/src/doc_math.rs — v67.8.0 Math-Aware Doc Generation

pub const DOC_MATH_HELP: &str = "\
fav doc --math — 数式対応ドキュメント生成

使用例:
  fav doc --math runes/autodiff/autodiff.fav
  fav doc --math runes/linalg/linalg.fav --format html
  fav doc --math pipeline.fav --format mdx

フラグ:
  --math          LaTeX 数式を MathJax 記法で出力
  --format <fmt>  出力フォーマット: markdown (default) / html / mdx
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
             <p>Gradient: \u{2207}f(x)</p>\n\
             <p>{}</p>\n\
             <!-- --math --format html output -->",
            src, math_block
        ),
        "mdx" => format!(
            "{{/* fav doc --math --format mdx: {} */}}\n\
             import MathJax from 'better-react-mathjax';\n\n\
             ## Math Reference\n\n\
             {}\n\n\
             Gradient: \u{2207}f(x) — 逆伝播法（backpropagation）で計算",
            src, math_block
        ),
        _ => format!(
            "<!-- fav doc --math: {} -->\n\n\
             ## Math Reference\n\n\
             {}\n\n\
             Gradient: \u{2207}f(x)\n\n\
             MathJax 形式で出力（`--format html` で HTML+MathJax 出力）",
            src, math_block
        ),
    }
}
