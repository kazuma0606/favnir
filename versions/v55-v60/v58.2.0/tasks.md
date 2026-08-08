# Tasks — v58.2.0 — カナリアリリース

## ステータス: COMPLETE

---

## 事前確認（T0）

- [x] `versions/roadmap/roadmap-v58.1-v59.0.md` の v58.2.0 セクションを確認
- [x] ベーステスト数 3279（v58.1.0 完了時点の実績値）を確認 — `cargo test 2>&1 | grep 'tests passed'` で 3279 であることを数値確認する（サブロードマップ記載の「3274」は誤値）
- [x] `fav/Cargo.toml` が `58.1.0` であることを確認（更新前）
- [x] `v58200_tests` が `driver.rs` に存在しないことを確認（新規追加対象）
- [x] `v58100_tests` が `driver.rs` に存在することを確認（`v58200_tests` の挿入位置として使用）
- [x] `cmd_deploy_strategy` に `"canary"` アームが存在しないことを確認（更新対象）
- [x] main.rs のディスパッチ条件が `sub == "rollback" || has_strategy` であることを確認（更新対象）
- [x] rolling チェック 5 件（v56300 / v56900 / v57000 / v57900 / v58000）が `"58.1.0"` を期待していることを確認

---

## 実装タスク（順序厳守）

- [x] T1: `fav/Cargo.toml` version を `58.2.0` に更新
- [x] T2: `fav/src/driver.rs` — `cmd_deploy_strategy` を拡張
  - [x] `is_rollback` フラグを廃止し `match sub` パターンに統合
  - [x] `"rollback"` アーム: 既存の出力を維持
  - [x] `"promote"` アーム: `"Canary promoted to 100% traffic."` を出力して 0 を返す
  - [x] `"abort"` アーム: `"Canary aborted. Traffic reverted to stable."` を出力して 0 を返す
  - [x] `"status"` アーム: ヘルス情報を出力して 0 を返す
  - [x] `match strategy` に `"canary"` アームを追加
  - [x] `--canary-weight` フラグのパースを追加（デフォルト 10）
  - [x] `v58100_tests::cmd_deploy_unknown_strategy` の引数が `"invalid-strategy"` に更新されている（T2 で実施）
- [x] T3: `fav/src/main.rs` — ディスパッチ条件を更新
  - [x] `let is_canary_sub = matches!(sub, "rollback" | "promote" | "abort" | "status");`
  - [x] `if is_canary_sub || has_strategy {` に変更
- [x] T4: `fav/src/driver.rs` — `v58200_tests` モジュールを `v58100_tests` の直前に追加
  - [x] `cmd_deploy_canary_weight`: canary deploy で exit code 0 を検証
  - [x] `cmd_deploy_canary_promote`: promote で exit code 0 を検証
  - [x] `use super::cmd_deploy_strategy;` をモジュール内に追加
- [x] T5: rolling バージョンチェック 5 件を `"58.1.0"` → `"58.2.0"` に更新
  - [x] v56300_tests
  - [x] v56900_tests
  - [x] v57000_tests
  - [x] v57900_tests
  - [x] v58000_tests

---

## テスト・検証

- [x] T6: `cargo build` でコンパイルエラーがないことを確認
- [x] T7: `cargo test` 全通過（**3283 tests passed, 0 failed**）（code-review 対応で +2 追加）
  - [x] `v58200_tests::cmd_deploy_canary_weight` ok
  - [x] `v58200_tests::cmd_deploy_canary_promote` ok
  - [x] `v58200_tests::cmd_deploy_canary_abort` ok（code-review 対応で追加）
  - [x] `v58200_tests::cmd_deploy_canary_status` ok（code-review 対応で追加）
  - [x] `v58100_tests::cmd_deploy_blue_green` / `cmd_deploy_rollback` 引き続き通過
  - [x] `v58100_tests::cmd_deploy_unknown_strategy` は引数を `"invalid-strategy"` に変更後に通過（T2 で更新済みのため）
  - [x] 既存 3279 件全通過
- [x] T8: `cargo clippy -- -D warnings` クリーン

---

## ポスト処理

- [x] T9: `CHANGELOG.md` に v58.2.0 エントリを追加（形式: `## [v58.2.0] — 2026-07-28 — カナリアリリース`）
- [x] T10: `versions/current.md` を v58.2.0 / 3281 tests に更新
- [x] T11: `versions/roadmap/roadmap-v58.1-v59.0.md` の v58.2.0 実績を COMPLETE に更新
  - [x] `3279 + 2 = 3281 tests passed, 0 failed（2026-07-28）` を追記
- [x] T12: `versions/v55-v60/v58.2.0/tasks.md` を COMPLETE に更新

---

## 完了確認

- [x] `cmd_deploy_canary_weight` pass
- [x] `cmd_deploy_canary_promote` pass
- [x] **3283 tests passed, 0 failed**（ベース 3279 + 4、code-review 対応で +2）
- [x] `cargo clippy -- -D warnings` クリーン
- [x] `fav/src/main.rs` のディスパッチが `is_canary_sub || has_strategy` になっている
- [x] rolling チェック 5 件が `"58.2.0"` になっている
- [x] `CHANGELOG.md` に `[v58.2.0]` エントリが追加されている
- [x] `versions/current.md` が v58.2.0 / 3281 tests を反映

---

## 実装メモ

- `cmd_deploy_strategy` のリファクタリング: `is_rollback` bool フラグ → `match sub` パターンに統合（より拡張しやすい設計）
- リファクタリング後も `v58100_tests::cmd_deploy_rollback` が通過することを確認済み
- rolling チェックは全バージョンで 5 件全件更新が必要（v56300 / v56900 / v57000 / v57900 / v58000）
- `--canary-weight` 未指定時のデフォルト値: `10`（ロードマップ例示に合わせた）
- `cmd_deploy_unknown_strategy` のテスト引数を `"canary"` → `"invalid-strategy"` に変更（canary が valid strategy になったため）
