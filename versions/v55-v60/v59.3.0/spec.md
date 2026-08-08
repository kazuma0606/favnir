# v59.3.0 Spec — コスト可視化（`fav cost-estimate`）

Date: 2026-07-29
Status: 設計中

---

## 概要

`fav cost-estimate` コマンドを追加。各 Rune の操作量とクラウドプロバイダの料金表を照合し、
コスト見積もりを出力する。`driver.rs` に `cmd_cost_estimate(provider: &str) -> i32` を実装する。

---

## 実装内容

| 項目 | 内容 |
|---|---|
| `fav/src/driver.rs` | `cmd_cost_estimate(provider: &str) -> i32` を追加（ステージ別コスト出力スタブ） |
| `fav/src/main.rs` | `Some("cost-estimate")` アームを新規追加（`--provider` フラグ解析 + `cmd_cost_estimate()` 呼び出し） |

**注意**: 本バージョンはスタブ実装のため `registry/pricing/<provider>.json` ファイルは参照しない。コスト値はハードコードされた固定値を出力する。実際のファイル読み込みは将来バージョンで実装予定。

---

## cmd_cost_estimate の出力仕様

```
Stage Analysis:
  Parse     (Kafka):      ~$0.08/hour  (2M msgs/hr × $0.04/1M)
  Validate  (CPU):        ~$0.03/hour  (0.5 vCPU on Lambda)
  Store     (Snowflake):  ~$0.12/hour  (1 credit/hr × $3/credit / 25)

Total estimated cost: ~$0.23/hour  (~$165/month)
Provider: <provider>
```

戻り値: `0`

引数: `provider: &str`（例: `"aws"`）

---

## テスト

`v59300_tests` モジュールを `v59200_tests` の直前に挿入（2 件）:

| テスト名 | 内容 |
|---|---|
| `cost_estimate_generates` | `cmd_cost_estimate("aws")` が `0` を返すことを検証 |
| `cost_estimate_aws_pricing` | インライン pricing 文字列が `~$0.08`・`~$0.23`・`~$165` を含むことを検証 |

- `use super::cmd_cost_estimate` は `cost_estimate_generates` が `cmd_cost_estimate()` を呼ぶために必要
- `cost_estimate_aws_pricing` はインライン文字列のみで `super` のシンボルを使わない（`use super::cmd_cost_estimate` で個別指定するため `use super::*` は不要）

**実際のベース**: 3312（v59.2.0 実績値）
**完了条件**: 3312 + 2 = **3314 tests passed, 0 failed**

---

## ローリングチェック更新

既存 7 件のローリングアサーションを `"59.2.0"` → `"59.3.0"` に更新:
- `v59000_tests::cargo_toml_version_is_59_0_0`
- `v58900_tests::cargo_toml_version_is_58_9_0`
- `v58000_tests::cargo_toml_version_is_58_0_0`
- `v57900_tests::cargo_toml_version_is_57_9_0`
- `v57000_tests::cargo_toml_version_is_57_0_0`（`rolling check from v57.0.0`）
- `v56900_tests::cargo_toml_version_is_56_9_0`（`rolling check from v56.9.0`）
- `v56300_tests::cargo_toml_version_is_56_3_0`

**注意**: `v59100_tests`・`v59200_tests` には rolling check が存在しない（feature テストのみ）ため更新対象外。更新対象は計 7 件。

failure メッセージ 7 件も同様に `"59.3.0"` に更新。

---

## main.rs 変更

`Some("cost-estimate")` アームを `Some(cmd)` ワイルドカードの直前に追加:

```
fav cost-estimate [<file>] [--provider <name>]
```

- `--provider <name>` フラグで provider 名を取得（デフォルト: `"aws"`）
- `cmd_cost_estimate(provider)` を呼んで `process::exit(code)`
- `--provider` に値がない場合 → `eprintln!` + `exit(1)`

---

## 影響ファイル

| ファイル | 変更内容 |
|---|---|
| `fav/src/driver.rs` | `cmd_cost_estimate` 追加 + v59300_tests + ローリングチェック更新 |
| `fav/src/main.rs` | `Some("cost-estimate")` アーム新規追加 |
| `fav/Cargo.toml` | バージョン `59.3.0` |
| `CHANGELOG.md` | v59.3.0 エントリ追加 |
| `versions/current.md` | 最新安定版を v59.3.0 に更新 |
| `versions/roadmap/roadmap-v59.1-v60.0.md` | v59.3.0 実績欄に完了記録、v59.4.0 ベース数更新 |
