# v72.6.0 実装計画 — `fav init` テンプレートギャラリー拡充

Date: 2026-08-12

---

## 依存順序

```
T0: 事前確認
  ↓
T1: TEMPLATE_GALLERY 拡充（5 エントリ追加）
  ↓
T2: create_ai_etl_project 実装
T2: create_streaming_project 実装
T2: create_enterprise_project 実装
T2: create_data_quality_project 実装
T2: create_distributed_project 実装
  ↓
T3: try_cmd_new に 5 アームを追加
  ↓
T4: v726000_tests モジュール追加
  ↓
T5: Cargo.toml バージョン更新 + driver.rs バージョンアサーション更新
  ↓
T6: 部分テスト確認（cargo test v726000）
  ↓
T7: 全体テスト確認（cargo test）
  ↓
T8: CHANGELOG.md 更新
T8: versions/current.md 更新
  ↓
T9: 最終確認
```

---

## ステップ詳細

### T0: 事前確認

- `fav/Cargo.toml` のバージョンが `72.5.0` であることを確認
- `cargo test` が 3630 tests pass（0 failures）であることを確認
- `driver.rs` に `v725000_tests` モジュールが存在することを確認
- `driver.rs` に `v726000_tests` が未存在であることを確認
- `driver.rs` 内の `try_cmd_new` が存在することを確認（12 アーム）
- `driver.rs` 内の `"72.5.0"` 文字列（バージョンアサーション）の件数を grep で確認

### T1: TEMPLATE_GALLERY 拡充

`TEMPLATE_GALLERY`（または相当するデータ構造）に以下を追加:
```
ai-etl       / "LLM抽出 → VectorDB パイプライン"
streaming    / "Kafka + MLスコアリング パイプライン"
enterprise   / "マルチテナント + 監査ログ パイプライン"
data-quality / "データ品質検証パイプライン"
distributed  / "マルチノード par パイプライン"
```

`cargo build` でエラーがないことを確認。

### T2: `make_*` + `create_*` 関数 10 件追加（v725000_tests モジュールの直前に挿入）

**パターン**: 各テンプレートにつき 2 関数を追加する。
- `make_<name>_main_fav(name: &str) -> String` — コード文字列生成（fs 非依存、テスト対象）
- `create_<name>_project(name: &str) -> Result<(), String>` — `make_*` を呼んで fs 書き込み

#### `ai-etl`

```
make_ai_etl_main_fav -> String:
  import rune "llm"
  import rune "vector_db"
  fn main(ctx: AppCtx) -> Result<Unit, String> {
      bind raw    <- ctx.io.read_file_raw("data/docs.txt")
      bind chunks <- Llm.extract(raw)
      bind _      <- VectorDb.upsert(chunks)
      ctx.io.println("AI ETL complete.")
  }

create_ai_etl_project: make_ai_etl_main_fav を呼び fav.toml / README.md も書き込む → Ok(())
```

#### `streaming`

```
make_streaming_main_fav -> String:
  import rune "kafka"
  fn main(ctx: AppCtx) -> Result<Unit, String> {
      // par [Consume, Score] |> Produce
      ctx.io.println("Streaming pipeline running.")
  }
```

#### `enterprise`

```
make_enterprise_main_fav -> String:
  schema TenantRow { tenant_id: String, payload: String }
  fn main(ctx: AppCtx) -> Result<Unit, String> {
      bind rows <- ctx.io.read_file_raw("data/tenants.json")
      ctx.io.println("[audit] processed tenant rows")
  }
```

#### `data-quality`

```
make_data_quality_main_fav -> String:
  schema DataRow { id: String, value: Float }
  fn main(ctx: AppCtx) -> Result<Unit, String> {
      bind raw   <- ctx.io.read_file_raw("data/input.csv")
      bind valid <- Schema.validate_all(raw)
      ctx.io.println("Data quality check complete.")
  }
```

#### `distributed`

```
make_distributed_main_fav -> String:
  fn main(ctx: AppCtx) -> Result<Unit, String> {
      // par [LoadA, LoadB] |> Merge |> Transform |> Output
      ctx.io.println("Distributed pipeline complete.")
  }
```

`cargo build` でエラーがないことを確認。

### T3: `try_cmd_new` に 5 アーム追加

既存のアームの後ろに追加:
```rust
"ai-etl"       => create_ai_etl_project(name),
"streaming"    => create_streaming_project(name),
"enterprise"   => create_enterprise_project(name),
"data-quality" => create_data_quality_project(name),
"distributed"  => create_distributed_project(name),
```

`cargo build` でエラーがないことを確認。

### T4: `v726000_tests` モジュール追加

`v725000_tests` モジュールの直後に追加。`make_*` ヘルパーを直接呼ぶ（fs 非依存）:

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

`cargo test v726000` で 2 件 pass することを確認。

### T5: バージョン更新

- `fav/Cargo.toml`: `version = "72.5.0"` → `version = "72.6.0"`
- `driver.rs` 内の `version = \"72.5.0\"` 文字列を `version = \"72.6.0\"` に replace_all
- `driver.rs` 内のエラーメッセージ `"Cargo.toml version should be 72.5.0"` を `"72.6.0"` に replace_all
- `driver.rs` 内のエラーメッセージ `"Cargo.toml should declare version 72.5.0"` を `"72.6.0"` に replace_all
- 残存 72.5.0 はコメント・セクションヘッダーのみで意図的保持を確認

### T6: 部分テスト確認

```
cargo test v726000
```
2 件 pass を確認。

### T7: 全体テスト確認

```
cargo test
```
3632 tests pass（0 failures）を確認。

### T8: `CHANGELOG.md` 更新

- `CHANGELOG.md` 先頭に `## [v72.6.0]` エントリを追加

### T9: `versions/current.md` 更新

- 「進行中バージョン」を `v72.6.0`（`fav init` テンプレートギャラリー拡充）に更新
- 「次に切る版」を `v72.7.0` に更新

### T10: 最終確認

- `cargo test v726000` 2 件 pass
- `cargo test` 全体 3632 pass
- `fav/Cargo.toml` バージョン = `72.6.0`
- `try_cmd_new` に 5 新アーム存在
- TEMPLATE_GALLERY 拡充済み
