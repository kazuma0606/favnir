# Plan: v53.4.0 — E2E 統合デモ Phase 1（Kafka → par transform → Snowflake）

---

## ステップ 1: 事前確認

```bash
cargo test -j 8 -- --test-threads=8 2>&1 | grep "test result"
# → 3169 passed, 0 failed を確認

# v53400_tests が未存在を確認
rg -n "v53400_tests" fav/src/driver.rs  # → 0 件

# v53300_tests の行番号を確認（挿入位置）
rg -n "v53300_tests" fav/src/driver.rs  # → 行番号を特定

# examples/v55-demo/ が未存在を確認
ls examples/v55-demo/ 2>/dev/null  # → エラーまたは空

# Cargo.toml が 53.3.0 であることを確認
grep "^version" fav/Cargo.toml  # → version = "53.3.0"
```

---

## ステップ 2: `examples/v55-demo/` 作成

```bash
mkdir -p examples/v55-demo/stages
```

### `examples/v55-demo/fav.toml`

```toml
[package]
name    = "v55-demo"
version = "0.1.0"

[runes]
kafka     = "2.1.0"
snowflake = "1.0.0"
```

### `examples/v55-demo/pipeline.fav`

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

### `examples/v55-demo/stages/enrich.fav`

```favnir
type Order        = { id: Int, amount: Float, status: String }
type EnrichedOrder = { id: Int, amount: Float, status: String, region: String }

fn run(order: Order) -> Result<EnrichedOrder, String> {
  Result.ok({ id: order.id, amount: order.amount, status: order.status, region: "us-east-1" })
}
```

### `examples/v55-demo/stages/validate.fav`

```favnir
type Order = { id: Int, amount: Float, status: String }

fn run(order: Order) -> Result<Order, String> {
  return Result.err("invalid status") if order.status != "pending"
  return Result.err("negative amount") if order.amount < 0.0
  Result.ok(order)
}
```

`cargo build` → コンパイルエラーなし確認。

---

## ステップ 3: `driver.rs` — `v53400_tests` 追加

`v53300_tests` モジュールの直前に `v53400_tests` を追加:

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

`cargo build` → コンパイルエラーなし確認。

---

## ステップ 4: `fav/Cargo.toml` バージョン更新

`version = "53.3.0"` → `version = "53.4.0"`

v53300_tests にはバージョンピンテストが存在しないため、空化対象なし。

---

## ステップ 5: テスト実行・確認

```bash
cd /c/Users/yoshi/favnir/fav && cargo test -j 8 -- --test-threads=8 2>&1 | grep "test result"
```

期待値: 3171 passed, 0 failed

```bash
cargo clippy -- -D warnings
```

---

## ステップ 6: 後処理

- `CHANGELOG.md` に v53.4.0 エントリ追加
- `versions/current.md` を v53.4.0（3171 tests）に更新
- `roadmap-v53.1-v54.0.md` の v53.4.0 実績欄を COMPLETE に更新
- `tasks.md` を COMPLETE に更新（T0〜T4 全 `[x]`）
