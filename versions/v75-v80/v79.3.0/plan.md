# v79.3.0 実装計画 — Provenance showcase パイプライン

Date: 2026-08-16

---

## 実装順序

### Step 1: pipeline.fav 更新

`infra/e2e-demo/favnir3-showcase/pipeline.fav` に `load_with_provenance` 関数を追加する。

追加位置: `load_with_freshness` 関数の直後。

```favnir
// --- Stage 2: Provenance（v76.x）---
fn load_with_provenance(ctx: AppCtx, rows: List<Row>) -> Result<TracedData, String> {
    bind source <- DataSource {
        name: "snowflake-crm",
        uri: "snowflake://warehouse/crm/users",
        source_type: Snowflake
    }
    bind raw    <- TracedData.wrap(rows, source)
    bind masked <- raw |> TracedData.map(mask_pii, label: "mask_pii")
    bind facet  <- OpenLineage.from_provenance(masked.provenance)
    Result.ok(masked)
}
```

`showcase_pipeline` の Stage 2 コメントを「load_with_provenance で実装済み」に更新する。

---

### Step 2: CHANGELOG.md 更新（テスト追加より先）

`CHANGELOG.md` の先頭に v79.3.0 エントリを追加。

---

### Step 3: driver.rs — v793000_tests モジュール追加

`fav/src/driver.rs` の末尾に以下を追加:

```rust
// --- v79.3.0: Provenance showcase パイプライン ---
#[cfg(test)]
mod v793000_tests {
    const PIPELINE: &str = include_str!("../../infra/e2e-demo/favnir3-showcase/pipeline.fav");

    #[test]
    fn showcase_provenance_traced() {
        assert!(PIPELINE.contains("load_with_provenance"), "pipeline.fav must define load_with_provenance");
        assert!(PIPELINE.contains("TracedData"), "pipeline.fav must reference TracedData");
        assert!(PIPELINE.contains("DataSource"), "pipeline.fav must reference DataSource");
        assert!(PIPELINE.contains("mask_pii"), "pipeline.fav must reference mask_pii");
    }

    #[test]
    fn showcase_provenance_openlineage_generated() {
        assert!(PIPELINE.contains("OpenLineage"), "pipeline.fav must reference OpenLineage");
        assert!(PIPELINE.contains("masked.provenance"), "pipeline.fav must reference masked.provenance");
    }
}
```

注意: `use super::*` 不要。`const PIPELINE` パターンを採用（v79.2.0 と同様）。

---

### Step 4: Cargo.toml バージョン更新

`fav/Cargo.toml` の `version = "79.2.0"` → `"79.3.0"` に更新。

driver.rs 内の `"79.2.0"` バージョン文字列アサーションを `"79.3.0"` に一括更新（sed）。

また、エラーメッセージ文字列（unescaped）の `79.2.0` も `79.3.0` に更新する。

更新後に `grep -c "79\.2\.0" /c/Users/yoshi/favnir/fav/src/driver.rs` → 出力が `1` であることを確認。
（残るのは `// --- v79.2.0: Temporal showcase パイプライン ---` コメント行の 1 件のみ）

---

### Step 5: versions/current.md 更新

- `## 進行中バージョン` → `**v79.3.0**（Provenance showcase パイプライン）`
- `## 次に切る版` → `**v79.4.0**（Verifiable showcase パイプライン）`

---

### Step 6: 最終確認

```bash
cargo test v793000 2>&1 | grep -E "test result|FAILED"
cargo test 2>&1 | grep "^test result"
```

3793 tests pass、v793000 2 件 pass を確認。

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
