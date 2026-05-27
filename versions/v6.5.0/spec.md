# Favnir v6.5.0 仕様書 — サイトドキュメント補完

作成日: 2026-05-27

---

## テーマ

実装済みだがドキュメントが存在しない機能の docs を追加する。

現状の課題:
- `stage` / `seq` / `|>` / `abstract seq` — 言語のコアだがドキュメントなし
- `schemas/*.yaml` の制約仕様 — 実装済みだが書き方が不明
- `fav infer` CLI — 使い方が docs に存在しない
- `fav deploy` / `fav build --schema` — `rune-cli.mdx` に未記載

---

## Phase A — `language/pipeline.mdx`

### 概要

`stage` / `seq` / `|>` / `abstract stage` / `abstract seq` を解説する。
Favnir の中核的差別化要素であり、最初に充実させるべきドキュメント。

### 構成

1. **stage とは** — 型契約とエフェクトを持つ変換の単位
   ```favnir
   stage ParseCsv: String -> List<Row> !Io = |path| { /* ... */ }
   stage Validate: Row -> Row            = |row|  { /* ... */ }
   stage SaveToDb: Row -> Int       !Db  = |row|  { /* ... */ }
   ```

2. **seq とは** — 名前を持つデータフローの構造
   ```favnir
   seq UserImport = ParseCsv |> Validate |> SaveToDb
   // UserImport : String -> Int  !Io !Db
   ```
   - `seq` は関数合成の結果ではなく「アーキテクチャの単位」
   - コンパイラがパイプライン構造を理解できる
   - エフェクトは自動的に合成される

3. **`|>` 演算子** — 型チェック付きのパイプ合成
   - 左の出力型と右の入力型が一致することをコンパイル時に検証

4. **abstract stage / abstract seq** — 依存注入
   ```favnir
   abstract stage Notify: String -> Unit !Io
   abstract seq ReportPipeline = LoadData |> Transform |> Notify

   // 注入: 実際の実装を bind
   stage EmailNotify: String -> Unit !Io = |msg| { /* send email */ }
   bind ReportPipeline = with { Notify = EmailNotify }
   ```

5. **fav explain 出力例**
   ```
   NAME          TYPE                       EFFECTS
   UserImport    String -> Int              !Io !Db
     ParseCsv    String -> List<Row>        !Io
     Validate    Row -> Row
     SaveToDb    Row -> Int                 !Db
   ```

### 完了条件

- `site/content/docs/language/pipeline.mdx` が存在する
- コード例がすべて有効な Favnir 構文である
- category: "言語仕様", order: 6

---

## Phase B — `language/schema.mdx`

### 概要

`schemas/*.yaml` の書き方と制約一覧を解説する。
型定義に付加する「ランタイム制約」の仕様書。

### 構成

1. **スキーマとは**
   - Favnir 型に付加する制約の宣言
   - `fav check` でコンパイル時に制約の整合性を確認
   - `T.validate` でランタイムに制約を検証（v6.6.0 で完全実装）

2. **基本構文 (`schemas/orders.yaml`)**
   ```yaml
   type: Order
   fields:
     id:
       type: Int
       constraints:
         positive: true
     customer_name:
       type: String
       constraints:
         max_length: 100
     email:
       type: String
       constraints:
         pattern: "^[^@]+@[^@]+$"
     amount:
       type: Float
       constraints:
         positive: true
     cancelled_at:
       type: String
       constraints:
         nullable: true
   ```

3. **制約一覧**

   | 制約キー | 対象型 | 説明 |
   |---------|-------|------|
   | `positive: true` | Int / Float | 0 より大きい値のみ許容 |
   | `non_negative: true` | Int / Float | 0 以上の値のみ許容 |
   | `max_length: N` | String | 最大 N 文字 |
   | `min_length: N` | String | 最小 N 文字 |
   | `pattern: "regex"` | String | 正規表現にマッチすること |
   | `nullable: true` | 任意 | null / None を許容 |
   | `one_of: [...]` | String | 列挙値のいずれかであること |

4. **スキーマファイルの配置**
   ```
   project/
     schemas/
       orders.yaml
       users.yaml
     src/
       main.fav
   ```

5. **`fav build --schema`** — スキーマから DDL 生成
   ```bash
   fav build --schema schemas/orders.yaml --dialect postgres
   # → CREATE TABLE orders (...)
   ```

### 完了条件

- `site/content/docs/language/schema.mdx` が存在する
- 制約一覧がすべて記載されている
- category: "言語仕様", order: 7

---

## Phase C — `stdlib/infer.mdx`

### 概要

`fav infer` CLI の使い方を解説する。
外部データソースから Favnir 型定義を自動生成する機能。

### 構成

1. **fav infer とは**
   - CSV / DB / プロトコルバッファから型定義を生成
   - 生成結果に `schemas/` の制約ヒントを付与

2. **CSV からの型推論**
   ```bash
   fav infer --csv data/orders.csv
   ```
   出力:
   ```favnir
   type Order = {
     id:            Int
     customer_name: String
     email:         String
     amount:        Float
   }
   ```

3. **データベースからの型推論**
   ```bash
   fav infer --db "postgresql://localhost/mydb" --table orders
   # SQLite:
   fav infer --db "sqlite://./dev.db" --table users
   ```

4. **Proto からの型推論**
   ```bash
   fav infer --proto schemas/user.proto
   ```

5. **スキーマファイルへの出力**
   ```bash
   fav infer --csv data/orders.csv --out schemas/order.yaml
   # → schemas/order.yaml を生成（制約ヒント付き）
   ```

6. **生成型の使い方**
   ```favnir
   // fav infer が生成した型をそのまま使う
   import rune "duckdb"

   type Order = { id: Int  customer_name: String  amount: Float }

   public fn main() -> Unit !Io !Db {
     bind conn   <- duckdb.open(":memory:")
     bind orders <- duckdb.query<Order>(conn, "SELECT * FROM 'data/orders.csv'")
     IO.println($"読み込み: {List.length(orders)} 件")
   }
   ```

### 完了条件

- `site/content/docs/stdlib/infer.mdx` が存在する
- 3 種類のデータソース（CSV / DB / Proto）が記載されている
- category: "標準ライブラリ", order: 6

---

## Phase D — `rune-cli.mdx` 更新

### 追記内容

既存の `rune-cli.mdx`（`rune` パッケージマネージャの解説）に
`fav` コマンド本体のサブコマンドも追記する。

#### `fav deploy`

```bash
# Lambda にデプロイ
fav deploy --target lambda

# ECS/Fargate にデプロイ（v6.7.0 で完全対応）
fav deploy --target ecs --dry-run

# デプロイ設定は fav.toml [deploy] セクションで管理
```

```toml
[deploy]
target   = "lambda"
function = "my-pipeline"
region   = "ap-northeast-1"
```

#### `fav build --schema`

```bash
# スキーマから DDL を生成
fav build --schema schemas/orders.yaml --dialect postgres
fav build --schema schemas/orders.yaml --dialect sqlite

# 出力先指定
fav build --schema schemas/ --out migrations/001_init.sql
```

### 完了条件

- `rune-cli.mdx` に `fav deploy` と `fav build --schema` のセクションが追加されている

---

## 完了条件まとめ

1. `site/content/docs/language/pipeline.mdx` が存在し、stage/seq/|>/abstract seq を解説している
2. `site/content/docs/language/schema.mdx` が存在し、制約一覧がすべて記載されている
3. `site/content/docs/stdlib/infer.mdx` が存在し、CSV / DB / Proto の使い方が記載されている
4. `rune-cli.mdx` に `fav deploy` と `fav build --schema` のセクションが追加されている
5. すべてのコード例が有効な Favnir 構文になっている
