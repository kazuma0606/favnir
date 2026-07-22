# Spec: v52.4.0 — `fav explain --lineage` インタラクティブ HTML レポート

Status: PLANNED
Date: 2026-07-21

---

## 目的

v52.3.0 で `--with-schema` オプションによるスキーマ情報付加を実装した。
v52.4.0 では `--format html` オプションを追加し、依存グラフを SVG でレンダリングして
クリックで stage 詳細（型・エフェクト・スキーマ）を表示できる**自己完結型 HTML** を生成する。

外部ライブラリ不要。生成された HTML をそのままブラウザで開ける。

---

## 使用例

```bash
$ fav explain --lineage pipeline.fav --format html -o lineage.html
# → lineage.html に書き出し（ブラウザで開けるインタラクティブ HTML）

$ fav explain --lineage pipeline.fav --format html
# → stdout に出力（-o 未指定時）
```

---

## 出力 HTML の構造

```html
<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="utf-8">
<title>Favnir Lineage Report</title>
<style>
  body { font-family: sans-serif; margin: 20px; background: #fff; }
  .node rect { fill: #eef6f9; stroke: #555; cursor: pointer; }
  .node rect:hover { fill: #d0e8f0; }
  .node text { pointer-events: none; }
  #detail { margin-top: 20px; padding: 12px; background: #f9f9f9;
            border: 1px solid #ddd; border-radius: 4px; min-height: 60px; }
  table { border-collapse: collapse; }
  td { padding: 4px 8px; border-bottom: 1px solid #eee; vertical-align: top; }
  td:first-child { font-weight: bold; color: #555; width: 100px; }
</style>
</head>
<body>
<h1>Favnir Lineage Report</h1>
<svg xmlns="http://www.w3.org/2000/svg" width="N" height="160">
  <!-- arrowhead marker -->
  <!-- stage nodes as <g class="node" onclick="showDetail('StageName')"> -->
  <!-- pipeline edges as <line> with marker-end -->
</svg>
<div id="detail"><em>Click a stage node to see details.</em></div>
<script>
var stages = {
  "StageName": {
    "kind": "transform",
    "effects": "Pure",
    "schema": "OrderRow",
    "sources": "",
    "sinks": ""
  }
};
function showDetail(name) {
  var s = stages[name];
  if (!s) return;
  document.getElementById('detail').innerHTML =
    '<table>' +
    '<tr><td>Name</td><td>' + name + '</td></tr>' +
    '<tr><td>Kind</td><td>' + s.kind + '</td></tr>' +
    '<tr><td>Effects</td><td>' + s.effects + '</td></tr>' +
    (s.schema ? '<tr><td>Schema</td><td>' + s.schema + '</td></tr>' : '') +
    (s.sources ? '<tr><td>Sources</td><td>' + s.sources + '</td></tr>' : '') +
    (s.sinks   ? '<tr><td>Sinks</td><td>'   + s.sinks   + '</td></tr>' : '') +
    '</table>';
}
</script>
</body>
</html>
```

---

## 変更ファイル一覧

| ファイル | 変更内容 |
|---|---|
| `fav/src/lineage.rs` | `render_lineage_html(report: &LineageReport) -> String` を追加（`render_lineage_svg` の直後） |
| `fav/src/driver.rs` | `pub use crate::lineage::` に `render_lineage_html` を追加、`cmd_explain_lineage` に `output: Option<&str>` 引数追加、`"html"` アームを追加、ファイル書き出しロジックを追加 |
| `fav/src/main.rs` | `--lineage` ブロックに `output_file` 変数と `-o <file>` フラグ解析を追加 |
| `fav/Cargo.toml` | version → `"52.4.0"` |
| `CHANGELOG.md` | v52.4.0 エントリ追加 |
| `versions/current.md` | v52.4.0（3143 tests）に更新 |
| `versions/roadmap/roadmap-v52.1-v53.0.md` | v52.4.0 実績欄を更新 |

---

## 詳細仕様

### 1. `render_lineage_html(report: &LineageReport) -> String`（lineage.rs）

`render_lineage_svg` の直後（`render_lineage_text` の前）に追加する。

**SVG レイアウト**:
- 既存の `render_lineage_svg` と同じ座標体系を使用（ノード幅 160px、間隔 200px、高さ 160px）
- ノードは `<g class="node" onclick="showDetail('NAME')">` でラップ（クリック可能）
- `onclick` の NAME はダブルクォートのエスケープが必要: `&quot;` を使う

**JSON データ生成**:
- `stages` オブジェクトの各エントリ:
  - `kind`: `entry.kind`
  - `effects`: `entry.effects` が空なら `"Pure"`、あれば `!X+!Y` 形式
  - `schema`: `entry.schema.as_deref().unwrap_or("")`
  - `sources`: `entry.sources.join(", ")`
  - `sinks`: `entry.sinks.join(", ")`
- JSON 値に含まれるダブルクォートは `\"` でエスケープする（`str::replace('"', "\\\"")` を使用）

**`showDetail` 関数**:
- `stages[name]` を参照してテーブル行を構築
- `schema` / `sources` / `sinks` は値が空文字のとき行を省略する

**実装例（骨格）**:

```rust
pub fn render_lineage_html(report: &LineageReport) -> String {
    let nodes: Vec<&LineageEntry> = report.transformations.iter().collect();
    let n = nodes.len();
    let svg_width = (n * 200 + 40).max(200);

    // ── SVG ──────────────────────────────────────────────────────────────────
    let mut svg = format!(
        "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"{}\" height=\"160\">\n",
        svg_width
    );
    svg.push_str("  <defs><marker id=\"arr\" markerWidth=\"10\" markerHeight=\"7\" refX=\"9\" refY=\"3.5\" orient=\"auto\">\n");
    svg.push_str("    <polygon points=\"0 0, 10 3.5, 0 7\" fill=\"#555\"/>\n");
    svg.push_str("  </marker></defs>\n");

    for (i, entry) in nodes.iter().enumerate() {
        let x = i * 200 + 20;
        let safe_name = entry.name.replace('"', "&quot;");
        svg.push_str(&format!(
            "  <g class=\"node\" onclick=\"showDetail(&quot;{}&quot;)\">\n",
            safe_name
        ));
        svg.push_str(&format!(
            "    <rect x=\"{}\" y=\"70\" width=\"160\" height=\"40\" rx=\"4\"/>\n", x
        ));
        svg.push_str(&format!(
            "    <text x=\"{}\" y=\"86\" text-anchor=\"middle\" font-size=\"12\" fill=\"#222\">{}</text>\n",
            x + 80, entry.name
        ));
        svg.push_str(&format!(
            "    <text x=\"{}\" y=\"100\" text-anchor=\"middle\" font-size=\"10\" fill=\"#666\">{}</text>\n",
            x + 80, entry.kind
        ));
        svg.push_str("  </g>\n");
    }

    // pipeline edges（render_lineage_svg と同じ座標）
    let name_to_idx: std::collections::HashMap<&str, usize> =
        nodes.iter().enumerate().map(|(i, e)| (e.name.as_str(), i)).collect();
    for pipeline in &report.pipelines {
        for i in 0..pipeline.steps.len().saturating_sub(1) {
            if let (Some(&fi), Some(&ti)) = (
                name_to_idx.get(pipeline.steps[i].as_str()),
                name_to_idx.get(pipeline.steps[i + 1].as_str()),
            ) {
                let x1 = fi * 200 + 180;
                let x2 = ti * 200 + 20;
                svg.push_str(&format!(
                    "  <line x1=\"{}\" y1=\"90\" x2=\"{}\" y2=\"90\" stroke=\"#555\" stroke-width=\"1.5\" marker-end=\"url(#arr)\"/>\n",
                    x1, x2
                ));
            }
        }
    }
    svg.push_str("</svg>\n");

    // ── JS stages data ────────────────────────────────────────────────────────
    let mut js_entries = String::new();
    for entry in &nodes {
        let effects = if entry.effects.is_empty() {
            "Pure".to_string()
        } else {
            entry.effects.iter()
                .map(|e| format!("!{}", e.trim_start_matches('!')))
                .collect::<Vec<_>>().join("+")
        };
        let schema  = entry.schema.as_deref().unwrap_or("").replace('"', "\\\"");
        let sources = entry.sources.join(", ").replace('"', "\\\"");
        let sinks   = entry.sinks.join(", ").replace('"', "\\\"");
        let key     = entry.name.replace('"', "\\\"");
        js_entries.push_str(&format!(
            "  \"{}\": {{\"kind\":\"{}\",\"effects\":\"{}\",\"schema\":\"{}\",\"sources\":\"{}\",\"sinks\":\"{}\"}},\n",
            key, entry.kind, effects, schema, sources, sinks
        ));
    }

    // ── HTML 組み立て ──────────────────────────────────────────────────────────
    format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="utf-8">
<title>Favnir Lineage Report</title>
<style>
body{{font-family:sans-serif;margin:20px;background:#fff}}
.node rect{{fill:#eef6f9;stroke:#555;cursor:pointer}}
.node rect:hover{{fill:#d0e8f0}}
.node text{{pointer-events:none}}
#detail{{margin-top:20px;padding:12px;background:#f9f9f9;border:1px solid #ddd;border-radius:4px;min-height:60px}}
table{{border-collapse:collapse}}
td{{padding:4px 8px;border-bottom:1px solid #eee;vertical-align:top}}
td:first-child{{font-weight:bold;color:#555;width:100px}}
</style>
</head>
<body>
<h1>Favnir Lineage Report</h1>
{}
<div id="detail"><em>Click a stage node to see details.</em></div>
<script>
var stages={{{}}};
function showDetail(name){{
  var s=stages[name];if(!s)return;
  var h='<table>';
  h+='<tr><td>Name</td><td>'+name+'</td></tr>';
  h+='<tr><td>Kind</td><td>'+s.kind+'</td></tr>';
  h+='<tr><td>Effects</td><td>'+s.effects+'</td></tr>';
  if(s.schema)h+='<tr><td>Schema</td><td>'+s.schema+'</td></tr>';
  if(s.sources)h+='<tr><td>Sources</td><td>'+s.sources+'</td></tr>';
  if(s.sinks)h+='<tr><td>Sinks</td><td>'+s.sinks+'</td></tr>';
  h+='</table>';
  document.getElementById('detail').innerHTML=h;
}}
</script>
</body>
</html>"#,
        svg, js_entries
    )
}
```

### 2. `cmd_explain_lineage` 更新（driver.rs）

`output: Option<&str>` 引数を追加。`"html"` アームを追加。
出力先が `Some(path)` の場合は `std::fs::write(path, content)` でファイル書き出し、
`None` の場合は `print!` で stdout に出力。

```rust
pub fn cmd_explain_lineage(
    file: Option<&str>,
    format: &str,
    show_dead: bool,
    with_schema: bool,
    output: Option<&str>,
) {
    // ...（ファイル収集・パース処理は変更なし）
    for path in &paths {
        // ...
        let content = match format {
            "json"    => render_lineage_json(&report),
            "mermaid" => render_lineage_mermaid_with_schema(&report, show_dead, with_schema),
            "d2"      => render_lineage_d2(&report),
            "dot"     => render_lineage_dot_with_schema(&report, with_schema),
            "svg"     => render_lineage_svg(&report),
            "html"    => render_lineage_html(&report),
            "text"    => render_lineage_text(&report, path),
            other     => {
                eprintln!("error: unknown format '{}'. valid: text, json, mermaid, d2, dot, svg, html", other);
                process::exit(1);
            }
        };
        if let Some(out_path) = output {
            std::fs::write(out_path, &content).unwrap_or_else(|e| {
                eprintln!("error: could not write to '{}': {}", out_path, e);
                process::exit(1);
            });
        } else {
            print!("{}", content);
        }
    }
}
```

**エラーメッセージの更新**:
既存の `"error: unknown format '{}'. valid: text, json, mermaid, d2, dot, svg"` に `html` を追加する。

### 3. `main.rs` — `-o <file>` フラグ解析

`--lineage` ブロックの変数宣言に追加:
```rust
let mut output_file: Option<String> = None;
```

`while` ループの `match` に追加（`"--format"` アームの直前に追加するのが自然）:
```rust
"-o" => {
    output_file = Some(
        args.get(i + 1)
            .unwrap_or_else(|| {
                eprintln!("error: -o requires a file path");
                process::exit(1);
            })
            .clone()
    );
    i += 2;
}
```

`cmd_explain_lineage` 呼び出しを更新:
```rust
cmd_explain_lineage(file, &format, show_dead, with_schema, output_file.as_deref());
```

---

## テスト（2 件）

追加先: `driver.rs` の `v52400_tests` モジュール（`v52300_tests` の直前）

### `lineage_html_output`

```rust
#[test]
fn lineage_html_output() {
    let src = include_str!("lineage.rs");
    assert!(src.contains("render_lineage_html"), "lineage.rs must have render_lineage_html");
    assert!(src.contains("<!DOCTYPE html>"), "render_lineage_html must emit DOCTYPE");
}
```

### `lineage_html_has_stage_detail`

```rust
#[test]
fn lineage_html_has_stage_detail() {
    let src = include_str!("lineage.rs");
    assert!(src.contains("showDetail"), "render_lineage_html must have showDetail JS function");
    assert!(src.contains("id=\\\"detail\\\""), "render_lineage_html must have detail div");
}
```

### `lineage_html_renders_stage_node`

```rust
#[test]
fn lineage_html_renders_stage_node() {
    let report = make_report();
    let html = render_lineage_html(&report);
    assert!(html.contains("<!DOCTYPE html>"), "must start with DOCTYPE");
    assert!(html.contains("showDetail"), "must have showDetail function");
    assert!(html.contains("id=\"detail\""), "must have detail div");
    assert!(html.contains("<svg"), "must have SVG element");
}
```

---

## テスト数

- ベース: **3141** tests（v52.3.0 完了時点）
- `v52300_tests` に version テストなし → 削除 0 件
- 追加: `v52400_tests` 3 件（`lineage_html_output` + `lineage_html_has_stage_detail` + `lineage_html_renders_stage_node`）
- **合計: 3144 tests**

---

## 完了条件

- `cargo test` 3144 passed, 0 failed
- `cargo clippy -- -D warnings` クリーン
- `fav explain --lineage <file> --format html -o out.html` で自己完結型 HTML が生成される
- 生成された HTML をブラウザで開くと SVG グラフが表示され、ノードをクリックすると詳細パネルが更新される
- `-o` 未指定時は stdout に出力される（既存の他フォーマットと同じ挙動）
- `--format html` の `cmd_explain_lineage` エラーメッセージに `html` が追加されている
- 複数ファイル + `-o` 指定時は最後のファイル出力で上書きされる（仕様として受け入れ、将来バージョンで改善余地あり）
