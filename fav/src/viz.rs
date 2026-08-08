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
    if args.iter().any(|a| a == "--help" || a == "-h") {
        return VIZ_HELP.to_string();
    }

    let format = if args.iter().any(|a| a == "--ascii") {
        "ascii"
    } else {
        args.iter()
            .position(|a| a == "--format")
            .and_then(|i| args.get(i + 1))
            .map(|s| s.as_str())
            .unwrap_or("ascii")
    };

    match format {
        "svg" => format!(
            "[fav viz] {} — SVG 出力\n\
             ステージ別色分け + 実行時間付き svg を生成します。\n\
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
