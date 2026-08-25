# Plan: v86.6.0 — シナリオ 1: マスタデータ同期（BusinessPartner → S3）

## 実装ステップ

### Step 1: `CHANGELOG.md` に v86.6.0 エントリ追加

v86.5.0 エントリの直前（先頭）に v86.6.0 エントリを追加する。

### Step 2: `infra/e2e-demo/sap-odata/pipeline.fav` を新規作成

`infra/e2e-demo/sap-odata/` ディレクトリに `pipeline.fav` を作成する。
（ディレクトリは v85.9.0 で `docker-compose.yml` 作成時に存在済み）

**前提**: v86.5.0 で定義済みの `sap_odata.sap_config_from_env` / `sap_odata.business_partners` 等のスタブ関数を参照する。

```favnir
-- シナリオ 1: マスタデータ同期（BusinessPartner → S3）（v86.6.0）
-- 注: v86.6.0 時点では Registry デプロイ（v86.8.0）前のため、
--     ローカル Rune ファイルを直接参照する。
--     v86.8.0 以降は import rune "sap-odata" に切り替える。

import rune "s3"

fn sync_business_partners(ctx: AppCtx) -> Result<Int, String> {
    bind cfg      <- sap_odata.sap_config_from_env()
    bind partners <- sap_odata.business_partners(cfg, BusinessPartnerFilter {
        country:       Option.some("JP"),
        changed_after: Option.some("2026-08-01"),
        top:           Option.some(500),
        category:      Option.none()
    })
    bind json     <- Json.encode(partners)
    bind _        <- ctx.s3.put_object("favnir-sap-demo", "partners/latest.json", json)
    Result.ok(List.length(partners))
}
```

### Step 3: `fav/src/driver.rs` に `mod v86600_tests` 追加

`mod v86500_tests { ... }` の直後に追加する。

```rust
// use super::* 不要（std::path::Path / std::fs::read_to_string のみ使用）
#[cfg(test)]
mod v86600_tests {
    #[test]
    fn sap_e2e_pipeline_fav_exists() {
        let path = std::path::Path::new("../infra/e2e-demo/sap-odata/pipeline.fav");
        assert!(path.exists(), "infra/e2e-demo/sap-odata/pipeline.fav should exist");
    }

    #[test]
    fn sap_e2e_pipeline_contains_sync_business_partners() {
        let content = std::fs::read_to_string("../infra/e2e-demo/sap-odata/pipeline.fav")
            .expect("infra/e2e-demo/sap-odata/pipeline.fav should exist");
        assert!(
            content.contains("sync_business_partners"),
            "pipeline.fav should define sync_business_partners function"
        );
    }
}
```

### Step 4: `cargo test` で全 pass 確認

```
cargo test 2>&1 | grep "test result"
```

期待: `3965 tests, 0 failures`
