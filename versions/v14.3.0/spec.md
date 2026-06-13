# v14.3.0 Spec — Azure lineage + fav explain 出力改善

Date: 2026-06-12

---

## 目的

CrossCloud パイプラインのリネージを **可視化** できるようにする。

v14.2.0 では `AzureCtx` / `AwsCtx` 型と `fav.toml [azure]` を実装した。
v14.3.0 では `fav explain --lineage` の出力を改善し、
AWS RDS から Azure Postgres へのデータフローを人間が読めるフォーマットで表示する。

また Azure Blob Storage（v14.5.0 実装予定）のリネージ基盤を前倒しで追加する。

---

## ユーザー体験（Before / After）

### Before（v14.2.0 まで）

```
$ fav explain --lineage migrate.fav

Lineage: migrate.fav

Sources:
  (none)

Sinks:
  (none)

Transformations:
  ExtractFromRds  [read]   DbRead    sources=[] sinks=[]
  TransformRows   [pure]   (pure)
  LoadToAzure     [write]  AzureDb   sources=[] sinks=[]

Pipelines:
  seq MigrationPipeline = ExtractFromRds |> TransformRows |> LoadToAzure
```

### After（v14.3.0）

```
$ fav explain --lineage migrate.fav

Lineage: migrate.fav

Sources:
  - AWS RDS (ExtractFromRds)

Sinks:
  - Azure Postgres (LoadToAzure)

CrossCloud Flow:
  [AWS RDS] → ExtractFromRds → TransformRows → LoadToAzure → [Azure Postgres]

Transformations:
  ExtractFromRds  [read]   !Postgres(read)    sources=[AWS RDS] sinks=[]
  TransformRows   [pure]   (pure)
  LoadToAzure     [write]  !AzureDb(write)    sources=[] sinks=[Azure Postgres]

Pipelines:
  seq MigrationPipeline = ExtractFromRds |> TransformRows |> LoadToAzure
    sources: AWS RDS
    sinks:   Azure Postgres
```

---

## スコープ

### In Scope

| 項目 | 内容 |
|---|---|
| `ast::Effect::AzureStorage` | Azure Blob 用エフェクト variant を ast.rs に追加（v14.5.0 の基盤） |
| lineage.rs AzureBlob 基盤 | `collect_azure_blob_call_kinds`、`azure_storage_effects`、`combined_effects` 拡張 |
| `render_lineage_text` CrossCloud 形式 | `!Postgres`/`!Db` + `!AzureDb` 両方存在する場合に CrossCloud Flow セクションを出力 |
| ソース/シンクの意味的ラベル | `!Postgres(read)` → sources=["AWS RDS"]、`!AzureDb(write)` → sinks=["Azure Postgres"] |
| `v143000_tests` | `azure_db_lineage_collected`、`crosscloud_lineage_format`（2 件） |
| バージョン `14.3.0` | `fav/Cargo.toml` バンプ |

### Out of Scope

- Azure Blob VM プリミティブ（v14.5.0）
- AWS Rune 正式パッケージング（v14.4.0）
- CrossCloud E2E デモ（v15.0.0）
- `self/cli.fav` の lineage 表示変更（改善は Rust 側で完結するため今回は対象外）

---

## CrossCloud Flow 検出ロジック

`render_lineage_text` で以下の条件を満たす場合に `CrossCloud Flow:` セクションを出力する:

```
条件: レポート内に以下が両方存在する
  1. `effects` に `!Postgres(read)` または `!Db(read)` または `!Snowflake` を含む変換
  2. `effects` に `!AzureDb(read)` または `!AzureDb(write)` を含む変換
```

フォーマット:
```
CrossCloud Flow:
  [AWS RDS] → StageA → StageB → ... → [Azure Postgres]
```

---

## AzureStorage 基盤（v14.5.0 先行）

`ast::Effect::AzureStorage` を追加し、lineage.rs で以下を定義する:
- `collect_azure_blob_call_kinds(expr)` → `(has_read, has_write)`
  - `AzureBlob.get_raw(...)` / `AzureBlob.list_raw(...)` → read
  - `AzureBlob.put_raw(...)` / `AzureBlob.delete_raw(...)` → write
- `azure_storage_effects(effects, blob_read, blob_write)` — `!AzureStorage(read/write)` 変換
- `combined_effects` を 6 引数に拡張（`az_blob_read`, `az_blob_write` 追加）

v14.5.0 で実際の VM プリミティブが追加されたとき、この基盤がそのまま使える。

---

## 完了条件

| 確認項目 | 目標 |
|---|---|
| `cargo test v143000` 全 2 件パス | ✅ |
| `cargo test` 全件パス（リグレッションなし） | ✅ |
| `CARGO_PKG_VERSION == "14.3.0"` | ✅ |
| `AzureStorage` が `ast::Effect` に存在する | ✅ |
| `collect_azure_blob_call_kinds` が lineage.rs に存在する | ✅ |
| `render_lineage_text` が CrossCloud Flow を出力する | ✅ |
