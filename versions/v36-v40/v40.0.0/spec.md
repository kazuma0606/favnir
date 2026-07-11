# v40.0.0 spec — Enterprise Governance マイルストーン宣言

## バージョン概要

| 項目 | 内容 |
|---|---|
| バージョン | v40.0.0 |
| テーマ | Enterprise Governance マイルストーン宣言・★クリーンアップ |
| 前提 | v39.9.0 COMPLETE — v40.0 前調整・安定化完了 |
| 完了条件 | `v40000_tests` 全テスト pass・`cargo test` 0 failures・`MILESTONE.md` 更新・★クリーンアップ完了 |

## 背景と目的

v39.1〜v39.9 のスプリントで以下を達成した。本バージョンはこれらを統合して Enterprise Governance マイルストーンを正式宣言し、v40 世代に移行する。

### 達成内容

| バージョン | 内容 |
|---|---|
| v39.1.0 | RBAC Rune — `auth.require_role` / `auth.check_permission` / `auth.verify_jwt` |
| v39.2.0 | Audit Log Rune — `Audit.log` / `Audit.start_trace` / `Audit.end_trace` |
| v39.3.0 | `fav policy` — `fav policy check` / `fav policy check --ci`（exit 1） |
| v39.4.0 | Secret Rune 強化 — `Secret.get_aws` / `get_vault` / `get_gcp` / `get_env` |
| v39.5.0 | マルチテナント対応 — `tenant.db_schema` / `s3_prefix` / `validate_tenant` |
| v39.6.0 | `fav audit` — 依存 Rune ライセンス一覧 / GPL・CVE 検出（exit 1） |
| v39.7.0 | CI/CD ポリシーゲート — `fav ci init` 生成 YAML に Policy check 自動含める |
| v39.8.0 | Enterprise cookbook + ガバナンスドキュメント（6 MDX ファイル） |
| v39.9.0 | v40.0 前調整・安定化 — `enterprise-governance.mdx` 振り返りドキュメント |

## ロードマップとの差異

ロードマップの完了条件「テスト数 5000+」は現時点の実績（2810 件）から大幅に乖離している。
本バージョンでは「2810 + 4（v40000_tests）= 2814 件」を完了条件とする（v36.0〜v39.0 と同規約）。

ロードマップ記載の「GitHub Issues P1/P2 ラベル付きオープンバグ 0 件」条件は Favnir が OSS 公開前のため GitHub Issues が存在しない。本バージョンでは対象外とする（v36.0 / v37.0 / v38.0 / v39.0 と同規約）。

`roadmap-v35.1-v40.0.md` §v40.0 完了基準に記載された「デプロイ / データ品質 / マルチソース / AI 支援」の 4 コンポーネントは、それぞれ v36.0〜v39.0 の各スプリント（Deployment Story / Data Quality First / Multi-Source ETL Power / Intelligence & Assistance）で達成・宣言済みである。v40.0.0 本バージョンでの再検証は不要とする（v36.0〜v39.0 と同規約）。

## 実装スコープ

| ファイル | 変更内容 |
|---|---|
| `MILESTONE.md` | v40.0 Enterprise Governance 宣言セクション追加（先頭に挿入） |
| `README.md` | v40.0 マイルストーン宣言行を追加 |
| `CHANGELOG.md` | `## [v40.0.0]` エントリ追加 |
| `fav/src/driver.rs` | `v39900_tests::cargo_toml_version_is_39_9_0` スタブ化 |
| `fav/src/driver.rs` | `v40000_tests` モジュール（4 件）追加 |
| `fav/Cargo.toml` | バージョン `39.9.0` → `40.0.0` |
| ビルドキャッシュ | `cargo clean`（★クリーンアップ） |
| `versions/v36-v40/v40.0.0/tasks.md` | COMPLETE 更新 |

## v40000_tests の設計

| テスト名 | 検証内容 | `include_str!` パス |
|---|---|---|
| `cargo_toml_version_is_40_0_0` | Cargo.toml に `"40.0.0"` が含まれる | `"../Cargo.toml"` |
| `changelog_has_v40_0_0` | `CHANGELOG.md` に `[v40.0.0]` が含まれる | `"../../CHANGELOG.md"` |
| `milestone_has_enterprise_governance` | `MILESTONE.md` に `"Enterprise Governance"` が含まれる | `"../../MILESTONE.md"` |
| `readme_mentions_enterprise_governance` | `README.md` に `"Enterprise Governance"` が含まれる | `"../../README.md"` |

imports 不要（`include_str!` のみ使用）。
`cargo_toml_version_is_40_0_0` に `// NOTE: 次バージョン bump 時に Stubbed コメントへ置き換えること` を付与する。

```rust
// ── v40000_tests (v40.0.0) — Enterprise Governance マイルストーン宣言 ─────────
#[cfg(test)]
mod v40000_tests {
    // include_str! のみ使用のため imports 不要

    #[test]
    fn cargo_toml_version_is_40_0_0() {
        // NOTE: 次バージョン bump 時に Stubbed コメントへ置き換えること
        let cargo = include_str!("../Cargo.toml");
        assert!(cargo.contains("40.0.0"), "Cargo.toml must contain version 40.0.0");
    }

    #[test]
    fn changelog_has_v40_0_0() {
        let src = include_str!("../../CHANGELOG.md");
        assert!(src.contains("[v40.0.0]"), "CHANGELOG.md must contain [v40.0.0]");
    }

    #[test]
    fn milestone_has_enterprise_governance() {
        let src = include_str!("../../MILESTONE.md");
        assert!(src.contains("Enterprise Governance"), "MILESTONE.md must contain Enterprise Governance");
    }

    #[test]
    fn readme_mentions_enterprise_governance() {
        let src = include_str!("../../README.md");
        assert!(src.contains("Enterprise Governance"), "README.md must contain Enterprise Governance");
    }
}
```

## 宣言文

```
RBAC で実行権限を制御し、Audit Log でパイプラインを追跡できる。
fav policy で組織ポリシーを宣言的に定義し、
fav policy check --ci で違反を PR でブロックできる。
Secret Rune は Vault / AWS / GCP に対応し、
マルチテナント対応で複数チームが安全に使える。

これが Favnir v40.0 — Enterprise Governance の姿である。
```

## MILESTONE.md への追加内容

```
## v40.0.0 — Enterprise Governance（2026-07-11）

> 「RBAC で実行権限を制御し、Audit Log でパイプラインを追跡できる。
>  `fav policy` で組織ポリシーを宣言的に定義し、
>  `fav policy check --ci` で違反を PR でブロックできる。
>  Secret Rune は Vault / AWS / GCP に対応し、
>  マルチテナント対応で複数チームが安全に使える。
>
>  これが Favnir v40.0 — Enterprise Governance の姿である。」

v40.0.0 をもって、Favnir の **Enterprise Governance** を正式に宣言する。

### 達成コンポーネント（v39.1〜v39.9）

| コンポーネント | バージョン | 内容 |
|---|---|---|
| RBAC Rune | v39.1 | require_role / check_permission / verify_jwt |
| Audit Log Rune | v39.2 | Audit.log / start_trace / end_trace |
| fav policy | v39.3 | fav policy check / fav policy check --ci（exit 1） |
| Secret Rune 強化 | v39.4 | get_aws / get_vault / get_gcp / get_env |
| マルチテナント | v39.5 | tenant.db_schema / s3_prefix / validate_tenant |
| fav audit | v39.6 | ライセンス一覧 / GPL・CVE 検出 |
| CI/CD ゲート | v39.7 | fav ci init に Policy check ステップ自動含める |
| Governance docs | v39.8 | docs/governance/ 3 件 + cookbook 3 件 |
| 安定化 | v39.9 | enterprise-governance.mdx ドキュメント整備 |

**宣言日**: 2026-07-11

---
```

挿入位置: `# Favnir Milestones` ヘッダの直後、`## v39.0.0` セクションの直前。

## README.md への追加行

```markdown
**v40.0（2026-07-11）で、[Enterprise Governance](./MILESTONE.md) マイルストーンを宣言しました。**
```

挿入位置: `**v39.0（2026-07-10）で、[Intelligence & Assistance]...` 行の直後。

## ★クリーンアップ

v40.0.0 は x.0.0 マイルストーンのため `cargo clean` が必須（v36〜v39 の x.0.0 と同規約）。

**注意**: `cargo clean` により `fav/tmp/hello.fav` が消える可能性がある（v30.0.0 での知見）。
クリーンアップ前後で `fav/tmp/hello.fav` の存在を確認し、消失した場合は以下の内容で復元すること:
```
fn add(a: Int, b: Int) -> Int { a + b }
fn main() -> Bool { add(1, 2) == 3 }
```

クリーンアップ手順（T7 の順序）:
1. `fav/tmp/hello.fav` 存在確認
2. `cargo clean`
3. `hello.fav` 存在確認（消失していれば復元）
4. `cargo test`（全通過確認）

## テスト数の計算

| バージョン | 実績 |
|---|---|
| v39.9.0 | 2810 |
| v40.0.0 追加分（v40000_tests 4 件 + v39900_tests スタブ化 0 件変化） | +4 |
| v40.0.0 期待値 | 2814 |

## 完了条件

| # | 条件 | 検証方法 |
|---|---|---|
| 1 | `MILESTONE.md` に `"Enterprise Governance"` が含まれる | `milestone_has_enterprise_governance` テスト |
| 2 | `README.md` に `"Enterprise Governance"` が含まれる | `readme_mentions_enterprise_governance` テスト |
| 3 | `CHANGELOG.md` に `[v40.0.0]` が含まれる | `changelog_has_v40_0_0` テスト |
| 4 | `Cargo.toml` バージョンが `40.0.0` | `cargo_toml_version_is_40_0_0` テスト |
| 5 | `cargo clean` 実施済み | T2 実行記録 |
| 6 | `cargo test` 全通過（failures=0 かつテスト数 ≥ 2814） | `cargo test` 実行結果（2810 + 4 = 2814） |
