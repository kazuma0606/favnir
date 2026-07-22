# Spec: v53.5.0 — E2E 統合デモ Phase 2（assert_schema + audit-log + OTel）

Status: 計画中
Date: 2026-07-22

---

## 概要

v53.4.0 で作成した `examples/v55-demo/` に対して、以下の 3 機能を統合する:

1. **`assert_schema`**: `SchemaCheck` stage で `assert_schema<ValidOrder>` を使用
2. **`--audit-log`**: `run.sh` に `fav run --audit-log ./audit.log` を含める
3. **OTel span コメント**: `pipeline.fav` の SchemaCheck stage 内にコメントで OTel span の動作を説明

`fav explain --error E0419` で確認できる型ガイダンス（v53.3.0 で強化済み）と
`--audit-log` フラグ（v52.x 系で実装済み）が実際のデモパイプラインで使用される形を示す。

---

## 実装スコープ

### 1. `examples/v55-demo/pipeline.fav` 更新

`ValidOrder` 型定義と `SchemaCheck` stage を追加する。

```favnir
import kafka
import snowflake
import "./stages/enrich" as enrich
import "./stages/validate" as validate

type RawOrder      = { id: Int, amount: Float, status: String, raw_ts: String }
type Order         = { id: Int, amount: Float, status: String }
type ValidOrder    = { id: Int, amount: Float, status: String }
type EnrichedOrder = { id: Int, amount: Float, status: String, region: String }

pipeline OrderIngestion {
  stage Consume: Stream<RawOrder> -> Stream<Order> = |_raw| {
    bind order <- kafka.consume("orders")
    Ok(order)
  }
  stage SchemaCheck: Order -> Result<ValidOrder> = |order| {
    bind checked <- assert_schema<ValidOrder>(order)
    Ok(checked)
    // fav run --audit-log 実行時: アクセスログに自動記録
    // OTel span に schema.name = "ValidOrder" が付与
  }
  stage Process: ValidOrder -> Result<EnrichedOrder> = |valid_order| {
    par [enrich.run(valid_order), validate.run(valid_order)] |> Merge.ordered
  }
  stage Store: EnrichedOrder -> Unit = |enriched| {
    bind _ <- snowflake.insert("orders_v2", enriched)
    Ok(Unit)
  }
}
```

**注意**: `stages/enrich.fav` / `stages/validate.fav` の `Order` 型定義と `ValidOrder` 型定義の
フィールドが同一のため、`par` アーム内で `ValidOrder` を `Order` として渡しても型が揃う。

### 2. `examples/v55-demo/run.sh` 新規作成

```bash
#!/bin/bash
# E2E integration demo runner (v55-demo)
# Usage: ./run.sh
set -euo pipefail

fav run pipeline.fav --audit-log ./audit.log
```

---

### 3. テスト仕様

`v53500_tests` モジュールを `driver.rs` に追加（`v53400_tests` の直前）:

```rust
// -- v53500_tests (v53.5.0) -- E2E 統合デモ Phase 2: assert_schema + audit-log --
#[cfg(test)]
mod v53500_tests {
    #[test]
    fn e2e_integration_demo_has_schema() {
        let content = include_str!("../../examples/v55-demo/pipeline.fav");
        assert!(
            content.contains("assert_schema"),
            "pipeline.fav must use assert_schema for schema validation"
        );
        assert!(
            content.contains("ValidOrder"),
            "pipeline.fav must define ValidOrder schema type"
        );
    }

    #[test]
    fn e2e_integration_demo_has_audit_log() {
        let content = include_str!("../../examples/v55-demo/run.sh");
        assert!(
            content.contains("fav run"),
            "run.sh must invoke fav run"
        );
        assert!(
            content.contains("--audit-log"),
            "run.sh must pass --audit-log flag to fav run"
        );
    }
}
```

---

## バージョン更新

- `fav/Cargo.toml`: `"53.4.0"` → `"53.5.0"`

---

## 完了条件

- `cargo test` 3173 passed, 0 failed（3171 + 2 件追加）
  （ベース 3171 = v53.4.0 完了時実績。ロードマップ推定値 3167 との差 +6 = 累積差 +4（v53.1.0 コードレビュー起因）+ 今回追加 +2）
- `v53500_tests` 2 件 pass:
  - `e2e_integration_demo_has_schema`
  - `e2e_integration_demo_has_audit_log`
- `cargo clippy -- -D warnings` クリーン
- `examples/v55-demo/pipeline.fav` に `assert_schema` / `ValidOrder` が含まれる
- `examples/v55-demo/run.sh` に `fav run` / `--audit-log` が含まれる

---

## 影響範囲

| ファイル | 変更種別 |
|---|---|
| `examples/v55-demo/pipeline.fav` | `ValidOrder` 型・`SchemaCheck` stage 追加 |
| `examples/v55-demo/run.sh` | 新規作成 |
| `fav/src/driver.rs` | `v53500_tests` 追加 |
| `fav/Cargo.toml` | version 更新 |
| `CHANGELOG.md` | v53.5.0 エントリ追加 |
| `versions/current.md` | v53.5.0 / 3173 tests に更新 |
| `versions/roadmap/roadmap-v53.1-v54.0.md` | v53.5.0 実績欄を COMPLETE に更新・推定値修正 |

---

## 設計上の注意

- `ValidOrder` のフィールドは `Order` と同一（`{ id: Int, amount: Float, status: String }`）— `assert_schema` の型ガード役
- `run.sh` は `include_str!("../../examples/v55-demo/run.sh")` で driver.rs テストから参照する
- `e2e_integration_demo_structure`（v53400_tests）は `run.sh` を assert していないため、今回の追加で既存テストは破壊されない
- v53400_tests にバージョンピンテストは存在しないため、`Cargo.toml` バージョン更新時の空化対象なし
- `SchemaCheck` stage は `Process` stage の前に挿入する（validate 前にスキーマを確認するのが自然な順序）
