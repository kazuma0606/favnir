# v59.4.0 Spec — Rune マーケットプレイス Phase 1（`fav marketplace`）

Date: 2026-07-29
Status: COMPLETE（2026-07-29）

---

## 概要

既存の `fav publish`（v29.1 実装済み `cmd_publish`）を Marketplace 向けに拡張。
`fav marketplace list` / `fav marketplace search <query>` / `fav marketplace publish --rune <name>`
コマンドを追加し、`driver.rs` に `cmd_marketplace_list` / `cmd_marketplace_publish` スタブを実装する。

**スコープ**: Phase 1 では `list` / `search` / `publish` の基本コマンドのみ実装する。
ロードマップに記載の「エンタープライズ向け Private Registry サポート」はスタブ実装の範囲外とし、Phase 2 以降（v59.x 後続バージョン）で対応する。

---

## 実装内容

| 項目 | 内容 |
|---|---|
| `fav/src/driver.rs` | `cmd_marketplace_list() -> i32` を追加（Rune 一覧出力スタブ） |
| `fav/src/driver.rs` | `cmd_marketplace_publish(rune: &str) -> i32` を追加（publish スタブ） |
| `fav/src/main.rs` | `Some("marketplace")` アームを新規追加（`list` / `search` / `publish` サブコマンド対応） |

---

## cmd_marketplace_list の出力仕様

```
Name          Author          Downloads  License
kafka         favnir-official  12,450    MIT
snowflake     favnir-official   8,320    MIT
salesforce    acme-corp           920    Apache-2.0
```

戻り値: `0`

---

## cmd_marketplace_publish の出力仕様

```
Publishing rune '<name>' to Favnir Marketplace...
[OK] Rune '<name>' published successfully.
```

引数: `rune: &str`
戻り値: `0`

---

## テスト

`v59400_tests` モジュールを `v59300_tests` の直前に挿入（2 件）:

| テスト名 | 内容 |
|---|---|
| `cmd_marketplace_list` | `cmd_marketplace_list()` が `0` を返すことを検証 |
| `cmd_marketplace_publish` | `cmd_marketplace_publish("my-rune")` が `0` を返すことを検証 |

- `use super::cmd_marketplace_list` および `use super::cmd_marketplace_publish` が必要
- テスト名がそのまま関数名と一致する（`fn cmd_marketplace_list()` / `fn cmd_marketplace_publish()`）

**実際のベース**: 3314（v59.3.0 実績値）
**完了条件**: 3314 + 2 = **3316 tests passed, 0 failed**

---

## 完了条件

- `v59400_tests::cmd_marketplace_list` pass
- `v59400_tests::cmd_marketplace_publish` pass
- **3316 tests passed, 0 failed**（ベース 3314 + 2）

---

## ローリングチェック更新

既存 7 件のローリングアサーションを `"59.3.0"` → `"59.4.0"` に更新:
- `v59000_tests::cargo_toml_version_is_59_0_0`
- `v58900_tests::cargo_toml_version_is_58_9_0`
- `v58000_tests::cargo_toml_version_is_58_0_0`
- `v57900_tests::cargo_toml_version_is_57_9_0`
- `v57000_tests::cargo_toml_version_is_57_0_0`（`rolling check from v57.0.0`）
- `v56900_tests::cargo_toml_version_is_56_9_0`（`rolling check from v56.9.0`）
- `v56300_tests::cargo_toml_version_is_56_3_0`

**注意**: `v59100_tests`〜`v59300_tests` には rolling check が存在しない（feature テストのみ）ため更新対象外。更新対象は計 7 件。

failure メッセージ 7 件も同様に `"59.4.0"` に更新。

---

## main.rs 変更

`Some("marketplace")` アームを `Some(cmd)` ワイルドカードの直前に追加:

```
fav marketplace list
fav marketplace search <query>
fav marketplace publish --rune <name>
```

- `list` → `cmd_marketplace_list()` を呼んで `process::exit(code)`
- `search <query>` → 検索結果を出力して `process::exit(0)`（スタブ）
- `publish --rune <name>` → `cmd_marketplace_publish(rune)` を呼んで `process::exit(code)`
- 不明サブコマンド → `eprintln!` + `exit(1)`

---

## 影響ファイル

| ファイル | 変更内容 |
|---|---|
| `fav/src/driver.rs` | `cmd_marketplace_list` / `cmd_marketplace_publish` 追加 + v59400_tests + ローリングチェック更新 |
| `fav/src/main.rs` | `Some("marketplace")` アーム新規追加、HELP テキストに `marketplace` 追加 |
| `fav/Cargo.toml` | バージョン `59.4.0` |
| `CHANGELOG.md` | v59.4.0 エントリ追加 |
| `versions/current.md` | 最新安定版を v59.4.0 に更新 |
| `versions/roadmap/roadmap-v59.1-v60.0.md` | v59.4.0 実績欄に完了記録、v59.5.0 ベース数更新 |
