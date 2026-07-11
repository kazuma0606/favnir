# Roadmap v39.1.0 〜 v40.0.0 — Enterprise Governance

Date: 2026-07-06
Status: 骨格確定（v35.0 完了時点）、詳細は v39.0 完了後に確定

---

## 目標

v39.0「Intelligence & Assistance」で「AI がパイプラインを補助する」を実現した。
このフェーズは **「チームで安全に運用できる」** を実現する。

---

## バージョン計画

### ✅ v39.1.0 — RBAC Rune

```favnir
bind _ <- auth.require_role(ctx, "admin")
bind _ <- auth.check_permission(ctx, "write:warehouse")
```

`runes/auth/auth.fav` — require_role / check_permission / verify_jwt

**完了条件**: Rust テスト 3 件（2788 tests passed, 0 failed）

---

### ✅ v39.2.0 — Audit Log Rune

`runes/audit/audit.fav` — `Audit.log` / `Audit.start_trace` / `Audit.end_trace`
`fav.toml` に `[audit]` セクション（`enabled`, `output = "file"/"webhook"`）

**完了条件**: Rust テスト 3 件（2791 tests passed, 0 failed）

---

### ✅ v39.3.0 — `fav policy`

```favnir
policy {
  deny_runes: ["experimental/*"]
  require_schema: true
  require_tests: true
  max_pipeline_stages: 20
}
```

`fav policy check` / `fav policy check --ci`（exit 1）

**完了条件**: Rust テスト 3 件

---

### ✅ v39.4.0 — Secret Rune 強化

- `Secret.get_aws(ctx, name)` / `Secret.get_vault(ctx, path)` / `Secret.get_gcp(ctx, name)`
- `fav.toml` に `[secrets] backend = "aws"/"vault"/"gcp"`
- `Secret.get_env(ctx, name)` — ローカル開発フォールバック

**完了条件**: Rust テスト 3 件

---

### ✅ v39.5.0 — マルチテナント対応

`ctx.tenant_id: String` を `AppCtx` に追加。DB スキーマ自動切り替え / S3 prefix 分離。

> **スコープ注記（v39.5.0 実装時に確定）**: `AppCtx` への `tenant_id: String` フィールド実追加は v39.9.0（前調整版）に移管。
> 本バージョンでは `runes/tenant/tenant.fav`（db_schema / s3_prefix / validate_tenant スタブ）と rune.toml の追加のみを実施。

**完了条件**: テナント分離 E2E テスト 2 件（functional）+ meta テスト 2 件 = 計 4 件（2801 tests passed）

---

### ✅ v39.6.0 — `fav audit`

`fav audit` — 依存 Rune ライセンス一覧 / `fav audit --check` — GPL・CVE 検出（exit 1）

**完了条件**: Rust テスト 2 件

---

### v39.7.0 — CI/CD ポリシーゲート ✅

`fav policy check --ci` で違反時に stderr 出力 + exit 1。`fav ci init` 生成 YAML に自動含める。

**完了条件**: Rust テスト 2 件

---

### v39.8.0 — Enterprise cookbook + ガバナンスドキュメント ✅

- `site/content/docs/governance/rbac.mdx`
- `site/content/docs/governance/audit-log.mdx`
- `site/content/docs/governance/policy.mdx`
- `site/content/cookbook/multi-tenant-etl.mdx`
- `site/content/cookbook/secret-manager-vault.mdx`
- `site/content/cookbook/ci-policy-gate.mdx`

**完了条件**: Rust テスト 1 件

---

### v39.9.0 — v40.0 前調整・安定化 + 全スプリント振り返り ✅

- `site/content/docs/enterprise-governance.mdx` 新規作成（v39.1〜v39.8 機能一覧テーブル + v40.0 宣言文）
- コードフリーズ（新規機能追加なし）

**完了条件**: meta テスト 2 件（2810 tests passed, 0 failed）

---

### v40.0.0 — Enterprise Governance マイルストーン宣言 ★クリーンアップ ✅

**宣言文（暫定）**:

> 「RBAC で実行権限を制御し、Audit Log でパイプラインを追跡できる。
>  `fav policy` で組織ポリシーを宣言的に定義し、
>  `fav policy check --ci` で違反を PR でブロックできる。
>  Secret Rune は Vault / AWS / GCP に対応し、
>  マルチテナント対応で複数チームが安全に使える。
>
>  これが Favnir v40.0 — Enterprise Governance の姿である。」

**完了条件**:
- v39.1〜v39.9 の全機能が動作する / テスト数 5000+
- GitHub Issues の P1/P2 ラベル付きオープンバグが **0 件**
- `★クリーンアップ` 完了

---

## 参考リンク

- マスタースケジュール: `versions/roadmap/roadmap-v35.1-v40.0.md`
- 前サブスプリント: `versions/roadmap/roadmap-v38.1-v39.0.md`
- 達成宣言: `MILESTONE.md`
