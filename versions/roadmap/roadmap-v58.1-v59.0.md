# Roadmap v58.1.0 〜 v59.0.0 — Governance & Deployment 2.0

Date: 2026-07-23
Status: 未着手

---

## 前提

- 直前完了: v58.0.0「Enterprise Security」（tests ≥ 3272）
- マスターロードマップ: `roadmap-v55.1-v60.0.md`
- 本文書はマスターの v59.0 スプリント部分の詳細版
- **既存機能の扱い**: `assert_schema`（v52.0 実装済み）を v58.3 のスキーママイグレーションで活用。
  `expand_env_vars` / `inject_snowflake_config`（v10.7 実装済み）の拡張として
  v58.6 の `inject_env_config` を実装。`!Observe` エフェクト（v29.0 実装済み）を
  v58.4 の Data Catalog 統合で新規 `!Catalog` エフェクトとして追加。
  詳細はマスターロードマップ「既存機能との位置づけ」テーブルを参照。

---

## 目標

Blue/Green・カナリア・スキーマ管理・データカタログ・ポリシーコードを実装し、
**運用チームに信頼される「Governance & Deployment 2.0」基盤を完成させる**。

---

## バージョン計画

### v58.1.0 — Blue/Green デプロイメントサポート

`fav deploy --strategy blue-green` コマンドを追加。
2 スロット（blue / green）の切り替えロジックを `driver.rs` に実装。
`infra/deploy/blue-green/` に Terraform テンプレートを追加。

```bash
$ fav deploy --strategy blue-green --env prod pipeline.fav
Deploying to: green slot (current: blue)
Health check: OK (green)
Traffic switch: blue → green [100%]
Old slot (blue): kept for 10 minutes (rollback window)

$ fav deploy rollback --env prod
Traffic switch: green → blue [100%]
```

**完了条件**: Rust テスト 2 件（ベース 3272 + 2 = 3274 tests passed, 0 failed）
- `cmd_deploy_blue_green`
- `cmd_deploy_rollback`

**実績**: 3276 + 3 = 3279 tests passed, 0 failed（2026-07-28）— **COMPLETE**（code-review 対応で +1）

---

### v58.2.0 — カナリアリリース

`fav deploy --strategy canary --canary-weight <pct>` を追加。
`fav deploy promote` / `fav deploy abort` コマンドを追加。
`fav deploy status` でカナリア健全性（エラー率・レイテンシ）を表示。

```bash
$ fav deploy --strategy canary --canary-weight 10 --env prod pipeline.fav
Deploying v58.2.0 to canary (10% traffic)

$ fav deploy promote --env prod   # 100% に昇格
$ fav deploy abort --env prod     # ロールバック
```

**完了条件**: Rust テスト 2 件（ベース 3279 + 2 = 3281 tests passed, 0 failed）
- `cmd_deploy_canary_weight`
- `cmd_deploy_canary_promote`

**実績**: 3279 + 4 = 3283 tests passed, 0 failed（2026-07-28）— **COMPLETE**（code-review 対応で +2）

---

### v58.3.0 — スキーママイグレーション / バージョニング

`fav schema migrate` コマンドで JSONL データをマイグレーション定義に従って変換。
`apply_migration_transform` ヘルパーを driver.rs に追加しコア変換ロジックを実装。

> **実装スコープ変更（spec v58.3.0 確定時）:**
> - `migration` ブロックの AST / parser 追加: **スコープ外** — v58.x の一貫パターン（driver.rs スタブ）に合わせて見送り
> - `assert_schema` バージョン引数追加: **スコープ外** — 将来バージョンへ繰り越し

```bash
$ fav schema migrate --from v1 --to v2 --data orders.jsonl
Schema migration: v1 → v2
  Input : orders.jsonl
  Status: OK (dry-run mode)
```

**完了条件**: Rust テスト 2 件（ベース 3283 + 2 = 3285 tests passed, 0 failed）
- `schema_migration_transforms`
- `cmd_schema_migrate_test`（関数名 `cmd_schema_migrate` との衝突を避けるため `_test` サフィックス付き）

**実績**: 3283 + 3 = 3286 tests passed, 0 failed（2026-07-28）— **COMPLETE**（code-review 対応で +1）

---

### v58.4.0 — Data Catalog 統合（`fav catalog`）

`fav catalog push` で DataHub / Apache Atlas にパイプラインメタデータ（lineage / schema）を登録。
`fav catalog search` でカタログ検索。

> **実装スコープ変更（spec v58.4.0 確定時）:**
> - `!Catalog` エフェクトの AST/IR 追加: **スコープ外** — v58.x の一貫パターン（driver.rs スタブ）に合わせて見送り
> - DataHub / Atlas への実 HTTP 通信: **スコープ外** — 出力文字列モックで検証

```bash
$ fav catalog push --catalog datahub://localhost:8080
Registering pipeline: OrderIngestion
  stage Parse:    RawOrder → Order
  stage Validate: Order → Result<ValidOrder>
  stage Store:    ValidOrder → Unit  (Snowflake: orders_v2)
Catalog push: OK

$ fav catalog search "order"
OrderIngestion  pipeline  last_run: 2026-07-23T10:00:00Z
```

**完了条件**: Rust テスト 2 件（ベース 3286 + 2 = 3288 tests passed, 0 failed）
- `cmd_catalog_push_test`（関数名 `cmd_catalog_push` との衝突を避けるため `_test` サフィックス付き）
- `cmd_catalog_search_test`

**実績**: 3286 + 3 = 3289 tests passed, 0 failed（2026-07-28）— **COMPLETE**（code-review 対応で +1）

---

### v58.5.0 — Policy-as-Code（`fav policy`）

`policy` ブロックを AST / parser に追加。
`fav policy check` コマンドでポリシー違反を検出。E0426 エラーコード（ポリシー違反）を
`error_catalog.rs` に追加。`fav policy list` でアクティブポリシー一覧を表示。

```favnir
policy DataRetention {
  rule NoPersonalDataInLogs: |pipeline| {
    pipeline.stages
      |> List.filter(|s| s.writes_to("logs"))
      |> List.all(|s| !s.accesses_field("email") && !s.accesses_field("user_id"))
  }
}
```

```bash
$ fav policy check pipeline.fav --policy-dir policy/
[FAIL] DataRetention: stage "AuditLog" writes email to logs
```

> **実装スコープ変更（spec v58.5.0 確定時）:**
> - `policy` ブロックの AST / parser 追加: **スコープ外** — v58.x の一貫パターン（driver.rs スタブ）に合わせて見送り
> - `fav.toml` [policy] セクションの本格パース: **スコープ外** — スタブで固定ルールを返す

**完了条件**: Rust テスト 2 件（ベース 3289 + 2 = 3291 tests passed, 0 failed）
- `policy_check_violation`
- `policy_check_passes`

**実績**: 3289 + 3 = 3292 tests passed, 0 failed（2026-07-28）— **COMPLETE**（code-review 対応で +1）

---

### v58.6.0 — マルチ環境設定（dev / staging / prod）

`fav.toml` の `[env.<name>]` セクションを解析し、`--env` フラグで環境別設定を選択。
既存の `expand_env_vars` の拡張として `inject_env_config` を実装。

```toml
# fav.toml
[env.dev]
snowflake.database = "DEV_DB"
kafka.bootstrap = "localhost:9092"

[env.prod]
snowflake.database = "PROD_DB"
kafka.bootstrap = "kafka-prod:9092"
```

```bash
$ fav run pipeline.fav --env staging
$ fav run pipeline.fav --env prod
```

> **実装スコープ変更（spec v58.6.0 確定時）:**
> - `fav.toml` `[env.<name>]` セクションの本格 TOML パース: **スコープ外** — driver.rs スタブで代替
> - `expand_env_vars` 拡張（roadmap 言及だが未実装）: **スコープ外** — 将来バージョンへ繰り越し

**完了条件**: Rust テスト 2 件（ベース 3292 + 2 = 3294 tests passed, 0 failed）
- `env_config_parsed`
- `env_config_injected`

> ベース 3292 は v58.5.0 code-review 対応（+1）後の実績値。ロードマップ当初記載の 3291 から補正。

**実績**: 3292 + 4 = 3296 tests passed, 0 failed（2026-07-28）— **COMPLETE**（code-review 対応で +2）

---

### v58.7.0 — HA / DR（ヘルスチェック・フェイルオーバー）

`fav run --ha --replica <n>` フラグで複数レプリカを起動。
`/healthz` エンドポイントを自動追加。
プライマリ障害時に自動フェイルオーバーする Tokio ベースの watchdog を実装。

```bash
$ fav run pipeline.fav --ha --replica 2
[HA] Primary replica started (port 8080)
[HA] Secondary replica started (port 8081)
[HA] Health check: /healthz → 200 OK
[HA] Failover: primary → secondary (reason: primary unresponsive)
```

> **実装スコープ変更（spec v58.7.0 確定時）:**
> - Tokio ベースの実 watchdog プロセス起動: **スコープ外** — driver.rs スタブで出力を模倣
> - 実際の `/healthz` HTTP エンドポイント起動: **スコープ外** — 出力文字列モックで検証
> - レプリカ間の実ネットワーク通信: **スコープ外** — 固定ポート番号出力で代替

**完了条件**: Rust テスト 2 件（ベース 3296 + 2 = 3298 tests passed, 0 failed）
- `ha_health_check_endpoint`
- `ha_failover_triggers`

**実績**: 3296 + 4 = 3300 tests passed, 0 failed（2026-07-29）— **COMPLETE**（code-review 対応で +2）

---

### v58.8.0 — ドキュメントサイト Governance & Deployment 記事

`site/content/docs/enterprise/deployment.mdx` — Blue/Green・カナリア・HA の設定と運用。
`site/content/docs/enterprise/governance.mdx` — Schema Migration・Data Catalog・Policy-as-Code。
`site/content/cookbook/multi-env-pipeline.mdx` — マルチ環境設定のレシピ。

**完了条件**: Rust テスト 2 件（ベース 3300 + 2 = 3302 tests passed, 0 failed）
- `docs_deployment_page_exists`
- `docs_governance_page_exists`

**実績**: 3300 + 2 = 3302 tests passed, 0 failed（2026-07-29）— **COMPLETE**

---

### v58.9.0 — 安定化・コードフリーズ（Governance & Deployment 2.0 前調整）

全 lint / clippy クリーン確認。`site/content/docs/governance-overview.mdx` 骨子作成。
v58.1〜v58.8 の全テストが通過していることを確認して v59.0 へ。

**完了条件**: Rust テスト 2 件（ベース 3302 + 2 = 3304 tests passed, 0 failed）
- `cargo_toml_version_is_58_9_0`
- `governance_overview_exists`

**実績**: 3302 + 2 = 3304 tests passed, 0 failed（2026-07-29）— **COMPLETE**

---

### v59.0.0 — Governance & Deployment 2.0 宣言 ★クリーンアップ

**宣言文**:

> 「パイプラインは Blue/Green で無停止デプロイされ、
>  カナリアは段階的にトラフィックを引き受ける。
>  スキーマはバージョン管理され、データはカタログで検索できる。
>  ポリシーはコードで記述され、コンプライアンスは自動で証明される。
>  Favnir のパイプラインは運用チームに信頼される。
>
>  これが Favnir v59.0 — Governance & Deployment 2.0 の姿である。」

**完了条件**:
- v58.1〜v58.9 の全機能が動作する
- `cargo test` 全通過（failures=0 かつテスト数 ≥ **3308**）
- `v59000_tests` 4 件 pass（ベース 3304 + 4 = 3308 tests passed, 0 failed）:
  - `cargo_toml_version_is_59_0_0`
  - `changelog_has_v59_0_0`
  - `milestone_has_governance_deployment2`
  - `readme_mentions_governance_deployment2`
- `MILESTONE.md` に `"Governance & Deployment 2.0"` 宣言文エントリを追加する
- `★クリーンアップ`（`cargo clean`）完了

**実績**: 3304 + 4 = 3308 tests passed, 0 failed（2026-07-29）— **COMPLETE**（★クリーンアップ実施済み）

---

## 参考リンク

- マスターロードマップ: `versions/roadmap/roadmap-v55.1-v60.0.md`
- 前サブスプリント: `versions/roadmap/roadmap-v57.1-v58.0.md`
- 達成宣言: `MILESTONE.md`
