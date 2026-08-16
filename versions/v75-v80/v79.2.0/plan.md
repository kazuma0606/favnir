# v79.2.0 実装計画 — Temporal showcase パイプライン

Date: 2026-08-16

---

## 実装順序

### Step 1: pipeline.fav 更新

`infra/e2e-demo/favnir3-showcase/pipeline.fav` に `load_with_freshness` 関数を追加する。

既存の `showcase_pipeline` 関数のコメントを「load_with_freshness で実装済み」に更新する。

追加内容:

```favnir
// --- Stage 1: Temporal（v75.x）---
fn load_with_freshness(ctx: AppCtx) -> Result<List<Row>, String> {
    bind snapshot <- AsOfQuery { table: "orders", as_of_ts: ctx.run_ts }
    bind _        <- FreshnessPolicy.check(snapshot, max_age: Duration.hours(1))
    bind history  <- apply_scd2_update(existing_customers, new_data, ctx.run_ts)
    Result.ok(history)
}
```

---

### Step 2: CHANGELOG.md 更新（テスト追加より先）

`CHANGELOG.md` の先頭に v79.2.0 エントリを追加。

---

### Step 3: driver.rs — v792000_tests モジュール追加

`fav/src/driver.rs` の末尾に以下を追加:

```rust
// --- v79.2.0: Temporal showcase パイプライン ---
#[cfg(test)]
mod v792000_tests {
    #[test]
    fn showcase_temporal_freshness_check() {
        let pipeline = include_str!("../../infra/e2e-demo/favnir3-showcase/pipeline.fav");
        assert!(pipeline.contains("load_with_freshness"), "pipeline.fav must define load_with_freshness");
        assert!(pipeline.contains("AsOfQuery"), "pipeline.fav must reference AsOfQuery");
        assert!(pipeline.contains("FreshnessPolicy"), "pipeline.fav must reference FreshnessPolicy");
    }

    #[test]
    fn showcase_temporal_scd2_applied() {
        let pipeline = include_str!("../../infra/e2e-demo/favnir3-showcase/pipeline.fav");
        assert!(pipeline.contains("apply_scd2_update"), "pipeline.fav must reference apply_scd2_update");
        assert!(pipeline.contains("ctx.run_ts"), "pipeline.fav must reference ctx.run_ts for temporal context");
    }
}
```

注意: `use super::*` は不要（外部シンボル未使用）。

---

### Step 4: Cargo.toml バージョン更新

`fav/Cargo.toml` の `version = "79.1.0"` → `"79.2.0"` に更新。

driver.rs 内の `"79.1.0"` バージョン文字列アサーションを `"79.2.0"` に一括更新。

更新後に `grep -c "79\.1\.0" driver.rs` → 出力が `1` であることを確認。
（残るのは `// --- v79.1.0: 統合ショーケース基盤 ---` の 1 件のみ）

---

### Step 5: versions/current.md 更新

- `## 進行中バージョン` → `**v79.2.0**（Temporal showcase パイプライン）`
- `## 次に切る版` → `**v79.3.0**（Provenance showcase パイプライン）`

---

### Step 6: 最終確認

```bash
cargo test v792000 2>&1 | grep -E "test result|FAILED"
cargo test 2>&1 | grep "^test result"
```

3791 tests pass、v792000 2 件 pass を確認。

---

## 依存順序サマリ

```
pipeline.fav 更新（Step 1）← showcase ファイルが先に存在すること
  → CHANGELOG 更新（Step 2）
  → driver.rs テスト追加（Step 3）← pipeline.fav が先に更新されていること
  → Cargo.toml バージョン更新（Step 4）
  → current.md 更新（Step 5）
  → 最終確認（Step 6）
```
