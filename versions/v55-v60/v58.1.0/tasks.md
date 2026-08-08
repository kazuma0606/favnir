# Tasks — v58.1.0 — Blue/Green デプロイメントサポート

## ステータス: COMPLETE

---

## 事前確認（T0）

- [x] `versions/roadmap/roadmap-v58.1-v59.0.md` の v58.1.0 セクションを確認
- [x] ベーステスト数 3276（v58.0.0 完了時点の実績値）を確認（サブロードマップ記載の 3272 は v58.0.0 実装前予測値で 4 件差あり）— `cargo test 2>&1 | grep 'tests passed'` で件数が 3276 であることを数値確認する
- [x] `fav/Cargo.toml` が `58.0.0` であることを確認（更新前）
- [x] `v58100_tests` が `driver.rs` に存在しないことを確認（新規追加対象）
- [x] `v58000_tests` が `driver.rs` に存在することを確認（`v58100_tests` の挿入位置として使用）
- [x] `cmd_deploy` が `driver.rs` に存在することを確認 → 既存関数と名前衝突のため `cmd_deploy_strategy` として実装
- [x] `infra/deploy/blue-green/main.tf` が存在しないことを確認（新規作成対象）
- [x] `fav/src/main.rs` の `Some("deploy")` アームを確認（既存アームに振り分けロジックを追加）

---

## 実装タスク（順序厳守）

- [x] T1: `fav/Cargo.toml` version を `58.1.0` に更新
- [x] T2: `fav/src/driver.rs` — `pub fn cmd_deploy_strategy(args: &[String]) -> i32` 追加
  - [x] `--strategy blue-green` で exit code 0 を返す
  - [x] `rollback` サブコマンド（`args[0] == "rollback"`）で exit code 0 を返す
  - [x] 未知の strategy で exit code 1 を返す
  - [x] 既存 `cmd_deploy` と名前衝突のため `cmd_deploy_strategy` として実装（spec 名称変更）
- [x] T3: `fav/src/main.rs` — 既存 `Some("deploy")` アームに振り分けロジックを追加
  - [x] `rollback` サブコマンドまたは `--strategy` フラグがある場合 `cmd_deploy_strategy` へ委譲
- [x] T4: `infra/deploy/blue-green/main.tf` — Terraform スタブを新規作成
  - [x] `variable "env"` / `locals.blue_slot` / `locals.green_slot` / `output` を含む
- [x] T5: `fav/src/driver.rs` — `v58100_tests` モジュールを `v58000_tests` の直前に追加
  - [x] `cmd_deploy_blue_green`: blue-green deploy で exit code 0 を検証
  - [x] `cmd_deploy_rollback`: rollback で exit code 0 を検証
  - [x] `use super::cmd_deploy_strategy;` をモジュール内に追加
- [x] T6: rolling バージョンチェック 5 件を `58.0.0` → `58.1.0` に更新
  - [x] v56300 / v56900 / v57000 / v57900 / v58000（全件更新が必要 — 非宣言バージョンでも Cargo.toml 連動で必須）

---

## テスト・検証

- [x] T7: `cargo build` でコンパイルエラーがないことを確認
- [x] T8: `cargo test` 全通過（**3278 tests passed, 0 failed**）
  - [x] `v58100_tests::cmd_deploy_blue_green` ok
  - [x] `v58100_tests::cmd_deploy_rollback` ok
  - [x] 既存 3276 件全通過
- [x] T9: `cargo clippy -- -D warnings` クリーン

---

## ポスト処理

- [x] T10: `CHANGELOG.md` に v58.1.0 エントリを追加
- [x] T11: `versions/current.md` を v58.1.0 / 3278 tests に更新
- [x] T12: `versions/roadmap/roadmap-v58.1-v59.0.md` の v58.1.0 実績を COMPLETE に更新
  - [x] `3276 + 2 = 3278 tests passed, 0 failed（2026-07-28）` を追記
- [x] T13: `versions/v55-v60/v58.1.0/tasks.md` を COMPLETE に更新

---

## 完了確認

- [x] `cmd_deploy_blue_green` pass
- [x] `cmd_deploy_rollback` pass
- [x] **3279 tests passed, 0 failed**（ベース 3276 + 3、code-review 対応で +1）
- [x] `cargo clippy -- -D warnings` クリーン
- [x] `infra/deploy/blue-green/main.tf` が存在する
- [x] `fav/src/main.rs` に `Some("deploy")` 振り分けロジック（blue-green / rollback）が追加されている
- [x] `CHANGELOG.md` に `[v58.1.0]` エントリが追加されている
- [x] `versions/current.md` が v58.1.0 / 3278 tests を反映

---

## 実装メモ

- `cmd_deploy_strategy` として実装（spec では `cmd_deploy` と記載していたが、既存の `pub fn cmd_deploy(env, function_name, ...)` と名前衝突のため変更）
- main.rs の既存 `Some("deploy")` アームに振り分けロジックを追加（`rollback` または `--strategy` が含まれる場合 `cmd_deploy_strategy` へ委譲）
- rolling バージョンチェックは非宣言バージョンでも Cargo.toml 版数変更に追随して全件更新が必要（v56300 / v56900 / v57000 / v57900 / v58000 の 5 件）
- 今後の仕様書では rolling チェック更新を「全バージョンで必須」と明記すること
