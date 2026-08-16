# v72.6.0 Spec — `fav init` テンプレートギャラリー拡充

Date: 2026-08-12
Status: 計画中

---

## 背景

`fav init`（`cmd_new` / `try_cmd_new`）はすでに driver.rs に実装されており、
script / pipeline / lib / postgres-etl / etl-csv-to-db / api-gateway / lambda-scheduled /
distributed-etl / data-contract / multi-source / rag-pipeline / ci-workflow の 12 テンプレートを持つ。

v73 以降のユースケース（AI ETL / streaming / enterprise / data-quality / distributed）に
対応したテンプレートを 5 件追加し、`TEMPLATE_GALLERY` を拡充する。

---

## 目標

1. `fav init --template ai-etl` — LLM 抽出 → VectorDB パイプライン雛形
2. `fav init --template streaming` — Kafka + ML スコアリング パイプライン雛形
3. `fav init --template enterprise` — マルチテナント + 監査ログ パイプライン雛形
4. `fav init --template data-quality` — データ品質検証パイプライン雛形
5. `fav init --template distributed` — マルチノード par パイプライン雛形

---

## API / 構文例

```bash
$ fav init --template ai-etl my-ai-project
Created project: my-ai-project/
  main.fav
  fav.toml
  README.md

$ fav init --template data-quality my-dq-project
Created project: my-dq-project/
  main.fav
  fav.toml
  README.md
```

---

## 実装詳細

### `driver.rs` — TEMPLATE_GALLERY 拡充

既存の `TEMPLATE_GALLERY` に 5 エントリを追加:

```rust
TemplateEntry { name: "ai-etl",       description: "LLM抽出 → VectorDB パイプライン" },
TemplateEntry { name: "streaming",    description: "Kafka + MLスコアリング パイプライン" },
TemplateEntry { name: "enterprise",   description: "マルチテナント + 監査ログ パイプライン" },
TemplateEntry { name: "data-quality", description: "データ品質検証パイプライン" },
TemplateEntry { name: "distributed",  description: "マルチノード par パイプライン" },
```

### `driver.rs` — create_* 関数追加（5 件）

**責任分担:**
- `make_<name>_main_fav(name: &str) -> String` — `main.fav` のコード文字列を生成（fs 非依存）
- `create_<name>_project(name: &str) -> Result<(), String>` — `make_*` を呼んでディレクトリ・ファイルを作成

テストは `make_*` ヘルパーを直接呼び、生成文字列を検証する（ファイルシステム非依存）。
`create_*` は `cmd_new` / `try_cmd_new` から呼ばれる公開エントリポイント。

```rust
fn make_ai_etl_main_fav(name: &str) -> String
fn create_ai_etl_project(name: &str) -> Result<(), String>  // make_ai_etl_main_fav を呼んで fs 書き込み

fn make_streaming_main_fav(name: &str) -> String
fn create_streaming_project(name: &str) -> Result<(), String>

fn make_enterprise_main_fav(name: &str) -> String
fn create_enterprise_project(name: &str) -> Result<(), String>

fn make_data_quality_main_fav(name: &str) -> String
fn create_data_quality_project(name: &str) -> Result<(), String>

fn make_distributed_main_fav(name: &str) -> String
fn create_distributed_project(name: &str) -> Result<(), String>
```

各 `make_*` が返す文字列に含まれるべき内容:
- `make_ai_etl_main_fav`: `llm` または `LLM`（Llm.extract の呼び出し）
- `make_streaming_main_fav`: `kafka` または `par`
- `make_enterprise_main_fav`: `TenantRow` または `tenant`
- `make_data_quality_main_fav`: `validate`
- `make_distributed_main_fav`: `par`

#### main.fav 内容要件

- `ai-etl`: `import rune "llm"` + `import rune "vector_db"` + AppCtx 引数 + LLM.extract / VectorDb.upsert 呼び出し
- `streaming`: `import rune "kafka"` + `par` ステージ（Kafka.consume / ML score / Kafka.produce）
- `enterprise`: `schema TenantRow` + マルチテナントフィルタ + 監査ログ出力
- `data-quality`: `Schema.validate_all` を含む検証パイプライン
- `distributed`: `par [StageA, StageB] |> Merge` を含むマルチノード構成

### `driver.rs` — `try_cmd_new` 拡充

既存の match アームの後ろに 5 アームを追加:

```rust
"ai-etl"       => create_ai_etl_project(name),
"streaming"    => create_streaming_project(name),
"enterprise"   => create_enterprise_project(name),
"data-quality" => create_data_quality_project(name),
"distributed"  => create_distributed_project(name),
```

---

## テスト

### `v726000_tests` モジュール（driver.rs に追加）

`make_*` ヘルパーを直接呼び、生成文字列を検証する（ファイルシステム非依存）。

```rust
#[cfg(test)]
mod v726000_tests {
    use super::{make_ai_etl_main_fav, make_data_quality_main_fav};

    #[test]
    fn init_template_ai_etl_valid() {
        let code = make_ai_etl_main_fav("my-project");
        assert!(code.contains("llm") || code.contains("LLM"),
            "ai-etl template should reference LLM rune");
        assert!(code.contains("AppCtx"),
            "ai-etl template should have AppCtx context argument");
    }

    #[test]
    fn init_template_data_quality_valid() {
        let code = make_data_quality_main_fav("my-project");
        assert!(code.contains("validate"),
            "data-quality template should reference Schema.validate_all");
        assert!(code.contains("AppCtx"),
            "data-quality template should have AppCtx context argument");
    }
}
```

---

## 成功基準

- `cargo test v726000` で 2 件 pass
- `cargo test` 全体で 3632 tests pass（3630 + 2）
  ※ ロードマップ記載値 3629 は v72.5.0 の実測 3630 以前の計算のため +2 で 3632 が正しい
- `fav init --template ai-etl`・`--template data-quality` が正常動作（テストで担保）
- 既存 12 テンプレートに対するリグレッションなし

---

## スコープ外

- 実際のファイルシステムへの書き込み（driver.rs テストは生成コード文字列のみ検証）
- WASM 対応・サイト側 UI 更新（v73.x 以降）
- rustyline / ~/.fav_history 統合（v72.7.0 以降）

---

## 変更ファイル

- `fav/src/driver.rs` — TEMPLATE_GALLERY 5 エントリ追加 + create_* 5 関数追加 + try_cmd_new 5 アーム追加 + v726000_tests モジュール追加 + バージョン更新
- `fav/Cargo.toml` — version `72.5.0` → `72.6.0`
- `CHANGELOG.md` — v72.6.0 エントリ追加
- `versions/current.md` — 進行中バージョンを v72.6.0 に更新
