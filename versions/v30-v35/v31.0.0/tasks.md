# v31.0.0 — タスクリスト

**ステータス**: COMPLETE

---

## 前提確認（T0）

- [x] `fav/Cargo.toml` の version が `30.9.0` であること
- [x] `cargo test 2>&1 | grep "test result"` を実行して通過件数（2418 passed 想定）を実測確認すること
- [x] `driver.rs` に `mod v310000_tests` が存在しないこと
- [x] v30.9.0 が COMPLETE であること
- [x] `versions/current.md` の「次に切る版」欄が `v30.9.0` のままであること（T8 で v31.1.0 に修正する）

---

## 実装タスク

- [x] **T1** `fav/Cargo.toml` — version を `30.9.0` → `31.0.0` に更新
- [x] **T2** `fav/src/driver.rs` — `cargo_toml_version_is_30_9_0` をスタブ化
- [x] **T3** `fav/src/driver.rs` — `v310000_tests`（4 件）を追加（`use super::*` なし）
- [x] **T4** `MILESTONE.md` — v31.0.0「Real-World Readiness」セクションを先頭に追記
- [x] **T5** `README.md` — v31.0 マイルストーン一行を追加
- [x] **T6** `CHANGELOG.md` — `[v31.0.0]` セクションを先頭に追記
- [x] **T7** `benchmarks/v31.0.0.json` — 新規作成
- [x] **T8** `versions/current.md` — 「最新安定版」欄を v31.0.0 に更新、「次に切る版」を `v31.1.0 — TBD` に修正、マイルストーン進捗表の `v31.0 — Real-World Readiness` を `**完了**` に変更
- [x] **T9** `cargo clean` 実行 + `fav/tmp/hello.fav` 復元（clean 後も存在を確認）
- [x] **T10** `cargo build` — ビルド成功を確認（`fav v31.0.0` Finished）
- [x] **T11a** `du -sh target/` — ビルドサイズ確認（参考値として記録）

---

## テスト確認

- [x] **T11** `cargo test --bin fav v310000 2>&1 | tail -8` — 4/4 PASS
- [x] **T12** `cargo test 2>&1 | grep "test result"` — 全件 PASS（2422 passed、0 failures）

---

## 完了処理

- [x] **T13** `benchmarks/v31.0.0.json` の `tests_passed` を実測値で更新（2422 — 初期値と一致）
- [x] **T14** このファイル（tasks.md）を COMPLETE に更新（全チェックボックス `[x]`）

---

## 完了条件チェックリスト（spec.md 対応）

- [x] `Cargo.toml` version = `"31.0.0"`
- [x] `MILESTONE.md` に `"Real-World Readiness"` セクション
- [x] `README.md` に `"v31.0"` の記述
- [x] `cargo test v310000` — 4/4 PASS
- [x] `cargo test`（`cargo clean` 後）— 全件 PASS（0 failures）
- [x] `fav/tmp/hello.fav` が `cargo clean` 後に復元されていること
- [x] `CHANGELOG.md` に `[v31.0.0]` セクション
- [x] `benchmarks/v31.0.0.json` 存在
- [x] `benchmarks/v31.0.0.json` の `tests_passed` が実測値で更新されていること（2422）
- [x] `versions/current.md` を v31.0.0 に更新（マイルストーン進捗表 v31.0 → `**完了**`）
- [x] tasks.md が COMPLETE

---

## コードレビューチェックリスト

- [x] `v310000_tests` に `use super::*` がないこと
- [x] `cargo_toml_version_is_30_9_0` が空スタブになっていること（コメント付き）
- [x] `milestone_real_world_readiness_declared` が `"Real-World Readiness"` を検索していること
- [x] `readme_mentions_v31_0` が `"v31.0"` を検索していること
- [x] `benchmark_v31_0_0_exists` が `"31.0.0"` を検索していること
- [x] `fav/tmp/hello.fav` が `cargo clean` 後に復元されていること
- [x] `versions/current.md` のマイルストーン進捗表で v31.0 が `**完了**` になっていること
- [x] `versions/current.md` の「次に切る版」が `v31.1.0 — TBD` になっていること
