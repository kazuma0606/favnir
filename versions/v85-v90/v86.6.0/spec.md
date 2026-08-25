# Spec: v86.6.0 — シナリオ 1: マスタデータ同期（BusinessPartner → S3）

## Background

v86.5.0 で BusinessPartner CRUD の全スタブ関数を定義した。
v86.6.0 では業務シナリオ 1 の E2E パイプラインを実装する。
SAP の得意先マスタを一覧取得し、JSON として S3 に同期する。

**注意**: v86.6.0 時点では Rune Registry デプロイ（v86.8.0）前のため、
ローカル Rune ファイルを直接参照する形式で実装する。
v86.8.0 以降は `import rune "sap-odata"` に切り替える。

## Goals

- `infra/e2e-demo/sap-odata/pipeline.fav` に `sync_business_partners()` 関数を実装する
- `import rune "s3"` + `sap_odata` 関数の組み合わせで E2E フローを記述する
- driver.rs に `mod v86600_tests`（2 件）を追加し、3963 → 3965 tests とする

## パイプライン実装（Favnir 構文）

```favnir
-- infra/e2e-demo/sap-odata/pipeline.fav
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

## ファイル構成

| ファイル | 変更 |
|---|---|
| `infra/e2e-demo/sap-odata/pipeline.fav` | 新規作成（`sync_business_partners()` 関数） |
| `fav/src/driver.rs` | `mod v86600_tests` 追加（2 テスト） |
| `CHANGELOG.md` | v86.6.0 エントリ追加 |

## Success Criteria

- `infra/e2e-demo/sap-odata/pipeline.fav` が存在する
- `infra/e2e-demo/sap-odata/pipeline.fav` が `sync_business_partners` を含む
- `cargo test 2>&1 | grep "test result"` → 3965 tests, 0 failures

## テスト詳細

```rust
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
```
