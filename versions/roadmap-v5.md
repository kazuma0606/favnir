# Favnir ロードマップ v4.0.0 → v6.0.0

作成日: 2026-05-15
更新日: 2026-05-16

v4.0.0 完了後の進化の方針。

---

## 時代背景・反省

v3.x〜v4.0.0 の開発を通じて、以下の構造的な問題が明確になった。

**1. Rune が「VMビルトインのリネーム」になっている**
現状の全 rune は 20〜80 行の単一ファイルで、実質的にすべてが
`VM.primitive_raw(...)` を呼ぶだけの薄いラッパーである。
Favnir で書かれた実質的なロジックがほぼ存在しない。

**2. 単一ファイル制約により機能が限定される**
`import rune "db"` は `db.fav` 1枚しか読めないため、
`connection.fav` / `query.fav` / `migration.fav` のような
責務分割が不可能。rune は必然的に小さくなる。

**3. Rune 間の compose が存在しない**
`http` rune が `json` rune を内部利用する、といった組み合わせができない。
各 rune は孤立した島になっている。

**4. 「型安全なデータパイプライン専用言語」というピッチを rune が体現していない**
`db.query` は `List<Map<String,String>>` を返すだけで、
型安全性・スキーマ検証・エラーマッピングが存在しない。

---

## 方針

- **v4.x**: Rune Ecosystem の再設計と充実。Rune が Favnir の言語能力を体現する。
  VMビルトインを最小の primitive 層に整理し、高レベル API はすべて Favnir で書く。
  LSP・MCP・Notebook で開発体験を整備し、AWS SDK を仕込んでおく（LocalStack で開発）。
- **v5.0.0**: Favnir を AWS 上で本番稼働させ、リファレンスサイトを公開する。
  「Favnir で書かれたサービスが動いている」状態を作る最初の本番リリース。
- **v6.0.0**: セルフホスト Phase 2。型チェッカーを Favnir で実装する。
  「Rust = VM の筋肉、Favnir = コンパイラの知性」の完成形。

---

## フェーズ構成

| フェーズ | バージョン | テーマ |
|---------|-----------|--------|
| インフラ整備 | v4.1 | Rune マルチファイル対応 |
| データ処理 Rune | v4.2〜v4.4 | DB/gRPC/HTTP → DuckDB → Gen |
| セキュリティ・運用 | v4.5〜v4.7 | Auth → Log → 環境変数管理 |
| 開発体験 | v4.8〜v4.10 | LSP → MCP → Notebook |
| AWS 統合 | v4.11〜v4.12 | AWS SDK → fav deploy + Registry |
| 本番公開 | v5.0 | CI/CD + インフラ + リファレンスサイト |
| セルフホスト | v6.0 | 型チェッカーを Favnir で実装 |

---

## v4.1.0 — Rune マルチファイル対応

**テーマ**: rune をディレクトリ単位のモジュールとして扱えるようにする。
これ以降の全ての rune 充実化の前提となるインフラ整備。

### 追加するもの

**ディレクトリ rune のロード**
```
runes/
  db/
    db.fav          ← public API（エントリポイント）
    connection.fav  ← 内部モジュール
    query.fav
    migration.fav
```
`import rune "db"` が `db/` ディレクトリを検出した場合、
`db.fav` をエントリポイントとして読み込み、
`db/` 配下の全 `.fav` を内部スコープとして利用可能にする。

**rune 内部 `use`**
```favnir
// db.fav 内から
use connection.{ connect, close }
use query.{ run, paginate }
```

**rune 間 `use`**
```favnir
// grpc.fav 内から json rune を使う
import rune "json"
```

### 完了条件
- `import rune "db"` が `runes/db/` ディレクトリを認識して動作する
- rune 内部ファイル間の `use` が型チェック・実行ともに通る
- 既存単一ファイル rune との後方互換性が保たれる

### 注意: v4.1.0 は「インフラ整備」であり、rune の中身は変わらない

v4.1.0 でファイル分割の仕組みは整うが、**各 rune の実装コードは依然として薄いラッパー**のままである。
たとえば `db/connection.fav` の `connect` は `DB.connect_raw(...)` を呼ぶだけ、
`http/request.fav` の `get` は `Http.get_raw(...)` を呼ぶだけ、という状態は変わらない。

これは意図的な分離である。v4.1.0 の目標は「マルチファイル rune が動く基盤を作ること」であり、
その基盤の上で rune に実質的な Favnir ロジック（トランザクション管理・リトライ・バリデーション等）を
書き込むのが v4.2.0 以降の仕事となる。

```
v4.1.0: runes/db/connection.fav に connect(url) -> DB.connect_raw(url) を書く
v4.2.0: runes/db/transaction.fav に with_transaction(...) を Favnir で実装する
```

「コード量が少ない＝まだ何もしていない」ではなく、「インフラが整った＝次のフェーズに進める」が正しい理解。

---

## v4.1.5 — 型制約システム（`schemas/*.yaml` + コンパイル時検査）

**テーマ**: Favnir の型定義に制約を付与し、コンパイル時にリテラル値を検査する。
ORM は使わず SQL は別で書くが、**型システムが DB 制約の番人になる**設計。
`schemas/*.yaml` は DB Rune・DuckDB・Gen Rune・`fav build --schema` が共有する。

### 設計思想

```
型定義（type T = {...}）
    + 制約定義（schemas/order.yaml）
         ↓
fav check（コンパイル時）
    ├── リテラル値が制約に違反 → コンパイルエラー（E0xxx）
    └── T.validate 関数を自動生成

fav build --schema
    └── 型定義 + 制約 → SQL DDL を生成

db.query<T> / aws.s3.read_csv<T>
    └── 外部データを T.validate で自動検査
```

### `schemas/*.yaml` の形式

```yaml
# schemas/order.yaml
Order:
  id:
    constraints: [primary_key, positive]
  email:
    constraints: [unique]
    max_length: 255
    pattern: "^[a-z0-9._%+-]+@[a-z0-9.-]+\\.[a-z]{2,}$"
  amount:
    constraints: [positive]
    min: 0.01
  note:
    nullable: true          # Option<String> と対応
```

同じファイルを複数の用途で共有する:

| 読む主体 | 用途 |
|---------|------|
| `fav check` | コンパイル時リテラル検査 + `T.validate` 自動生成 |
| `fav build --schema` | SQL DDL 生成 |
| Gen Rune 2.0（v4.4.0） | 制約を満たす検証データ生成 |
| `db.query<T>` | クエリ結果の自動バリデーション |

### コンパイル時チェック

**リテラル値の検査（静的に分かるもの）**
```favnir
// E0xxx: id must be positive
let o = Order { id: -1, email: "user@example.com", amount: 100.0 }

// E0xxx: amount must be positive
let o = Order { id: 1, email: "user@example.com", amount: -5.0 }

// E0xxx: email does not match pattern
let o = Order { id: 1, email: "not-an-email", amount: 100.0 }
```

**外部データは実行時に自動検査**
```favnir
// db.query<Order> が内部で Order.validate を自動呼び出し
bind orders <- db.query<Order>(conn, "SELECT * FROM orders")
// → Result<List<Order>, DbError | ValidationError>

// CSV / S3 から読んだデータも同様
bind rows <- aws.s3.read_csv<Order>("bucket", "orders.csv")
// → Result<List<Order>, AwsError | ValidationError>
```

**自動生成される `T.validate`**
```favnir
// schemas/order.yaml から fav check が自動生成
// Order.validate : Map<String,String> -> Result<Order, List<ValidationError>>
bind result <- Order.validate(raw_map)
match result {
    Ok(order) => process(order)
    Err(errs) => log.error("LE100", Map.set((), "errors", errs))
}
```

### `fav build --schema` — SQL DDL 生成

```
fav build --schema src/types.fav --out migrations/001_create_tables.sql
```

```sql
-- 生成される DDL（schemas/order.yaml の制約を反映）
CREATE TABLE orders (
    id      INTEGER PRIMARY KEY AUTOINCREMENT,
    email   VARCHAR(255) UNIQUE NOT NULL
            CHECK (email ~ '^[a-z0-9._%+-]+@[a-z0-9.-]+\.[a-z]{2,}$'),
    amount  REAL NOT NULL CHECK (amount >= 0.01),
    note    TEXT
);
```

生成された SQL を `migrations/` に置いて `fav db migrate` で適用する流れ。
DDL の最終編集は人間が行う — 生成物は出発点。

### 追加する VM primitive・コンパイラ変更

- `schemas/*.yaml` のロード: `driver.rs` でプロジェクト起動時に読み込み
- `Checker` に制約情報を渡してリテラルチェックを追加
- `compiler.rs` で `T.validate` 関数を型定義から自動生成
- `fav build --schema` コマンドを `driver.rs` に追加

### 完了条件
- `schemas/*.yaml` がプロジェクト起動時に自動ロードされる
- リテラル値の制約違反がコンパイルエラーになる（テスト 10 件以上）
- `T.validate` が自動生成され、外部データの検査に使える
- `fav build --schema` が SQL DDL を生成できる
- 既存テストがすべて通る（破壊的変更なし）

---

## v4.2.0 — DB / HTTP / gRPC Rune 2.0

**テーマ**: 既存の rune 3本をまとめて充実化し、Favnir らしい高レベル API を実装する。
マルチファイル対応（v4.1.0）を活かして責務を分割し、Favnir で書かれたロジックを増やす。

### DB Rune 2.0

**ファイル構成**
```
runes/db/
  db.fav           ← public API
  connection.fav   ← connect / close / with_conn
  query.fav        ← query / query_one / paginate / batch_insert
  transaction.fav  ← with_transaction / savepoint
  migration.fav    ← migrate / rollback / status
```

**高レベル API（Favnir で実装）**
```favnir
// with_transaction: コールバック内で失敗したら自動ロールバック
public fn with_transaction<T>(
    conn: DbHandle,
    f: DbHandle -> Result<T, DbError>
) -> Result<T, DbError> !Db {
    bind tx <- DB.begin_tx(conn)
    match f(tx) {
        Ok(v)  => bind _ <- DB.commit_tx(tx)   Result.ok(v)
        Err(e) => bind _ <- DB.rollback_tx(tx) Result.err(e)
    }
}

// paginate: LIMIT/OFFSET を自動付加
public fn paginate(conn: DbHandle, sql: String, page: Int, size: Int)
    -> Result<List<Map<String, String>>, DbError> !Db {
    bind offset <- Result.ok(page * size)
    DB.query_raw(conn, $"{sql} LIMIT {size} OFFSET {offset}")
}
```

**マイグレーション**
```
fav db migrate           # 未適用マイグレーションを実行
fav db migrate --status  # 適用状態を一覧表示
fav db migrate --rollback
```

### HTTP Rune 2.0

**ファイル構成**
```
runes/http/
  http.fav       ← public API
  client.fav     ← get / post / put / delete / patch
  retry.fav      ← with_retry / exponential_backoff
  auth.fav       ← bearer / basic / api_key
  response.fav   ← json_body / text_body / status_ok
```

**高レベル API（Favnir で実装）**
```favnir
public fn with_retry<T>(
    max_attempts: Int,
    f: Unit -> Result<T, HttpError>
) -> Result<T, HttpError> !Network {
    fn attempt(n: Int) -> Result<T, HttpError> !Network {
        match f() {
            Ok(v)  => Result.ok(v)
            Err(e) => if n <= 1 then Result.err(e) else attempt(n - 1)
        }
    }
    attempt(max_attempts)
}
```

### gRPC Rune 2.0

**現状の問題（v4.0.0〜v4.1.0 時点）**: `proto_bytes_to_string_map` が `field1`/`field2` の
位置キーしか返さない。ハンドラは実際のフィールド名ではなく位置番号でアクセスしなければならない。

```favnir
// v4.0.0〜v4.1.0 のハンドラ — 位置キー "field1"/"field2" しか使えない
public fn handle_echo(req: Map<String, String>) -> Map<String, String> {
    bind msg <- Option.unwrap_or(Map.get(req, "field1"), "(empty)")
    Map.set((), "field1", msg)
}
```

これは gRPC が「動いていない」のではなく、**protobuf のフィールド名情報が失われている**ことが原因である。
HTTP/2 トランスポート・フレーミング・ステータスコードは v4.0.0 時点で本物の gRPC として動作している。
不足しているのは「`proto_bytes_to_string_map` がデコード時にフィールド名を復元する」実装だけである。

**解決方針**: encode/decode 時に `type_metas` のフィールド名情報を使い、
宣言順のフィールド名を保持する。

```favnir
// v4.2.0 以降のハンドラ — フィールド名が保持される
public fn handle_get_user(req: Map<String, String>) -> Map<String, String> {
    bind id <- Option.unwrap_or(Map.get(req, "id"), "0")
    ...
}
```

**ファイル構成**
```
runes/grpc/
  grpc.fav       ← public API
  server.fav     ← serve / serve_stream / handler helpers
  client.fav     ← call / call_stream / with_deadline
  codec.fav      ← encode / decode（型名情報を利用）
  error.fav      ← ok / err / status_to_error
```

### 完了条件
- `db.with_transaction` / `db.paginate` / `fav db migrate` が動く（統合テスト 20 件以上）
- `http.with_retry(3, || http.get(url))` が動く（テスト 15 件以上）
- gRPC ハンドラが受け取る Map のキーがフィールド名（`id`, `name` 等）になる
- 各 rune のコードが Favnir で書かれたロジックを体現している

---

## v4.3.0 — DuckDB Rune（組み込み OLAP + Parquet / S3 統合）

**テーマ**: データエンジニアの日常作業（Parquet 分析・集計・変換）を
SQL で直接書けるようにする。サーバー不要の組み込み型 OLAP エンジン。
Parquet rune（既存）と AWS SDK（v4.11.0）の橋渡しになる。

### なぜ DuckDB か

- **サーバー不要** — SQLite と同じ組み込み型。`duckdb.open(":memory:")` で即使える
- **Parquet / CSV / JSON を直接 SQL でクエリ** — ファイルをテーブルとして扱える
- **S3 統合がネイティブ** — `read_parquet('s3://bucket/*.parquet')` がそのまま動く
- **既存 db rune との使い分け** — OLTP（トランザクション）は db rune、OLAP（分析）は duckdb rune

### ファイル構成

```
runes/duckdb/
  duckdb.fav    ← public API
  query.fav     ← query / query_one / execute / explain
  io.fav        ← read_parquet / read_csv / write_parquet / write_csv
  s3.fav        ← s3_scan / s3_query
```

### 使用イメージ

```favnir
import rune "duckdb"

type OrderSummary = { customer: String total: Float count: Int }

public fn main() -> Unit !Io !Db {
    bind conn   <- duckdb.open(":memory:")
    bind result <- duckdb.query<OrderSummary>(conn,
        "SELECT customer, SUM(amount) AS total, COUNT(*) AS count
         FROM 'data/orders/*.parquet'
         GROUP BY customer ORDER BY total DESC LIMIT 10")
    IO.println(result)
}
```

```favnir
// CSV → Parquet 変換（ETL の基本操作）
public fn convert() -> Unit !Io !Db {
    bind conn <- duckdb.open(":memory:")
    duckdb.execute(conn,
        "COPY (SELECT * FROM read_csv_auto('input.csv'))
         TO 'output.parquet' (FORMAT PARQUET)")
}
```

### VM primitives（最小セット）

- `DuckDb.open_raw(path) -> Result<DbHandle, DbError>` — 既存 `DbHandle` 型を再利用
- `DuckDb.query_raw(conn, sql) -> Result<List<Map<String,String>>, DbError>`
- `DuckDb.execute_raw(conn, sql) -> Result<Unit, DbError>`

**依存クレート追加**
```toml
duckdb = { version = "0.10", features = ["bundled"] }
```

### 完了条件
- `duckdb.open(":memory:")` + `duckdb.query<T>` がローカル Parquet に対して動く
- CSV → Parquet 変換が動く
- 統合テスト 10 件以上（サーバー不要、CI で完結）
- Parquet rune との組み合わせサンプルが examples に追加される

---

## v4.4.0 — Gen Rune 2.0（検証データ強化）

**テーマ**: 現状の gen rune はランダム値を生成するだけで実用に足りない。
フィールド名ヒント・制約定義・大量データ出力・DuckDB 統合で
データパイプラインのテストを現実的なレベルに引き上げる。

### 現状の問題

```favnir
gen.one<Order>(42)
// → { id: 7823, customer: "xkqpz", amount: 0.3821 }
//                          ↑ランダム文字列  ↑非現実的な値
```

### フィールド名ヒントによるリアルデータ生成

```favnir
type Order = { id: Int customer_name: String email: String amount: Float created_at: String }

gen.one<Order>(42)
// → { id: 1, customer_name: "田中 太郎", email: "tanaka@example.com",
//     amount: 12800.0, created_at: "2026-03-15T10:23:44Z" }
```

| フィールド名パターン | 生成される値 |
|---------------------|-------------|
| `*_name` / `name` | 人名（日/英） |
| `email` / `*_email` | `xxx@example.com` 形式 |
| `*_at` / `*_date` | ISO 8601 日時 |
| `id` / `*_id` | 連番または UUID |
| `price` / `amount` / `*_fee` | 正の実数（現実的な範囲） |
| `age` | 0〜120 の整数 |

### YAML による制約・分布指定

```yaml
# gen/order.yaml
amount:
  distribution: pareto   # 80/20 則
  min: 100
  max: 1000000
created_at:
  range: last_90_days
customer_name:
  locale: ja
```

```favnir
gen.one_with<Order>("order", 42)  // gen/order.yaml の制約を適用
```

### 大量データの Parquet / CSV 直接出力

```favnir
// 100万行をメモリに乗せずにストリーム書き込み
gen.to_parquet<Order>("test_data/orders.parquet", 1_000_000, 42)
gen.to_csv<Order>("test_data/orders.csv", 50_000, 42)
```

### DuckDB 統合（v4.3.0 との連携）

```favnir
// 生成 → DuckDB に直接ロード → SQL でクエリ・検証
bind conn <- duckdb.open(":memory:")
bind _    <- gen.load_into<Order>(conn, "orders", 10_000, 42)
bind result <- duckdb.query<Summary>(conn,
    "SELECT customer_name, SUM(amount) FROM orders GROUP BY customer_name")
```

### エッジケース・境界値生成（プロパティベーステスト向け）

```favnir
gen.edge_cases<Order>()
// → [ { id: 0, amount: -1.0, email: "" },       ← 空・負値
//     { id: Int.MAX, amount: Float.MAX, ... },   ← 最大値
//     { customer_name: None, ... } ]             ← None
```

### 完了条件
- フィールド名ヒントによるリアルデータ生成が動く
- `gen/order.yaml` の制約が反映される
- `gen.to_parquet` で 100 万行ストリーム書き込みが動く
- `gen.load_into` で DuckDB と統合できる
- `gen.edge_cases<T>` が境界値を網羅する
- 統合テスト 15 件以上

---

## v4.5.0 — Auth Rune（JWT / OAuth2 / RBAC）

**テーマ**: 認証・認可を Favnir の型システムに統合する。
ローカルでは Rust の crypto primitive で厳密に検証し、
AWS 本番では ALB + Cognito に委譲する。切り替えは設定1行で済む。

### アーキテクチャ

```
【ローカル開発】
クライアント → Favnir サービス
              auth.verify_jwt(token, secret)
                ↓ VM primitive
              Rust: Crypto.jwt_verify_raw（署名検証）

【AWS 本番】
クライアント → ALB + Cognito（署名検証はここで完結）
              → Favnir サービス
                auth.from_cognito_header(req)
                  ↓ VM primitive
                Rust: Crypto.jwt_decode_raw（検証済みなのでパースのみ）
```

**`fav.toml` でモードを切り替え**
```toml
[auth]
mode = "jwt"       # ローカル: Rust で署名検証
# mode = "cognito" # AWS 本番: ALB ヘッダーを信頼
```

### VM primitives

- `Crypto.jwt_verify_raw(token, secret, alg) -> Result<Map<String,String>, String>`
- `Crypto.jwt_decode_raw(token) -> Result<Map<String,String>, String>`
- `Crypto.hmac_sha256_raw(key, data) -> String`

### ファイル構成

```
runes/auth/
  auth.fav        ← public API
  jwt.fav         ← verify_jwt / from_cognito_header / decode_claims
  rbac.fav        ← require_role / require_any_role / has_permission
  oauth2.fav      ← authorization_url / exchange_code / refresh_token
  apikey.fav      ← verify_api_key / generate_api_key
```

### 使用イメージ

```favnir
import rune "auth"

type Claims = { sub: String role: String exp: Int }

public fn handle_orders(req: HttpRequest) -> Result<HttpResponse, HttpError> !Network !Auth {
    bind claims <- auth.verify_jwt<Claims>(req, Env.get_or("JWT_SECRET", ""))
    bind _      <- auth.require_role(claims, "data_engineer")
    ...
}
```

**`!Auth` エフェクト**: `!Auth` を宣言しない関数から `auth.*` を呼ぶと型エラー。

### 完了条件
- `auth.verify_jwt` が HS256 / RS256 トークンを正しく検証できる
- `auth.from_cognito_header` が ALB の `X-Amzn-Oidc-Data` ヘッダーをパースできる
- `fav.toml [auth] mode` で jwt / cognito を切り替えられる
- 統合テスト 15 件以上

---

## v4.6.0 — Log Rune（構造化ログ + メトリクス）

**テーマ**: すべての `!*` エフェクトにログを統合し、CloudWatch / Grafana で即座に使える
構造化ログ基盤を作る。`!Log` という独立エフェクトは持たない —
エフェクトを持つ関数はすべて自動的にランタイムエラーをログに出力する。

### 設計思想

```
純粋関数（エフェクトなし）   → ログなし（副作用ゼロを保証）
!Io / !Db / !Network / ...  → VM primitive 層がエラー時に自動で LE コードを出力
アプリケーションイベント     → log.info / log.success / log.warn / log.error で明示的に出力
```

### ログコード体系

| プレフィクス | 区分 | 例 |
|------------|------|-----|
| `I` | INFO | `I001` |
| `S` | SUCCESS | `S001` |
| `W` | WARN | `W001` |
| `LE` | LOG ERROR | `LE001` |

コンパイラエラーコード（`E0001`〜、4桁）と区別するため3桁を使う。
アプリ定義コードは `I100`〜 / `S100`〜 / `W100`〜 / `LE100`〜 を使うことで組み込みと衝突しない。

**組み込みコード（YAML不要）**
```
I000  Application started        LE010 DB error
I001  Application stopped        LE020 Network error
S000  Operation completed        LE030 Auth error
W001  Retry attempted            LE040 RPC error
W002  Slow operation             LE050 AWS error
```

### YAML 拡張

```yaml
# logs/success.yaml        # logs/error.yaml
S100:                       LE100:
  message: "Pipeline done"    message: "API unreachable"
  tags: [pipeline]            severity: critical
```

### 出力フォーマット

**`text`（ローカル）**
```
[2026-05-16 10:30:00] SUCCESS S100  Pipeline completed  inserted=1500
```

**`json`（CloudWatch / Grafana Loki）**
```json
{"ts":"2026-05-16T10:30:01Z","level":"SUCCESS","code":"S100","msg":"Pipeline completed","ctx":{"inserted":1500}}
```

### メトリクス拡張

```yaml
# metrics/pipeline.yaml
processed_rows:
  unit: Count
pipeline_duration:
  unit: Milliseconds
```

```favnir
log.metric("processed_rows", List.len(rows))
log.metric("pipeline_duration", elapsed_ms)
```

`format = "json"` のとき **CloudWatch EMF** で出力。ログストリームからメトリクスが自動抽出される。

### `fav.toml` 設定

```toml
[log]
level   = "info"        # debug | info | warn | error
format  = "json"        # json | text
output  = "stdout"      # stdout | stderr
service = "my-pipeline"
```

### VM primitives

- `Log.emit_raw(level, code, message, context_json) -> Unit`
- `Log.metric_raw(name, value, unit) -> Unit`

### 完了条件
- `log.info / success / warn / error` が動く
- `logs/*.yaml` のカスタムコードが使える
- `log.metric` が CloudWatch EMF 形式で出力される
- 各 VM primitive がエラー時に `LE*` コードを自動出力する
- 統合テスト 15 件以上

---

## v4.7.0 — 環境変数管理（`.env.*` + Secrets Manager）

**テーマ**: ローカル開発から本番 AWS まで、環境変数の切り替えを一貫した仕組みで管理する。

### `.env.*` ファイル切り替え

```
fav run src/main.fav               → .env を読む（デフォルト）
fav run src/main.fav --env local   → .env.local を読む
fav run src/main.fav --env prod    → .env.prod を読む
```

```
project-root/
  .env           ← 共通デフォルト（git 管理してよい値のみ）
  .env.local     ← ローカル開発用（git 管理外）
  .env.prod      ← 本番値のうちシークレット以外（git 管理外）
```

### Secrets Manager フォールバック

```toml
# fav.toml
[secrets]
provider = "aws"    # aws | local | none
prefix   = "prod/"
region   = "ap-northeast-1"
```

`provider = "aws"` のとき `Env.get("DATABASE_URL")` は以下の順で解決:

```
1. 実行時環境変数
2. .env.* ファイル
3. AWS Secrets Manager の "prod/DATABASE_URL"
```

```favnir
// ローカル (.env.local): DATABASE_URL=sqlite://./dev.db
// 本番 Fargate (Secrets Manager): postgresql://...
bind conn <- db.connect(Env.get_or("DATABASE_URL", "sqlite://./dev.db"))
```

### 完了条件
- `fav run --env local` が `.env.local` を読み込む
- `fav.toml [secrets] provider = "aws"` で Secrets Manager フォールバックが動く
- LocalStack の Secrets Manager でテストが完結する

---

## v4.8.0 — LSP（microserver 方式）

**テーマ**: 開発体験を一変させる言語サーバーを実装する。
エディタ補完・型表示・リアルタイムエラーを手に入れる。

### 起動方式

```
fav lsp                    # stdio モード（VS Code デフォルト）
fav lsp --daemon           # バックグラウンドで常駐（デフォルトポート: 2087）
fav lsp --daemon --port N  # ポート指定
fav lsp --status           # 常駐プロセスの状態確認
fav lsp --stop             # 常駐プロセスを停止
```

microserver 方式を採用する主な理由:
- VS Code + Favnir Notebook（v4.10.0）が同じ LSP インスタンスを共有できる
- 起動コストが1回で済む（補完が速い）

### 実装するもの（優先度順）

```
1. 診断（Diagnostics）   ← fav check の結果をリアルタイムで表示
2. ホバー（Hover）       ← 型情報・effect 情報の表示
3. 補完（Completion）    ← 関数名・フィールド名・rune 名
4. 定義ジャンプ（Go to Definition）
5. リネーム（Rename）
```

### 完了条件
- `fav lsp` が VS Code から起動され、型エラーがリアルタイムで表示される
- ホバーで型情報・effect 情報が表示される
- `fav lsp --daemon` で常駐プロセスが起動し、複数クライアントから接続できる

---

## v4.9.0 — MCP サーバー

**テーマ**: AI アシスタント（Claude 等）が Favnir プロジェクトを理解・操作できるようにする。
LSP（v4.8.0）の最低限（diagnostics + hover）が動いた後に実装する。

### LSP との役割分担

```
VS Code / エディタ     ─── LSP ───▶  fav lsp --daemon  ← 型情報・補完・診断
Claude / AI アシスタント ─── MCP ───▶  fav mcp          ← コード実行・型チェック・docs
```

### 起動方式

```
fav mcp              # stdio モード（Claude Desktop / Claude Code デフォルト）
fav mcp --http       # HTTP モード
fav mcp --port N     # ポート指定
```

### MCP Tools

```
fav_check(source)      → 型チェック結果を構造化して返す
fav_run(source, env?)  → 実行結果（!Io のみ許可、!Aws / !Db 等はサンドボックス拒否）
fav_infer(path, fmt)   → CSV / SQLite から型定義を生成
fav_rune_docs(name)    → rune の API 一覧・エフェクト・説明
fav_explain_error(code)→ エラーコードの詳細説明・修正例
```

### MCP Resources

```
favnir://project/types      ← プロジェクトの全型定義
favnir://project/functions  ← public fn 一覧 + シグネチャ
favnir://runes/{name}       ← rune ソース + docs
favnir://errors             ← エラーカタログ全文
```

### 完了条件
- `fav mcp` が Claude Desktop / Claude Code から接続できる
- `fav_check` / `fav_run` が動く
- `fav lsp --daemon` との共存が確認できる

---

## v4.10.0 — Favnir Notebook

**テーマ**: Favnir ネイティブのノートブック環境を実装する。
LSP（v4.8.0）と MCP（v4.9.0）を統合してセル補完・AI支援を実現する。

### 概要

```
fav notebook              # ローカルサーバーを起動してブラウザで開く
fav notebook src/demo.fnb # 指定ファイルを開く
fav notebook --port 8888  # ポート指定
```

ファイル形式は `.fnb`（Favnir Notebook）。内部構造は `.ipynb` 互換の JSON。

### セル構成

```
[markdown] セルでドキュメントを記述
[code]     セルで Favnir コードを実行（前セルの環境を引き継ぐ）
[output]   セルで実行結果を表示（テキスト / テーブル / グラフ）
```

### 内部構成

```
fav notebook
  ├── HTTP サーバー（フロントエンド配信 + REST API）
  ├── WebSocket（セル実行・出力ストリーミング）
  └── LSP クライアント（fav lsp --daemon に接続、または内部起動）
```

### AWS Glue / Databricks との関係

`.fnb` ファイルは `.ipynb` 互換なので、将来的に Glue Studio / Databricks への
エクスポートが検討できる。v4.10.0 の時点ではローカル動作に集中する。

### 完了条件
- `fav notebook` でブラウザが開き、コードセルを実行できる
- LSP 統合により補完・型表示が動く
- `.fnb` ファイルが `.ipynb` として Jupyter で開ける

---

## v4.11.0 — AWS SDK Rune（LocalStack 開発）

**テーマ**: Favnir プログラムが AWS リソースをネイティブに扱えるようにする。
すべて LocalStack（Docker）で開発し、本番切り替えは設定のみ。

### 開発戦略

```bash
docker run -d -p 4566:4566 localstack/localstack

# fav run --aws-local で AWS_ENDPOINT_URL=http://localhost:4566 を注入
```

`AWS_ENDPOINT_URL` 環境変数を差し替えるだけで本番に繋がる設計を維持する。

### ファイル構成

```
runes/aws/
  aws.fav         ← public API
  s3.fav          ← get_object / put_object / list_objects / read_csv / write_parquet
  sqs.fav         ← send / receive / delete / poll
  dynamodb.fav    ← get_item / put_item / query / scan / batch_write
  secrets.fav     ← get_secret（Secrets Manager）
  auth.fav        ← 認証共通処理
```

### 組み込みイベント型

```favnir
type S3Event    = { bucket: String key: String size: Int event_type: String region: String }
type SqsMessage = { message_id: String body: String receipt_handle: String attributes: Map<String, String> }
```

### 使用イメージ

```favnir
import rune "aws"

type Order = { id: Int customer: String amount: Float }

// S3 から CSV を読んで Parquet に変換して書き戻す
public fn main() -> Unit !Io !Aws {
    bind orders <- aws.s3.read_csv<Order>("my-bucket", "input/orders.csv")
    aws.s3.write_parquet("my-bucket", "output/orders.parquet", orders)
}

// S3 イベント → SQS → Favnir のイベント駆動パターン
public fn pipeline() -> Unit !Io !Aws {
    aws.sqs.poll(Env.get_or("QUEUE_URL", ""), |msg| {
        bind event  <- json.parse<S3Event>(msg.body)
        bind orders <- aws.s3.read_csv<Order>(event.bucket, event.key)
        process(orders)
    })
}
```

### VM primitives

- `Aws.s3_get_raw / s3_put_raw / s3_list_raw`
- `Aws.sqs_send_raw / sqs_receive_raw / sqs_delete_raw`
- `Aws.dynamodb_get_raw / dynamodb_put_raw`
- `Aws.secrets_get_raw`

**`!Aws` effect**: 単一エフェクト。IAM は Terraform で管理するため細分化しない。

**依存クレート追加**
```toml
aws-sdk-s3             = "1"
aws-sdk-sqs            = "1"
aws-sdk-dynamodb       = "1"
aws-sdk-secretsmanager = "1"
aws-config             = "1"
```

### 完了条件
- LocalStack 上で `aws.s3.read_csv<T>` / `write_parquet` が動く
- LocalStack 上で `aws.sqs.poll` が動く
- LocalStack 上で `aws.dynamodb.get_item` / `put_item` が動く
- 統合テスト 15 件以上（LocalStack 使用、本物の AWS 不要）
- Favnir Notebook（v4.10.0）上でデモが動く

---

## v4.12.0 — `fav deploy`（ECS/Fargate + Lambda）+ Rune Registry + `fav run --cron`

**テーマ**: Favnir エコシステムのインフラ整備。
AWS へのデプロイワークフロー・外部 rune の配布基盤・スケジュール実行を整える。

### `fav deploy`

**`fav.toml` デプロイ設定**
```toml
[deploy]
target  = "ecs"
region  = "ap-northeast-1"
ecr     = "123456789.dkr.ecr.ap-northeast-1.amazonaws.com/my-pipeline"
cluster = "my-cluster"
service = "my-pipeline-service"
cpu     = 256
memory  = 512

[deploy.env]
DATABASE_URL = { secret = "prod/db-url" }
BATCH_SIZE   = "1000"
```

**CLI**
```
fav deploy                     # ビルド → ECR push → ECS デプロイ
fav deploy --target lambda     # Lambda 関数として zip デプロイ
fav deploy --dry-run           # 差分確認のみ
fav deploy --env staging       # 環境別設定を使用
fav logs                       # CloudWatch Logs をストリーミング表示
```

**内部フロー（ECS）**
```
1. cargo build --release
2. Dockerfile を自動生成（Alpine + fav バイナリ + ソース）
3. docker build → ECR push
4. ECS タスク定義を更新 → rolling update → ヘルスチェック待機
```

### `fav run --cron`（AWS 無依存のスケジュール実行）

```
fav run src/main.fav --cron "0 2 * * *"    # 毎日 02:00 UTC
fav run src/main.fav --cron "*/15 * * * *" # 15 分おき
```

- `fav` プロセス自身がスケジューラとして動作（EventBridge 不要）
- Fargate 上で動かすだけで定期バッチになる
- 実行のたびに Log Rune が `I000 started` / `S000 completed` を出力

```toml
[run]
cron = "0 2 * * *"   # fav run だけで --cron が自動適用
```

### Rune Registry

```
fav rune install         # fav.toml の runes を取得・キャッシュ
fav rune publish         # rune をレジストリに公開
fav rune search <query>  # レジストリを検索
```

**アーキテクチャ**
```
fav rune install/publish
  → API Gateway → ECS（registry server）
     ├── S3（.fav パッケージ tarball）
     └── DynamoDB（rune メタデータ）
```

**`fav.toml` rune 依存定義**
```toml
[runes]
db   = { version = "2.0.0" }
http = { version = "2.0.0" }
slack = { git = "https://github.com/example/favnir-slack" }
```

### 完了条件
- `fav deploy` が ECS / Lambda デプロイを完走する
- `fav run --cron` が期待通りにスケジュール実行できる
- `fav rune install` が git URL と registry URL の両方から動く

---

## v5.0.0 — AWS 本番稼働 + CI/CD + リファレンスサイト

**テーマ**: v4.x で作ったすべてのピースを使って Favnir 自身を AWS に乗せる。
「Favnir で書かれたサービスが AWS で動いている」状態を作る最初の本番リリース。

### 全体構成

```
GitHub
  └── CI/CD（GitHub Actions）
        ├── fav check / fav test
        ├── docker build → ECR push
        └── fav deploy → ECS rolling update

AWS（Terraform 管理）
  ├── ECS/Fargate — Rune Registry API サーバー
  ├── S3 + DynamoDB — Rune パッケージ + メタデータ
  ├── Secrets Manager — 認証情報
  ├── CloudWatch — Log Rune の出力がそのまま監視に
  └── CloudFront + S3 — リファレンスサイト

リファレンスサイト（Next.js + @favnir/wasm）
  ├── 言語仕様 docs
  ├── エラーカタログ
  ├── Rune カタログ（Registry API から動的取得）
  └── Live Playground（WASM でブラウザ内実行）
```

**Terraform の詳細設計は v5.0.0 着手タイミングで別途策定する。**

### Phase A: CI/CD

```yaml
# .github/workflows/deploy.yml
- run: fav check src/
- run: fav test
- run: docker build → ECR push
- run: fav deploy --env prod
```

### Phase B: `@favnir/wasm` — ブラウザ内 Favnir ランタイム

**既存の `fav build --wasm`（Favnir プログラム → WASM）とは別物。**
Favnir の**ランタイム自体**を WASM にコンパイルし、JS から呼び出せるようにする。

```rust
// crates/favnir-wasm/src/lib.rs
#[wasm_bindgen]
pub fn fav_check(source: &str) -> JsValue { ... }

#[wasm_bindgen]
pub fn fav_run(source: &str) -> JsValue {
    // !Io のみ許可、!Db / !Aws 等はサンドボックスで拒否
}
```

```toml
[dependencies]
wasm-bindgen = "0.2"

[lib]
crate-type = ["cdylib"]
```

### Phase C: リファレンスサイト（Next.js）

```
site/
  app/
    page.tsx                 ← ランディングページ
    docs/[...slug]/page.tsx  ← 言語仕様（MDX）
    errors/[code]/page.tsx   ← エラーカタログ
    runes/page.tsx           ← Rune カタログ
    playground/page.tsx      ← Live Playground
```

```typescript
// playground/page.tsx
import init, { fav_check, fav_run } from '@favnir/wasm';

const run = async (code: string) => {
    await init();
    const result = fav_run(code);
    setOutput(result.stdout);
};
```

サーバー不要・ネットワークラウンドトリップなし。ブラウザ内で完結。

### Phase D: Dogfooding

- Rune Registry サーバー自体を Favnir HTTP サービスとして実装
- CI スクリプトの一部を Favnir で記述

### 完了条件
- GitHub Actions で `fav check → test → deploy` が動く
- Rune Registry が AWS 上で稼働し `fav rune install db` が動く
- `@favnir/wasm` が npm パッケージとしてビルドできる
- リファレンスサイトが CloudFront で公開されている
- Playground でブラウザ内 Favnir 実行が動く

---

## v6.0.0 — セルフホスト Phase 2（型チェッカーを Favnir で実装）

**テーマ**: 「Rust = VM の筋肉、Favnir = コンパイラの知性」の完成。
v5.0.0 で本番稼働・リファレンス公開が済んだ後の内部的な進化。

### 目標

```
現状: Favnir ソース → (Rust) Lexer → (Rust) Parser → (Rust) Type Checker → (Rust) VM
v6.0: Favnir ソース → (Rust) Lexer → (Favnir) Parser → (Favnir) Type Checker → (Rust) VM
```

### マイルストーン

1. **Phase A**: 純粋関数の型チェックのみ Favnir で実装
2. **Phase B**: エフェクト型チェックを Favnir で実装
3. **Phase C**: Rust 型チェッカーと並走テスト（全テストで一致確認）
4. **Phase D**: Rust 型チェッカーを削除、完全移行

### 完了条件
- `fav check` のコア処理が Favnir で動作する
- 全既存テスト（800件以上）が Favnir 型チェッカー経由で通過する
- Playground（WASM）でも Favnir 型チェッカーが動く

---

## 全体スケジュール概観

| バージョン | テーマ | フェーズ |
|-----------|--------|---------|
| v4.1.0 | Rune マルチファイル対応 | インフラ整備 |
| v4.1.5 | **型制約システム**（schemas/*.yaml + コンパイル時検査） | インフラ整備 |
| v4.2.0 | DB / HTTP / gRPC Rune 2.0 | データ処理 Rune |
| v4.3.0 | DuckDB Rune（組み込み OLAP・Parquet/S3） | データ処理 Rune |
| v4.4.0 | Gen Rune 2.0（検証データ強化） | データ処理 Rune |
| v4.5.0 | Auth Rune（JWT / OAuth2 / RBAC） | セキュリティ・運用 |
| v4.6.0 | Log Rune（構造化ログ・メトリクス・EMF） | セキュリティ・運用 |
| v4.7.0 | 環境変数管理（`.env.*` + Secrets Manager） | セキュリティ・運用 |
| v4.8.0 | LSP（microserver 方式） | 開発体験 |
| v4.9.0 | MCP サーバー（Claude / AI 統合） | 開発体験 |
| v4.10.0 | Favnir Notebook（LSP + MCP 統合） | 開発体験 |
| v4.11.0 | AWS SDK Rune（LocalStack 開発） | AWS 統合 |
| v4.12.0 | `fav deploy` + Rune Registry + `fav run --cron` | AWS 統合 |
| **v5.0.0** | **AWS 本番稼働 + CI/CD + リファレンスサイト** | **本番公開** |
| v6.0.0 | セルフホスト Phase 2（型チェッカー） | セルフホスト |

---

## 設計原則（v4.x 全体）

**VM primitive は最小に保つ**
新機能を追加する際、まず「Favnir で書けないか？」を問う。
`DB.query_raw` / `Http.get_raw` 等の I/O 境界層だけを Rust に残し、
それ以上のロジックはすべて Favnir で実装する。

**Rune は言語の実証である**
各 rune の実装コードは、Favnir の表現力・型安全性・エフェクト管理の
生きたデモンストレーションでなければならない。
「動く」だけでなく「Favnir らしく書かれている」ことが品質基準。

**LSP・MCP・Notebook は一体で設計する**
`fav lsp --daemon` は `fav notebook` の内部コンポーネントとしても動作する。
LSP サーバーを先に作ることで、Notebook は補完・型表示を自動的に得る。
MCP は同じ型チェックエンジンを共有し、AI アシスタントに同じ情報を提供する。

**AWS は LocalStack ファーストで開発する**
`AWS_ENDPOINT_URL` を差し替えるだけで本番に繋がる設計を維持する。
コストゼロで開発し、デプロイ直前にのみ本物の AWS に触れる。
IAM・ネットワーク等のインフラは Terraform で管理し、Favnir は関与しない。

**後方互換性を守る**
v4.x における rune API の変更は、既存ユーザーへの影響を最小化する。
破壊的変更は deprecation period を設けて v5.0.0 に集約する。
