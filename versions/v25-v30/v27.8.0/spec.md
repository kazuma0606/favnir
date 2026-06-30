# v27.8.0 仕様書 — dbt 連携 Rune

## 概要

`runes/dbt/` を新規作成し、dbt（data build tool）の `manifest.json` を解析して
`Dbt.ref[T]` / `Dbt.source[T]` API からコンパイル済み SQL を型安全に実行できるようにする。

---

## 背景

ロードマップ v27.8「dbt 連携」より。Data Lakehouse フェーズの第 8 コンポーネント。
dbt はデータ変換の事実上の標準。`dbt ref()` で参照するモデルを Favnir パイプラインから
型安全に読み込めるようにする。

v27.8.0 は stub 実装。`Dbt.ref_raw` / `Dbt.source_raw` は引数検証のみを行い、
固定の空配列 `"[]"` を返す。実際の `manifest.json` 解析と SQL 実行は v28.x に延期する。

---

## Rune API

```favnir
import rune "dbt"

// dbt モデルの出力を Favnir パイプラインで後処理
stage LoadCustomerSummary: Unit -> Result<String, String> !Db = |_| {
  Dbt.ref("./dbt_project", "customer_summary")
}

// dbt の source 定義を参照
stage LoadRawEvents: Unit -> Result<String, String> !Db = |_| {
  Dbt.source("./dbt_project", "raw", "events")
}

seq DbtRefPipeline = LoadCustomerSummary
```

---

## 実装対象ファイル

### 1. `fav/src/backend/vm.rs` — 新 primitive 2 件追加

| primitive 名 | シグネチャ（引数） | 実装方針 |
|---|---|---|
| `Dbt.ref_raw` | `(config: String, model_name: String)` | stub: 引数検証、固定 `"[]"` 返却 |
| `Dbt.source_raw` | `(config: String, source_name: String, table_name: String)` | stub: 引数検証、固定 `"[]"` 返却 |

> **挿入位置**:
> - JSONL ブロック末尾（`"JSONL.append_raw"` の wasm32 アーム直後、**行 18013 付近**）に追加
> - Azure Blob Storage ブロック（`// ── Azure Blob Storage primitives (v14.5.0)`）の直前
> - 両方とも `#[cfg(not(target_arch = "wasm32"))]` / `#[cfg(target_arch = "wasm32")]` ガード付き

### 2. `runes/dbt/dbt.fav` — 新規作成（2 関数）

```favnir
// runes/dbt/dbt.fav
// v27.8.0 stub — manifest.json の実解析は v28.x 以降
// TODO(v28.x): DbtConfig { project_dir: String, profiles_dir: String, target: String } 型に昇格予定

public fn ref(config: String, model_name: String) -> Result<String, String> !Db {
    Dbt.ref_raw(config, model_name)
}

public fn source(config: String, source_name: String, table_name: String) -> Result<String, String> !Db {
    Dbt.source_raw(config, source_name, table_name)
}
```

**エフェクト**: `!Db`（dbt はデータベースにクエリを実行するため）

### 3. `examples/dbt_pipeline.fav` — 新規作成

```favnir
// examples/dbt_pipeline.fav
// dbt モデル参照パイプラインデモ（v27.8.0 stub）

stage LoadCustomerSummary: Unit -> Result<String, String> !Db = |_| {
    Dbt.ref("./dbt_project", "customer_summary")
}

stage LoadRawEvents: Unit -> Result<String, String> !Db = |_| {
    Dbt.source("./dbt_project", "raw", "events")
}

seq DbtRefPipeline = LoadCustomerSummary
```

### 4. `fav/tests/fixtures/dbt_manifest.json` — 新規作成

dbt manifest.json のモックフィクスチャ（v28.x でのモックテスト基盤）:

```json
{
  "nodes": {
    "model.my_project.customer_summary": {
      "compiled_sql": "SELECT * FROM customer_summary_raw",
      "name": "customer_summary",
      "resource_type": "model"
    }
  },
  "sources": {
    "source.my_project.raw.events": {
      "name": "events",
      "source_name": "raw",
      "schema": "raw"
    }
  }
}
```

### 5. `site/content/docs/runes/dbt.mdx` — 新規作成

dbt 連携 Rune の使用方法・API リファレンス・v28.x 予定事項を記載。

### 6. `fav/self/checker.fav` — `ns_to_effect` 更新（T10）

`Dbt` は新規 namespace。`!Db` エフェクトに対応するため `ns_to_effect` に追加する:

```favnir
if ns == "Dbt" { "Db" } else { "" }
```

`"JSONL" => "IO"` のブロックの直後（`else { "" }` の直前）に追加する。

> **実施タイミング**: T10 は T9（全テスト実行）より前に完了すること（v27.6.0 の教訓）。

---

## テスト

### driver.rs v278000_tests（8 件）

| テスト名 | 内容 |
|---|---|
| `dbt_rune_has_ref_fn` | `runes/dbt/dbt.fav` に `fn ref(` が含まれること |
| `dbt_rune_has_source_fn` | `runes/dbt/dbt.fav` に `fn source(` が含まれること |
| `dbt_rune_uses_db_effect` | `runes/dbt/dbt.fav` に `!Db` が含まれること |
| `vm_has_dbt_ref_raw` | `vm.rs` に `Dbt.ref_raw` が含まれること |
| `vm_has_dbt_source_raw` | `vm.rs` に `Dbt.source_raw` が含まれること |
| `dbt_example_has_pipeline` | `examples/dbt_pipeline.fav` に `DbtRefPipeline` が含まれること |
| `dbt_manifest_fixture_has_nodes` | `tests/fixtures/dbt_manifest.json` に `"nodes"` が含まれること |
| `changelog_has_v27_8_0` | `CHANGELOG.md` に `[v27.8.0]` が含まれること |

### `cargo test dbt` / `cargo test v278000` 期待値

- `cargo test v278000 --bin fav` — 8/8 PASS（全テスト名でフィルタ）
- `cargo test dbt --bin fav` — 7 件 PASS（`changelog_has_v27_8_0` はテスト名に `dbt` を含まないため除外）
- ロードマップ要件「3 件以上 PASS」を超過

> 詳細な `include_str!` パス対応表は tasks.md メモを参照。

---

## 完了条件

- [ ] `fav/Cargo.toml` が `version = "27.8.0"` であること
- [ ] `fav/src/backend/vm.rs` に `Dbt.ref_raw` が含まれること
- [ ] `fav/src/backend/vm.rs` に `Dbt.source_raw` が含まれること
- [ ] `runes/dbt/dbt.fav` に `public fn ref(` が含まれること
- [ ] `runes/dbt/dbt.fav` に `public fn source(` が含まれること
- [ ] `runes/dbt/dbt.fav` に `!Db` エフェクトが使われていること
- [ ] `examples/dbt_pipeline.fav` に `DbtRefPipeline` が含まれること
- [ ] `fav/tests/fixtures/dbt_manifest.json` に `"nodes"` が含まれること
- [ ] `site/content/docs/runes/dbt.mdx` が存在すること
- [ ] `fav/self/checker.fav` の `ns_to_effect` に `"Dbt"` が登録されていること
- [ ] `CHANGELOG.md` に `[v27.8.0]` エントリが存在すること
- [ ] `benchmarks/v27.8.0.json` が存在すること（test_count: 2204）
- [ ] `v278000_tests` 9 件すべて PASS
- [ ] 総テスト数 ≥ 2204 件

---

## 設計注記

### `ref` 関数名について
`ref` は Rust の予約語だが `dbt.fav` は Favnir ファイルのため現時点では問題ない。
将来 Favnir セルフホストパーサーが `ref` を予約語として扱うリスクがある場合は
`dbt_ref` / `dbt_source` にリネームする（v28.x で DbtConfig 昇格時に合わせて検討）。

### DbtConfig 型の延期について
ロードマップ v27.8 には `DbtConfig { project_dir: String, profiles_dir: String, target: String }` が
実装内容として記載されているが、v27.8.0 stub 段階では `config: String`（プロジェクトパス）で代替する。
**これはロードマップとの意図的な差異**。v28.x で `manifest.json` 実解析を実装する際に
DbtConfig 型に昇格し、API を更新する（STABILITY.md v1.x ポリシー適用外の stub 段階変更）。

### エフェクト設計（`!Db` の選択理由）
dbt は実際にデータベース（postgres / BigQuery / Snowflake 等）にクエリを実行するため `!Db` が適切。
`ClickHouse` / `Redshift` が `"Db"` に登録済みのパターンに準拠する。
`BigQuery` / `SQLite` の `ns_to_effect` 登録は各バージョン（v27.4 / v27.9）の責務（本バージョンでは対象外）。

---

## スコープ外（v28.x 以降）

- `manifest.json` の実解析（compiled SQL の抽出・実行）
- `DbtConfig { project_dir, profiles_dir, target }` 構造体型への昇格
- `Dbt.run_model(config, model_name)` — dbt CLI 呼び出し
- Snowflake / BigQuery / ClickHouse との dbt 統合テスト
- `dbt ref()` の型パラメータ `[T]` によるデシリアライズ
