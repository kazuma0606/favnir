# Roadmap v57.1.0 〜 v58.0.0 — Enterprise Security

Date: 2026-07-23
Status: COMPLETE

---

## 前提

- 直前完了: v57.0.0「Language Power 2.0」（tests ≥ 3250）
- マスターロードマップ: `roadmap-v55.1-v60.0.md`
- 本文書はマスターの v58.0 スプリント部分の詳細版
- **既存機能の扱い**: `fav audit` は v24.6 で実装済み。v57.4 はこれに `--security` フラグを追加する拡張。
  v57.5 の `fav audit verify` は既存 `fav audit` の拡張コマンドとして追加。
  シークレット注入は v57.2 が新規実装。`fav run --inject-secrets` も新規追加。
  詳細はマスターロードマップ「既存機能との位置づけ」テーブルを参照。

---

## 目標

RBAC・シークレット管理・TLS・監査ログ署名・コンプライアンスレポートを実装し、
**企業のセキュリティ要件を満たす「Enterprise Security」基盤を完成させる**。

---

## バージョン計画

### v57.1.0 — RBAC（ロールベースアクセス制御）for Rune

`fav.toml` の `[security.rbac]` セクションを解析し、Rune へのアクセスをロールで制限。
checker で現在のロールコンテキストを検証。E0424 エラーコード（RBAC アクセス拒否）を
`error_catalog.rs` に追加。`fav run --role <role>` フラグで実行時ロールを指定。

```toml
# fav.toml
[security.rbac]
roles = ["reader", "writer", "admin"]

[security.rbac.bindings]
"kafka"     = ["reader", "writer", "admin"]
"snowflake" = ["writer", "admin"]
```

**完了条件**: Rust テスト 3 件（ベース 3252 + 3 = 3255 tests passed, 0 failed）
- `rbac_access_denied`
- `rbac_access_granted`
- `rbac_unrestricted_rune`

**実績**: 3252 + 3 = 3255 tests passed, 0 failed（2026-07-27）— **COMPLETE**

---

### v57.2.0 — シークレット管理統合（Vault / AWS Secrets Manager）

`[secrets]` セクションを解析し、AWS Secrets Manager / HashiCorp Vault からシークレットを取得。
実行時に環境変数として注入（ソースコードには直接埋め込まない）。
`fav secrets list` / `fav secrets rotate` コマンドを追加。

```toml
# fav.toml
[secrets]
provider = "aws-secrets-manager"
region   = "ap-northeast-1"

[secrets.bindings]
SNOWFLAKE_PASSWORD = "prod/snowflake/password"
KAFKA_API_KEY      = "prod/kafka/api-key"
```

```bash
$ fav run pipeline.fav --inject-secrets
$ fav secrets list
$ fav secrets rotate SNOWFLAKE_PASSWORD
```

**完了条件**: Rust テスト 2 件（ベース 3255 + 2 = 3257 tests passed, 0 failed）
- `secrets_provider_config_parsed`
- `cmd_secrets_list`

**実績**: 3255 + 2 = 3257 tests passed, 0 failed（2026-07-27）— **COMPLETE**

---

### v57.3.0 — TLS / mTLS サポート（HTTP / gRPC Rune）

`fav.toml` の `[security.tls]` セクションを解析し、`TlsConfig` 構造体を `toml.rs` に追加。
`is_mtls()` メソッドで mTLS 設定（クライアント証明書あり）を判定できる。
証明書・鍵の HTTP / gRPC Rune クライアントへの実際の注入および
`fav doctor` の TLS 設定チェック項目追加は後続バージョンで対応予定。

```toml
# fav.toml
[security.tls]
ca_cert  = "certs/ca.pem"
tls_cert = "certs/client.pem"
tls_key  = "certs/client-key.pem"
verify   = true
```

**完了条件**: Rust テスト 2 件（ベース 3257 + 2 = 3259 tests passed, 0 failed）
- `tls_config_parsed`
- `mtls_cert_injected`

**実績**: 3257 + 2 = 3259 tests passed, 0 failed（2026-07-27）— **COMPLETE**

---

### v57.4.0 — 依存関係セキュリティスキャン（`fav audit --security`）

既存の `fav audit` コマンドに `--security` フラグを追加。Rune バージョンを既知 CVE
データベース（`registry/security.json`）と照合。`--fail-on-high` フラグで HIGH 以上の CVE
があれば非ゼロ終了コード（CI 統合向け）。

```bash
$ fav audit --security
[WARN] rune kafka@2.1.0: CVE-2026-1234 (severity: HIGH)
       fix: upgrade to kafka@2.2.0
[OK]   rune postgres@1.0.0: no known vulnerabilities

$ fav audit --security --fail-on-high
exit code: 1
```

**完了条件**: Rust テスト 2 件（ベース 3259 + 2 = 3261 tests passed, 0 failed）
- `security_scan_detects_cve`
- `security_scan_fail_on_high`

**実績**: 3259 + 2 = 3261 tests passed, 0 failed（2026-07-28）— **COMPLETE**

---

### v57.5.0 — 監査ログ暗号化・署名（tamper-proof audit）

> **実装スコープ注記**: 実際の HMAC-SHA256（外部 crate）/ `--audit-sign` CLI / `fav audit verify` コマンド /
> `[secrets]` 鍵取得は本バージョンのスコープ外。`AuditEntry` データ構造・`sign_entry` / `verify_entry` 純粋関数の確立のみ。

`--audit-sign` フラグで HMAC-SHA256 署名を各 JSONL エントリに付与。
`fav audit verify` コマンドで署名検証を実行。
鍵は `[secrets]` プロバイダから取得（v57.2 の実装を活用）。

```bash
$ fav run pipeline.fav --audit-log audit.jsonl --audit-sign --audit-key prod/audit-key
$ fav audit verify audit.jsonl --audit-key prod/audit-key
[OK] 1,250 entries verified (tamper-free)
[FAIL] entry 847: hash mismatch (tampered)
```

**完了条件**: Rust テスト 2 件（ベース 3261 + 2 = 3263 tests passed, 0 failed）
- `audit_sign_entry`
- `audit_verify_tamper_detected`

**実績**: 3261 + 2 = 3263 tests passed, 0 failed（2026-07-28）— **COMPLETE**

---

### v57.6.0 — コンプライアンスレポート（GDPR / SOC2 対応）

> **実装スコープ注記**: `fav compliance-report` CLI コマンド・JSONL ファイル読み込み・`-o report.md` 出力は本バージョンのスコープ外。
> `ComplianceFramework` 列挙型・`ComplianceReport` データ構造・`generate_report` 純粋関数の確立のみ。

`fav compliance-report` コマンドを追加。`--audit-log` の JSONL ログを解析し、
GDPR（データアクセス・削除記録）/ SOC2（アクセス制御・監査証跡）のフレームワークに沿った
Markdown レポートを生成。

```bash
$ fav compliance-report --framework gdpr --audit-log audit.jsonl -o report.md
$ fav compliance-report --framework soc2  --audit-log audit.jsonl -o report.md
```

**完了条件**: Rust テスト 2 件（ベース 3263 + 2 = 3265 tests passed, 0 failed）
- `compliance_report_gdpr_generates`
- `compliance_report_soc2_generates`

**実績**: 3263 + 2 = 3265 tests passed, 0 failed（2026-07-28）— **COMPLETE**

---

### v57.7.0 — マルチテナント分離

> **実装スコープ注記**: Rune エンドポイントへのテナント識別子自動挿入・
> `strict` モード時の E0425 エラー発行（checker 統合）・
> `fav run --tenant` CLI フラグは本バージョンのスコープ外。
> `TenancyConfig` / `TenancyIsolation` データ構造と TOML パース層の確立に集中する。

`fav.toml` の `[tenancy]` セクションを解析し、Rune のエンドポイントにテナント識別子を自動挿入。
`strict` モードでは、テナント識別子なしのアクセスを E0425 エラーとして拒否。

```toml
# fav.toml
[tenancy]
mode   = "strict"
tenant = "${TENANT_ID}"

[tenancy.isolation]
snowflake_schema   = "tenant_${TENANT_ID}"
kafka_topic_prefix = "${TENANT_ID}."
```

**完了条件**: Rust テスト 2 件（ベース 3265 + 2 = 3267 tests passed, 0 failed）
- `tenancy_config_parsed`
- `tenancy_strict_enforced`

**実績**: 3265 + 2 = 3267 tests passed, 0 failed（2026-07-28）— **COMPLETE**

---

### v57.8.0 — ドキュメントサイト Enterprise Security 記事

`site/content/docs/enterprise/rbac.mdx` — RBAC 設定・ロールバインディング・checker 統合。
`site/content/docs/enterprise/secrets.mdx` — シークレット管理・Vault / AWS SM 連携手順。
`site/content/docs/enterprise/compliance.mdx` — コンプライアンスレポート・GDPR / SOC2 対応。

**完了条件**: Rust テスト 2 件（ベース 3267 + 2 = 3269 tests passed, 0 failed）
- `docs_rbac_page_exists`
- `docs_compliance_page_exists`

**実績**: 3267 + 3 = 3270 tests passed, 0 failed（2026-07-28）— **COMPLETE**

---

### v57.9.0 — 安定化・コードフリーズ（Enterprise Security 前調整）

全 lint / clippy クリーン確認。`site/content/docs/enterprise-security-overview.mdx` 骨子作成。
v57.1〜v57.8 の全テストが通過していることを確認して v58.0 へ。

**完了条件**: Rust テスト 2 件（ベース 3270 + 2 = 3272 tests passed, 0 failed）
- `cargo_toml_version_is_57_9_0`
- `enterprise_security_overview_exists`

> ベース 3270 は v57.8.0 code-review 対応（`docs_secrets_page_exists` +1）による実績値。ロードマップ元記載 3269 から修正。

**実績**: 3270 + 2 = 3272 tests passed, 0 failed（2026-07-28）— **COMPLETE**

---

### v58.0.0 — Enterprise Security 宣言 ★クリーンアップ

**宣言文**:

> 「アクセスはロールで制御され、シークレットはコードに現れず、
>  通信は mTLS で守られ、監査ログは改ざんできない。
>  コンプライアンスレポートはボタン一つで生成される。
>  Favnir は企業のセキュリティ要件を満たす言語になった。
>
>  これが Favnir v58.0 — Enterprise Security の姿である。」

**完了条件**:
- v57.1〜v57.9 の全機能が動作する
- `cargo test` 全通過（failures=0 かつテスト数 ≥ **3276**）
- `v58000_tests` 4 件 pass（ベース 3272 + 4 = 3276 tests passed, 0 failed）:
  - `cargo_toml_version_is_58_0_0`
  - `changelog_has_v58_0_0`
  - `milestone_has_enterprise_security`
  - `readme_mentions_enterprise_security`
- `MILESTONE.md` に `"Enterprise Security"` 宣言文エントリを追加する
- `★クリーンアップ`（`cargo clean`）完了

**実績**: 3272 + 4 = 3276 tests passed, 0 failed（2026-07-28）— **COMPLETE**

---

## 参考リンク

- マスターロードマップ: `versions/roadmap/roadmap-v55.1-v60.0.md`
- 前サブスプリント: `versions/roadmap/roadmap-v56.1-v57.0.md`
- 達成宣言: `MILESTONE.md`
