# Plan: v96.4.0 — SAP → Snowflake リアルタイム同期

## 実装ステップ

### Step 1: `infra/e2e-demo/sap-odata/pipeline_snowflake_sync.fav` を新規作成

`pipeline_export.fav` と同スタイルで作成する。

```favnir
-- SAP → Snowflake リアルタイム同期パイプライン（v96.4.0）
-- BusinessPartner エンティティを SAP OData から取得し、Snowflake テーブルにロードする。
-- v11.0 Snowflake 統合（ctx.snowflake.execute_raw）と接続する。
import rune "sap-odata"
import rune "snowflake"

-- BusinessPartner を Snowflake 行形式（JSON 文字列）に変換するヘルパー
fn bp_to_snowflake_row(bp: BusinessPartner) -> String {
    Json.encode(bp)
}

pipeline sync_bp_to_snowflake !SapOData !Snowflake {
    stage Fetch {
        bind bps <- ctx.sap.business_partners(BusinessPartnerFilter {
            country:       Option.some("JP"),
            category:      Option.none(),
            changed_after: Option.none(),
            top:           Option.some(500)
        })
    }
    |> stage Load {
        bind rows <- List.map(bps, fn(bp) { bp_to_snowflake_row(bp) })
        bind _    <- ctx.snowflake.execute_raw(
            "INSERT INTO SAP_BUSINESS_PARTNERS SELECT * FROM VALUES ?",
            rows
        )
    }
}
```

### Step 2: `fav/src/driver.rs` に `mod v96400_tests` を追加

`mod v96300_tests` の直後に追加する。

```rust
#[cfg(test)]
mod v96400_tests {
    #[test]
    fn pipeline_snowflake_sync_fav_exists() {
        let content = std::fs::read_to_string(
            "../infra/e2e-demo/sap-odata/pipeline_snowflake_sync.fav",
        )
        .expect("pipeline_snowflake_sync.fav should exist");
        assert!(
            content.contains("sync_bp_to_snowflake"),
            "pipeline_snowflake_sync.fav should define sync_bp_to_snowflake pipeline"
        );
    }

    #[test]
    fn pipeline_snowflake_sync_uses_execute_raw() {
        let content = std::fs::read_to_string(
            "../infra/e2e-demo/sap-odata/pipeline_snowflake_sync.fav",
        )
        .expect("pipeline_snowflake_sync.fav should exist");
        assert!(
            content.contains("execute_raw"),
            "pipeline_snowflake_sync.fav should use execute_raw"
        );
    }

    #[test]
    fn pipeline_snowflake_sync_defines_bp_to_snowflake_row() {
        let content = std::fs::read_to_string(
            "../infra/e2e-demo/sap-odata/pipeline_snowflake_sync.fav",
        )
        .expect("pipeline_snowflake_sync.fav should exist");
        assert!(
            content.contains("bp_to_snowflake_row"),
            "pipeline_snowflake_sync.fav should define bp_to_snowflake_row helper"
        );
    }
}
```

## 依存関係

```
Step 1 (pipeline_snowflake_sync.fav 新規作成) → Step 2 (driver.rs テスト)
```
