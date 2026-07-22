# Spec: v53.4.0 — E2E 統合デモ Phase 1（Kafka → par transform → Snowflake）

Status: 計画中
Date: 2026-07-22

---

## 概要

v51〜v53 で実装した機能（`par` 並列 stage・新 import 構文・Kafka/Snowflake Rune）を統合した
大規模 E2E デモを `examples/v55-demo/` に作成する。

`fav bench`・`fav explain` との連携を想定したデモパイプラインであり、
`par [enrich, validate]` による並列処理と Snowflake への書き込みを示す。

Phase 1（v53.4.0）ではディレクトリ構造と `pipeline.fav` の基本形を作成する。
`assert_schema` の統合は Phase 2（v53.5.0）で追加する。

---

## 実装スコープ

### 1. `examples/v55-demo/` ディレクトリ作成

作成ファイル:

```
examples/v55-demo/
├── fav.toml
├── pipeline.fav
└── stages/
    ├── enrich.fav
    └── validate.fav
```

#### `examples/v55-demo/fav.toml`

```toml
[package]
name    = "v55-demo"
version = "0.1.0"

[runes]
kafka     = "2.1.0"
snowflake = "1.0.0"
```

#### `examples/v55-demo/pipeline.fav`

```favnir
import kafka
import snowflake
import "./stages/enrich" as enrich
import "./stages/validate" as validate

type RawOrder     = { id: Int, amount: Float, status: String }
type Order        = { id: Int, amount: Float, status: String }
type EnrichedOrder = { id: Int, amount: Float, status: String, region: String }

pipeline OrderIngestion {
  stage Consume: Stream<RawOrder> -> Stream<Order> = |_raw| {
    bind order <- kafka.consume("orders")
    Ok(order)
  }
  stage Process: Order -> Result<EnrichedOrder> = |order| {
    par [enrich.run(order), validate.run(order)] |> Merge.ordered
  }
  stage Store: EnrichedOrder -> Unit = |enriched| {
    bind _ <- snowflake.insert("orders_v2", enriched)
    Ok(Unit)
  }
}
```

#### `examples/v55-demo/stages/enrich.fav`

```favnir
type Order        = { id: Int, amount: Float, status: String }
type EnrichedOrder = { id: Int, amount: Float, status: String, region: String }

fn run(order: Order) -> Result<EnrichedOrder, String> {
  Result.ok({ id: order.id, amount: order.amount, status: order.status, region: "us-east-1" })
}
```

#### `examples/v55-demo/stages/validate.fav`

```favnir
type Order = { id: Int, amount: Float, status: String }

fn run(order: Order) -> Result<Order, String> {
  return Result.err("invalid status") if order.status != "pending"
  return Result.err("negative amount") if order.amount < 0.0
  Result.ok(order)
}
```

---

### 2. テスト仕様

`v53400_tests` モジュールを `driver.rs` に追加（`v53300_tests` の直前）:

```rust
// -- v53400_tests (v53.4.0) -- E2E 統合デモ Phase 1 --
#[cfg(test)]
mod v53400_tests {
    #[test]
    fn e2e_integration_demo_structure() {
        let base = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../examples/v55-demo");
        assert!(base.exists(), "examples/v55-demo/ must exist");
        assert!(
            base.join("fav.toml").exists(),
            "examples/v55-demo/fav.toml must exist"
        );
        assert!(
            base.join("pipeline.fav").exists(),
            "examples/v55-demo/pipeline.fav must exist"
        );
        assert!(
            base.join("stages/enrich.fav").exists(),
            "examples/v55-demo/stages/enrich.fav must exist"
        );
        assert!(
            base.join("stages/validate.fav").exists(),
            "examples/v55-demo/stages/validate.fav must exist"
        );
    }

    #[test]
    fn e2e_integration_demo_uses_par() {
        let content = include_str!("../../examples/v55-demo/pipeline.fav");
        assert!(
            content.contains("par ["),
            "pipeline.fav must use par [] parallel syntax"
        );
        assert!(
            content.contains("Merge.ordered"),
            "pipeline.fav must use Merge.ordered after par"
        );
        assert!(
            content.contains("import kafka"),
            "pipeline.fav must import kafka"
        );
        assert!(
            content.contains("import snowflake"),
            "pipeline.fav must import snowflake"
        );
    }
}
```

---

## バージョン更新

- `fav/Cargo.toml`: `"53.3.0"` → `"53.4.0"`

---

## 完了条件

- `cargo test` 3171 passed, 0 failed（3169 + 2 件追加）
- `v53400_tests` 2 件 pass:
  - `e2e_integration_demo_structure`
  - `e2e_integration_demo_uses_par`
- `cargo clippy -- -D warnings` クリーン
- `examples/v55-demo/` ディレクトリが以下の構造で存在すること:
  - `fav.toml`
  - `pipeline.fav`（`par [` / `Merge.ordered` / `import kafka` / `import snowflake` を含む）
  - `stages/enrich.fav`
  - `stages/validate.fav`

---

## 影響範囲

| ファイル | 変更種別 |
|---|---|
| `examples/v55-demo/fav.toml` | 新規作成 |
| `examples/v55-demo/pipeline.fav` | 新規作成 |
| `examples/v55-demo/stages/enrich.fav` | 新規作成 |
| `examples/v55-demo/stages/validate.fav` | 新規作成 |
| `fav/src/driver.rs` | `v53400_tests` 追加 |
| `fav/Cargo.toml` | version 更新 |
| `CHANGELOG.md` | v53.4.0 エントリ追加 |
| `versions/current.md` | v53.4.0 / 3171 tests に更新 |
| `versions/roadmap/roadmap-v53.1-v54.0.md` | v53.4.0 実績欄を COMPLETE に更新 |

---

## 設計上の注意

- `examples/v55-demo/` は `examples/v50-demo/` と同じ階層（`fav/` の親ディレクトリ下の `examples/`）
- テスト内の `include_str!("../../examples/v55-demo/pipeline.fav")` は `fav/src/driver.rs` から `../../` で 2 階層上（リポジトリルート）を指す
  - `fav/src/` + `../../` = リポジトリルート（`C:/Users/yoshi/favnir/`） → `examples/v55-demo/pipeline.fav` ✓
- `env!("CARGO_MANIFEST_DIR")` は `fav/` を指すため `.join("../examples/v55-demo")` で正しく解決される（v50-demo テストと同パターン）
- `stages/enrich.fav` / `stages/validate.fav` は型が揃っていれば十分（VM 実行テストは対象外）
- Phase 2（v53.5.0）で `assert_schema` を追加するため、スキーマ型定義は phase 1 から含めておく
- `examples/v55-demo/` を v53.4.0（v53 スプリント中）で先行作成するのは、v53.5.0〜v53.9.0 の各フェーズで段階的に拡張するためである。`v50-demo` が v49.1.0 で作成されて v50.x 系で利用されたのと同じ先行作成パターン。v55.0 到達時に最終形を確定する。
