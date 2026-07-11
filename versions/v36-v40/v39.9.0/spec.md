# v39.9.0 spec — v40.0 前調整・安定化 + 全スプリント振り返り

## バージョン概要

| 項目 | 内容 |
|---|---|
| バージョン | v39.9.0 |
| テーマ | v40.0 前調整・安定化 + 全スプリント振り返り — v39.1〜v39.8 の成果を Enterprise Governance overview として文書化 |
| 前提 | v39.8.0 COMPLETE — Enterprise cookbook + ガバナンスドキュメント完了 |
| 完了条件 | `v39900_tests` 全テスト pass・`cargo test` 0 failures（≥ 2810 件） |

## 背景と目的

v39.1.0〜v39.8.0 で Enterprise Governance フェーズの機能実装（RBAC / Audit Log / Policy / Secret / マルチテナント / fav audit / CI/CD ゲート）とドキュメント整備（governance docs + cookbook）が完了した。

v39.9.0 では v40.0 マイルストーン宣言に向けて:
1. v39 スプリント全体の振り返り文書として `site/content/docs/enterprise-governance.mdx` を作成
2. v40.0 宣言文の preview を文書内に掲載（暫定）
3. 安定化のためのコードフリーズ（新規機能追加なし）

> 本バージョンはロードマップに完了条件のテスト数指定なし。標準パターン（meta 2 件）を採用。実際の新規テスト数: 2 件（2808 + 2 = 2810）。

## 実装スコープ

### 1. `site/content/docs/enterprise-governance.mdx` — Enterprise Governance 概要ドキュメント

```mdx
---
title: "Enterprise Governance — v39 スプリント完了"
description: "RBAC / Audit / Policy / Secret / マルチテナント / CI ゲートを統合した Enterprise Governance の全体像"
---
```

**内容**:
- v39.0 から v39.9 の概要（何を達成したか）
- 機能一覧テーブル（バージョン・機能・コマンド）
- v40.0 宣言文（暫定）の掲載
- 各ドキュメントへの内部リンク（docs/governance/* / cookbook/*）

### 2. `fav/src/driver.rs` — テストモジュール更新

#### `v39800_tests::cargo_toml_version_is_39_8_0` のスタブ化

```rust
// Stubbed: version bumped to 39.9.0 — assertion intentionally removed
```

#### `v39900_tests` モジュール新規追加（`v39800_tests` の閉じ `}` の後に追加）

```rust
// ── v39900_tests (v39.9.0) — v40.0 前調整・安定化 ────────────────────────────
#[cfg(test)]
mod v39900_tests {
    // include_str! のみ使用のため imports 不要

    #[test]
    fn cargo_toml_version_is_39_9_0() {
        // NOTE: 次バージョン bump 時に Stubbed コメントへ置き換えること
        let cargo = include_str!("../Cargo.toml");
        assert!(cargo.contains("39.9.0"), "Cargo.toml must contain version 39.9.0");
    }

    #[test]
    fn changelog_has_v39_9_0() {
        let src = include_str!("../../CHANGELOG.md");
        assert!(src.contains("[v39.9.0]"), "CHANGELOG.md must contain [v39.9.0]");
    }
}
```

### 3. `CHANGELOG.md` — `[v39.9.0]` エントリ追加

`## [v39.8.0]` ヘッダ行の直前に挿入（`### Added` セクション使用）:

```
## [v39.9.0] — YYYY-MM-DD

### Added
- `site/content/docs/enterprise-governance.mdx` — v39 スプリント振り返り + Enterprise Governance 概要ドキュメント追加
- `v39900_tests` 2 テスト追加（meta 2 件）

---
```

**セパレータは `—`（全角ダッシュ U+2014）**

### 4. その他ドキュメント更新

- `fav/Cargo.toml`: `39.8.0` → `39.9.0`
- `versions/current.md`: 最新安定版 → v39.9.0、次に切る版 → v40.0.0
- `versions/roadmap/roadmap-v39.1-v40.0.md`: v39.9.0 を ✅ 完了済みにマーク

## `enterprise-governance.mdx` の詳細内容

### 機能一覧テーブル

| バージョン | 機能 | コマンド / Rune |
|---|---|---|
| v39.1.0 | RBAC | `auth.require_role` / `auth.check_permission` / `auth.verify_jwt` |
| v39.2.0 | Audit Log | `Audit.log` / `Audit.start_trace` / `Audit.end_trace` |
| v39.3.0 | Policy | `fav policy check` / `fav policy check --ci` |
| v39.4.0 | Secret | `Secret.get_aws` / `Secret.get_vault` / `Secret.get_gcp` / `Secret.get_env` |
| v39.5.0 | マルチテナント | `tenant.db_schema` / `tenant.s3_prefix` / `tenant.validate_tenant` |
| v39.6.0 | fav audit | `fav audit` / `fav audit --check` |
| v39.7.0 | CI/CD ゲート | `fav ci init`（Policy check ステップ自動含む） |
| v39.8.0 | Governance docs | `docs/governance/` 3 件 + cookbook 3 件 |

### v40.0 宣言文（暫定）

> 「RBAC で実行権限を制御し、Audit Log でパイプラインを追跡できる。
>  `fav policy` で組織ポリシーを宣言的に定義し、
>  `fav policy check --ci` で違反を PR でブロックできる。
>  Secret Rune は Vault / AWS / GCP に対応し、
>  マルチテナント対応で複数チームが安全に使える。
>
>  これが Favnir v40.0 — Enterprise Governance の姿である。」

## 注意事項

### 新規 Rust ソースファイル・main.rs 変更なし

v39.9.0 は MDX 1 件と meta テスト 2 件のみ。新規 Rust ソースファイルの作成・`main.rs` への `mod` 追加・ディスパッチアーム追加は不要。`compiler.fav` / `checker.fav` 等のセルフホスト側ファイルへの変更も不要。

### MILESTONE.md は更新しない

MILESTONE.md への Enterprise Governance 記入は v40.0.0 マイルストーン宣言バージョンで行う（v39.9.0 のスコープ外）。

## テスト数の計算

| バージョン | 実績 |
|---|---|
| v39.8.0 | 2808 |
| v39.9.0 追加分 | +2（meta 2 件） |
| v39.9.0 期待値 | 2810 |

## 完了条件

| # | 条件 | 検証方法 |
|---|---|---|
| 1 | `CHANGELOG.md` に `[v39.9.0]` が含まれる | `changelog_has_v39_9_0` テスト |
| 2 | `Cargo.toml` バージョンが `39.9.0` | `cargo_toml_version_is_39_9_0` テスト |
| 3 | `cargo test` 全通過（failures=0 かつテスト数 ≥ 2810） | `cargo test` 実行結果（2808 + 2 = 2810） |
| 4 | `enterprise-governance.mdx` が存在する | T6 で目視確認（`include_str!` は使用しないため cargo での自動検証なし） |
| 5 | `roadmap-v39.1-v40.0.md` の v39.9.0 が ✅ | T6 後に目視確認 |
