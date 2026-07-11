# v37.9.0 spec — v38.0 前調整・安定化

## バージョン概要

| 項目 | 内容 |
|---|---|
| バージョン | v37.9.0 |
| テーマ | v38.0 前調整・安定化 — Multi-Source ETL 機能群の統合と品質向上 |
| 前提 | v37.8.0 COMPLETE — Multi-Source cookbook 5 本整備済み |
| 完了条件 | `v37900_tests` 全テスト pass・`cargo test` 0 failures（≥ 2737 件） |

## 背景と目的

v37.1〜v37.8 で実装した Multi-Source ETL 機能群（`List.join_on` / `List.fan_out` / CDC Rune / lineage DOT/SVG / multi-source テンプレート / cookbook 5 本）を v38.0 マイルストーン宣言前に統合し、品質を引き上げる。

主な調整内容:

1. **`render_lineage_text` にサマリー行追加** — テキスト形式のリネージ出力末尾に `Total: N stage(s), M pipeline(s)` のサマリー行を追加し、一目でパイプライン規模を把握できるようにする。
2. **`site/content/docs/multi-source-etl.mdx` 新規作成** — v37.x 系の Multi-Source ETL 機能を一覧化するドキュメントを作成し、v38.0 マイルストーンの前に公式ドキュメントを整備する。

## 実装スコープ

### 1. `fav/src/lineage.rs` — `render_lineage_text` にサマリー行追加

`render_lineage_text` 関数の末尾（`Pipelines:` ブロックの出力後、`out` を返す直前）にサマリー行を追加する。

**変更箇所（`lineage.rs` 行 1065 付近）:**

```rust
    out.push_str("Pipelines:\n");
    if report.pipelines.is_empty() {
        out.push_str("  (none)\n");
    } else {
        for p in &report.pipelines {
            out.push_str(&format!("  seq {} = {}\n", p.name, p.steps.join(" |> ")));
            if !p.sources.is_empty() {
                out.push_str(&format!("    sources: {}\n", p.sources.join(", ")));
            }
            if !p.sinks.is_empty() {
                out.push_str(&format!("    sinks:   {}\n", p.sinks.join(", ")));
            }
        }
    }

    // v37.9.0: サマリー行
    out.push('\n');
    out.push_str(&format!(
        "Total: {} stage(s), {} pipeline(s)\n",
        report.transformations.len(),
        report.pipelines.len(),
    ));

    out
```

**変更前の末尾:**
```rust
    }

    out
}
```

### 2. `site/content/docs/multi-source-etl.mdx` — Multi-Source ETL ドキュメント

```
---
title: "Multi-Source ETL"
description: "Favnir v37.x — 複数ソースを型安全につなげる Multi-Source ETL 機能群"
---

# Multi-Source ETL

Favnir v37.x では **複数ソースを型安全につなげる** 機能群を提供します。

## `List.join_on` — 2 ソース結合

`List.join_on(left, right, pred)` で 2 つのリストを述語関数でマッチングして結合します（left semi-join）。

```favnir
stage Join(customers: List<String>, orders: List<String>) -> List<String> {
    List.join_on(customers, orders, |c, o| String.contains(o, c))
}
```

## `List.fan_out` / `List.fan_in` — 分散処理

`List.fan_out(list, n)` でリストを n チャンクに分割し、`List.fan_in(chunks)` でマージします。

```favnir
stage Distribute(records: List<String>) -> List<String> {
    bind chunks  <- List.fan_out(records, 4)
    bind results <- List.map(chunks, |chunk| List.map(chunk, process))
    List.fan_in(results)
}
```

## CDC Rune — Debezium イベント処理

Debezium JSON 形式の CDC（Change Data Capture）イベントを処理します。

```favnir
import runes/cdc

stage FilterInserts(events: List<String>) -> List<String> {
    CDC.filter_inserts(events)
}
```

## `fav explain --lineage` — リネージグラフ出力

```bash
fav explain --lineage --format dot src/main.fav > lineage.dot
fav explain --lineage --format svg src/main.fav > lineage.svg
```

## `fav new --template multi-source` — プロジェクトテンプレート

```bash
fav new --template multi-source my_etl_project
```

Postgres + CSV の 2 ソース ETL プロジェクトを生成します。

## 関連 Cookbook

- [2 テーブル結合（List.join_on）](/cookbook/join-two-tables)
- [CDC（Postgres → データウェアハウス）](/cookbook/cdc-postgres-to-warehouse)
- [地域別 fan_out 処理](/cookbook/fan-out-by-region)
- [ジェネリック ETL 関数](/cookbook/generic-etl-function)
- [リネージグラフの可視化](/cookbook/lineage-visualization)
```

### 3. `driver.rs` — `v37900_tests` モジュール

```rust
// ── v37900_tests (v37.9.0) — v38.0 前調整・安定化 ──────────────────────────────
#[cfg(test)]
mod v37900_tests {
    // include_str! のみ使用のため imports 不要

    #[test]
    fn cargo_toml_version_is_37_9_0() {
        let cargo = include_str!("../Cargo.toml");
        assert!(cargo.contains("37.9.0"), "Cargo.toml must contain version 37.9.0");
    }

    #[test]
    fn changelog_has_v37_9_0() {
        let src = include_str!("../../CHANGELOG.md");
        assert!(src.contains("[v37.9.0]"), "CHANGELOG.md must contain [v37.9.0]");
    }

    #[test]
    fn lineage_text_has_summary_line() {
        let src = include_str!("lineage.rs");
        assert!(
            src.contains("Total: {} stage(s), {} pipeline(s)"),
            "lineage.rs must contain Total summary line in render_lineage_text"
        );
    }

    #[test]
    fn multi_source_etl_doc_exists() {
        let doc = include_str!("../../site/content/docs/multi-source-etl.mdx");
        assert!(
            doc.contains("List.join_on"),
            "multi-source-etl.mdx must contain List.join_on"
        );
    }
}
```

**`include_str!` のみ使用のため `use super::*` / imports 不要。**

## 注意事項

### `render_lineage_text` の変更範囲

変更は `lineage.rs` の末尾近く、`out` を返す直前（`out` という変数の最後の `push_str` の後）に 3 行を追加するだけ。他の `render_lineage_*` 関数（`render_lineage_json` / `render_lineage_mermaid` / `render_lineage_dot` / `render_lineage_svg`）は変更しない。

### サマリー行のテスト方法

`lineage_text_has_summary_line` テストは `include_str!("lineage.rs")` でソースコードの文字列定数を確認する（`render_lineage_text` の出力を実際に実行して確認するより簡単）。

### MDX の frontmatter 形式

既存 cookbook・docs の形式に合わせる:
```
---
title: "..."
description: "..."
---
```

### spec.md 内コードブロックのネストについて

§2（MDX コンテンツ）では外側コードブロック内に内側コードブロックをネストして示している。
実際のファイル作成時は **Write ツールで直接書き込む**こと。

### テスト数の計算

| バージョン | 実績 |
|---|---|
| v37.8.0 | 2733 |
| v37.9.0 追加分 | +4 |
| v37.9.0 期待値 | 2737 |

## ロードマップとの整合

ロードマップ v37.9.0:「v38.0 前調整・安定化」（詳細未記載）

本 spec は 4 テストを追加する。ロードマップ更新時に件数を記録する。

## 完了条件

| # | 条件 | 検証方法 |
|---|---|---|
| 1 | `Cargo.toml` バージョンが `37.9.0` | `cargo_toml_version_is_37_9_0` テスト |
| 2 | `CHANGELOG.md` に `[v37.9.0]` が含まれる | `changelog_has_v37_9_0` テスト |
| 3 | `lineage.rs` の `render_lineage_text` がサマリー行を含む | `lineage_text_has_summary_line` テスト |
| 4 | `site/content/docs/multi-source-etl.mdx` が存在し `List.join_on` を含む | `multi_source_etl_doc_exists` テスト |
| 5 | `cargo test` 全通過（failures=0 かつテスト数 ≥ 2737） | `cargo test` 実行結果（v37.8.0 実績 2733 + 4 件 = 2737） |
