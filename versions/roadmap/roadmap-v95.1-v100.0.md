# Favnir ロードマップ v95.1〜v100.0 — SAP Platform Era

Date: 2026-08-30
Status: 完了（v100.0.0 宣言 2026-09-04）

---

## 背景と方針

v95.0.0「SAP Advanced 1.0」をもって、SAP Advanced Era が完成した。
`$batch` / `QueryBuilder<T>` / `fav infer --sap-metadata` / Lambda SnapStart の 4 柱が揃い、
Favnir は「SAP を型安全に操作できる言語」として完成した。

しかし SAP Advanced Era には**3 つの未解決課題**が残った。

**課題 1 — リアルタイム連携の欠落**
現状はすべてポーリング型。SAP 側でデータが変わっても Favnir は「次回実行まで」気づけない。
OData `$delta`（差分リンク）/ SAP Event Mesh（AMQP イベント受信）が未実装。

**課題 2 — マルチシステム・クロスプラットフォームの壁**
PRD / QAS / DEV 環境の切り替えが `SapConfig` 手動差し替えのみ。
SAP + Snowflake / DuckDB をまたぐ型安全なクロスシステムクエリもない。

**課題 3 — エンタープライズ本番対応の不足**
OAuth2 PKCE / 監査ログ / Rate Limiting / GDPR マスキング / SLA モニタリングが未整備。
本番 SAP 環境への投入には信頼性・セキュリティ層が必要。

v95.1〜v100.0 では、これら 3 課題を段階的に解決し、
Favnir を「SAP エンタープライズ本番対応プラットフォーム」として宣言する。

```
v96.0 — SAP Real-time 1.0    : 「$delta / Event Mesh でリアルタイム同期できる」
v97.0 — SAP Multi-system 1.0 : 「PRD/QAS/DEV 切替・Snowflake クロスシステムが動く」
v98.0 — SAP Workflow 1.0     : 「承認フロー・BTP Integration が Favnir から起動できる」
v99.0 — SAP Analytics 1.0    : 「SAC/BW へのレポート自動生成・KPI モニタリングが動く」
v100.0 — SAP Platform 1.0    : 「OAuth2・監査・GDPR・SLA — 本番エンタープライズ対応完成」
```

### 設計方針

- **ctx パターン統一（必須）**: v91.0 で確立した ctx パターンを本スプリント全体で徹底する。
  新機能はすべて `AppCtx` の interface フィールドとして追加し、`ctx.xxx.*` 形式でアクセスする。
  `effect X { ... }` による独立エフェクト宣言は行わない。
  pipeline シグネチャのエフェクトマーカー（`!SapOData` / `!SapEvent` 等）は
  ctx interface の「型アノテーション」として機能し、Rust の `Effect` enum への追加を伴う。
- **SAP Real-time**: `ctx.sap_event.*` — SAP Event Mesh への接続を ctx interface として追加。
  pipeline シグネチャは `!SapEvent` マーカーで宣言し、Rust の `Effect::SapEvent` を追加する。
- **Multi-system**: `ctx.sap_env("PRD")` / `ctx.sap_env("QAS")` — 環境を型安全に切り替える
- **Cross-system JOIN**: SAP エンティティと Snowflake テーブルを Favnir の型安全 JOIN で結合する
- **Workflow**: `ctx.approval.*` — 承認フローを ctx interface として追加。
  pipeline シグネチャは `!Approval` マーカーで宣言し、Rust の `Effect::Approval` を追加する。
  「人間の承認が必要」という意図を型シグネチャで表現する。
- **Analytics**: `KpiDefinition<T>` / `BwQuery<T>` — レポートを型で設計する
- **Enterprise**: `ctx.audit.*` / `ctx.unmask.*` — 監査・アンマスクも ctx interface として追加。
  `Masked<T>` 型ラッパー / `RetryPolicy` / `CircuitBreaker<T>` は純粋な Favnir 型として実装。
  `RetryPolicy` は v96.8.0 で定義し、v99.3.0 の `CircuitBreaker<T>` がそれを内包する。
- **Rust 側実装（各エフェクト追加時）**: `Effect` enum への追加（`effect_catalog.rs` 相当）、
  `checker.fav` の exhaustive match 対応、WASM ビルドの cfg guard 確認を各スプリントで行う。

### 現状評価（v95.0.0 時点）

| カテゴリ | 状態 | 評価 |
|---|---|---|
| BusinessPartner / SalesOrder / Material / PurchaseOrder / JournalEntry CRUD | 完成 | ★★★★★ |
| OData `$batch` / `QueryBuilder<T>` | 完成 | ★★★★★ |
| `fav infer --sap-metadata` | 完成 | ★★★★☆ |
| AppCtx 統合（`ctx.sap.*`） | 完成 | ★★★★★ |
| Lambda SnapStart / `fav bench --sap` | 完成 | ★★★★☆ |
| リアルタイム連携（$delta / Event Mesh） | **未実装** | ★☆☆☆☆ |
| マルチシステム（PRD/QAS/DEV 切替） | **未実装** | ★☆☆☆☆ |
| ワークフロー / 承認フロー | **未実装** | ★☆☆☆☆ |
| Analytics / BI 連携 | 基本集計のみ | ★★☆☆☆ |
| セキュリティ / 監査 / GDPR | **未実装** | ★☆☆☆☆ |

---

## テスト数推移（本スプリント全体）

| バージョン | テスト数 | 増加 |
|---|---|---|
| v95.0.0（ベース） | 4,164 | — |
| v95.1.0〜v95.9.0 | +2 × 9 = +18 | 4,182 |
| v96.0.0（宣言） | +4 | 4,186 |
| v96.1.0〜v96.9.0 | +2 × 9 = +18 | 4,204 |
| v97.0.0（宣言） | +4 | 4,208 |
| v97.1.0〜v97.9.0 | +2 × 9 = +18 | 4,226 |
| v98.0.0（宣言） | +4 | 4,230 |
| v98.1.0〜v98.9.0 | +2 × 9 = +18 | 4,248 |
| v99.0.0（宣言） | +4 | 4,252 |
| v99.1.0〜v99.9.0 | +2 × 9 = +18 | 4,270 |
| v100.0.0（宣言） | +4 | 4,274 |

**本スプリント合計**: +110 tests（4,164 → 4,274）

---

## ━━━━━━━━━━━━━━━━━━━━━━━━━━
## Sprint 1: SAP Real-time（v95.1〜v96.0）
## ━━━━━━━━━━━━━━━━━━━━━━━━━━

**テーマ**: OData `$delta` による差分同期 + SAP Event Mesh によるイベント駆動 pipeline。

| バージョン | 内容 | テスト数 | 状態 |
|---|---|---|---|
| v95.1.0 | OData `$delta` / `DeltaLink` 型定義（差分取得基盤） | 4164 + 2 = 4166 | 未着手 |
| v95.2.0 | `ctx.sap.delta<T>()` — 前回以降の差分エンティティ取得 | 4166 + 2 = 4168 | 未着手 |
| v95.3.0 | SAP Event Mesh 接続基盤（AMQP クライアント / `!SapEvent` エフェクト） | 4168 + 2 = 4170 | 未着手 |
| v95.4.0 | イベント駆動 pipeline（`on_event` stage トリガー）| 4170 + 2 = 4172 | 未着手 |
| v95.5.0 | Deep Insert（SalesOrder + Items を 1 リクエストで作成） | 4172 + 2 = 4174 | 未着手 |
| v95.6.0 | Function Import / Action Import（OData RPC スタイル） | 4174 + 2 = 4176 | 未着手 |
| v95.7.0 | バッチ部分失敗ハンドリング（`PartialSuccess<T>` / `BatchItemResult<T>`） | 4176 + 2 = 4178 | 未着手 |
| v95.8.0 | `fav sap-mock`（オフラインテスト用モックサーバー） | 4178 + 2 = 4180 | 未着手 |
| v95.9.0 | 安定化・コードフリーズ | 4180 + 2 = 4182 | 未着手 |
| v96.0.0 | SAP Real-time 1.0 宣言 ★クリーンアップ | 4182 + 4 = 4186 | 未着手 |

詳細: [roadmap-v95.1-v96.0.md](roadmap-v95.1-v96.0.md)

---

## ━━━━━━━━━━━━━━━━━━━━━━━━━━
## Sprint 2: SAP Multi-system（v96.1〜v97.0）
## ━━━━━━━━━━━━━━━━━━━━━━━━━━

**テーマ**: PRD / QAS / DEV 環境切替 + Snowflake / DuckDB クロスシステム連携。

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

詳細: [roadmap-v96.1-v97.0.md](roadmap-v96.1-v97.0.md)

---

## ━━━━━━━━━━━━━━━━━━━━━━━━━━
## Sprint 3: SAP Workflow（v97.1〜v98.0）
## ━━━━━━━━━━━━━━━━━━━━━━━━━━

**テーマ**: SAP Workflow Management / BTP Integration Suite — 承認フローを Favnir pipeline で制御する。

| バージョン | 内容 | テスト数 | 状態 |
|---|---|---|---|
| v97.1.0 | `WorkflowInstance` 型 + `ctx.sap.workflow_start()` | 4208 + 2 = 4210 | 未着手 |
| v97.2.0 | タスク照会 `ctx.sap.workflow_tasks()` + 完了操作 `workflow_task_complete()` | 4210 + 2 = 4212 | 未着手 |
| v97.3.0 | `!Approval` エフェクト型（人間の承認を型で表現） | 4212 + 2 = 4214 | 未着手 |
| v97.4.0 | 条件分岐 pipeline（ワークフロー結果に基づく `match` stage） | 4214 + 2 = 4216 | 未着手 |
| v97.5.0 | SAP BTP Integration Suite connector（`iFlowClient`） | 4216 + 2 = 4218 | 未着手 |
| v97.6.0 | E2E デモ（発注 → 自動承認ルーティング → SAP 反映） | 4218 + 2 = 4220 | 未着手 |
| v97.7.0 | `MockWorkflowClient`（承認フローのオフラインテスト） | 4220 + 2 = 4222 | 未着手 |
| v97.8.0 | サイトドキュメント（Workflow / Approval パターンガイド） | 4222 + 2 = 4224 | 未着手 |
| v97.9.0 | 安定化・コードフリーズ | 4224 + 2 = 4226 | 未着手 |
| v98.0.0 | SAP Workflow 1.0 宣言 ★クリーンアップ | 4226 + 4 = 4230 | 未着手 |

詳細: [roadmap-v97.1-v98.0.md](roadmap-v97.1-v98.0.md)

---

## ━━━━━━━━━━━━━━━━━━━━━━━━━━
## Sprint 4: SAP Analytics（v98.1〜v99.0）
## ━━━━━━━━━━━━━━━━━━━━━━━━━━

**テーマ**: SAP Analytics Cloud / BW/4HANA — レポート・KPI モニタリングを Favnir pipeline で自動化する。

| バージョン | 内容 | テスト数 | 状態 |
|---|---|---|---|
| v98.1.0 | `KpiDefinition<T>` / `KpiSnapshot<T>` 型定義 | 4230 + 2 = 4232 | 未着手 |
| v98.2.0 | `BwQuery<T>` 型 + `ctx.sap.bw_query()` — BW/4HANA クエリインターフェース | 4232 + 2 = 4234 | 未着手 |
| v98.3.0 | SAP Analytics Cloud データプッシュ API（`ctx.sap.sac_push()`） | 4234 + 2 = 4236 | 未着手 |
| v98.4.0 | レポート自動生成 pipeline（Favnir → SAC ダッシュボードデータ） | 4236 + 2 = 4238 | 未着手 |
| v98.5.0 | KPI 閾値アラート（`KpiAlert` 型 + Slack / メール通知 pipeline） | 4238 + 2 = 4240 | 未着手 |
| v98.6.0 | `fav report --sap`（ローカル HTML レポート生成コマンド） | 4240 + 2 = 4242 | 未着手 |
| v98.7.0 | E2E デモ（日次売上 KPI → SAC プッシュ → Slack アラート） | 4242 + 2 = 4244 | 未着手 |
| v98.8.0 | サイトドキュメント（Analytics / KPI パターンガイド） | 4244 + 2 = 4246 | 未着手 |
| v98.9.0 | 安定化・コードフリーズ | 4246 + 2 = 4248 | 未着手 |
| v99.0.0 | SAP Analytics 1.0 宣言 ★クリーンアップ | 4248 + 4 = 4252 | 未着手 |

詳細: [roadmap-v98.1-v99.0.md](roadmap-v98.1-v99.0.md)

---

## ━━━━━━━━━━━━━━━━━━━━━━━━━━
## Sprint 5: SAP Platform 1.0 宣言（v99.1〜v100.0）
## ━━━━━━━━━━━━━━━━━━━━━━━━━━

**テーマ**: セキュリティ / 監査 / GDPR / SLA — エンタープライズ本番対応の完成。

| バージョン | 内容 | テスト数 | 状態 |
|---|---|---|---|
| v99.1.0 | OAuth2 PKCE / SAP BTP Trust Configuration（`BtpCredential` 型） | 4257 + 2 = 4259 | 完了 |
| v99.2.0 | `!Audit` エフェクト型 + 監査ログ（`AuditTrail` / `AuditEvent`） | 4259 + 2 = 4261 | 完了 |
| v99.3.0 | Rate Limiting / Circuit Breaker（`CircuitBreaker<T>` 型） | 4261 + 2 = 4263 | 完了 |
| v99.4.0 | マルチテナント対応（`TenantContext` / `ctx.sap.for_tenant()`） | 4263 + 2 = 4265 | 完了 |
| v99.5.0 | GDPR データマスキング（`Masked<T>` 型ラッパー + `unmask` 権限エフェクト） | 4265 + 2 = 4267 | 完了 |
| v99.6.0 | SLA モニタリング（`SlaDefinition` / `SlaViolation` / `fav sla-check`） | 4267 + 2 = 4269 | 完了 |
| v99.7.0 | 負荷テスト・総合ベンチマーク（全 5 章横断） | 4269 + 2 = 4271 | 未着手 |
| v99.8.0 | 総合ドキュメント（SAP Platform 完全ガイド / Migration ガイド） | 4271 + 2 = 4273 | 未着手 |
| v99.9.0 | コードフリーズ・最終確認 | 4273 + 2 = 4275 | 未着手 |
| v100.0.0 | Favnir SAP Platform 1.0 宣言 ★大クリーンアップ | 4275 + 4 = 4279 | 未着手 |

詳細: [roadmap-v99.1-v100.0.md](roadmap-v99.1-v100.0.md)

---

## スプリント総括

| スプリント | バージョン範囲 | テーマ | 宣言バージョン | テスト累計 |
|---|---|---|---|---|
| Sprint 1 | v95.1〜v96.0 | SAP Real-time（$delta / Event Mesh / Deep Insert / sap-mock） | v96.0.0 | 4,186 |
| Sprint 2 | v96.1〜v97.0 | SAP Multi-system（PRD/QAS/DEV / Snowflake JOIN / Clean Core） | v97.0.0 | 4,213 |
| Sprint 3 | v97.1〜v98.0 | SAP Workflow（承認フロー / BTP Integration / !Approval エフェクト） | v98.0.0 | 4,235 |
| Sprint 4 | v98.1〜v99.0 | SAP Analytics（SAC / BW / KPI / fav report） | v99.0.0 | 4,252 |
| Sprint 5 | v99.1〜v100.0 | SAP Platform 1.0（OAuth2 / Audit / GDPR / SLA / マルチテナント） | v100.0.0 | 4,274 |

**合計**: +110 tests（4,164 → 4,274）

### 参考リンク

- 前フェーズ（完了）: [roadmap-v90.1-v95.0.md](roadmap-v90.1-v95.0.md)
- Sprint 1 詳細: [roadmap-v95.1-v96.0.md](roadmap-v95.1-v96.0.md)
- Sprint 2 詳細: [roadmap-v96.1-v97.0.md](roadmap-v96.1-v97.0.md)
- Sprint 3 詳細: [roadmap-v97.1-v98.0.md](roadmap-v97.1-v98.0.md)
- Sprint 4 詳細: [roadmap-v98.1-v99.0.md](roadmap-v98.1-v99.0.md)
- Sprint 5 詳細: [roadmap-v99.1-v100.0.md](roadmap-v99.1-v100.0.md)
- 進行状況: [../current.md](../current.md)
- マイルストーン: [../../MILESTONE.md](../../MILESTONE.md)

---

## 宣言文（予定）

> 「Favnir が、SAP のプラットフォームになった。
>
>  `$delta` で差分を受け取り、Event Mesh でリアルタイムに動き、
>  `ctx.sap_env("PRD")` で本番に切り替え、
>  `!Approval` で人間の判断を型に閉じ込め、
>  `Masked<T>` で個人情報を守り、
>  `!Audit` で証跡を残す。
>
>  それが、Favnir SAP Platform 1.0 である。」
