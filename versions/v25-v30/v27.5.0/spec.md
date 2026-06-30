# v27.5.0 仕様書 — redshift Rune 追加

## 概要

AWS Redshift Rune を新規追加する。
`import rune "redshift"` → `Redshift.*` 名前空間に 5 関数を追加（connect / query / execute / copy_from_s3 / unload_to_s3）。

---

## 背景

ロードマップ v27.5「redshift Rune 追加」より。Data Lakehouse フェーズの第 5 コンポーネント。
Redshift は postgres 互換 API を持つため `!Db` エフェクト（ClickHouse / BigQuery と統一）を採用。
S3 COPY コマンド（`COPY FROM S3`）・UNLOAD（`UNLOAD TO S3`）による高速バルク処理が特徴。

ローカルテストは postgres 互換 API を使用。実 Redshift 接続は v28.x 以降。

---

## 実装する関数

| 関数 | シグネチャ | 内容 |
|---|---|---|
| `Redshift.connect` | `(config: String) -> Result<String, String> !Db` | 接続確立（postgres ドライバ利用）。接続ハンドルを返す |
| `Redshift.query` | `(conn: String, sql: String) -> Result<String, String> !Db` | 型付きクエリ（SELECT）。JSON 配列文字列を返す |
| `Redshift.execute` | `(conn: String, sql: String, params: String) -> Result<Int, String> !Db` | DDL / DML 実行。影響行数を返す |
| `Redshift.copy_from_s3` | `(conn: String, table: String, s3_uri: String, opts: String) -> Result<Unit, String> !Db` | S3 COPY（CSV / Parquet / JSON。大量バルクロード向き） |
| `Redshift.unload_to_s3` | `(conn: String, query: String, s3_uri: String, opts: String) -> Result<Unit, String> !Db` | UNLOAD（SELECT 結果を S3 に出力） |

> **エフェクト**: `!Db`（ClickHouse / BigQuery と統一。postgres 互換 API のため）

---

## VM Primitive（vm.rs に追加）

| primitive 名 | シグネチャ（引数） | 実装方針 |
|---|---|---|
| `Redshift.connect_raw` | `(config: String)` | stub: 引数検証のみ、`ok_vm(VMValue::Str("redshift-stub-conn".into()))` 返却 |
| `Redshift.query_raw` | `(conn: String, sql: String)` | stub: 引数検証、`ok_vm(VMValue::Str("[]".into()))` 返却 |
| `Redshift.execute_raw` | `(conn: String, sql: String, params: String)` | stub: 引数検証、`ok_vm(VMValue::Int(0))` 返却（影響行数 0） |
| `Redshift.copy_from_s3_raw` | `(conn: String, table: String, s3_uri: String, opts: String)` | stub: 引数検証、`ok_vm(VMValue::Unit)` 返却 |
| `Redshift.unload_to_s3_raw` | `(conn: String, query: String, s3_uri: String, opts: String)` | stub: 引数検証、`ok_vm(VMValue::Unit)` 返却 |

> **挿入位置**: ClickHouse ブロック末尾（`"ClickHouse.async_insert_raw" => Ok(err_vm(...))` の wasm32 アーム直後、行 17878 付近）・Azure Blob Storage ブロック（`// ── Azure Blob Storage primitives (v14.5.0)`）直前。
>
> **wasm32 ガード**: ClickHouse / BigQuery と同パターン（`#[cfg(not(target_arch = "wasm32"))]` / `#[cfg(target_arch = "wasm32")]` 両アーム）で追加する。wasm32 アームは `err_vm("Redshift not supported on wasm32")` を返す。

---

## runes/redshift/redshift.fav（新規作成）

```favnir
// runes/redshift/redshift.fav — Redshift Rune (v27.5.0)
// AWS Redshift（postgres 互換 API）への接続・クエリ・S3 COPY / UNLOAD。
// postgres ドライバを使った実接続は v28.x 以降。現バージョンは引数検証 stub。
// TODO(v28.x): postgres クレートを使った実接続に移行予定。
//              _config（DSN / 接続文字列）を postgres クライアントの初期化に渡す。
public fn connect(config: String) -> Result<String, String> !Db {
    Redshift.connect_raw(config)
}
public fn query(conn: String, sql: String) -> Result<String, String> !Db {
    Redshift.query_raw(conn, sql)
}
public fn execute(conn: String, sql: String, params: String) -> Result<Int, String> !Db {
    Redshift.execute_raw(conn, sql, params)
}
public fn copy_from_s3(conn: String, table: String, s3_uri: String, opts: String) -> Result<Unit, String> !Db {
    Redshift.copy_from_s3_raw(conn, table, s3_uri, opts)
}
public fn unload_to_s3(conn: String, query: String, s3_uri: String, opts: String) -> Result<Unit, String> !Db {
    Redshift.unload_to_s3_raw(conn, query, s3_uri, opts)
}
```

---

## examples/redshift_analytics.fav

```favnir
// examples/redshift_analytics.fav — Redshift Analytics デモ (v27.5.0)
import rune "redshift"

stage LoadFromS3: Unit -> Result<Unit, String> !Db = |_| {
    bind conn <- Redshift.connect("host:localhost,port:5439,dbname:dev")
    Redshift.copy_from_s3(conn, "events", "s3://my-bucket/events/*.csv", "FORMAT CSV")
}

// seq pipeline は前ステージの成功値（Unit）を次ステージの引数として渡す
stage QuerySummary: Unit -> Result<String, String> !Db = |_| {
    bind conn <- Redshift.connect("host:localhost,port:5439,dbname:dev")
    Redshift.query(conn, "SELECT count(*) FROM events")
}

stage UnloadToS3: String -> Result<Unit, String> !Db = |query| {
    bind conn <- Redshift.connect("host:localhost,port:5439,dbname:dev")
    Redshift.unload_to_s3(conn, query, "s3://my-bucket/output/", "FORMAT PARQUET")
}

seq RedshiftAnalyticsPipeline = LoadFromS3 |> QuerySummary |> UnloadToS3
```

---

## テスト

### driver.rs v275000_tests（12 件）

| テスト名 | 内容 |
|---|---|
| `redshift_rune_has_connect_fn` | `redshift.fav` に `"fn connect("` が含まれること |
| `redshift_rune_has_query_fn` | `redshift.fav` に `"fn query("` が含まれること |
| `redshift_rune_has_execute_fn` | `redshift.fav` に `"fn execute("` が含まれること |
| `redshift_rune_has_copy_from_s3_fn` | `redshift.fav` に `"fn copy_from_s3("` が含まれること |
| `redshift_rune_has_unload_to_s3_fn` | `redshift.fav` に `"fn unload_to_s3("` が含まれること |
| `redshift_rune_vm_has_connect_raw` | `vm.rs` に `"Redshift.connect_raw"` が含まれること |
| `redshift_rune_vm_has_query_raw` | `vm.rs` に `"Redshift.query_raw"` が含まれること |
| `redshift_rune_vm_has_execute_raw` | `vm.rs` に `"Redshift.execute_raw"` が含まれること |
| `redshift_rune_vm_has_copy_from_s3_raw` | `vm.rs` に `"Redshift.copy_from_s3_raw"` が含まれること |
| `redshift_rune_vm_has_unload_to_s3_raw` | `vm.rs` に `"Redshift.unload_to_s3_raw"` が含まれること |
| `redshift_example_has_pipeline` | `examples/redshift_analytics.fav` に `"RedshiftAnalyticsPipeline"` が含まれること |
| `changelog_has_v27_5_0` | `CHANGELOG.md` に `"[v27.5.0]"` が含まれること |

### `cargo test redshift` 期待値

- `v275000_tests::redshift_rune_has_*` 5 件
- `v275000_tests::redshift_rune_vm_has_*` 5 件
- `v275000_tests::redshift_example_has_pipeline` 1 件
- 合計 11 件（`changelog_has_v27_5_0` は `redshift` を含まないため除外）（ロードマップ要件「3 件以上」超過）

---

## 完了条件

- [ ] `fav/Cargo.toml` が `version = "27.5.0"` であること
- [ ] `runes/redshift/redshift.fav` に `public fn connect(` が含まれること
- [ ] `runes/redshift/redshift.fav` に `public fn query(` が含まれること
- [ ] `runes/redshift/redshift.fav` に `public fn execute(` が含まれること
- [ ] `runes/redshift/redshift.fav` に `public fn copy_from_s3(` が含まれること
- [ ] `runes/redshift/redshift.fav` に `public fn unload_to_s3(` が含まれること
- [ ] `fav/src/backend/vm.rs` に `Redshift.connect_raw` が含まれること
- [ ] `fav/src/backend/vm.rs` に `Redshift.query_raw` が含まれること
- [ ] `fav/src/backend/vm.rs` に `Redshift.execute_raw` が含まれること
- [ ] `fav/src/backend/vm.rs` に `Redshift.copy_from_s3_raw` が含まれること
- [ ] `fav/src/backend/vm.rs` に `Redshift.unload_to_s3_raw` が含まれること
- [ ] `examples/redshift_analytics.fav` に `RedshiftAnalyticsPipeline` が含まれること
- [ ] `site/content/docs/runes/redshift.mdx` が存在すること
- [ ] `CHANGELOG.md` に `[v27.5.0]` エントリが存在すること
- [ ] `benchmarks/v27.5.0.json` が存在すること（test_count: 2176）
- [ ] `v275000_tests` 12 件すべて PASS
- [ ] `cargo test redshift --bin fav` で 11 件 PASS（`changelog_has_v27_5_0` は `redshift` を含まないため除外）
- [ ] 総テスト数 ≥ 2176 件

---

## スコープ外（v28.x 以降）

- postgres クレートを使った実 Redshift 接続（実データ連携）
- `Redshift.query[T]` ジェネリック API（ロードマップ記載）
  - **延期根拠**: v27.5.0 は stub 実装段階であり、VM レベルでジェネリック型引数の評価ができないため v28.0 以降に延期する
- S3 COPY の IAM ロール認証（`iam_role` オプション）
- COPY の format 検証（CSV / PARQUET / JSON / ORC 等）
- Redshift Spectrum（S3 上のデータを直接クエリ）
- 接続プール管理
- ローカル postgres での API 互換テスト（実通信なし）
