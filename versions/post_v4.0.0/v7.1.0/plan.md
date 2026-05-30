# Favnir v7.1.0 Plan — fav explain --lineage

作成日: 2026-05-27

## 変更ファイル一覧

| ファイル | 変更種別 | 内容 |
|---------|---------|------|
| `fav/src/main.rs` | 変更 | `explain` コマンドに `--lineage` フラグ追加、`cmd_explain_lineage` 呼び出し |
| `fav/src/driver.rs` | 変更 | `cmd_explain_lineage` / `lineage_analysis` / `render_lineage_text` / `render_lineage_json` を追加 |
| `site/content/docs/language/pipeline.mdx` | 変更 | `fav explain --lineage` の使い方を追記 |

---

## Phase A — CLI フラグ追加（main.rs）

`fav explain` の引数パーサーに `--lineage` を追加する。

```rust
// main.rs の explain ブランチ
"--lineage" => {
    lineage = true;
    i += 1;
}
```

`lineage == true` の場合：
```rust
cmd_explain_lineage(file, &format);
return;
```

`cmd_explain` のシグネチャは変更しない（後方互換）。

---

## Phase B — `lineage_analysis` 関数（driver.rs）

```rust
fn lineage_analysis(program: &ast::Program) -> LineageReport
```

**処理フロー:**

1. `program.items` を走査
2. `Item::TrfDef` / `Item::FnDef` から:
   - エフェクトリストを収集 → `StageLineage.effects`
   - 本体 Expr を再帰走査 → SQL 文字列リテラルを抽出
3. SQL 文字列からテーブル名を正規表現で抽出:
   - `FROM (\w+)` / `JOIN (\w+)` → sources (`!DbRead`)
   - `INSERT INTO (\w+)` / `UPDATE (\w+)` / `DELETE FROM (\w+)` → sinks (`!DbWrite`)
4. `Item::FlowDef`（seq）から `PipelineLineage` を構築:
   - `steps` フィールドから stage 名のチェーンを取得

**SQL 抽出の補助関数:**

```rust
fn extract_tables_from_sql(sql: &str) -> (Vec<String>, Vec<String>)
// → (read_tables, write_tables)
```

```rust
fn collect_sql_literals(expr: &ast::Expr) -> Vec<String>
// AST を再帰走査して DB.*_raw の文字列引数を返す
```

---

## Phase C — `render_lineage_text` / `render_lineage_json`（driver.rs）

### テキスト形式

```
LINEAGE: <filename>
======================

Sources:
  !DbRead   → users, orders
  !Io       → (none detected)

Sinks:
  !DbWrite  → audit_log

Transformations:
  LoadOrders  : String → List<Order>        !DbRead
  Summarize   : List<Order> → List<Summary>  (pure)
  SaveReport  : List<Summary> → Int          !DbWrite

Pipelines:
  OrderReport = LoadOrders |> Summarize |> SaveReport
```

### JSON 形式（`--format json`）

```json
{
  "lineage": {
    "sources": [
      { "effect": "!DbRead", "targets": ["users", "orders"] }
    ],
    "sinks": [
      { "effect": "!DbWrite", "targets": ["audit_log"] }
    ],
    "transformations": [
      { "name": "LoadOrders", "input": "String", "output": "List<Order>", "effects": ["!DbRead"] }
    ],
    "pipelines": [
      { "name": "OrderReport", "chain": ["LoadOrders", "Summarize", "SaveReport"] }
    ]
  }
}
```

---

## Phase D — テスト追加（driver.rs）

テスト対象:
1. `extract_tables_from_sql` のユニットテスト:
   - `SELECT * FROM users` → read: ["users"], write: []
   - `INSERT INTO orders (...)` → read: [], write: ["orders"]
   - `UPDATE users SET ...` → read: [], write: ["users"]
   - `SELECT u.id FROM users u JOIN orders o ON ...` → read: ["users", "orders"], write: []
2. `lineage_analysis` の統合テスト:
   - stage/seq を含む .fav ソースを入力 → LineageReport の内容を確認
3. `render_lineage_text` のスモークテスト

---

## Phase E — ドキュメント更新

`site/content/docs/language/pipeline.mdx` に `fav explain --lineage` セクションを追記:
- コマンド例
- 出力フォーマットの説明
- Schema Authority との連携（!DbRead/!DbWrite によるデータフロー可視化）
