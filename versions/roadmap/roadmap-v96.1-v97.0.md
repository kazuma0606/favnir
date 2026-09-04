# Roadmap v96.1.0 〜 v97.0.0 — SAP Multi-system 1.0

Date: 2026-08-30
Status: 未着手

マスターロードマップ: [roadmap-v95.1-v100.0.md](roadmap-v95.1-v100.0.md)

---

## 前提

- 直前完了: v96.0.0「SAP Real-time 1.0 宣言」（tests = 4,188）
- 本スプリントは SAP Platform Era の第 2 スプリント
- 目標: v97.0.0「SAP Multi-system 1.0 宣言」（tests = 4,211）

### 着手前チェックリスト

- `versions/current.md` の最新安定版が v96.0.0 になっていることを確認する
- `runes/sap-odata/event_mesh.fav` が存在することを確認する（v95.3.0 完了済みの証拠）
- `fav/src/driver.rs` に `mod v96000_tests` が存在することを確認する（v96.0.0 完了済みの証拠）
- `fav/Cargo.toml` の version が `96.0.0` であることを確認する

### スプリントの性格

SAP Platform Era の**マルチシステム・クロスプラットフォームスプリント**。

本番 SAP 環境では PRD / QAS / DEV の複数環境が当たり前。
型安全な環境切替（`ctx.sap_env("PRD")`）を実装し、
さらに Snowflake / DuckDB とのクロスシステム型安全 JOIN で
「SAP + データレイク」を Favnir pipeline で統合する。

---

## バージョン一覧

| バージョン | 内容 | テスト数 | 状態 |
|---|---|---|---|
| v96.1.0 | `SapEnvironment` 型（`PRD` / `QAS` / `DEV`）+ `ctx.sap_env("PRD")` | 4188 + 2 = 4190 | 未着手 |
| v96.2.0 | `fav.toml [sap.environments]` マルチ環境設定 | 4190 + 2 = 4192 | 未着手 |
| v96.3.0 | SAP → Parquet / DuckDB エクスポートパイプライン | 4192 + 2 = 4194 | 未着手 |
| v96.4.0 | SAP → Snowflake リアルタイム同期（v11.0 Snowflake 統合と接続） | 4194 + 3 = 4197 | 未着手 |
| v96.5.0 | カスタム OData サービス対応（任意メタデータから型生成・`fav infer` 拡張） | 4197 + 4 = 4201 | 未着手 |
| v96.6.0 | S/4HANA Clean Core REST API wrapper（`CleanCoreClient`） | 4201 + 2 = 4203 | 未着手 |
| v96.7.0 | Cross-system 型安全 JOIN（SAP エンティティ × Snowflake テーブル） | 4203 + 2 = 4205 | 未着手 |
| v96.8.0 | 接続プール / キャッシュ / リトライ（`RetryPolicy` 型） | 4205 + 2 = 4207 | 未着手 |
| v96.9.0 | 安定化・コードフリーズ | 4207 + 2 = 4209 | 未着手 |
| v97.0.0 | SAP Multi-system 1.0 宣言 ★クリーンアップ | 4209 + 4 = 4213 | 未着手 |

---

## v96.1.0 — `SapEnvironment` 型 + `ctx.sap_env()`

PRD / QAS / DEV を型で表現し、pipeline 内で安全に切り替えられるようにする。

```favnir
type SapEnvironment =
    | Prd
    | Qas
    | Dev
    | Custom(String)

-- AppCtx に複数の SapClient を保持
bind sap_prd <- ctx.sap_env("PRD")
bind bps     <- sap_prd.business_partners(filter)
```

**修正ファイル**: `runes/sap-odata/types.fav`、`runes/ctx/ctx.fav`、`fav/src/driver.rs`

---

## v96.2.0 — `fav.toml [sap.environments]` マルチ環境設定

`fav.toml` に複数 SAP 環境の接続設定を記述できるようにする。

```toml
[sap.environments.PRD]
base_url    = "${SAP_PRD_URL}"
client      = "100"
username    = "${SAP_PRD_USER}"
password    = "${SAP_PRD_PASS}"

[sap.environments.QAS]
base_url    = "${SAP_QAS_URL}"
client      = "200"
username    = "${SAP_QAS_USER}"
password    = "${SAP_QAS_PASS}"
```

**修正ファイル**: `fav/src/toml.rs`、`fav/src/driver.rs`

---

## v96.3.0 — SAP → Parquet / DuckDB エクスポートパイプライン

SAP エンティティを Parquet ファイルに書き出し、DuckDB で分析する pipeline を実装する。

```favnir
pipeline export_bp_to_parquet !SapOData !Io {
    stage Fetch {
        bind bps <- ctx.sap.business_partners(BusinessPartnerFilter {
            country: Option.some("JP"), category: Option.none(),
            changed_after: Option.none(), top: Option.some(1000)
        })
    }
    |> stage Write {
        bind _ <- ctx.io.write_parquet("output/business_partners.parquet", bps)
    }
}
```

**修正ファイル**: `infra/e2e-demo/sap-odata/pipeline_export.fav`（新規）、`fav/src/driver.rs`

---

## v96.4.0 — SAP → Snowflake リアルタイム同期

v11.0 の Snowflake 統合と接続し、SAP データを Snowflake に直接ロードする pipeline を実装する。

```favnir
pipeline sync_bp_to_snowflake !SapOData !Snowflake {
    stage Fetch {
        bind bps <- ctx.sap.business_partners(filter)
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

**修正ファイル**: `infra/e2e-demo/sap-odata/pipeline_snowflake_sync.fav`（新規）、`fav/src/driver.rs`

---

## v96.5.0 — カスタム OData サービス対応

標準 S/4HANA サービス以外のカスタム OData サービスにも `fav infer` で型生成できるようにする。
`--sap-service-name` フラグでカスタムサービスのエンドポイント名を指定できる。

```
$ fav infer --from sap \
    --sap-metadata https://my-sap/sap/opu/odata/sap/ZMY_CUSTOM_SRV/$metadata \
    --sap-service-name ZMY_CUSTOM_SRV \
    --output runes/sap-odata/custom_service.fav
```

**修正ファイル**: `fav/src/sap_metadata.rs`、`fav/src/main.rs`、`fav/src/driver.rs`

---

## v96.6.0 — S/4HANA Clean Core REST API wrapper

SAP S/4HANA Cloud の新しい "Clean Core" REST API（OData ではなく JSON REST）に対応する
`CleanCoreClient` 型を追加する。

```favnir
type CleanCoreClient = {
    base_url: String,
    token:    String
}

-- Clean Core API 呼び出し
bind result <- ctx.sap_clean_core.get<BusinessPartnerV2>(
    "/API_BUSINESS_PARTNER/A_BusinessPartner('BP001')"
)
```

**修正ファイル**: `runes/sap-odata/clean_core.fav`（新規）、`fav/src/driver.rs`

---

## v96.7.0 — Cross-system 型安全 JOIN

SAP エンティティと Snowflake テーブルを Favnir の型で JOIN する。

```favnir
type SapSnowflakeJoin<A, B> = {
    sap_entity:       A,
    snowflake_record: B,
    join_key:         String
}

-- SAP BusinessPartner × Snowflake CRM テーブルを partner_id で JOIN
bind joined <- CrossSystem.join<BusinessPartner, CrmRecord>(
    bps, crm_records,
    fn(bp) { bp.partner_id },
    fn(crm) { crm.sap_id }
)
```

**修正ファイル**: `runes/sap-odata/cross_system.fav`（新規）、`fav/src/driver.rs`

---

## v96.8.0 — 接続プール / キャッシュ / リトライ

本番運用に必要な接続管理機能を `RetryPolicy` 型として追加する。

```favnir
type RetryPolicy = {
    max_attempts:    Int,
    backoff_ms:      Int,
    retry_on_status: List<Int>   -- 503, 429 など
}

type SapConnectionPool = {
    pool_size:    Int,
    timeout_ms:   Int,
    retry_policy: RetryPolicy
}
```

**修正ファイル**: `runes/sap-odata/connection.fav`（新規）、`fav/src/driver.rs`

---

## v96.9.0 — 安定化・コードフリーズ

- 全テスト通過確認（4,209 tests, 0 failures）
- `cargo clippy --locked -- -D warnings` 通過
- `./target/debug/fav fmt --check self/compiler.fav` 通過
- `./target/debug/fav fmt --check self/checker.fav` 通過

---

## v97.0.0 — SAP Multi-system 1.0 宣言

**宣言文**:

> 「Favnir が、SAP の境界を越えた。
>
>  `ctx.sap_env("PRD")` で本番に向き、
>  SAP のデータが Snowflake に流れ、
>  カスタムサービスの型も `fav infer` が生み出す。
>
>  それが、Favnir SAP Multi-system 1.0 である。」

**v97000_tests（4 テスト）**:
- `cargo_toml_version_is_97_0_0`
- `changelog_has_v97_0_0`
- `milestone_has_sap_multi_system`
- `readme_mentions_sap_multi_system`

---

## スプリント終了時の確認

- [ ] 4,213 tests, 0 failures
- [ ] `cargo clean` を実施する（★クリーンアップ）
- [ ] `cargo test` で 4,213 tests, 0 failures を再確認する（cargo clean 後）
- [ ] `cargo clippy --locked -- -D warnings` pass
- [ ] `./target/debug/fav fmt --check self/compiler.fav` pass
- [ ] `./target/debug/fav fmt --check self/checker.fav` pass
- [ ] `versions/current.md` を v97.0.0 に更新（テスト数 4,213）
- [ ] `MILESTONE.md` に v97.0.0 エントリを追加
- [ ] `README.md` に `## v97.0 — SAP Multi-system 1.0` セクションを追加
