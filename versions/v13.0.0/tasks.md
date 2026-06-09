# Favnir v13.0.0 Tasks

Date: 2026-06-09
Theme: 言語信頼性宣言 — E2E デモ W006 修正 + CHANGELOG/README 更新 + バージョン宣言

---

## Phase A — fav2py pipeline.fav W006 修正

- [ ] A-1: `infra/e2e-demo/fav2py/src/pipeline.fav` の `bind _ <- Postgres.execute_raw(...)` を `chain _ <- Postgres.execute_raw(...)` に変更（4 箇所）
  - `CREATE TABLE IF NOT EXISTS txn ...`
  - `DELETE FROM txn`
  - `INSERT INTO txn ...`
  - ※ `IO.println(...)` は Unit 戻り値 → 変更不要
- [ ] A-2: `./target/debug/fav lint --deny-warnings ../infra/e2e-demo/fav2py/src/pipeline.fav` → exit 0 を確認
- [ ] A-3: `./target/debug/fav check ../infra/e2e-demo/fav2py/src/pipeline.fav` → no errors を確認

---

## Phase B — airgap analyze.fav 確認

- [ ] B-1: `./target/debug/fav lint --deny-warnings ../infra/e2e-demo/airgap/src/analyze.fav` → exit 0 を確認（変更不要のはず）

---

## Phase C — CHANGELOG.md 更新

- [ ] C-1: v12.1.0 エントリを CHANGELOG.md に追記
  - `bind` 再束縛禁止（E0018）+ `checker.fav` 実装 + 1353 tests
- [ ] C-2: v12.2.0 エントリを追記
  - `bind _` + Result 戻り値 → W006 警告（`is_result_returning_call`）
- [ ] C-3: v12.3.0 エントリを追記
  - `bind` → monadic bind 修正（`--legacy` モード、LegacyBindCheck opcode）
- [ ] C-4: v12.4.0 エントリを追記
  - `seq` pipeline fail-fast（SeqStageCheck opcode）
- [ ] C-5: v12.5.0 エントリを追記
  - `fav run --verbose/--trace` + `fav check --json/--show-types` + 1386 tests
- [ ] C-6: v12.6.0 エントリを追記
  - Postgres Rune TLS 対応（sslmode=disable/prefer/require）+ エラー詳細化
- [ ] C-7: v12.7.0 エントリを追記
  - `fav doc --builtins [--format json]` + `fav explain <code>`
- [ ] C-8: v12.8.0 エントリを追記
  - `fav scaffold <template>` — stage / seq / postgres-etl / rune
- [ ] C-9: v12.9.0 エントリを追記
  - CI `fav test self/*.fav` + `services: postgres:16` 統合テスト + 1415 tests
- [ ] C-10: v12.10.0 エントリを追記
  - 全エラー `help:` + `fav check --strict` + `fav lint --deny-warnings` + `fav.toml [lint]`
- [ ] C-11: v13.0.0 エントリを追記
  - 言語信頼性宣言 + fav2py W006 修正

---

## Phase D — README.md 更新

- [ ] D-1: README.md の `v12.0.0` 宣言行の後に v13.0.0 宣言文を追記
  ```
  v13.0.0（2026-06-09）で、言語信頼性宣言を完了しました。
  型安全・エラー伝播・デバッグ可視性の三点において、Favnir のランタイム挙動は
  型システムの宣言と一致することを保証します。
  ```

---

## Phase E — バージョン更新・テスト・コミット

- [ ] E-1: `fav/Cargo.toml` version → `"13.0.0"`
- [ ] E-2: `driver.rs` の `version_is_12_10_0` を comment out（次バージョンテストに委譲）
- [ ] E-3: `driver.rs` 末尾に `v130000_tests` モジュールを追加
  - `version_is_13_0_0` — `CARGO_PKG_VERSION == "13.0.0"`
  - `fav2py_pipeline_no_w006` — pipeline.fav に W006 がないこと
- [ ] E-4: `cargo build` — Cargo.lock 更新
- [ ] E-5: `cargo test` — 全通過確認
- [ ] E-6: `git commit -m "feat: v13.0.0 — 言語信頼性宣言"`
- [ ] E-7: `git push` → CI 確認（`gh run watch`）

---

## 完了条件サマリー

| 確認項目 | 状態 |
|---|---|
| fav2py/pipeline.fav で `fav lint --deny-warnings` exit 0 | |
| airgap/analyze.fav で `fav lint --deny-warnings` exit 0 | |
| CHANGELOG.md に v12.1.0〜v13.0.0 全エントリ記載 | |
| README.md に v13.0.0 宣言文あり | |
| `CARGO_PKG_VERSION == "13.0.0"` | |
| `cargo test` 全通過 | |
| CI 全 green | |
