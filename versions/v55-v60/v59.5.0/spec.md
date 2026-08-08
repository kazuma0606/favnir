# v59.5.0 Spec — Migration Toolkit（v1 → Enterprise マイグレーション）

Date: 2026-07-30
Status: 設計中

---

## 概要

`fav migrate --from v1 --to enterprise` に Enterprise マイグレーション機能を追加する。

- `--dry-run`: Enterprise 機能移行ガイダンス（W035 / TLS / RBAC / multi-env）を出力
- `--apply`（`--in-place` 相当）: `import rune "X"` → `import X` の W035 自動修正を適用

既存の `cmd_migrate` は `from_version` / `to_version` パラメータを受け取る設計だが、
`to_version == "enterprise"` の分岐が未実装。
本バージョンでは以下 2 つの新規 pub fn を追加することで対応する。

---

## 実装内容

| 項目 | 内容 |
|---|---|
| `fav/src/driver.rs` | `pub fn cmd_migrate_dry_run() -> String` を追加 |
| `fav/src/driver.rs` | `pub fn migrate_enterprise_import(src: &str) -> String` を追加 |
| `fav/src/driver.rs` | `v59500_tests` モジュールを追加（2 件） |
| `fav/Cargo.toml` | バージョン `59.5.0` |

---

## cmd_migrate_dry_run の仕様

サンプルソース（`import rune "kafka"` を含む 2 行）を内部で保持し、
dry-run 時に enterprise 移行ガイダンスを文字列で返す。

```
[analyze] pipeline.fav
  [WARN] import rune "kafka" → import kafka  (W035: legacy_import_rune)
  [WARN] !Http effect: add TLS config to fav.toml  (new in v57.3)
  [INFO] No RBAC config detected: add [security.rbac] if needed
  [INFO] No [env.*] sections: consider multi-env config (v58.6)
```

戻り値: `String`（上記ガイダンス文字列）

---

## migrate_enterprise_import の仕様

`import rune "X"` → `import X` の変換（1 行単位、インデント保持）。

- 入力: `"import rune \"kafka\"\nstage Parse: Stream<Event> -> Stream<Order> = |e| Ok(e)"`
- 出力: `"import kafka\nstage Parse: Stream<Event> -> Stream<Order> = |e| Ok(e)"`

`import rune` でない行はそのまま通過させる。
末尾の `\n` は元のソースに従う。

---

## テスト

`v59500_tests` モジュールを `v59400_tests` の直前に挿入（2 件）。
新規 pub fn（`cmd_migrate_dry_run` / `migrate_enterprise_import`）は `cmd_marketplace_publish` の直後に追加し、
テストモジュールは関数追加後に挿入する（plan.md Step 3〜5 参照）。

| テスト名 | 検証内容 |
|---|---|
| `cmd_migrate_dry_run` | `cmd_migrate_dry_run()` 戻り値が `[WARN]` / `import rune` / `RBAC` を含む |
| `cmd_migrate_auto_fix_import` | `migrate_enterprise_import(...)` が `import kafka` を含み `import rune "kafka"` を含まない |

**ベース**: 3316（v59.4.0 実績値）
**完了条件**: 3316 + 2 = **3318 tests passed, 0 failed**

---

## 完了条件

- `v59500_tests::cmd_migrate_dry_run` pass
- `v59500_tests::cmd_migrate_auto_fix_import` pass
- **3318 tests passed, 0 failed**（ベース 3316 + 2）

---

## ローリングチェック更新

既存 7 件のローリングアサーションを `"59.4.0"` → `"59.5.0"` に更新:
- `v59000_tests::cargo_toml_version_is_59_0_0`
- `v58900_tests::cargo_toml_version_is_58_9_0`
- `v58000_tests::cargo_toml_version_is_58_0_0`
- `v57900_tests::cargo_toml_version_is_57_9_0`
- `v57000_tests::cargo_toml_version_is_57_0_0`（rolling check from v57.0.0）
- `v56900_tests::cargo_toml_version_is_56_9_0`（rolling check from v56.9.0）
- `v56300_tests::cargo_toml_version_is_56_3_0`

failure メッセージ 7 件も同様に `"59.5.0"` に更新。
**注意**: `v59000_tests` には rolling check が存在するため更新対象に含む。
`v59100_tests`〜`v59400_tests` は rolling check なし（feature テストのみ）のため更新対象外。

---

## 影響ファイル

| ファイル | 変更内容 |
|---|---|
| `fav/src/driver.rs` | `cmd_migrate_dry_run` / `migrate_enterprise_import` 追加 + `v59500_tests` + ローリングチェック更新 |
| `fav/Cargo.toml` | バージョン `59.5.0` |
| `CHANGELOG.md` | v59.5.0 エントリ追加 |
| `versions/current.md` | 最新安定版を v59.5.0 に更新 |
| `versions/roadmap/roadmap-v59.1-v60.0.md` | v59.5.0 実績欄に完了記録・v59.6.0 ベース数を確定 |
| `versions/v55-v60/v59.5.0/tasks.md` | COMPLETE ステータスに更新・コードレビュー記録追記 |
