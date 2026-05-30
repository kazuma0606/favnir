# Favnir v7.1.0 Spec — fav explain --lineage（データリネージ）

作成日: 2026-05-27

## テーマ

エフェクトシグネチャを静的解析してデータの流れを可視化する。
数百万円のデータカタログと同等の情報をコードから自動生成する。

## 背景

v7.0.0 で `!DbRead` / `!DbWrite` / `!DbAdmin` のエフェクト細分化が完成した。
v7.1.0 では、これらのエフェクトと SQL 文字列リテラルを静的解析することで
**どのテーブルを読んで・どのテーブルに書いているか**をコードから自動出力する。

## 目標出力

```bash
fav explain --lineage pipeline.fav
```

```
LINEAGE: pipeline.fav
======================

Sources:
  !DbRead   → users, orders
  !Io       → /data/config.csv

Sinks:
  !DbWrite  → audit_log
  !AWS      → s3://my-bucket/reports/

Transformations:
  LoadOrders  : String  → List<Order>    !DbRead
  Summarize   : List<Order> → List<Summary>  (pure)
  SaveReport  : List<Summary> → Int      !DbWrite

Pipeline:
  OrderReport = LoadOrders |> Summarize |> SaveReport
```

## 現状分析

| 項目 | 現状 |
|------|------|
| `fav explain` | VIS/NAME/TYPE/EFFECTS/DEPS テーブルを出力 |
| `--lineage` フラグ | なし |
| SQL テーブル名抽出 | なし |
| seq/stage 変換チェーン表示 | DEPS 列で部分的に表示 |
| `cmd_explain` シグネチャ | `(file, schema, format, focus)` |

## 実装設計

### データ構造

```rust
struct LineageReport {
    sources: Vec<LineageEntry>,   // 読み込み元
    sinks:   Vec<LineageEntry>,   // 書き込み先
    stages:  Vec<StageLineage>,   // stage/fn の変換情報
    pipelines: Vec<PipelineLineage>, // seq の変換チェーン
}

struct LineageEntry {
    effect: String,     // "!DbRead", "!AWS", "!Io" 等
    targets: Vec<String>, // テーブル名 / ファイルパス / S3 パス
}

struct StageLineage {
    name:    String,
    input:   String,
    output:  String,
    effects: Vec<String>,
}

struct PipelineLineage {
    name:  String,
    chain: Vec<String>, // stage 名の順序
}
```

### SQL テーブル名抽出（ベストエフォート）

`DB.query_raw` / `DuckDb.query_raw` / `DB.execute_raw` の文字列リテラル引数を正規表現で解析：

| パターン | 対象 |
|---------|------|
| `FROM\s+(\w+)` | SELECT ソーステーブル |
| `JOIN\s+(\w+)` | JOIN テーブル |
| `INSERT\s+INTO\s+(\w+)` | INSERT 先テーブル |
| `UPDATE\s+(\w+)` | UPDATE 対象テーブル |
| `DELETE\s+FROM\s+(\w+)` | DELETE 対象テーブル |

動的クエリ（文字列リテラルでない場合）は `(dynamic)` と表示。

### AST 走査方針

- `ast::Program` の items を走査
- `Item::TrfDef` / `Item::FnDef` のエフェクトリストと本体を収集
- 本体 `Expr` を再帰的に走査して `Expr::MethodCall { object: "DB"|"DuckDb", args }` を検出
- `args[0]` が `Expr::Str(sql)` なら SQL パース、それ以外は `(dynamic)`
- `Item::FlowDef`（seq）の steps から変換チェーンを構築

## スコープ

### 実装するもの

- `--lineage` フラグ（main.rs の `explain` コマンドパーサーに追加）
- `cmd_explain_lineage(file, format)` 関数（driver.rs）
- `fn lineage_analysis(program: &ast::Program) -> LineageReport`
- `fn render_lineage_text(report: &LineageReport) -> String`
- `fn render_lineage_json(report: &LineageReport) -> String`
- テキスト形式とJSON形式（`--format json`）の両対応

### スコープ外（v7.x 以降）

- `!Io` のファイルパス抽出（`IO.read_file_raw` 引数のパース）
- `!AWS` の S3 パス抽出（`AWS.s3_get_raw` 等の引数）
- 変換グラフ内の join/filter 等の内部演算の可視化
- `fav explain --lineage` の複数ファイル対応（初版は単一ファイルのみ）

## 完了条件

- `fav explain --lineage pipeline.fav` が Sources / Sinks / Transformations / Pipeline セクションを出力する
- `--format json` でも lineage データが JSON として出力される
- `!DbRead` / `!DbWrite` エフェクトを持つ stage の SQL が静的文字列の場合、テーブル名が抽出される
- `seq` 定義からパイプラインチェーンが正しく表示される
- 既存テスト 1044 件が全件通る
