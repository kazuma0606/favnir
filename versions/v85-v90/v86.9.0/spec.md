# Spec: v86.9.0 — 安定化・コードフリーズ

## Background

v86.1.0〜v86.8.0 で SAP Master Data Sprint 2 の全機能が揃った。

| バージョン | 内容 |
|---|---|
| v86.1.0 | BusinessPartner 型定義 |
| v86.2.0 | business_partners() フィルタ検索 |
| v86.3.0 | business_partner_by_id() 単件取得 |
| v86.4.0 | create_business_partner() 作成 |
| v86.5.0 | update_business_partner() 更新 |
| v86.6.0 | E2E パイプライン（BusinessPartner → S3） |
| v86.7.0 | CRUD テスト（sap_odata.test.fav） |
| v86.8.0 | Rune Registry デプロイ（rune.toml v86.8.0） |

v86.9.0 は安定化スプリント。新機能追加は行わず、全機能を通しで確認する。

また v86.8.0 で発覚した `rune-registry/src/main.fav` の `!Effect` 注釈構文問題（E0374）も
このスプリントで修正済みとして記録する。

## Goals

1. `cargo test` が 3971 tests, 0 failures で完全 pass することを確認する
2. BusinessPartner CRUD 全操作（Create / Read / Update / List）がスタブとして定義されていることを Rust テストで確認する
3. シナリオ 1 パイプライン（`infra/e2e-demo/sap-odata/pipeline.fav`）の存在を確認する
4. `import rune "sap-odata"` の動作確認（v86.8.0 でデプロイ済みの Rune Registry 経由）— API Gateway `/runes/sap-odata` が HTTP 200 を返すことを curl で手動確認

## Scope

### Rust テスト（`mod v86900_tests`）

- `sap_master_data_business_partner_crud_covered`: `business_partner.fav` に Create / Read / Update / List の 4 関数が定義されていることを確認
- `sap_master_data_scenario1_pipeline_exists`: `infra/e2e-demo/sap-odata/pipeline.fav` が存在することを確認

### 安定化チェックリスト

- `cargo test` 3971 tests, 0 failures
- `cargo clippy --locked -- -D warnings` pass
- `fav fmt --check` pass（compiler.fav / checker.fav）
- `rune-registry/src/main.fav` が `fav run --legacy` で正常動作すること（v86.8.0 修正済み）

## Files to Modify

| ファイル | 操作 |
|---|---|
| `CHANGELOG.md` | v86.9.0 エントリ追加（先頭） |
| `fav/src/driver.rs` | `mod v86900_tests` 追加 |

## Success Criteria

- `cargo test 2>&1 | grep "test result"` が 3971 tests, 0 failures を出力する
- `sap_master_data_business_partner_crud_covered` が pass する
- `sap_master_data_scenario1_pipeline_exists` が pass する
- `curl -s -H "x-fav-token: fav-registry-v1-dk9p2mxw4qhz" https://32qp3qwhdh.execute-api.ap-northeast-1.amazonaws.com/runes/sap-odata` が HTTP 200 + `"name":"sap-odata"` を含むレスポンスを返す
