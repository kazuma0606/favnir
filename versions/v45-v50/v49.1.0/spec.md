# Spec: v49.1.0 — 全機能統合テスト + E2E デモ更新

## 概要

v46〜v49 の全機能（`return` ガード節 / 新 import 構文 / stdlib 2.0 / `fav test` ブロック）を
統合したパイプラインデモ `examples/v50-demo/` を作成し、Rust テスト 2 件で構造と内容を検証する。

---

## 変更ファイル

| ファイル | 変更内容 |
|---|---|
| `examples/v50-demo/fav.toml` | 新規作成（プロジェクト設定・`[runes] kafka`）|
| `examples/v50-demo/pipeline.fav` | 新規作成（メインパイプライン・新 import 構文使用）|
| `examples/v50-demo/stages/validate.fav` | 新規作成（validate ステージ）|
| `fav/src/driver.rs` | `v491000_tests` 追加（2テスト）|
| `fav/Cargo.toml` | version → `"49.1.0"` |
| `fav/src/driver.rs`（`v49000_tests`）| `cargo_toml_version_is_49_0_0` をスタブ化（バージョンバンプにより）|
| `CHANGELOG.md` | v49.1.0 エントリ追加 |

---

## デモ内容

### `examples/v50-demo/fav.toml`

```toml
[package]
name    = "v50-demo"
version = "0.1.0"

[runes]
kafka = "2.1.0"
```

### `examples/v50-demo/stages/validate.fav`

```favnir
type RawOrder = { id: Int, amount: Float, status: String }
type Order    = { id: Int, amount: Float, status: String }

fn run(raw: RawOrder) -> Result<Order, String> {
  return Result.err("invalid status") if raw.status != "pending"
  return Result.err("negative amount") if raw.amount < 0.0
  Result.ok({ id: raw.id, amount: raw.amount, status: raw.status })
}
```

### `examples/v50-demo/pipeline.fav`

```favnir
import "./stages/validate" as validate
import kafka

pipeline OrderIngestion {
  stage Consume: Stream<RawOrder> -> Stream<Order> = |raw| {
    bind order <- kafka.consume("orders")
    bind valid <- validate.run(order)
    Ok(valid)
  }
}

#[test]
fn test_validate_ok() {
  let good = { id: 1, amount: 100.0, status: "pending" }
  assert_eq(validate.run(good), Result.ok({ id: 1, amount: 100.0, status: "pending" }))
}

#[test]
fn test_validate_err_status() {
  let bad = { id: 2, amount: 50.0, status: "shipped" }
  assert_eq(validate.run(bad), Result.err("invalid status"))
}
```

---

## テスト（+2）

`v491000_tests` を `v49000_tests` の直前に追加:

```rust
#[cfg(test)]
mod v491000_tests {
    #[test]
    fn e2e_demo_v50_structure() {
        let base = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../examples/v50-demo");
        assert!(base.exists(), "examples/v50-demo/ must exist");
        assert!(base.join("fav.toml").exists(), "examples/v50-demo/fav.toml must exist");
        assert!(base.join("pipeline.fav").exists(), "examples/v50-demo/pipeline.fav must exist");
        assert!(base.join("stages/validate.fav").exists(),
            "examples/v50-demo/stages/validate.fav must exist");
    }

    #[test]
    fn e2e_demo_uses_new_import() {
        let content = include_str!("../../examples/v50-demo/pipeline.fav");
        assert!(content.contains("import kafka"),
            "pipeline.fav should use new package import syntax: import kafka");
        assert!(content.contains("import \"./stages/validate\""),
            "pipeline.fav should use local import syntax");
        assert!(!content.contains("import rune"),
            "pipeline.fav should NOT use legacy import rune syntax");
    }
}
```

テスト数: 3069 → **3071**（+2）

---

## 注意事項

- `examples/v50-demo/` は `favnir/examples/` 直下（`fav/examples/` ではない）
- `env!("CARGO_MANIFEST_DIR")` = `fav/` → `.join("../examples/v50-demo")` = `favnir/examples/v50-demo/`
- `include_str!("../../examples/v50-demo/pipeline.fav")` — `fav/src/driver.rs` からの相対パス
- `pipeline.fav` は `import rune` を使わないこと（W035 非推奨構文の回避・テスト `e2e_demo_uses_new_import` で検証）
- ロードマップの推定テスト数 3064 は旧推定値（v49.0.0 実績 3069 より）。実際は 3069 + 2 = **3071**
- `RawOrder`/`Order` 型は `stages/validate.fav` にのみ定義し、`pipeline.fav` では再定義しないこと（本バージョンは Rust 構造テストのみでコンパイル実行は行わないため即時ブロックにはならないが、型衝突を避けるためコード品質として重要）

---

## 完了条件

- `cargo test` 3071 passed, 0 failed（3069 + 2 件）
- `cargo clippy -- -D warnings` クリーン
- `fav/Cargo.toml` version → `"49.1.0"`
- `CHANGELOG.md` に v49.1.0 エントリ追加
- `versions/current.md` を v49.1.0（3071 tests）に更新、進行中バージョンを `v49.2.0` に更新
- `versions/roadmap/roadmap-v49.1-v50.0.md` の v49.1.0 実績を記入
- `tasks.md` を COMPLETE に更新（T0〜T3 全 `[x]`）
