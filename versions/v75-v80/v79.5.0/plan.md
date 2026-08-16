# v79.5.0 実装計画 — Execution Effects showcase パイプライン

Date: 2026-08-16

---

## 実装順序

### Step 1: pipeline.fav 更新

`infra/e2e-demo/favnir3-showcase/pipeline.fav` の `load_with_provenance` と `showcase_pipeline` の間に以下を追加する:

```favnir
// --- Stage 3: Execution Effects（v78.x）---
fn join_stage(ctx: AppCtx, customers: List<Row>, orders: List<Row>) -> Result<List<Row>, String> !Adaptive !Cached {
    bind joined <- customers |> join(orders, on: "id")
    // !Adaptive → row 数に応じて broadcast/hash を自動選択
    // !Cached   → TTL 内は同じ入力に対してキャッシュを返す
    Result.ok(joined)
}
```

`showcase_pipeline` のコメント行も更新:
```favnir
    // Stage 3: Verifiable（v77.x）— contract.fav で実装済み
    // Stage 4: Execution Effects（v78.x）— join_stage で実装済み
```

---

### Step 2: CHANGELOG.md 更新（テスト追加より先）

`CHANGELOG.md` の先頭に v79.5.0 エントリを追加。

---

### Step 3: driver.rs — v795000_tests モジュール追加

`fav/src/driver.rs` の末尾に以下を追加:

```rust
// --- v79.5.0: Execution Effects showcase パイプライン ---
#[cfg(test)]
mod v795000_tests {
    const PIPELINE: &str = include_str!("../../infra/e2e-demo/favnir3-showcase/pipeline.fav");

    #[test]
    fn showcase_execution_cached_effect() {
        assert!(PIPELINE.contains("join_stage"), "pipeline.fav must define join_stage");
        assert!(PIPELINE.contains("!Cached"), "pipeline.fav must declare !Cached effect");
    }

    #[test]
    fn showcase_execution_adaptive_effect() {
        assert!(PIPELINE.contains("!Adaptive"), "pipeline.fav must declare !Adaptive effect");
        assert!(PIPELINE.contains("join(orders, on:"), "pipeline.fav must reference join with on: key");
    }
}
```

注意: `use super::*` 不要。`const PIPELINE` パターンを採用。

---

### Step 4: Cargo.toml バージョン更新

`fav/Cargo.toml` の `version = "79.4.0"` → `"79.5.0"` に更新。

driver.rs 内の escaped `\"79.4.0\"` を `\"79.5.0\"` に一括更新（sed）。
エラーメッセージ文字列（unescaped）の `79.4.0` も `79.5.0` に更新。

更新後に `grep -c "79\.4\.0" /c/Users/yoshi/favnir/fav/src/driver.rs` → 出力が `1` であることを確認。
（残るのは `// --- v79.4.0: Verifiable showcase パイプライン ---` コメント行の 1 件のみ）

---

### Step 5: versions/current.md 更新

- `## 進行中バージョン` → `**v79.5.0**（Execution Effects showcase パイプライン）`
- `## 次に切る版` → `**v79.6.0**（Temporal 深化 / SCD2 拡張）`

---

### Step 6: 最終確認

```bash
cargo test v795000 2>&1 | grep -E "test result|FAILED"
cargo test 2>&1 | grep "^test result"
```

3797 tests pass、v795000 2 件 pass を確認。

---

## 依存順序サマリ

```
pipeline.fav 更新（Step 1）
  → CHANGELOG 更新（Step 2）
  → driver.rs テスト追加（Step 3）← pipeline.fav が先に更新されていること
  → Cargo.toml + エラーメッセージ更新（Step 4）
  → current.md 更新（Step 5）
  → 最終確認（Step 6）
```
