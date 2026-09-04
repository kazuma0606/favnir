# Roadmap v99.1.0 〜 v100.0.0 — Favnir SAP Platform 1.0

Date: 2026-08-30
Status: 完了（v100.0.0 宣言 2026-09-04）

マスターロードマップ: [roadmap-v95.1-v100.0.md](roadmap-v95.1-v100.0.md)

---

## 前提

- 直前完了: v99.0.0「SAP Analytics 1.0 宣言」（tests = 4,257）
- 本スプリントは SAP Platform Era の第 5 スプリント（最終）
- 目標: v100.0.0「Favnir SAP Platform 1.0 宣言」（tests = 4,279）

### 着手前チェックリスト

- `versions/current.md` の最新安定版が v99.0.0 になっていることを確認する
- `site/content/docs/guides/sap-analytics.mdx` が存在することを確認する（v98.8.0 完了済みの証拠）
- `fav/src/driver.rs` に `mod v99000_tests` が存在することを確認する（v99.0.0 完了済みの証拠）
- `fav/Cargo.toml` の version が `99.0.0` であることを確認する

### スプリントの性格

SAP Platform Era の**エンタープライズ本番対応スプリント（最終章）**。

OAuth2 PKCE / BTP Trust / 監査ログ / Rate Limiting / GDPR マスキング / SLA モニタリングを実装し、
Favnir が「本番 SAP 環境に安心して投入できるプラットフォーム」であることを宣言する。
v100.0.0 は 5 年間の SAP 統合開発の集大成として、Favnir SAP Platform 1.0 を宣言する。

---

## バージョン一覧

| バージョン | 内容 | テスト数 | 状態 |
|---|---|---|---|
| v99.1.0 | OAuth2 PKCE / SAP BTP Trust Configuration（`BtpCredential` 型） | 4257 + 2 = 4259 | 完了 |
| v99.2.0 | `!Audit` エフェクト型 + 監査ログ（`AuditTrail` / `AuditEvent`） | 4259 + 2 = 4261 | 未着手 |
| v99.3.0 | Rate Limiting / Circuit Breaker（`CircuitBreaker<T>` 型） | 4261 + 2 = 4263 | 未着手 |
| v99.4.0 | マルチテナント対応（`TenantContext` / `ctx.sap.for_tenant()`） | 4263 + 2 = 4265 | 未着手 |
| v99.5.0 | GDPR データマスキング（`Masked<T>` 型ラッパー + `unmask` 権限エフェクト） | 4265 + 2 = 4267 | 未着手 |
| v99.6.0 | SLA モニタリング（`SlaDefinition` / `SlaViolation` / `fav sla-check`） | 4267 + 2 = 4269 | 未着手 |
| v99.7.0 | 負荷テスト・総合ベンチマーク（全 5 章横断） | 4269 + 2 = 4271 | 未着手 |
| v99.8.0 | 総合ドキュメント（SAP Platform 完全ガイド / Migration ガイド） | 4271 + 2 = 4273 | 未着手 |
| v99.9.0 | コードフリーズ・最終確認 | 4273 + 2 = 4275 | 未着手 |
| v100.0.0 | Favnir SAP Platform 1.0 宣言 ★大クリーンアップ | 4275 + 4 = 4279 | 未着手 |

---

## v99.1.0 — OAuth2 PKCE / SAP BTP Trust Configuration

本番 SAP 環境への認証を OAuth2 PKCE フローで行う `BtpCredential` 型を追加する。
SAP BTP の Service Key / XSUAA トークン取得を Favnir から自動化する。

```favnir
type BtpCredential = {
    client_id:     String,
    client_secret: String,
    token_url:     String,
    scope:         List<String>
}

type BtpToken = {
    access_token: String,
    expires_in:   Int,
    token_type:   String
}

-- BTP トークン取得
bind token <- ctx.sap_btp.acquire_token(BtpCredential {
    client_id:     Env.get("BTP_CLIENT_ID"),
    client_secret: Env.get("BTP_CLIENT_SECRET"),
    token_url:     Env.get("BTP_TOKEN_URL"),
    scope:         ["API_BUSINESS_PARTNER"]
})
```

**修正ファイル**: `runes/sap-odata/btp_auth.fav`（新規）、`fav/src/driver.rs`

---

## v99.2.0 — `!Audit` エフェクトマーカー + 監査ログ ctx interface

SAP データへのアクセスを監査ログとして記録する `!Audit` エフェクトマーカーと `AuditClient` ctx interface を追加する。
ctx パターンに従い、`AppCtx` に `audit: AuditClient` フィールドとして追加する（`effect Audit { ... }` 宣言は行わない）。

```favnir
type AuditEvent = {
    actor:      String,
    action:     String,
    resource:   String,
    timestamp:  String,
    result:     String       -- "success" | "failure"
}

-- !Audit マーカーを持つ pipeline — 型から監査証跡が必要なことが分かる
-- ctx.audit.log() は AuditClient interface 経由でアクセス
pipeline read_business_partner !SapOData !Audit {
    stage Fetch {
        bind bp <- ctx.sap.business_partner_by_id(partner_id, false)
        bind _  <- ctx.audit.log(AuditEvent {
            actor:     Env.get("CURRENT_USER"),
            action:    "READ",
            resource:  "BusinessPartner/" ++ partner_id,
            timestamp: DateTime.now(),
            result:    "success"
        })
    }
}
```

**修正ファイル**: `runes/sap-odata/audit.fav`（新規）、`runes/ctx/ctx.fav`（`audit` フィールド追加）、`fav/src/driver.rs`
**Rust 側**: `Effect::Audit` を `Effect` enum に追加、`checker.fav` の exhaustive match 更新

---

## v99.3.0 — Rate Limiting / Circuit Breaker

SAP API の過負荷保護と障害時の自動 fallback を `CircuitBreaker<T>` 型で実装する。
`RetryPolicy` 型は v96.8.0 で定義済みであり、`CircuitBreaker<T>` はそれを内包する形で実装する。

```favnir
type CircuitState =
    | Closed       -- 通常動作
    | Open         -- トリップ中（リクエストを遮断）
    | HalfOpen     -- 復旧試行中

-- RetryPolicy は v96.8.0 で定義済み（runes/sap-odata/connection.fav）
type CircuitBreaker<T> = {
    state:            CircuitState,
    failure_count:    Int,
    threshold:        Int,
    reset_timeout_ms: Int,
    retry_policy:     RetryPolicy,    -- v96.8.0 の RetryPolicy を内包
    fallback:         Option<fn() -> T>
}

-- SAP API 呼び出しを Circuit Breaker で保護
bind result <- CircuitBreaker.call(ctx.sap_circuit, fn() {
    ctx.sap.business_partners(filter)
})
```

**修正ファイル**: `runes/sap-odata/resilience.fav`（新規）、`fav/src/driver.rs`

---

## v99.4.0 — マルチテナント対応

SaaS 製品での使用を想定し、テナントごとに SAP 接続を切り替える `TenantContext` を追加する。
`SapEnvironment` 型は v96.1.0 で定義済み（`runes/sap-odata/types.fav`）。

```favnir
type TenantId = String

-- SapEnvironment は v96.1.0 で定義済み（Prd / Qas / Dev / Custom(String)）
type TenantContext = {
    tenant_id: TenantId,
    sap_env:   SapEnvironment,
    schema:    String        -- Snowflake スキーマ等のテナント別リソース
}

-- テナントごとの SAP 接続
bind tenant_ctx <- ctx.sap.for_tenant("CUSTOMER_A")
bind bps        <- tenant_ctx.sap.business_partners(filter)
```

**修正ファイル**: `runes/sap-odata/tenant.fav`（新規）、`runes/ctx/ctx.fav`、`fav/src/driver.rs`

---

## v99.5.0 — GDPR データマスキング

個人情報（PII）を `Masked<T>` 型でラップし、権限なしでは参照できないようにする。
`!Unmask` マーカーと `UnmaskClient` ctx interface を追加する（`effect Unmask { ... }` 宣言は行わない）。

```favnir
-- 純粋な型ラッパー（Rust 実装。effect 宣言なし）
type Masked<T> = { inner: T }

-- PII フィールドをマスク
type BusinessPartnerPii = {
    partner_id: String,
    email:      Masked<String>,    -- GDPR 対象フィールド
    phone:      Masked<String>
}

-- アンマスクは !Unmask マーカーを持つ pipeline のみ可能
-- ctx.unmask.unmask() は UnmaskClient interface 経由でアクセス
pipeline export_with_pii !SapOData !Unmask {
    stage Fetch {
        bind bp    <- ctx.sap.business_partner_by_id(id, false)
        bind email <- ctx.unmask.unmask(bp.email)
    }
}
```

**修正ファイル**: `runes/sap-odata/privacy.fav`（新規）、`runes/ctx/ctx.fav`（`unmask` フィールド追加）、`runes/sap-odata/sap_odata.fav`（re-export）、`fav/src/driver.rs`
**注意（v99.5.0）**: `Effect::Unmask` の Rust `Effect` enum 追加と `checker.fav` exhaustive match 更新は将来バージョンに持ち越し（`effect_catalog.rs` / `checker.rs` / `compiler.rs` / `vm.rs` 波及のため）。

---

## v99.6.0 — SLA モニタリング + `fav sla-check`

SAP API の応答時間 SLA を定義し、違反を検出する `SlaDefinition` 型と `fav sla-check` コマンドを追加する。

```favnir
type SlaDefinition = {
    endpoint:       String,
    max_latency_ms: Int,
    availability:   Float    -- 0.999 = 99.9%
}

type SlaViolation = {
    sla:            SlaDefinition,
    actual_ms:      Int,
    timestamp:      String
}

-- CLI: SLA 準拠チェック
-- $ fav sla-check --config sla.toml --from 2026-08-01 --to 2026-08-31
-- BusinessPartner API: avg 245ms (SLA: 500ms) ✓
-- SalesOrder API:      avg 812ms (SLA: 500ms) ✗ VIOLATION
```

**修正ファイル**: `fav/src/main.rs`、`fav/src/driver.rs`
**注意（v99.6.0）**: `fav sla-check` は `fav sap-mock`（v95.8.0）と同様に `main.rs` にサブコマンドを追加する。
`cmd_sla_check(config: &str, from: &str, to: &str)` 関数として実装し、driver.rs でパスの存在確認テストを追加する。

---

## v99.7.0 — 負荷テスト・総合ベンチマーク

Sprint 1〜5 の全機能を対象に負荷テストと総合ベンチマークを実施する。

**計測対象**:
- `ctx.sap.delta_fetch<BusinessPartner>()` スループット
- `ctx.sap_env("PRD")` 環境切替オーバーヘッド
- `CircuitBreaker.call()` オーバーヘッド
- `Masked<T>` / `unmask()` のコスト
- マルチテナント 100 並列リクエスト時のレイテンシ

**修正ファイル**: `fav/src/driver.rs`、`versions/v95-v100/v99.7.0/benchmark_results.md`（新規）
**注意**: `versions/v95-v100/` ディレクトリを v99.7.0 着手前に作成すること（`mkdir versions/v95-v100/v99.7.0/`）。

---

## v99.8.0 — 総合ドキュメント

SAP Platform 完全ガイドと Migration ガイドを作成する。

**新規作成**:
- `site/content/docs/guides/sap-platform.mdx` — SAP Platform 1.0 全体像
- `site/content/docs/guides/sap-migration.mdx` — v95.0 → v100.0 移行ガイド
- `site/content/docs/guides/sap-enterprise-checklist.mdx` — 本番投入チェックリスト

**修正ファイル**: 上記 3 ファイル（新規）、`fav/src/driver.rs`

---

## v99.9.0 — コードフリーズ・最終確認

- 全テスト通過確認（4,275 tests, 0 failures）
- `cargo clippy --locked -- -D warnings` 通過
- `./target/debug/fav fmt --check self/compiler.fav` 通過
- `./target/debug/fav fmt --check self/checker.fav` 通過
- 全 SAP ガイドドキュメントのリンク切れチェック
- `versions/current.md` の次バージョン欄を v100.0.0 に更新

---

## v100.0.0 — Favnir SAP Platform 1.0 宣言 ★大クリーンアップ

**宣言文**:

> 「Favnir が、SAP のプラットフォームになった。
>
>  `$delta` で差分を受け取り、Event Mesh でリアルタイムに動き、
>  `ctx.sap_env("PRD")` で本番に向き、
>  Snowflake と型安全に JOIN し、
>  `!Approval` で人間の承認を型に閉じ込め、
>  KPI が SAC に流れ、Slack が鳴り、
>  `Masked<T>` が個人情報を守り、
>  `!Audit` が証跡を刻む。
>
>  OAuth2 が認証し、Circuit Breaker が守り、SLA が測る。
>
>  これが、Favnir SAP Platform 1.0 である。
>  SAP と Favnir の 5 年間の旅が、今、完成した。」

**v100000_tests（4 テスト）**:
- `cargo_toml_version_is_100_0_0`
- `changelog_has_v100_0_0`
- `milestone_has_sap_platform`
- `readme_mentions_sap_platform`

**命名規則注意（v100.0.0 初）**: テストモジュール名は `v100000_tests`（6桁）、関数名は `cargo_toml_version_is_100_0_0`。
Rust の識別子として有効だが、初めての 3 桁メジャーバージョンのため実装時に注意すること。

**大クリーンアップ**:
- `cargo clean` 実施（target/ ディレクトリ削除）
- `fav/tmp/hello.fav` 復元確認（cargo clean 後に消えないことを確認）
- 全 SAP ロードマップファイル（v95.1〜v100.0）の Status を「完了」に更新
- `roadmap-v95.1-v100.0.md` の Status を「完了」に更新

---

## スプリント終了時の確認

- [ ] 4,279 tests, 0 failures
- [ ] `cargo clean` を実施する（★大クリーンアップ）
- [ ] `cargo test` で 4,279 tests, 0 failures を再確認する（cargo clean 後）
- [ ] `cargo clippy --locked -- -D warnings` pass
- [ ] `./target/debug/fav fmt --check self/compiler.fav` pass
- [ ] `./target/debug/fav fmt --check self/checker.fav` pass
- [ ] `fav/tmp/hello.fav` が cargo clean 後も存在することを確認する
- [ ] `versions/current.md` を v100.0.0 に更新
- [ ] `MILESTONE.md` に v100.0.0 エントリを追加（SAP Platform 1.0 宣言）
- [ ] `README.md` に `## v100.0 — Favnir SAP Platform 1.0` セクションを追加
- [ ] 全ロードマップファイル（v95.1〜v100.0）の Status を「完了」に更新
- [ ] `roadmap-v95.1-v100.0.md` の Status を「完了」に更新
