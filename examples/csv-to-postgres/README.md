# csv-to-postgres — CSV → Postgres ETL サンプル

Favnir で CSV ファイルを読み込み、バリデーションして Postgres に書き込む ETL パイプラインの 30 分クイックスタートです。

## 構成

```
csv-to-postgres/
├── fav.toml                 # プロジェクト設定
├── src/
│   ├── types.fav            # RawRow / ValidRow 型定義
│   ├── validators.fav       # 純粋バリデーション関数
│   ├── stages.fav           # LoadCsv / ValidateRows / WriteToDb + seq
│   └── main.fav             # エントリポイント
├── data/
│   └── sample.csv           # サンプルデータ（10 行、行 9 は意図的に無効）
└── tests/
    └── pipeline_test.fav    # DB 不要の純粋バリデーションテスト
```

## クイックスタート（約 30 分）

### 1. 前提条件

- Favnir (`fav`) インストール済み
- PostgreSQL サーバー起動済み

```bash
fav --version   # 30.5.0 以上
psql --version  # PostgreSQL 14 以上推奨
```

### 2. Postgres テーブル作成

```sql
CREATE TABLE records (
    id     INTEGER PRIMARY KEY,
    name   TEXT    NOT NULL,
    amount NUMERIC NOT NULL,
    date   DATE    NOT NULL
);
```

### 3. 環境変数設定

```bash
export DATABASE_URL="postgresql://user:password@localhost:5432/mydb"
```

`fav.toml` の `[postgres]` セクションにある `# url = "$DATABASE_URL"` のコメントを外すと、
環境変数から接続先が読み込まれます。

### 4. パイプライン実行

```bash
cd examples/csv-to-postgres
fav run src/main.fav
# => Inserted 9 rows successfully.
```

行 9（`bad_id,Ivan,...`）は `ValidateRows` ステージで自動スキップされるため、9 行が挿入されます。

### 5. 結果確認

```sql
SELECT COUNT(*) FROM records;   -- 9
SELECT * FROM records ORDER BY id;
```

## パイプライン設計

```
LoadCsv          ValidateRows         WriteToDb
String ──────> List<RawRow> ──────> List<ValidRow> ──────> Int
  !IO                                                !Postgres
```

- **LoadCsv**: CSV テキストをパースして `RawRow` のリストに変換
- **ValidateRows**: `id` が整数、`amount` が浮動小数点であることを検証。失敗行はスキップ
- **WriteToDb**: 検証済み行を `INSERT INTO records` で挿入。挿入件数を返す

## テスト（DB 不要）

```bash
fav check src/types.fav
fav check src/validators.fav
fav check --legacy-check src/stages.fav
fav check --legacy-check src/main.fav
```

純粋バリデーションテスト（DB 接続不要）は `pipeline_test.fav` に 3 件記載しています。
