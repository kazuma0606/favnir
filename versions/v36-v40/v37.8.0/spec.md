# v37.8.0 spec — Multi-Source cookbook 5 本

## バージョン概要

| 項目 | 内容 |
|---|---|
| バージョン | v37.8.0 |
| テーマ | Multi-Source cookbook 5 本 — v37.x スプリントで追加した機能の実用レシピ |
| 前提 | v37.7.0 COMPLETE — `fav new --template multi-source` 実装済み |
| 完了条件 | `v37800_tests` 全テスト pass・`cargo test` 0 failures（≥ 2733 件） |

## 背景と目的

v37.1〜v37.7 で追加した以下の機能を、実際のユースケースとして示す cookbook を整備する。

| cookbook ファイル | 対応機能 |
|---|---|
| `join-two-tables.mdx` | v37.3.0: `List.join_on` |
| `cdc-postgres-to-warehouse.mdx` | v37.5.0: CDC Rune |
| `fan-out-by-region.mdx` | v37.4.0: `List.fan_out` / `List.fan_in` |
| `generic-etl-function.mdx` | v37.1.0/v37.2.0: 境界付きジェネリクス / 行多相 |
| `lineage-visualization.mdx` | v37.6.0: `fav explain --lineage --format dot/svg` |

## 実装スコープ

### 1. `site/content/cookbook/join-two-tables.mdx`

```mdx
---
title: "2 テーブル結合（List.join_on）"
description: "List.join_on を使って 2 つのリストをキーで結合する"
---

# 2 テーブル結合（List.join_on）

`List.join_on(left, right, pred)` で 2 つのリストを述語関数でマッチングして結合します（left semi-join）。

## コード例

```favnir
import runes/postgres as db

stage LoadUsers -> List<String> {
    db.query(ctx, "SELECT id, name FROM users")
}

stage LoadOrders -> List<String> {
    db.query(ctx, "SELECT user_id, amount FROM orders")
}

stage JoinUsersOrders(users: List<String>, orders: List<String>) -> List<String> {
    List.join_on(users, orders, |u, o| String.contains(o, u))
}

pipeline main {
    LoadUsers, LoadOrders |> JoinUsersOrders
}
```

## ポイント

- `List.join_on` は left semi-join: left リストの各要素に対して right リストの要素をマッチング。
- 述語 `pred` は `(left_elem, right_elem) -> Bool` を返すクロージャ。
- 大規模リストの場合は `List.fan_out` でチャンク分割してから並列処理することを推奨。

## 関連

- [fan-out-by-region](/cookbook/fan-out-by-region) — 地域別 fan_out と fan_in の組み合わせ
```

### 2. `site/content/cookbook/cdc-postgres-to-warehouse.mdx`

```mdx
---
title: "CDC（Postgres → データウェアハウス）"
description: "CDC Rune で Debezium イベントをフィルタリングしてデータウェアハウスに書き込む"
---

# CDC（Postgres → データウェアハウス）

CDC Rune（v37.5.0）と Debezium JSON イベントを使い、INSERT/UPDATE イベントのみをウェアハウスに書き込みます。

## コード例

```favnir
import runes/cdc
import runes/postgres as wh

stage ConsumeEvents -> List<String> {
    // Kafka / Kinesis 等から Debezium JSON イベントを取得（省略）
    []
}

stage FilterAndLoad(events: List<String>) -> Int {
    bind inserts <- CDC.filter_inserts(events)
    inserts
        |> List.map(|e| wh.execute(ctx, "INSERT INTO warehouse.events (payload) VALUES ($1)", [e]))
        |> List.length
}

pipeline cdc_pipeline {
    ConsumeEvents |> FilterAndLoad
}
```

## ポイント

- `CDC.filter_inserts(events)` は `"op":"c"` を含むイベントのみを返します。
- `CDC.extract_op(json)` で個別イベントの op コード（"c"/"u"/"d"/"r"）を取得できます。
- JSON はコンパクト形式（スペースなし）のみ対応: `{"op":"c",...}`

## 関連

- [CDC Rune ドキュメント](/docs/runes/cdc)
```

### 3. `site/content/cookbook/fan-out-by-region.mdx`

```mdx
---
title: "地域別 fan_out 処理"
description: "List.fan_out でリストをチャンクに分割し、地域別並列処理後に List.fan_in でマージする"
---

# 地域別 fan_out 処理

`List.fan_out(list, n)` でリストを n チャンクに分割し、`List.fan_in(chunks)` でフラット化します。

## コード例

```favnir
stage LoadRecords -> List<String> {
    // 大量レコードをロード（省略）
    []
}

stage ProcessByRegion(records: List<String>) -> List<String> {
    bind chunks  <- List.fan_out(records, 4)
    bind results <- List.map(chunks, |chunk|
        List.map(chunk, |r| String.concat(["processed:", r]))
    )
    List.fan_in(results)
}

pipeline fan_out_pipeline {
    LoadRecords |> ProcessByRegion
}
```

## ポイント

- `List.fan_out(list, n)` はリストを最大 n チャンクに等分割します（余りは先頭チャンクに加算）。
- `List.fan_in(chunks)` は `List<List<T>>` を 1 レベルフラット化します。
- `par` ステージと組み合わせることで真の並列処理が可能です。

## 関連

- [join-two-tables](/cookbook/join-two-tables) — チャンク後に join_on を適用するパターン
```

### 4. `site/content/cookbook/generic-etl-function.mdx`

```mdx
---
title: "ジェネリック ETL 関数"
description: "境界付きジェネリクスと行多相を使って型安全な汎用 ETL 関数を書く"
---

# ジェネリック ETL 関数

v37.1.0 の `T with Serialize/Deserialize` 制約と v37.2.0 の行多相 `R with { id: Int }` を活用します。

## コード例

```favnir
// T with Serialize: シリアライズ可能な任意の型を受け取る汎用ロード関数
fn load_and_serialize<T with Serialize>(items: List<T>) -> List<String> {
    List.map(items, |item| Json.encode(item))
}

// 行多相: id フィールドを持つ任意のレコード型を受け取る
fn filter_by_id<R with { id: Int }>(records: List<R>, min_id: Int) -> List<R> {
    List.filter(records, |r| r.id >= min_id)
}
```

## ポイント

- `T with Serialize` は型変数 T に制約を付与します（v37.1.0）。
- `R with { id: Int, .. }` は `id` フィールドを持つ任意のレコード型にマッチします（v37.2.0）。
- 制約違反は型チェック時（`fav check`）に型チェックエラーとして検出されます。

## 関連

- [言語リファレンス: 境界付きジェネリクス](/docs/language/generics)
```

### 5. `site/content/cookbook/lineage-visualization.mdx`

```mdx
---
title: "リネージグラフの可視化"
description: "fav explain --lineage --format dot/svg でパイプラインの依存関係をグラフ出力する"
---

# リネージグラフの可視化

v37.6.0 で追加された `dot` / `svg` 形式を使い、パイプラインのデータフローをビジュアル化します。

## 使い方

```bash
# DOT 形式（Graphviz）
fav explain --lineage --format dot src/main.fav > lineage.dot
dot -Tpng lineage.dot -o lineage.png

# インライン SVG（外部ツール不要）
fav explain --lineage --format svg src/main.fav > lineage.svg
```

## 出力例（DOT）

```dot
digraph lineage {
    rankdir=LR;
    node [shape=box style=filled fillcolor="#eef6f9"];
    LoadUsers [label="LoadUsers\nread"];
    JoinAndLoad [label="JoinAndLoad\ntransform"];
    SaveResult [label="SaveResult\nwrite"];
    LoadUsers -> JoinAndLoad;
    JoinAndLoad -> SaveResult;
}
```

## ポイント

- `--format dot` は Graphviz の `dot` / `neato` / `fdp` コマンドで PNG/SVG に変換できます。
- `--format svg` は外部ツール不要のインライン SVG を出力します（CI レポートへの埋め込みに最適）。
- `--format mermaid` / `--format d2` も引き続き利用可能です。

## 関連

- [リネージ解析](/docs/tools/lineage)
```

### 6. `driver.rs` — `v37800_tests` モジュール

```rust
// ── v37800_tests (v37.8.0) — Multi-Source cookbook 5 本 ──────────────────────
#[cfg(test)]
mod v37800_tests {
    // include_str! のみ使用のため imports 不要

    #[test]
    fn cargo_toml_version_is_37_8_0() {
        let cargo = include_str!("../Cargo.toml");
        assert!(cargo.contains("37.8.0"), "Cargo.toml must contain version 37.8.0");
    }

    #[test]
    fn changelog_has_v37_8_0() {
        let src = include_str!("../../CHANGELOG.md");
        assert!(src.contains("[v37.8.0]"), "CHANGELOG.md must contain [v37.8.0]");
    }

    #[test]
    fn multi_source_cookbook_files_exist() {
        let join   = include_str!("../../site/content/cookbook/join-two-tables.mdx");
        let cdc    = include_str!("../../site/content/cookbook/cdc-postgres-to-warehouse.mdx");
        let fanout = include_str!("../../site/content/cookbook/fan-out-by-region.mdx");
        let gen    = include_str!("../../site/content/cookbook/generic-etl-function.mdx");
        let lin    = include_str!("../../site/content/cookbook/lineage-visualization.mdx");

        assert!(join.contains("List.join_on"),       "join-two-tables.mdx must contain List.join_on");
        assert!(cdc.contains("CDC.filter_inserts"),  "cdc-postgres-to-warehouse.mdx must contain CDC.filter_inserts");
        assert!(fanout.contains("List.fan_out"),     "fan-out-by-region.mdx must contain List.fan_out");
        assert!(gen.contains("Serialize"),           "generic-etl-function.mdx must contain Serialize");
        assert!(lin.contains("--format dot"),        "lineage-visualization.mdx must contain --format dot");
    }
}
```

**`include_str!` のみ使用のため `use super::*` / imports 不要。**

## 注意事項

### MDX の frontmatter 形式

既存 cookbook（`duckdb-query.mdx` 等）の形式に合わせる:
```
---
title: "..."
description: "..."
---
```

`kafka-consumer.mdx` のように frontmatter なし（H1 直書き）の形式もあるが、
新規追加は `duckdb-query.mdx` 形式（frontmatter あり）で統一する。

### コードブロックの言語タグ

- Favnir コード: ` ```favnir `
- シェルコマンド: ` ```bash `
- DOT: ` ```dot `

### spec.md 内コードブロックのネストについて

spec.md の §1〜§5 では外側 ` ```mdx ` の中に内側 ` ```favnir ` 等をネストして示している。
実際のファイル作成時は **Write ツールで直接書き込む**こと。
spec.md からのコピーペーストは行わないこと（内側の ` ``` ` でフェンスが誤って閉じる誤認を避けるため）。

### テスト数の計算

| バージョン | 実績 |
|---|---|
| v37.7.0 | 2730 |
| v37.8.0 追加分 | +3 |
| v37.8.0 期待値 | 2733 |

ロードマップは「Rust テスト 1 件」と記載しているが、meta 2 件 + 機能 1 件の計 3 件を追加する。
T7 でロードマップを 3 件に更新する。

## ロードマップとの整合

ロードマップ v37.8.0:
- 5 ファイルが存在する
- Rust テスト 1 件（→ 3 件に更新）

**5 ファイル一覧:**
1. `site/content/cookbook/join-two-tables.mdx` ✓
2. `site/content/cookbook/cdc-postgres-to-warehouse.mdx` ✓
3. `site/content/cookbook/fan-out-by-region.mdx` ✓
4. `site/content/cookbook/generic-etl-function.mdx` ✓
5. `site/content/cookbook/lineage-visualization.mdx` ✓

## 完了条件

| # | 条件 | 検証方法 |
|---|---|---|
| 1 | `Cargo.toml` バージョンが `37.8.0` | `cargo_toml_version_is_37_8_0` テスト |
| 2 | `CHANGELOG.md` に `[v37.8.0]` が含まれる | `changelog_has_v37_8_0` テスト |
| 3 | 5 つの cookbook ファイルがすべて存在し各ファイルのキーワードを含む | `multi_source_cookbook_files_exist` テスト |
| 4 | `cargo test` 全通過（failures=0 かつテスト数 ≥ 2733） | `cargo test` 実行結果（v37.7.0 実績 2730 + 3 件 = 2733） |
| 5 | 5 つの MDX の frontmatter が `title` / `description` を含む正しい YAML 形式 | 目視確認 |
