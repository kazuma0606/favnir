# v59.6.0 Spec — Enterprise 認定チェックリスト（`fav certify`）

Date: 2026-07-30
Status: 設計中

---

## 概要

`fav certify --level enterprise` コマンドを追加する。
`fav.toml` と CI 設定を解析して Enterprise 1.0 要件の充足を確認し、
証明書 JSON（`enterprise-cert.json`）を生成する。
`enterprise-cert.json` はコマンド実行時のカレントディレクトリに作成される（`--output <path>` オプションは将来課題）。

---

## 実装内容

| 項目 | 内容 |
|---|---|
| `fav/src/driver.rs` | `pub fn cmd_certify() -> String` を追加 |
| `fav/src/driver.rs` | `pub fn generate_enterprise_cert() -> String` を追加 |
| `fav/src/driver.rs` | `v59600_tests` モジュールを追加（2 件） |
| `fav/src/main.rs` | `Some("certify")` アームを追加 |
| `fav/Cargo.toml` | バージョン `59.6.0` |

---

## cmd_certify の仕様

Enterprise 1.0 の 6 項目をチェックし、結果を文字列で返す。
ハードコードされたサンプル設定（RBAC / Secrets / TLS / Audit / Compliance は OK、SLA は WARN）を用いる。

```
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

戻り値: `String`

---

## generate_enterprise_cert の仕様

証明書 JSON 文字列を返す。以下のフィールドを含む。

```json
{
  "version": "enterprise-1.0",
  "issued_at": "2026-07-30",
  "checks_passed": 5,
  "checks_total": 6,
  "warnings": 1,
  "certification": "Enterprise 1.0 (5/6 passed, 1 warning)"
}
```

戻り値: `String`（JSON 文字列）

**注意**: `issued_at` は v59.6.0 リリース日を固定値としてハードコードする（動的生成は将来課題）。
JSON 構文正当性の検証はテスト内では部分文字列一致のみで行う（`serde_json` による parse チェックは将来課題）。

---

## テスト

`v59600_tests` モジュールを `v59500_tests` の直前（`// ─────────` セパレータ行の後ろ、`// -- v59500_tests` コメント行の前）に挿入（2 件）。
新規 pub fn は `migrate_enterprise_import` の直後に追加し、テストモジュールは関数追加後に挿入する。
テスト関数名（`cmd_certify_passes` / `cmd_certify_generates_cert`）は pub fn 名と一致しないため、`use super::*;` を使用してよい（v59500_tests とは異なる）。

| テスト名 | 検証内容 |
|---|---|
| `cmd_certify_passes` | `cmd_certify()` 戻り値が `[OK]` / `RBAC` / `5/6 checks passed` を含む |
| `cmd_certify_generates_cert` | `generate_enterprise_cert()` 戻り値が `enterprise-1.0` / `checks_passed` / `certification` を含む |

**ベース**: 3318（v59.5.0 実績値）
**完了条件**: 3318 + 2 = **3320 tests passed, 0 failed**

---

## CLI 接続（main.rs）

`Some("certify")` アームを `Some("migrate")` の直後に追加。

```
fav certify --level enterprise      → cmd_certify() + generate_enterprise_cert()
```

`--level` 引数を解析し、`enterprise` の場合は `cmd_certify()` を出力して
`generate_enterprise_cert()` の内容を `enterprise-cert.json` に書き出す。

---

## ローリングチェック更新

既存 7 件のローリングアサーションを `"59.5.0"` → `"59.6.0"` に更新:
- `v59000_tests::cargo_toml_version_is_59_0_0`
- `v58900_tests::cargo_toml_version_is_58_9_0`
- `v58000_tests::cargo_toml_version_is_58_0_0`
- `v57900_tests::cargo_toml_version_is_57_9_0`
- `v57000_tests::cargo_toml_version_is_57_0_0`（rolling check from v57.0.0）
- `v56900_tests::cargo_toml_version_is_56_9_0`（rolling check from v56.9.0）
- `v56300_tests::cargo_toml_version_is_56_3_0`

failure メッセージ 7 件も同様に `"59.6.0"` に更新。
**注意**: `v59000_tests` は rolling check あり（対象）。`v59100_tests`〜`v59500_tests` は rolling check なし（対象外）。

---

## 影響ファイル

| ファイル | 変更内容 |
|---|---|
| `fav/src/driver.rs` | `cmd_certify` / `generate_enterprise_cert` 追加 + `v59600_tests` + ローリングチェック更新 |
| `fav/src/main.rs` | `Some("certify")` アーム追加 + インポート追加 |
| `fav/Cargo.toml` | バージョン `59.6.0` |
| `CHANGELOG.md` | v59.6.0 エントリ追加 |
| `versions/current.md` | 最新安定版を v59.6.0 に更新 |
| `versions/roadmap/roadmap-v59.1-v60.0.md` | v59.6.0 実績欄に完了記録・v59.7.0 ベース数を確定 |
| `versions/v55-v60/v59.6.0/tasks.md` | COMPLETE ステータスに更新 |
