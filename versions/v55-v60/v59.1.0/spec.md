# v59.1.0 Spec — エンタープライズ E2E ハーネス強化

Date: 2026-07-29
Status: 設計中

---

## 概要

`examples/enterprise-demo/` ディレクトリに全エンタープライズ機能を統合したデモを作成。
`fav test --suite enterprise` コマンドを追加し、`driver.rs` に `cmd_test_enterprise` スタブを実装する。

---

## 実装内容

| 項目 | 内容 |
|---|---|
| `examples/enterprise-demo/pipeline.fav` | 8 機能を統合したエンタープライズデモパイプライン（新規作成） |
| `fav/src/driver.rs` | `cmd_test_enterprise() -> i32` を追加（8 チェック出力スタブ） |
| `fav/src/main.rs` | `Some("test")` アームに `--suite enterprise` フラグ検出を追加 |

---

## cmd_test_enterprise の出力仕様

```
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

戻り値: `0`

---

## examples/enterprise-demo/pipeline.fav の要件

- **`"RBAC"` という文字列を含む**（テストで検証）
- Blue/Green・Secret・mTLS・監査ログ・コンプライアンス・ポリシー・カタログへのコメント言及

---

## テスト

`v59100_tests` モジュールを `v59000_tests` の直前に挿入（2 件）:

| テスト名 | 内容 |
|---|---|
| `enterprise_e2e_demo_structure` | `include_str!("../../examples/enterprise-demo/pipeline.fav")` が `"RBAC"` を含むことを検証 |
| `cmd_test_enterprise_suite` | `cmd_test_enterprise()` が `0` を返すことを検証 |

- `use super::cmd_test_enterprise` が必要（`cmd_test_enterprise_suite` が `super` の関数を呼ぶため）
- `enterprise_e2e_demo_structure` は `include_str!` のみで `use super::*` 不要

**実際のベース**: 3308（v59.0.0 実績値）
**完了条件**: 3308 + 2 = **3310 tests passed, 0 failed**

---

## ローリングチェック更新

既存 7 件のローリングアサーションを `"59.0.0"` → `"59.1.0"` に更新:
- `v59000_tests::cargo_toml_version_is_59_0_0`
- `v58900_tests::cargo_toml_version_is_58_9_0`
- `v58000_tests::cargo_toml_version_is_58_0_0`
- `v57900_tests::cargo_toml_version_is_57_9_0`
- `v57000_tests::cargo_toml_version_is_57_0_0`（`rolling check from v57.0.0`）
- `v56900_tests::cargo_toml_version_is_56_9_0`（`rolling check from v56.9.0`）
- `v56300_tests::cargo_toml_version_is_56_3_0`

failure メッセージ 7 件も同様に更新。

---

## main.rs 変更

`Some("test")` アームの既存フラグ解析ループ内に `"--suite"` フラグを追加:
- `--suite enterprise` → `cmd_test_enterprise()` を呼んで `return;`
- `--suite` に値がない場合 → `eprintln!` + `exit(1)`

---

## 影響ファイル

| ファイル | 変更内容 |
|---|---|
| `examples/enterprise-demo/pipeline.fav` | 新規作成 |
| `fav/src/driver.rs` | `cmd_test_enterprise` 追加 + v59100_tests + ローリングチェック更新 |
| `fav/src/main.rs` | `Some("test")` アームに `--suite enterprise` 対応追加 |
| `fav/Cargo.toml` | バージョン `59.1.0` |
| `CHANGELOG.md` | v59.1.0 エントリ追加 |
| `versions/current.md` | 最新安定版を v59.1.0 に更新 |
| `versions/roadmap/roadmap-v59.1-v60.0.md` | v59.1.0 実績欄に完了記録 |
