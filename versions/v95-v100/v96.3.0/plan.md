# Plan: v96.3.0 — SAP → Parquet エクスポートパイプライン

## 実装ステップ

### Step 1: `infra/e2e-demo/sap-odata/pipeline_export.fav` を新規作成

既存の `pipeline.fav` / `pipeline_realtime.fav` と同スタイルで作成する。

```favnir
-- SAP → Parquet エクスポートパイプライン（v96.3.0）
-- BusinessPartner エンティティを SAP OData から取得し、Parquet ファイルに書き出す。
-- v96.3.0: ctx.sap.* / ctx.io.write_parquet スタイル（cfg 明示取得なし）
import rune "sap-odata"

pipeline export_bp_to_parquet !SapOData !Io {
    stage Fetch {
        bind bps <- ctx.sap.business_partners(BusinessPartnerFilter {
            country:       Option.some("JP"),
            category:      Option.none(),
            changed_after: Option.none(),
            top:           Option.some(1000)
        })
    }
    |> stage Write {
        bind _ <- ctx.io.write_parquet("output/business_partners.parquet", bps)
    }
}
```

### Step 2: `fav/src/driver.rs` に `mod v96300_tests` を追加

`mod v96200_tests` の直後に追加する。

```rust
#[cfg(test)]
mod v96300_tests {
    #[test]
    fn pipeline_export_fav_exists() {
        let content = std::fs::read_to_string(
            "../infra/e2e-demo/sap-odata/pipeline_export.fav",
        )
        .expect("pipeline_export.fav should exist");
        assert!(
            content.contains("export_bp_to_parquet"),
            "pipeline_export.fav should define export_bp_to_parquet pipeline"
        );
    }

    #[test]
    fn pipeline_export_uses_write_parquet() {
        let content = std::fs::read_to_string(
            "../infra/e2e-demo/sap-odata/pipeline_export.fav",
        )
        .expect("pipeline_export.fav should exist");
        assert!(
            content.contains("write_parquet"),
            "pipeline_export.fav should use write_parquet"
        );
    }
}
```

## 依存関係

```
Step 1 (pipeline_export.fav 新規作成) → Step 2 (driver.rs テスト)
```
