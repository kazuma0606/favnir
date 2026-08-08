# Roadmap v59.1.0 〜 v60.0.0 — Enterprise 1.0 宣言

Date: 2026-07-23
Status: 未着手

---

## 前提

- 直前完了: v59.0.0「Governance & Deployment 2.0」（tests ≥ 3308）
- マスターロードマップ: `roadmap-v55.1-v60.0.md`
- 本文書はマスターの v60.0 スプリント部分の詳細版
- **既存機能の扱い**: `sla` Rune は v52.5 で実装済み。v59.2 はこれを SLA Guarantee モードとして
  上位統合し `fav run --sla-enforce` と `fav sla report` を追加する。
  `fav publish`（v29.1 実装済み `cmd_publish`）を Marketplace 向けに拡張して
  `fav marketplace publish` として公開する。
  詳細はマスターロードマップ「既存機能との位置づけ」テーブルを参照。

---

## 目標

v56〜v59 で実装した全エンタープライズ機能を統合・検証し、
**「企業で安心して選ばれる言語」として Favnir v60.0 — Enterprise 1.0 を宣言する**。

---

## バージョン計画

### v59.1.0 — エンタープライズ E2E ハーネス強化

`examples/enterprise-demo/` ディレクトリに全エンタープライズ機能を統合したデモを作成。
`fav test --suite enterprise` コマンドを追加。`driver.rs` に `cmd_test_enterprise` を実装。

```bash
$ fav test --suite enterprise
[OK] RBAC enforcement (v57.1)
[OK] Secret injection — AWS SM mock (v57.2)
[OK] mTLS connection (v57.3)
[OK] Audit log signing + verification (v57.5)
[OK] Blue/Green deploy simulation (v58.1)
[OK] Compliance report — GDPR (v57.6)
[OK] Policy check — DataRetention (v58.5)
[OK] Data catalog push — DataHub mock (v58.4)
All 8 enterprise checks passed.
```

**完了条件**: Rust テスト 2 件（ベース 3308 + 2 = 3310 tests passed, 0 failed）
- `enterprise_e2e_demo_structure`
- `cmd_test_enterprise_suite`

**実績**: 3310 tests passed, 0 failed（2026-07-29 完了）

---

### v59.2.0 — SLA 保証ティア（SLA Guarantee + アラート統合）

既存の `sla` Rune（v52.5 実装済み）をより上位の SLA Guarantee モードとして統合。
`fav run --sla-enforce` フラグで実行時 SLA 監視を有効化し、違反時に自動アラートを発火。
`fav sla report` コマンドで SLA 達成率レポートを生成。

```toml
# fav.toml
[sla]
latency_p99_ms   = 200
error_rate_pct   = 0.1
availability_pct = 99.9

[sla.alerting]
channels           = ["pagerduty", "slack"]
escalation_policy  = "prod-oncall"
```

```bash
$ fav run pipeline.fav --sla-enforce
$ fav sla report --audit-log audit.jsonl -o sla-report.md
```

**完了条件**: Rust テスト 2 件（ベース 3310 + 2 = 3312 tests passed, 0 failed）
- `sla_guarantee_config_parsed`
- `sla_report_generates`

**実績**: 3312 tests passed, 0 failed（2026-07-29 完了）

---

### v59.3.0 — コスト可視化（`fav cost-estimate`）

`fav cost-estimate` コマンドを追加。各 Rune の操作量とクラウドプロバイダの料金表
（`registry/pricing/<provider>.json`）を照合してコスト見積もりを計算。

```bash
$ fav cost-estimate pipeline.fav --provider aws
Stage Analysis:
  Parse     (Kafka):      ~$0.08/hour  (2M msgs/hr × $0.04/1M)
  Validate  (CPU):        ~$0.03/hour  (0.5 vCPU on Lambda)
  Store     (Snowflake):  ~$0.12/hour  (1 credit/hr × $3/credit / 25)

Total estimated cost: ~$0.23/hour  (~$165/month)
```

**完了条件**: Rust テスト 2 件（ベース 3312 + 2 = 3314 tests passed, 0 failed）
- `cost_estimate_generates`
- `cost_estimate_aws_pricing`

**実績**: 3314 tests passed, 0 failed（2026-07-29 完了）

---

### v59.4.0 — Rune マーケットプレイス Phase 1（`fav marketplace`）

既存の `fav publish`（v29.1 実装済み `cmd_publish`）を Marketplace 向けに拡張。
`fav marketplace list` / `fav marketplace search` を追加。
エンタープライズ向け Private Registry サポートを追加。

> **実装注記**: Private Registry サポートは Phase 1 スコープ外とし、v59.x 後続バージョン（Phase 2）へ延期。
> Phase 1 では `list` / `search` / `publish` の基本コマンドのみ実装した。

```bash
$ fav marketplace list
Name          Author          Downloads  License
kafka         favnir-official  12,450    MIT
snowflake     favnir-official   8,320    MIT
salesforce    acme-corp           920    Apache-2.0

$ fav marketplace publish --rune my-rune
$ fav marketplace search kafka
```

**完了条件**: Rust テスト 2 件（ベース 3314 + 2 = 3316 tests passed, 0 failed）
- `cmd_marketplace_list`
- `cmd_marketplace_publish`

**実績**: 3316 tests passed, 0 failed（2026-07-29 完了）

---

### v59.5.0 — Migration Toolkit（v1 → Enterprise マイグレーション）

`fav migrate --from <version> --to <target>` コマンドを追加。
W035（legacy import）の自動修正と、Enterprise 機能への移行ガイダンスを生成。
`--dry-run` で変更内容を確認し、`--apply` で自動修正を適用。

```bash
$ fav migrate --from v1 --to enterprise --dry-run
[analyze] pipeline.fav
  [WARN] import rune "kafka" → import kafka  (W035: legacy_import_rune)
  [WARN] !Http effect: add TLS config to fav.toml  (new in v57.3)
  [INFO] No RBAC config detected: add [security.rbac] if needed
  [INFO] No [env.*] sections: consider multi-env config (v58.6)

$ fav migrate --from v1 --to enterprise --apply
[fixed] import rune "kafka" → import kafka
```

**完了条件**: Rust テスト 2 件（ベース 3316 + 2 = 3318 tests passed, 0 failed）
- `cmd_migrate_dry_run`
- `cmd_migrate_auto_fix_import`

**実績**: 3318 tests passed, 0 failed（2026-07-30 完了）

---

### v59.6.0 — Enterprise 認定チェックリスト（`fav certify`）

`fav certify --level enterprise` コマンドを追加。
`fav.toml` と CI 設定を解析して Enterprise 1.0 要件の充足を確認。
証明書 JSON（`enterprise-cert.json`）を生成。

```bash
$ fav certify --level enterprise
Checking Favnir Enterprise 1.0 requirements...
[OK]  RBAC configured ([security.rbac])
[OK]  Secrets managed (provider: aws-secrets-manager)
[OK]  TLS enabled ([security.tls])
[OK]  Audit logging active (--audit-sign enabled in CI)
[OK]  Compliance report: GDPR (last generated: 2026-07-23)
[WARN] SLA enforcement: not enabled in production pipeline
       Add: [sla] + fav run --sla-enforce

Enterprise 1.0 certification: 5/6 checks passed (1 warning)
```

**完了条件**: Rust テスト 2 件（ベース 3318 + 2 = 3320 tests passed, 0 failed）
- `cmd_certify_passes`
- `cmd_certify_generates_cert`

**実績**: 3320 tests passed, 0 failed（2026-07-30 完了）

---

### v59.7.0 — README / MILESTONE Enterprise 1.0 整備

`README.md` に Enterprise 1.0 への言及・v56〜v60 機能サマリーを追加。
`MILESTONE.md` に `## v60.0.0（予定）— Enterprise 1.0` エントリを追加。
`site/content/docs/enterprise/enterprise1-overview.mdx` を作成。

**完了条件**: Rust テスト 2 件（ベース 3320 + 2 = 3322 tests passed, 0 failed）
- `readme_has_enterprise1_mention`
- `docs_enterprise1_overview_exists`

**実績**: 3322 tests passed, 0 failed（2026-07-30 完了）

---

### v59.8.0 — ドキュメントサイト Enterprise 1.0 総括記事

`site/content/docs/enterprise/index.mdx` — Enterprise 1.0 の全機能一覧・認定要件・移行ガイド。
`site/content/cookbook/enterprise-checklist.mdx` — Enterprise 運用に必要な設定チェックリスト。

**完了条件**: Rust テスト 2 件（ベース 3322 + 2 = 3324 tests passed, 0 failed）
- `docs_enterprise_index_exists`
- `cookbook_enterprise_checklist_exists`

**実績**: 3324 tests passed, 0 failed（2026-07-30 完了）

---

### v59.9.0 — 安定化・コードフリーズ（Enterprise 1.0 前調整）

全 lint / clippy クリーン確認。`site/content/docs/enterprise/enterprise1-overview.mdx` を完成させる（v59.7.0 で作成済みのファイルを拡充）。
v59.1〜v59.8 の全テストが通過していることを確認して v60.0 へ。

**完了条件**: Rust テスト 2 件（ベース 3324 + 2 = 3326 tests passed, 0 failed）
- `cargo_toml_version_is_59_9_0`
- `enterprise1_overview_doc_complete`

**実績**: 3326 tests passed, 0 failed（2026-07-30 完了）

---

### v60.0.0 — Enterprise 1.0 宣言 ★クリーンアップ

**宣言文**:

> 「ストリームはウィンドウで区切られ、型システムは制約で守られる。
>  アクセスはロールで制御され、シークレットはコードに現れない。
>  デプロイは無停止で切り替わり、ポリシーはコードで記述される。
>  コストは可視化され、SLA は保証され、コンプライアンスは証明される。
>
>  Favnir はデータエンジニアリングのエンタープライズ標準になった。
>
>  これが Favnir v60.0 — Enterprise 1.0 の姿である。」

**完了条件**:
- v59.1〜v59.9 の全機能が動作する
- `cargo test` 全通過（failures=0 かつテスト数 ≥ **3330**）
- `v60000_tests` 4 件 pass（ベース 3326 + 4 = 3330 tests passed, 0 failed）:
  - `cargo_toml_version_is_60_0_0`
  - `changelog_has_v60_0_0`
  - `milestone_has_enterprise1`
  - `readme_mentions_enterprise1`
- `MILESTONE.md` に `"Enterprise 1.0"` 宣言文エントリを追加する
- `★クリーンアップ`（`cargo clean`）完了

**実績**: 3330 tests passed, 0 failed（2026-07-30 完了）

---

## 参考リンク

- マスターロードマップ: `versions/roadmap/roadmap-v55.1-v60.0.md`
- 前サブスプリント: `versions/roadmap/roadmap-v58.1-v59.0.md`
- 達成宣言: `MILESTONE.md`
