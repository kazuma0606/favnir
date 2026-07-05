# v30.1.0 — タスクリスト

**ステータス**: COMPLETE

---

## 前提確認（T0）

- [x] `fav/Cargo.toml` の version が `30.0.0` であること
- [x] `cargo test --bin fav 2>&1 | grep "^test result"` が `2372 passed` を含むこと
- [x] `driver.rs` に `mod v301000_tests` が存在しないこと
- [x] `fav/Cargo.toml` に `[profile.dev]` セクションが存在しないこと
- [x] v30.0.0 が COMPLETE であること

---

## 実装タスク

- [x] **T1** `fav/Cargo.toml` — version を `30.0.0` → `30.1.0` に更新
- [x] **T2** `fav/Cargo.toml` — `[profile.dev]` セクションを末尾に追加（`debug = 0` / `split-debuginfo = "off"`）
- [x] **T3** ビルド確認 — `cargo build 2>&1 | tail -3` が `Finished` を含むこと
- [x] **T4** `fav/src/driver.rs` — `v301000_tests`（6 件）を末尾に追加
- [x] **T5** `CHANGELOG.md` — `[v30.1.0]` セクションを先頭に追記
- [x] **T6** `benchmarks/v30.1.0.json` — 新規作成（test_count: 2378）
- [x] **T6b** `versions/current.md` — 進行中バージョンを `v30.1.0` に更新

---

## テスト確認

- [x] **T7** `cargo test --bin fav v301000 2>&1 | tail -5` — 6/6 PASS
- [x] **T8** `cargo test 2>&1 | grep "test result"` — `2378 passed` を含むこと（0 failures）

---

## 完了処理

- [x] **T9** このファイル（tasks.md）を COMPLETE に更新（全チェックボックス `[x]`）

---

## コードレビューチェックリスト

- [x] セキュリティ: `[profile.dev]` 設定はビルドのみに影響し、実行時の動作・セキュリティに変化なし
- [x] 副作用: `cargo test` の動作に影響しないこと（デバッグシンボル不要）
- [x] CI: GitHub Actions の CI ワークフローに影響しないこと
- [x] 重複: `Cargo.toml` に既存の `[profile.dev]` セクションがないこと

---

## 完了条件チェックリスト（spec.md 対応）

- [x] `Cargo.toml` version = "30.1.0"
- [x] `Cargo.toml` に `[profile.dev]` セクションが存在する
- [x] `[profile.dev]` に `debug = 0` が設定されている
- [x] `[profile.dev]` に `split-debuginfo = "off"` が設定されている
- [x] `cargo build` がエラーなく完了する
- [x] `cargo test` — 2378 tests PASS
- [x] `CHANGELOG.md` に `[v30.1.0]` セクションあり
- [x] `benchmarks/v30.1.0.json` 存在（test_count: 2378）
- [x] `cargo test --bin fav v301000` — 6/6 PASS
- [x] `versions/current.md` を `v30.1.0` に更新
- [x] tasks.md を COMPLETE に更新

---

## コードレビュー指摘・対応記録

code-reviewer 指摘（実装後）:
- [MED] `split-debuginfo = "off"` が Windows MSVC では no-op → `Cargo.toml` にコメント追記で意図を明文化
- [LOW] `versions/current.md` が「実装中」のまま → 「最新安定版 v30.1.0」に更新

spec-reviewer 指摘（実装前）:
- [HIGH] T0 確認手順の欠如 → plan.md に確認コマンドブロックを追加
- [HIGH] ロードマップの `.cargo/config.toml` 表記 → `roadmap-v30.1-v35.0.md` 241行修正
- [MED] `split-debuginfo_off` テストが `"off"` 未検証 → `contains("split-debuginfo = \"off\"")` に修正
- [MED] tasks.md に完了条件チェックリスト漏れ → セクション追加
- [MED] `split-debuginfo` 効果説明の混在 → spec.md 表を修正
- [LOW] `Cargo.lock` の扱い未明記 → spec.md 対象コンポーネント表に追記
- [LOW] `current.md` 更新タスク漏れ → T6b として追加
