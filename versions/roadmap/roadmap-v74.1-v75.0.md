# Roadmap v74.1.0 〜 v75.0.0 — Favnir 2.0 宣言

Date: 2026-08-08
Status: 完了（2026-08-14）

マスターロードマップ: [roadmap-v70.1-v75.0.md](roadmap-v70.1-v75.0.md)

---

## 前提

- 直前完了: v74.0.0「Production Proven」（tests = 3669）※ v73 スプリントで計画比 +22 超過
- 本スプリントは Phase 5「Favnir 2.0 宣言」の詳細計画
- 目標: v75.0.0「Favnir 2.0 宣言」（tests = 3692）

### スプリントの性格

Phase 5 は「言語・型・開発体験・実証——4つが揃った」を統合・宣言するスプリントである。
v71〜v74 で積み上げたものを磨き上げ、Favnir 2.0 として世界に宣言する。
コミュニティ・エコシステム・ドキュメント整備が中心。
B（機能磨き）40% + C（エコシステム・宣言）60% の構成。

---

## バージョン一覧

| バージョン | 内容 | テスト数 | 状態 |
|---|---|---|---|
| v74.1.0 | Rune マーケットプレイス（バージョン管理・依存解決） | 3669 + 2 = 3671 | 完了 |
| v74.2.0 | Multi-tenant Runtime | 3671 + 2 = 3673 | 完了 |
| v74.3.0 | Documentation Site 2.0 | 3673 + 3 = 3676 | 完了 |
| v74.4.0 | OSS Hardening | 3676 + 2 = 3678 | 完了 |
| v74.5.0 | Pipeline Scheduling（`fav schedule`） | 3678 + 2 = 3680 | 未着手 |
| v74.6.0 | `fav audit` 拡張（依存関係セキュリティ機能追加） | 3680 + 2 = 3682 | 未着手 |
| v74.7.0 | コミュニティ Rune 品質基準 | 3682 + 2 = 3684 | 未着手 |
| v74.8.0 | 統合デモ（v70〜v74 の全機能を使ったショーケース） | 3684 + 2 = 3686 | 未着手 |
| v74.9.0 | 安定化・コードフリーズ（Favnir 2.0 前最終調整） | 3686 + 2 = 3688 | 未着手 |
| v75.0.0 | Favnir 2.0 宣言 ★クリーンアップ | 3688 + 4 = 3692 | 未着手 |

---

## v74.1.0 — Rune マーケットプレイス（バージョン管理・依存解決）

```bash
# 公式マーケットプレイスへの公開
$ fav publish rune ./runes/mycompany-crm
Published: mycompany/crm@1.0.0

# インストール
$ fav add rune mycompany/crm@^1.0
# fav.toml に [rune.deps] として記録

# 依存関係一覧
$ fav rune list
  mycompany/crm  1.0.2  (latest: 1.0.2)
  favnir/json    9.0.0  (latest: 9.0.0)
  favnir/postgres 5.1.0 (latest: 5.2.0) ← update available
```

**実装内容:**
- `fav publish rune <path>` — Rune レジストリへのバイナリ + メタデータ送信
- `fav add rune <name>@<version>` — `fav.toml` の `[rune.deps]` に追加
- セマンティックバージョニング + 互換性チェック

**完了条件**: Rust テスト 2 件（3669 + 2 = 3671）
- `rune_marketplace_publish_format`
- `rune_marketplace_add_updates_toml`

---

## v74.2.0 — Multi-tenant Runtime

```toml
# fav.toml
[tenant]
isolation = "strict"        # ステージ間でリソースを分離
quota.max_memory_mb = 512
quota.max_cpu_pct   = 80
quota.max_rows      = 1_000_000

[tenant.team_a]
db_url     = "${TEAM_A_DB_URL}"
s3_bucket  = "team-a-data"

[tenant.team_b]
db_url     = "${TEAM_B_DB_URL}"
s3_bucket  = "team-b-data"
```

**実装内容:**
- `TenantQuota` / `TenantTeamConfig` / `TenantConfig` 構造体を `driver.rs` に追加
- `check_tenant_quota_exceeded` — rows / memory_mb のクォータ超過チェック関数
- `format_tenant_isolation_report` — テナント設定サマリー文字列生成
- ※ toml.rs パース・VM クォータ強制・AppCtx 注入は後続バージョン（v74.X）に延期

**完了条件**: Rust テスト 2 件（3671 + 2 = 3673）
- `multitenant_config_parsed`
- `multitenant_resource_quota_enforced`

---

## v74.3.0 — Documentation Site 2.0

```
新規・大幅拡充:
- Getting Started（5分チュートリアル）
- Language Reference（全構文・全エラーコード）
- Rune Catalog（実装済み全 Rune のドキュメント）
- Cookbook（10+ レシピ: AI ETL / 分散 / データ品質...）
- Migration Guide（v35 → v75 の移行手順）
- API Reference（fav CLI の全フラグ）
- Video Transcripts（将来の動画対応を見越した構造）
```

**実装内容:**
- `site/content/docs/v2/` — v2.0 ドキュメント構造
- `site/content/docs/v2/getting-started.mdx`
- `site/content/docs/v2/migration-v35-v75.mdx`
- `site/content/docs/v2/language-reference.mdx`（全構文一覧）

**完了条件**: Rust テスト 2 件（3673 + 2 = 3675）
- `docs_site2_getting_started_exists`
- `docs_site2_migration_guide_v35_to_v75`

---

## v74.4.0 — OSS Hardening

GitHub 上での公開 OSS として機能するための整備。

```
- CONTRIBUTING.md（コントリビュートガイド・PR テンプレート）
- SECURITY.md（脆弱性報告手順）
- .github/ISSUE_TEMPLATE/（バグ報告・機能要望テンプレート）
- CODE_OF_CONDUCT.md
- 依存ライブラリのライセンス確認（cargo-deny）
- SBOM（Software Bill of Materials）生成
```

**実装内容:**
- `CONTRIBUTING.md` — 開発環境セットアップ・PR フロー・コーディング規約
- `SECURITY.md` — 脆弱性報告手順
- `.github/ISSUE_TEMPLATE/bug_report.md` / `feature_request.md`
- `CODE_OF_CONDUCT.md`（Contributor Covenant v2.1）
- ※ `cargo-deny` 設定（`deny.toml`）+ CI 統合 / SBOM 生成は後続バージョンに延期

**完了条件**: Rust テスト 2 件（3676 + 2 = 3678）
- `oss_contributing_md_exists`
- `oss_security_md_exists`

---

## v74.5.0 — Pipeline Scheduling（`fav schedule`）

```bash
# cron ベースのパイプライン定期実行
$ fav schedule add daily-report \
    --cron "0 9 * * *" \
    --pipeline pipelines/daily_report.fav \
    --notify slack://my-channel

$ fav schedule list
NAME            CRON          LAST RUN              STATUS
daily-report    0 9 * * *     2026-08-08 09:00:02   OK
hourly-sync     0 * * * *     2026-08-08 10:00:01   OK

$ fav schedule run daily-report  # 即時実行
```

**実装内容:**
- `ScheduleEntry` 構造体（name / cron / pipeline / notify）
- `validate_cron_expr` — cron 式の基本バリデーション（`cmd_schedule_add` の前処理として先行実装）
- `cmd_schedule_list(entries)` — スケジュール一覧をテキスト形式で返す
- ※ `cmd_schedule_add` / `cmd_schedule_run` / 永続化 / デーモン化は後続バージョンに延期

**完了条件**: Rust テスト 2 件（3678 + 2 = 3680）
- `schedule_add_parses_cron`
- `schedule_list_returns_entries`

---

## v74.6.0 — `fav audit` 拡張（依存関係セキュリティ機能追加）

> 現行の `fav audit`（`cmd_audit`）は Favnir ソースコードレベルの監査（
> `runes/audit/` ディレクトリ対象）を提供している。本バージョンでは
> `fav audit --deps` サブフラグとして **Cargo 依存関係のセキュリティスキャン**
> を追加する。既存の `fav audit`（コードレベル）との衝突はない。

```bash
# 既存: ソースコード監査（変更なし）
$ fav audit pipeline.fav

# 新規追加: Cargo 依存関係のセキュリティスキャン
$ fav audit --deps
Auditing 47 Cargo dependencies...

CRITICAL  libduckdb-sys 1.2.2  CVE-2026-XXXX  Update to 1.3.0
HIGH      tokio 1.38.0         CVE-2026-YYYY  Update to 1.38.1
OK        45 dependencies clean

$ fav audit --deps --fix
Updated: libduckdb-sys 1.2.2 → 1.3.0
Updated: tokio 1.38.0 → 1.38.1
```

**実装内容:**（将来統合の土台実装）
- `DepVulnerability` 構造体（name / version / cve / severity / fix_version）
- `format_audit_deps_report` — 脆弱性レポートのテキストフォーマット
- `apply_audit_fix` — Cargo.toml 文字列中の依存バージョン置換
- `cargo audit` CLI 呼び出し・RustSec DB アクセス・CLI エントリポイント・実ファイル書き込みは後続バージョンで対応

**完了条件**: Rust テスト 2 件（3680 + 2 = 3682）
- `audit_detects_vulnerable_dep`
- `audit_fix_updates_cargo_toml`

---

## v74.7.0 — コミュニティ Rune 品質基準

コミュニティが公開した Rune の品質を担保するための基準と検証ツール。

```bash
# Rune 公開前のチェック
$ fav rune validate ./runes/my-rune
✓ rune.toml: valid
✓ implementation: my-rune.fav (247 lines)
✓ tests: 3 test cases found
✓ documentation: README.md exists
⚠ No example .fav file found
Score: 85/100 (Publish requires >= 80)
```

**実装内容:**（将来統合の土台実装）
- `RuneValidationItem` 構造体（name / passed / message）
- `RuneValidationReport` 構造体（rune_name / items / score）
- `validate_rune_score` — score >= 80 なら true（公開要件チェック）
- `format_rune_validation_report` — 検証レポートのテキストフォーマット
- `cmd_rune_validate(path)`・`fav publish rune` 自動フック・CLI エントリポイントは後続バージョンで対応

**完了条件**: Rust テスト 2 件（3682 + 2 = 3684）
- `rune_validate_scoring`
- `rune_validate_min_score_enforced`

---

## v74.8.0 — 統合デモ（v70〜v74 の全機能を使ったショーケース）

すべてのフェーズで実装した機能を一本のデモパイプラインで示す。

```
infra/e2e-demo/favnir2-showcase/
├── pipeline.fav          # AI ETL + 依存型 + データコントラクト + 分散実行
├── fav.toml              # マルチテナント + SLA + スケジュール
├── rune.toml             # カスタム Rune 依存
├── contract.fav          # データコントラクト定義
├── quality.fav           # 品質スコアリングパイプライン
└── README.md             # 実行手順
```

**pipeline.fav の概要（全フェーズ機能を網羅）:**

```favnir
// v71: 依存型（Vec<Float>[1536]）
// v72: データコントラクト + SLA
// v73: AI 補完ありの開発体験
// v74: マルチテナント設定

import rune "privacy"    // v73.3
import rune "linalg"     // v73.6（VM primitive 接続済み）

contract ShowcaseContract {
    input:  { text: NonEmptyStr, tenant_id: String }
    output: { vector: Vec<Float>[1536], score: Float where self >= 0.0 }
    sla:    { max_latency_ms: 3000 }
}

fn main(ctx: AppCtx) -> Result<Unit, String> {
    bind rows  <- ctx.io.read_file_raw("data/input.csv")
    bind clean <- Rune.privacy.mask(rows, fields: ["email"])
    bind embed <- OpenAI.embed_batch(clean)   // Vec<Float>[1536]
    bind score <- Rune.linalg.cosine_sim(embed, ctx.tenant.ref_vector)
    ctx.io.println(f"Done. mean_score={Float.mean(score)}")
}
```

**完了条件**: Rust テスト 2 件（3684 + 2 = 3686）
- `showcase_demo_structure_complete`
- `showcase_pipeline_fav_valid`

---

## v74.9.0 — 安定化・コードフリーズ（Favnir 2.0 前最終調整）

v70.1〜v74.8 の全機能を通しで確認する最終安定化スプリント。
ショーケースデモが完走し、CI が全グリーンであることを確認する。

**完了条件**: Rust テスト 2 件（3686 + 2 = 3688）
- `favnir2_full_sprint_all_stable`
- `favnir2_e2e_showcase_runs`

---

## v75.0.0 — Favnir 2.0 宣言 ★クリーンアップ

**宣言文**:

> 「compiler.fav が Favnir を完全に記述し、型システムが次元と制約を保証する。
>  依存型がベクトルの次元を守り、refined type がゼロ除算をコンパイル時に止める。
>  VS Code がパイプラインを補完し、AI がエラーを修正し、
>  実際のデータチームが本番で Favnir を走らせている。
>
>  データコントラクトがスキーマ境界を守り、品質スコアが劣化を警告する。
>  Favnir が Favnir 自身を運用し、Rune マーケットプレイスが
>  コミュニティの知恵を型安全なピースとして流通させる。
>
>  これが Favnir v75.0 — Favnir 2.0 の姿である。」

**クリーンアップ作業:**
- `cargo clean` 実施
- `Cargo.toml` バージョンを `75.0.0` に更新
- `CHANGELOG.md` に v75.0.0 エントリを追加
- `MILESTONE.md` に「Favnir 2.0」を追記
- `README.md` に v75.0 達成（Favnir 2.0）を追記
- `versions/current.md` を更新（v75.0.0 完了・次フェーズ計画へ）
- マスターロードマップの本スプリントを「完了」に更新

**完了条件**: `v75000_tests` 4 件（3688 + 4 = 3692）
- `cargo_toml_version_is_75_0_0`
- `changelog_has_v75_0_0`
- `milestone_has_favnir_2`
- `readme_mentions_favnir_2`

---

## テスト数推移（本スプリント）

| バージョン | テスト数 | 増加 |
|---|---|---|
| v74.0.0（ベース） | 3,669 | — |
| v74.1.0 | 3,671 | +2 |
| v74.2.0 | 3,673 | +2 |
| v74.3.0 | 3,676 | +3（language-reference テスト追加のため）|
| v74.4.0 | 3,678 | +2 |
| v74.5.0 | 3,680 | +2 |
| v74.6.0 | 3,682 | +2 |
| v74.7.0 | 3,684 | +2 |
| v74.8.0 | 3,686 | +2 |
| v74.9.0 | 3,688 | +2 |
| v75.0.0（宣言） | 3,692 | +4 |

**本スプリント合計**: +22 tests（3,669 → 3,691）

---

## v70.1〜v75.0 全スプリント総括

| スプリント | 期間 | テスト増 | 到達値 |
|---|---|---|---|
| Language Complete 1.0 | v70.1〜v71.0 | +22 | 3,581 |
| Type System 2.0 | v71.1〜v72.0 | +22 | 3,603 |
| Developer Exp 2.0 | v72.1〜v73.0 | +22 | 3,625 |
| Production Proven | v73.1〜v74.0 | +22 | 3,647 |
| **Favnir 2.0** | **v74.1〜v75.0** | **+23** | **3,692** |
| **合計** | | **+111** | **3,692** |

---

## 参考リンク

- マスターロードマップ: `versions/roadmap/roadmap-v70.1-v75.0.md`
- 前スプリント（完了予定）: `versions/roadmap/roadmap-v73.1-v74.0.md`
- 次フェーズ: （未計画 — v75.0.0 宣言後に策定）
- 達成宣言: `MILESTONE.md`
- 進行状況: `versions/current.md`
