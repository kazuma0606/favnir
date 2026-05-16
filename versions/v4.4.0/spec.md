# Favnir v4.4.0 仕様書 — Gen Rune 2.0（検証データ強化）

作成日: 2026-05-17

## 概要

現状の gen rune はランダム値を生成するだけで、実際のデータパイプライン開発に使えるレベルではない。
v4.4.0 では以下の 4 点を強化し、「テストデータ生成」から「本番相当データ生成」へ引き上げる。

1. **フィールド名ヒントによるリアルデータ生成** — `email`, `*_name`, `*_at` 等のパターンで意味ある値を生成
2. **`gen/*.yaml` による制約・分布指定** — 数値範囲・ロケール・分布関数をファイルで管理
3. **大量データの Parquet / CSV ストリーム書き込み** — メモリに乗せずに 100 万行を出力
4. **DuckDB 統合** — 生成データを直接 DuckDB テーブルに INSERT
5. **エッジケース・境界値生成** — プロパティベーステスト向けの境界値リスト

---

## 1. 現状の問題

```favnir
gen.one("Order")
// → { id: "7823", customer_name: "xkqpz", amount: "0.3821" }
//                              ↑ランダム文字列   ↑非現実的な値
```

- フィールド名を無視してランダム文字列・数値を生成するだけ
- 生成データが「本番データ」に見えないためパイプラインの品質検証に使えない
- 大量行の生成は `gen.list(type_name, n, seed)` で全行をメモリに保持してから出力（100万行で OOM）
- DuckDB と連携する手段がない（v4.3.0 で DuckDB が入ったが gen との橋渡しがない）

---

## 2. 設計方針

### 2.1 既存 API との後方互換

| 既存 API | v4.4.0 での扱い |
|---------|----------------|
| `gen.one(type_name)` | そのまま維持（ランダム生成） |
| `gen.list(type_name, n, seed)` | そのまま維持 |
| `gen.simulate(type_name, n, noise, seed)` | そのまま維持 |
| `gen.profile(type_name, data)` | そのまま維持 |
| `gen.int_val / float_val / bool_val / string_val / choice` | そのまま維持 |

既存テストはすべて pass のまま。新機能は新 API として追加する。

### 2.2 rune ファイル構成

```
runes/gen/
  gen.fav           ← public API（barrel file）— 既存 + 新 use を追加
  primitives.fav    ← 既存（変更なし）
  structured.fav    ← 既存（変更なし）
  hint.fav          ← 新規: フィールド名ヒント生成
  output.fav        ← 新規: Parquet / CSV ストリーム出力
  integration.fav   ← 新規: DuckDB 統合
  edge.fav          ← 新規: 境界値・エッジケース生成
  gen.test.fav      ← 既存テスト維持 + 新テスト追加
```

### 2.3 新規 VM プリミティブ（最小セット）

```
Gen.hint_one_raw(type_name)                         -> Map<String, String>
Gen.hint_list_raw(type_name, n)                     -> List<Map<String, String>>
Gen.to_parquet_raw(type_name, path, n, seed)        -> Result<Int, String>
Gen.to_csv_raw(type_name, path, n, seed)            -> Result<Int, String>
Gen.load_into_raw(type_name, handle_id, table, n, seed) -> Result<Int, String>
Gen.edge_cases_raw(type_name)                       -> List<Map<String, String>>
Gen.set_yaml_config_raw(type_name, yaml_path)       -> Result<Unit, String>
```

高レベルのロジック（seed セット・DuckDB ハンドル解決・エラーマッピング）は Favnir の rune 層で実装する。

---

## 3. フィールド名ヒントシステム（`Gen.hint_one_raw`）

### 3.1 フィールド名パターンマッチ

| パターン | 生成される値の例 |
|---------|----------------|
| `id` / `*_id` | 連番整数（"1", "2", ...） |
| `uuid` / `*_uuid` | UUID v4 形式の文字列 |
| `name` / `*_name` / `full_name` | 人名（"田中 太郎", "Taro Tanaka"）|
| `first_name` / `given_name` | 名のみ（"太郎"） |
| `last_name` / `family_name` | 姓のみ（"田中"） |
| `email` / `*_email` | `xxx@example.com` 形式 |
| `phone` / `*_phone` | 電話番号形式 |
| `*_at` / `created_at` / `updated_at` | ISO 8601 日時（"2026-03-15T10:23:44Z"） |
| `*_date` / `birth_date` | 日付（"2026-03-15"） |
| `price` / `amount` / `*_fee` / `*_price` | 正の実数（"1280.00"〜"98000.00"） |
| `age` | 20〜80 の整数文字列 |
| `count` / `*_count` | 正の整数（"1"〜"999"） |
| `url` / `*_url` | `https://example.com/xxx` 形式 |
| `zip` / `postal_code` | 郵便番号形式（"100-0001"） |
| `address` | 住所形式 |
| `description` / `body` / `content` | 意味のある文章（固定文例から選択） |
| `status` | "active" / "inactive" / "pending" から選択 |
| `flag` / `is_*` / `has_*` | "true" または "false" |
| その他 | 既存ランダム文字列生成（`gen.one` と同じ） |

### 3.2 ロケール（デフォルト: `ja`）

デフォルトで日本語ロケール（人名は日本人名、住所は日本形式）を使用する。
`gen/*.yaml` の `locale: en` で英語に切り替えられる。

### 3.3 連番 ID の管理

`hint_one_raw` を連続して呼ぶと `id` / `*_id` フィールドが自動インクリメントされる。
`Random.seed(n)` でリセットすると 1 から再開する。
スレッドローカルなカウンタで管理する（複数型が独立した連番を持つ）。

---

## 4. YAML 制約・分布設定（`gen/*.yaml`）

### 4.1 設定ファイルの場所と形式

プロジェクト内の `gen/<type_name_lower>.yaml` を参照する。
`schemas/*.yaml`（型制約、v4.1.5）とは別ファイルで gen 固有の設定を持つ。

```yaml
# gen/order.yaml — Order 型の生成設定
amount:
  distribution: pareto    # uniform（デフォルト）/ normal / pareto
  min: 100
  max: 1000000

created_at:
  range: last_90_days     # last_N_days / last_N_years / fixed_range

customer_name:
  locale: ja              # ja / en（フィールド単位で上書き可能）

status:
  values: ["active", "completed", "cancelled"]  # choice リストを上書き
  weights: [0.7, 0.2, 0.1]                       # 重み付き確率
```

### 4.2 サポートする設定項目

| キー | 値 | 説明 |
|-----|-----|------|
| `distribution` | `uniform` / `normal` / `pareto` | 数値の分布関数 |
| `min` / `max` | 数値 | 生成値の範囲 |
| `range` | `last_N_days` / `last_N_years` | 日時の範囲 |
| `locale` | `ja` / `en` | ロケール |
| `values` | 文字列リスト | choice の選択肢を上書き |
| `weights` | 数値リスト | 重み付き確率 |
| `nullable` | `true` / `false` | "null" を生成する確率を与える |
| `null_rate` | 0.0〜1.0 | nullable のとき `"null"` を返す確率 |

### 4.3 YAML 設定の読み込み

```favnir
// gen/hint.fav 内から使う
public fn one_from_yaml(type_name: String, yaml_name: String, seed: Int) -> Map<String, String> !Random {
    Random.seed(seed)
    match Gen.set_yaml_config_raw(type_name, yaml_name) {
        Ok(_)  => Gen.hint_one_raw(type_name)
        Err(e) => Gen.one_raw(type_name)    // フォールバック
    }
}
```

`Gen.set_yaml_config_raw` は `gen/<yaml_name>.yaml` を読み込み、スレッドローカルな設定マップに格納する。
以降の `hint_one_raw` / `hint_list_raw` 呼び出しでその設定が適用される。

---

## 5. 大量データのストリーム出力

### 5.1 `Gen.to_parquet_raw(type_name, path, n, seed)`

- `hint_one_raw` を内部で N 回呼び、バッチ（1000行単位）で Parquet ファイルに書き込む
- 既存の `arrow = "52"` + `parquet = "52"` クレートを使用（Cargo.toml 追加不要）
- スキーマは `type_metas` から動的に生成（全フィールドを `Utf8` 型で扱う）
- 成功時は書き込んだ行数 N を `Ok(N)` で返す
- 失敗時は `Err("WRITE_ERROR: ...")`

### 5.2 `Gen.to_csv_raw(type_name, path, n, seed)`

- `hint_one_raw` を内部で N 回呼び、バッチで CSV に書き込む
- 既存の `csv = "1"` クレートを使用（Cargo.toml 追加不要）
- 1 行目にヘッダー行（フィールド名）を出力
- 成功時は N を `Ok(N)` で返す

### 5.3 Favnir rune API（`output.fav`）

```favnir
// to_parquet: N行をメモリに乗せずに Parquet に書き込む
public fn to_parquet(type_name: String, path: String, n: Int, seed: Int) -> Result<Int, String> !Io {
    Gen.to_parquet_raw(type_name, path, n, seed)
}

// to_csv: N行を CSV に書き込む
public fn to_csv(type_name: String, path: String, n: Int, seed: Int) -> Result<Int, String> !Io {
    Gen.to_csv_raw(type_name, path, n, seed)
}
```

エフェクトは `!Io`（ファイル書き込み）のみ。`!Random` は VM primitive 内部で処理する。

---

## 6. DuckDB 統合

### 6.1 `Gen.load_into_raw(type_name, handle_id, table_name, n, seed)`

- DuckDB の `DUCKDB_CONNS` から接続を取り出し、テーブルが存在しなければ自動 CREATE TABLE
- `hint_one_raw` を 1000 行単位でバッチ生成し、パラメータ付き INSERT を実行
- 成功時は INSERT した合計行数 N を `Ok(N)` で返す
- 失敗時は `Err("LOAD_ERROR: ...")`

テーブルのスキーマは `type_metas` のフィールドから `TEXT NOT NULL` で自動生成する（DDL を Favnir 側に渡さない）。

### 6.2 Favnir rune API（`integration.fav`）

```favnir
// load_into: 生成データを DuckDB テーブルに直接 INSERT
// conn は duckdb rune の open() で得た DbHandle
public fn load_into(type_name: String, conn: DbHandle, table_name: String, n: Int, seed: Int) -> Result<Int, DbError> !Db {
    match Gen.load_into_raw(type_name, conn, table_name, n, seed) {
        Ok(rows) => Result.ok(rows)
        Err(e)   => Result.err(DbError { code: "LOAD_ERROR" message: e })
    }
}
```

---

## 7. エッジケース・境界値生成

### 7.1 `Gen.edge_cases_raw(type_name)`

型定義のフィールド型を見て、境界値の組み合わせを複数行で生成して返す。

| フィールド型 | 生成される境界値（文字列として） |
|------------|-------------------------------|
| `Int` | "0", "-1", "1", "9223372036854775807", "-9223372036854775808" |
| `Float` | "0.0", "-1.0", "1.0", "3.4028235e38", "-3.4028235e38" |
| `String` | `""`, 1文字, 255文字, 1000文字（全て "a" で埋め） |
| `Bool` | "true", "false" |

各境界値バリアントを 1 行として生成し、全フィールドが同時に境界値になるセットを返す。
返す行数は `境界値の最大バリアント数`（フィールド型の中で最も種類が多い型の値数）。

### 7.2 Favnir rune API（`edge.fav`）

```favnir
// edge_cases: 境界値のリストを生成（プロパティベーステスト向け）
// エフェクトなし（純粋に型メタ情報から静的に決定）
public fn edge_cases(type_name: String) -> List<Map<String, String>> {
    Gen.edge_cases_raw(type_name)
}

// first_edge: 最初の境界値のみ返す（単体テスト用）
public fn first_edge(type_name: String) -> Option<Map<String, String>> {
    List.first(Gen.edge_cases_raw(type_name))
}
```

---

## 8. `gen.fav` barrel ファイル更新

```favnir
// runes/gen/gen.fav — Gen Rune public API (v4.4.0)
use primitives.{ int_val, float_val, bool_val, string_val, choice }
use structured.{ one, list, simulate, profile }
use hint.{ one_with_hints, list_with_hints, one_from_yaml }
use output.{ to_parquet, to_csv }
use integration.{ load_into }
use edge.{ edge_cases, first_edge }
```

---

## 9. checker.rs への追加

### 9.1 新規ビルトイン呼び出しシグネチャ

| メソッド | 戻り値型 |
|---------|---------|
| `Gen.hint_one_raw` | `Map<String, String>` |
| `Gen.hint_list_raw` | `List<Map<String, String>>` |
| `Gen.to_parquet_raw` | `Result<Int, String>` |
| `Gen.to_csv_raw` | `Result<Int, String>` |
| `Gen.load_into_raw` | `Result<Int, String>` |
| `Gen.edge_cases_raw` | `List<Map<String, String>>` |
| `Gen.set_yaml_config_raw` | `Result<Unit, String>` |

`Gen.hint_one_raw` / `hint_list_raw` / `edge_cases_raw` — `!Random` エフェクト要求
`Gen.to_parquet_raw` / `to_csv_raw` — `!Io` エフェクト要求
`Gen.load_into_raw` — `!Db` エフェクト要求
`Gen.set_yaml_config_raw` — `!Io` エフェクト要求

---

## 10. 使用例

### 10.1 フィールド名ヒント生成

```favnir
import rune "gen"

type Order = { id: Int customer_name: String email: String amount: Float created_at: String }

public fn main() -> Unit !Io !Random {
    bind row <- gen.one_with_hints("Order")
    IO.println($"id={Map.get_or(row, \"id\", \"?\")}")
    IO.println($"name={Map.get_or(row, \"customer_name\", \"?\")}")
    IO.println($"email={Map.get_or(row, \"email\", \"?\")}")
    // → id=1  customer_name=田中 太郎  email=tanaka@example.com
}
```

### 10.2 YAML 制約付き生成

```favnir
import rune "gen"

type Transaction = { id: Int amount: Float created_at: String status: String }

public fn main() -> Unit !Io !Random {
    // gen/transaction.yaml: amount.distribution=pareto, status.values=[active, completed, cancelled]
    bind row <- gen.one_from_yaml("Transaction", "transaction", 42)
    IO.println($"amount={Map.get_or(row, \"amount\", \"?\")}")
    IO.println($"status={Map.get_or(row, \"status\", \"?\")}")
}
```

### 10.3 大量データ生成 → DuckDB 集計

```favnir
import rune "gen"
import rune "duckdb"

type Sale = { id: Int product_name: String amount: Float created_at: String }

public fn main() -> Unit !Io !Db !Random {
    bind conn_result <- duckdb.open(":memory:")
    match conn_result {
        Ok(conn) => {
            // 10万行を DuckDB の sales テーブルに直接 INSERT
            bind load_result <- gen.load_into("Sale", conn, "sales", 100000, 42)
            match load_result {
                Ok(n) => {
                    IO.println($"Loaded {n} rows")
                    // SQL で集計
                    bind result <- duckdb.query(conn,
                        "SELECT product_name, SUM(CAST(amount AS DOUBLE)) AS total FROM sales GROUP BY product_name ORDER BY total DESC LIMIT 5")
                    match result {
                        Ok(rows) => IO.println($"Top 5 products: {List.length(rows)} rows")
                        Err(e)   => IO.println($"Query error: {e.message}")
                    }
                }
                Err(e) => IO.println($"Load error: {e.message}")
            }
            duckdb.close(conn)
        }
        Err(e) => IO.println($"Open error: {e.message}")
    }
}
```

### 10.4 Parquet ストリーム書き込み → DuckDB 読み込み

```favnir
import rune "gen"
import rune "duckdb"

type Event = { id: Int user_id: Int event_type: String created_at: String }

public fn main() -> Unit !Io !Db !Random {
    // 100万行を Parquet に書き込み（メモリ効率的）
    bind write_result <- gen.to_parquet("Event", "tmp/events.parquet", 1000000, 42)
    match write_result {
        Ok(n) => {
            IO.println($"Written {n} rows to Parquet")
            // DuckDB で集計
            bind conn_result <- duckdb.open(":memory:")
            match conn_result {
                Ok(conn) => {
                    bind result <- duckdb.query(conn,
                        "SELECT event_type, COUNT(*) AS cnt FROM read_parquet('tmp/events.parquet') GROUP BY event_type")
                    match result {
                        Ok(rows) => IO.println($"Event types: {List.length(rows)}")
                        Err(e)   => IO.println($"Error: {e.message}")
                    }
                    duckdb.close(conn)
                }
                Err(e) => IO.println($"Open error: {e.message}")
            }
        }
        Err(e) => IO.println($"Write error: {e}")
    }
}
```

### 10.5 エッジケース生成

```favnir
import rune "gen"

type Product = { id: Int name: String price: Float }

public fn main() -> Unit !Io {
    bind cases <- gen.edge_cases("Product")
    IO.println($"Edge case count: {List.length(cases)}")
    // 各境界値ケースでバリデーション等を実行
}
```

---

## 11. テスト方針

### 11.1 `vm_stdlib_tests.rs`（Rust レベル・4 件追加）

- `gen_hint_one_raw_email_field` — `email` フィールドが `@example.com` を含む値を返す
- `gen_hint_one_raw_id_sequential` — 連続 `hint_one_raw` 呼び出しで `id` が "1", "2" と増える
- `gen_to_csv_raw_writes_file` — `to_csv_raw` でファイルが生成され行数が正しい
- `gen_edge_cases_raw_returns_multiple_rows` — `edge_cases_raw` が 2 件以上返す

### 11.2 `runes/gen/gen.test.fav`（既存 11 件維持 + 新規 10 件追加）

- `test_one_with_hints_email_contains_at` — email フィールドが "@" を含む
- `test_one_with_hints_id_is_numeric` — id フィールドが数値文字列
- `test_one_with_hints_name_not_empty` — name フィールドが空でない
- `test_list_with_hints_count` — N 件返る
- `test_list_with_hints_ids_sequential` — id が "1", "2", ... になる
- `test_to_csv_writes_correct_row_count` — tmp CSV に N 行 + ヘッダー
- `test_to_parquet_file_exists` — Parquet ファイルが生成される
- `test_to_parquet_readable_by_duckdb` — 生成 Parquet を duckdb.read_parquet で読める（※ duckdb rune 必要）
- `test_edge_cases_not_empty` — edge_cases が空でない
- `test_edge_cases_has_expected_count` — 型のフィールド型に応じた件数

### 11.3 `driver.rs` 統合テスト（4 件追加）

- `gen_rune_test_file_passes` — `runes/gen/gen.test.fav` 全 pass（既存が更新されたもの）
- `gen_hint_in_favnir_source` — inline ソースでヒント生成確認
- `gen_to_csv_in_favnir_source` — inline ソースで CSV 書き込み確認
- `gen_load_into_duckdb_in_source` — inline ソースで DuckDB ロード確認

---

## 12. 完了条件

- `gen.one_with_hints("Order")` で `email` フィールドが `@` を含む値を返す
- `gen.list_with_hints("Order", 5, 42)` の `id` フィールドが "1"〜"5" の連番
- `gen.to_parquet("Event", "tmp/test.parquet", 1000, 42)` でファイルが生成される
- `gen.to_csv("Event", "tmp/test.csv", 1000, 42)` でファイルが生成される
- `gen.load_into("Sale", conn, "sales", 10000, 42)` が DuckDB テーブルに INSERT できる
- `gen.edge_cases("Order")` が型のフィールド数に応じた境界値リストを返す
- 既存 826 件のテストがすべて通る（破壊的変更なし）
- 新規テスト 18 件以上が pass
- `examples/gen2_demo/` が動く
