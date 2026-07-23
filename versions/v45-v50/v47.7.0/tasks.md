# Tasks: v47.7.0 — `Result` 拡充

Status: COMPLETE
Date: 2026-07-17

---

## T0 — 事前確認

- [x] `cargo test` 3033 passed, 0 failed を確認
- [x] `vm.rs` に `Result.map`（4379）・`Result.map_err`（4406）・`Result.and_then`（4433）が存在することを確認
- [x] `vm.rs` に `Result.is_ok`（4499）・`Result.is_err`（4519）が存在することを確認
- [x] `checker.rs` に `("Result", "map")` / `("Result", "map_err")` / `("Result", "and_then")` が存在することを確認（line 6153/6176/6168）
- [x] `checker.rs` に `("Result", "is_ok")` / `("Result", "is_err")` が存在することを確認（line 6187）

## T1 — `driver.rs` に `v477000_tests` 追加

- [x] `v476000_tests` の直前に `v477000_tests` モジュールを追加（3 テスト）
  - [x] `result_map`: `Result.map(ok(5), |n| n*2)` → `unwrap_or(0)` == `10`
  - [x] `result_map_err`: `Result.map_err(err("oops"), |e| "wrapped: " ++ e)` → `Result.is_err` == `true`
  - [x] `result_and_then`: `Result.and_then(ok(5), |n| ok(n+1))` → `unwrap_or(0)` == `6`

## T2 — バージョン更新・テスト・完了

- [x] `fav/Cargo.toml` version → `"47.7.0"`
- [x] `CHANGELOG.md` に v47.7.0 エントリ追加
- [x] `cargo test` 3036 passed, 0 failed（3033 + 3 件）
- [x] `cargo clippy -- -D warnings` クリーン
- [x] `versions/current.md` を v47.7.0（3036 tests）に更新、進行中バージョンを `v47.8.0` に更新
- [x] `versions/roadmap/roadmap-v47.1-v48.0.md` の v47.7.0 完了条件テスト数（3036）を実績で確認・必要に応じて更新
- [x] tasks.md を COMPLETE に更新（T0〜T2 全チェック）

> **注記**: `site/content/docs/stdlib/` の Result ドキュメント更新は v47.9.0 stdlib ドキュメントスプリントで一括実施（本バージョンでは対象外）

---

## コードレビュー指摘と対応（spec-reviewer）

| 重大度 | 内容 | 対応 |
|---|---|---|
| [MED] | `result_map_err` テストが `is_err` のみで変換内容未検証 | `match` + 文字列直接比較に変更（spec/plan 両方更新） |
| [MED] | spec の注意事項で「is_err が明確」の根拠が不正確 | `match` 採用の理由を正確に書き直し |
| [LOW] | T0 に `is_ok`/`is_err` の確認行が欠落 | vm.rs・checker.rs の行番号確認を T0 に追加 |
| [LOW] | `site/` MDX 更新の扱いが未明記 | T2 末尾に「v47.9.0 で一括実施」の注記追加 |
