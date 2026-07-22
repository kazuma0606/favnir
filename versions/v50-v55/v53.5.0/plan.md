# Plan: v53.5.0 — E2E 統合デモ Phase 2（assert_schema + audit-log + OTel）

---

## ステップ 1: 事前確認

```bash
cargo test -j 8 -- --test-threads=8 2>&1 | grep "test result"
# → 3171 passed, 0 failed を確認

# v53500_tests が未存在を確認
rg -n "v53500_tests" fav/src/driver.rs  # → 0 件

# v53400_tests の行番号を確認（挿入位置）
rg -n "v53400_tests" fav/src/driver.rs  # → 行番号を特定

# examples/v55-demo/run.sh が未存在を確認
ls examples/v55-demo/run.sh 2>/dev/null  # → エラー

# pipeline.fav に assert_schema が未存在を確認
rg "assert_schema" examples/v55-demo/pipeline.fav  # → 0 件

# Cargo.toml が 53.4.0 であることを確認
grep "^version" fav/Cargo.toml  # → version = "53.4.0"
```

---

## ステップ 2: `examples/v55-demo/pipeline.fav` 更新

`ValidOrder` 型と `SchemaCheck` stage を追加する。

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

内容確認:
```bash
grep "assert_schema" examples/v55-demo/pipeline.fav  # → 1 件以上ヒット
grep "ValidOrder" examples/v55-demo/pipeline.fav     # → 1 件以上ヒット
```

---

## ステップ 3: `examples/v55-demo/run.sh` 新規作成

```bash
#!/bin/bash
# E2E integration demo runner (v55-demo)
# Usage: ./run.sh
set -euo pipefail

fav run pipeline.fav --audit-log ./audit.log
```

```bash
chmod +x examples/v55-demo/run.sh
```

内容確認:
```bash
grep "\-\-audit-log" examples/v55-demo/run.sh  # → 1 件以上ヒット
```

---

## ステップ 4: `driver.rs` — `v53500_tests` 追加

`v53400_tests` モジュールの直前に `v53500_tests` を追加:

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

`cargo build` → コンパイルエラーなし確認。

---

## ステップ 5: `fav/Cargo.toml` バージョン更新

`version = "53.4.0"` → `version = "53.5.0"`

v53400_tests にはバージョンピンテストが存在しないため、空化対象なし。

---

## ステップ 6: テスト実行・確認

```bash
cd /c/Users/yoshi/favnir/fav && cargo test -j 8 -- --test-threads=8 2>&1 | grep "test result"
```

期待値: 3173 passed, 0 failed

```bash
cargo clippy -- -D warnings
```

---

## ステップ 7: 後処理

- `CHANGELOG.md` に v53.5.0 エントリ追加
- `versions/current.md` を v53.5.0（3173 tests）に更新
- `roadmap-v53.1-v54.0.md` の v53.5.0 実績欄を COMPLETE に更新・推定値（3167 → 3173）を修正
- `tasks.md` を COMPLETE に更新（T0〜T5 全 `[x]`）
