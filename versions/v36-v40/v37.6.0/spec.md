# v37.6.0 spec — `fav lineage --graph`

## バージョン概要

| 項目 | 内容 |
|---|---|
| バージョン | v37.6.0 |
| テーマ | `fav lineage --graph` — リネージグラフの DOT / SVG 出力 |
| 前提 | v37.5.0 COMPLETE — CDC Rune 実装済み |
| 完了条件 | `v37600_tests` 全テスト pass・`cargo test` 0 failures（≥ 2727 件） |

## 背景と目的

v7.1.0 で実装した `fav explain --lineage` は `text / json / mermaid / d2` の 4 形式をサポートしている。
v37.6.0 では **DOT（Graphviz）形式** と **インライン SVG 形式** を追加し、
リネージグラフをビジュアルツールで活用できるようにする。

**今バージョンで行うこと（スコープ確定）:**
- `lineage.rs` に `render_lineage_dot` / `render_lineage_svg` を追加
- `driver.rs` の `cmd_explain_lineage` に `"dot"` / `"svg"` 形式を追加
- `driver.rs` の `pub use` と help テキストを更新
- `v37600_tests` 4 テスト追加（meta 2 件 + 機能 2 件）

**スコープ外（縮小）:**
- `fav lineage` という独立サブコマンドの追加（`fav explain --lineage` を継続使用）
- `--graph` フラグの独立実装（`--format dot/svg` で同機能を提供）
- Graphviz バイナリ（`dot` コマンド）の外部依存（Rust のみで DOT/SVG を生成）

## 実装スコープ

### 1. `lineage.rs` — `render_lineage_dot`

Graphviz DOT 形式でリネージグラフを出力する。

```rust
pub fn render_lineage_dot(report: &LineageReport) -> String {
    let mut out = String::from("digraph lineage {\n    rankdir=LR;\n    node [shape=box style=filled fillcolor=\"#eef6f9\"];\n");

    // ノード定義
    for entry in &report.transformations {
        let id = sanitize_mermaid_id(&entry.name);
        let label = format!("{}\\n{}", entry.name, entry.kind);
        out.push_str(&format!("    {} [label=\"{}\"];\n", id, label));
    }

    // エッジ定義: pipeline の steps を順に接続
    for pipeline in &report.pipelines {
        let steps = &pipeline.steps;
        for i in 0..steps.len().saturating_sub(1) {
            let from = sanitize_mermaid_id(&steps[i]);
            let to   = sanitize_mermaid_id(&steps[i + 1]);
            out.push_str(&format!("    {} -> {};\n", from, to));
        }
    }

    out.push('}');
    out
}
```

**DOT ID サニタイズ:** 同ファイル内の既存関数 `sanitize_mermaid_id` を直接流用する。
英数字・アンダースコア以外を `_` に変換。先頭が数字なら `n_` プレフィックス。
別名関数 `sanitize_dot_id` は作らない。

**出力例:**
```dot
digraph lineage {
    rankdir=LR;
    node [shape=box style=filled fillcolor="#eef6f9"];
    LoadUsers [label="LoadUsers\nread"];
    FilterActive [label="FilterActive\ntransform"];
    SaveResult [label="SaveResult\nwrite"];
    LoadUsers -> FilterActive;
    FilterActive -> SaveResult;
}
```

### 2. `lineage.rs` — `render_lineage_svg`

Graphviz 非依存のインライン SVG を生成する（外部バイナリ不要）。
各ノードを 160×40px の矩形として横に並べ、矢印で接続する。

```rust
pub fn render_lineage_svg(report: &LineageReport) -> String {
    // ノードインデックス: name → position
    // 各ノードを x=idx*200+20, y=60 に配置
    // ノード幅=160, 高さ=40
    // SVG 全体幅 = nodes * 200 + 40
    let nodes: Vec<&LineageEntry> = report.transformations.iter().collect();
    let n = nodes.len();
    let width  = (n * 200 + 40).max(200);
    let height = 140;

    let mut out = format!(
        "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"{}\" height=\"{}\">\n",
        width, height
    );
    out.push_str("  <defs><marker id=\"arr\" markerWidth=\"10\" markerHeight=\"7\" refX=\"9\" refY=\"3.5\" orient=\"auto\">\n");
    out.push_str("    <polygon points=\"0 0, 10 3.5, 0 7\" fill=\"#555\"/>\n");
    out.push_str("  </marker></defs>\n");

    // ノード描画
    for (i, entry) in nodes.iter().enumerate() {
        let x = i * 200 + 20;
        let y = 60;
        out.push_str(&format!(
            "  <rect x=\"{}\" y=\"{}\" width=\"160\" height=\"40\" rx=\"4\" fill=\"#eef6f9\" stroke=\"#555\"/>\n",
            x, y
        ));
        out.push_str(&format!(
            "  <text x=\"{}\" y=\"{}\" text-anchor=\"middle\" font-size=\"12\" fill=\"#222\">{}</text>\n",
            x + 80, y + 16, entry.name
        ));
        out.push_str(&format!(
            "  <text x=\"{}\" y=\"{}\" text-anchor=\"middle\" font-size=\"10\" fill=\"#666\">{}</text>\n",
            x + 80, y + 30, entry.kind
        ));
    }

    // エッジ描画（pipeline steps → 矢印）
    // ノード名 → x インデックスのマップを利用
    let name_to_idx: std::collections::HashMap<&str, usize> =
        nodes.iter().enumerate().map(|(i, e)| (e.name.as_str(), i)).collect();

    for pipeline in &report.pipelines {
        let steps = &pipeline.steps;
        for i in 0..steps.len().saturating_sub(1) {
            let from_name = steps[i].as_str();
            let to_name   = steps[i + 1].as_str();
            if let (Some(&fi), Some(&ti)) = (name_to_idx.get(from_name), name_to_idx.get(to_name)) {
                let x1 = fi * 200 + 180; // ノード右端
                let y1 = 80;
                let x2 = ti * 200 + 20;  // ノード左端
                let y2 = 80;
                out.push_str(&format!(
                    "  <line x1=\"{}\" y1=\"{}\" x2=\"{}\" y2=\"{}\" stroke=\"#555\" stroke-width=\"1.5\" marker-end=\"url(#arr)\"/>\n",
                    x1, y1, x2, y2
                ));
            }
        }
    }

    out.push_str("</svg>");
    out
}
```

**出力例（ノード 2 個の場合）:**
```xml
<svg xmlns="http://www.w3.org/2000/svg" width="440" height="140">
  ...
  <rect x="20" y="60" width="160" height="40" .../>
  <text ...>LoadUsers</text>
  ...
  <line x1="180" y1="80" x2="220" y2="80" ... marker-end="url(#arr)"/>
  ...
</svg>
```

### 3. `driver.rs` — `pub use` 更新

```rust
pub use crate::lineage::{
    extract_tables_from_sql, lineage_analysis,
    render_lineage_json, render_lineage_text,
    render_lineage_mermaid, render_lineage_d2,
    render_lineage_dot, render_lineage_svg,   // ← 追加
};
```

### 4. `driver.rs` — `cmd_explain_lineage` 更新

```rust
match format {
    "json"    => print!("{}", render_lineage_json(&report)),
    "mermaid" => print!("{}", render_lineage_mermaid(&report)),
    "d2"      => print!("{}", render_lineage_d2(&report)),
    "dot"     => print!("{}", render_lineage_dot(&report)),    // ← 追加
    "svg"     => print!("{}", render_lineage_svg(&report)),    // ← 追加
    "text"    => print!("{}", render_lineage_text(&report, path)),
    other     => {
        eprintln!("error: unknown format '{}'. valid: text, json, mermaid, d2, dot, svg", other);
        process::exit(1);
    }
}
```

### 5. help テキスト更新

`explain --lineage [--format <text|json|mermaid|d2>]` →
`explain --lineage [--format <text|json|mermaid|d2|dot|svg>]`

### 6. `driver.rs` — `v37600_tests` モジュール追加

```rust
// ── v37600_tests (v37.6.0) — fav lineage --graph (DOT / SVG 出力) ──────────
#[cfg(test)]
mod v37600_tests {
    use super::{render_lineage_dot, render_lineage_svg};
    use crate::lineage::{LineageReport, LineageEntry, PipelineLineage};

    fn make_report() -> LineageReport {
        LineageReport {
            transformations: vec![
                LineageEntry {
                    name: "LoadUsers".to_string(),
                    kind: "read".to_string(),
                    capability: None,
                    effects: vec![],
                    sources: vec!["users".to_string()],
                    sinks: vec![],
                },
                LineageEntry {
                    name: "SaveResult".to_string(),
                    kind: "write".to_string(),
                    capability: None,
                    effects: vec![],
                    sources: vec![],
                    sinks: vec!["results".to_string()],
                },
            ],
            pipelines: vec![PipelineLineage {
                name: "main_pipeline".to_string(),
                steps: vec!["LoadUsers".to_string(), "SaveResult".to_string()],
                sources: vec![],
                sinks: vec![],
            }],
        }
    }

    #[test]
    fn cargo_toml_version_is_37_6_0() {
        let cargo = include_str!("../Cargo.toml");
        assert!(cargo.contains("37.6.0"), "Cargo.toml must contain version 37.6.0");
    }

    #[test]
    fn changelog_has_v37_6_0() {
        let src = include_str!("../../CHANGELOG.md");
        assert!(src.contains("[v37.6.0]"), "CHANGELOG.md must contain [v37.6.0]");
    }

    #[test]
    fn lineage_dot_contains_digraph() {
        let report = make_report();
        let dot = render_lineage_dot(&report);
        assert!(dot.contains("digraph lineage"), "DOT output must start with 'digraph lineage'");
        assert!(dot.contains("LoadUsers"), "DOT output must contain node 'LoadUsers'");
        assert!(dot.contains("SaveResult"), "DOT output must contain node 'SaveResult'");
        assert!(dot.contains("LoadUsers -> SaveResult"), "DOT output must contain edge");
    }

    #[test]
    fn lineage_svg_contains_svg_tag() {
        let report = make_report();
        let svg = render_lineage_svg(&report);
        assert!(svg.contains("<svg"), "SVG output must contain <svg");
        assert!(svg.contains("LoadUsers"), "SVG output must contain node 'LoadUsers'");
        assert!(svg.contains("SaveResult"), "SVG output must contain node 'SaveResult'");
        assert!(svg.contains("marker-end"), "SVG output must contain arrow marker");
    }
}
```

**`use super::` の理由:** `render_lineage_dot` / `render_lineage_svg` は `driver.rs` の `pub use` で再エクスポートされており、`super::` でアクセスできる。`LineageReport` 等の struct は `crate::lineage::` から直接インポート。

## 注意事項

### DOT ID のサニタイズ

`render_lineage_dot` 内では `sanitize_mermaid_id`（同ファイル内プライベート関数）を直接呼ぶ。
`sanitize_mermaid_id` は `fn`（プライベート）だが `lineage.rs` 内から呼べるため問題なし。
別名関数を作る必要はない。

### SVG の `name_to_idx` の型

`std::collections::HashMap<&str, usize>` を使うため、ライフタイムに注意。
`nodes` は `Vec<&LineageEntry>` であり、`entry.name.as_str()` のライフタイムは `nodes` と一致する。

### テスト内の `LineageEntry` コンストラクタ

`LineageEntry` は `pub struct` で全フィールド `pub` なので、struct literal で直接構築できる。

### ロードマップの「2 件」との差異

ロードマップは「Rust テスト 2 件」と記載しているが、本バージョンでは meta 2 件 + 機能 2 件の計 4 件を追加する。
理由: 他バージョン（v37.3〜v37.5）との統一パターン（meta 2 件必須）。
T8 でロードマップを 4 件に更新する。

## ロードマップとの整合

ロードマップ v37.6.0:
- `fav lineage --graph --format dot/svg` でリネージグラフを出力する
- 完了条件: DOT / SVG が生成される / Rust テスト 2 件（→ 4 件に更新）

**スコープ縮小の理由（`--graph` フラグ省略）:**
`fav explain --lineage --format dot/svg` で同機能を提供する。
`--graph` フラグを `fav explain --lineage` に追加すると引数解析が複雑になるため、
`--format` の新オプションとして実装するのが最小変更で安全。

## 完了条件

| # | 条件 | 検証方法 |
|---|---|---|
| 1 | `Cargo.toml` バージョンが `37.6.0` | `cargo_toml_version_is_37_6_0` テスト |
| 2 | `CHANGELOG.md` に `[v37.6.0]` が含まれる | `changelog_has_v37_6_0` テスト |
| 3 | `render_lineage_dot` が `digraph lineage` と正しいエッジを出力する | `lineage_dot_contains_digraph` テスト |
| 4 | `render_lineage_svg` が `<svg` タグと矢印マーカーを出力する | `lineage_svg_contains_svg_tag` テスト |
| 5 | `cargo test` 全通過（failures=0 かつテスト数 ≥ 2727） | `cargo test` 実行結果（v37.5.0 実績 2723 + 4 件 = 2727） |
