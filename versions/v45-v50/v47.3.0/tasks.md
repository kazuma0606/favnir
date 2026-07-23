# Tasks: v47.3.0 — `List.scan` / `List.take_while` / `List.drop_while`

Status: COMPLETE
Date: 2026-07-17

---

## T0 — 事前確認

- [x] `cargo test` 3021 passed, 0 failed を確認
- [x] `vm.rs` に `List.take_while`（line 3389）・`List.drop_while`（line 3422）・`List.scan`（line 3466）の実装が存在することを確認
- [x] `checker.rs` に `("List", "take_while")` / `("List", "drop_while")`（line 5992）・`("List", "scan")`（line 6005）が存在することを確認

## T1 — `driver.rs` に `v473000_tests` 追加

- [x] `v472000_tests` の直後に `v473000_tests` モジュールを追加（3 テスト）
  - [x] `list_scan_cumulative`: `scan(range(1,4), 0, |acc,x| acc+x)` → `length == 4`（init値含む）
  - [x] `list_take_while`: `take_while(range(1,6), |x| x<3)` → `length == 2`
  - [x] `list_drop_while`: `drop_while(range(1,6), |x| x<3)` → `length == 3`

## T2 — バージョン更新・テスト・完了

- [x] `fav/Cargo.toml` version → `"47.3.0"`
- [x] `CHANGELOG.md` に v47.3.0 エントリ追加
- [x] `cargo test` 3024 passed, 0 failed（3021 + 3 件）
- [x] `cargo clippy -- -D warnings` クリーン
- [x] `versions/current.md` を v47.3.0（3024 tests）に更新、進行中バージョンを `v47.4.0` に更新
- [x] `versions/roadmap/roadmap-v47.1-v48.0.md` の v47.3.0〜v48.0 完了条件テスト数を実績ベースに全更新
- [x] tasks.md を COMPLETE に更新（T0〜T2 全チェック）

---

## コードレビュー指摘と対応（spec-reviewer）

| 重大度 | 内容 | 対応 |
|---|---|---|
| [HIGH] | ロードマップの推定テスト数が全バージョンで -5 ずれ | v47.3〜v48.0 の全推定値を更新（3024/3027/3030/3033/3036/3039/3041/≥3045） |
| [MED] | ロードマップの `List.scan(init, f)` が引数順を誤解させる | `List.scan(list, init, f)` に修正し「初期値を含む」の注釈を追加 |
| [LOW] | tasks.md にロードマップ修正タスクが未記載 | T2 にチェックボックスを追加 |
| [LOW] | tasks.md の `current.md` 更新に進行中バージョンが未明示 | `進行中バージョンを v47.4.0 に更新` を追記 |
