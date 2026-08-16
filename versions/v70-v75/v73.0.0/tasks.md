# v73.0.0 タスクリスト — Developer Experience 2.0 宣言 ★クリーンアップ

Date: 2026-08-13
Status: 完了

---

## T0: 事前確認

- [x] `fav/Cargo.toml` のバージョンが `72.9.0` であることを確認
- [x] `cargo test` が 3642 tests pass（0 failures）であることを確認
- [x] `driver.rs` に `v729000_tests` モジュールが存在することを確認
- [x] `driver.rs` に `v73000_tests` が未存在であることを確認
- [x] `driver.rs` 内の `"72.9.0"` 文字列（バージョンアサーション）の件数を grep で確認しておく

---

## T1: `cargo clean`

- [x] `cargo clean` を実施した
- [x] `cargo build -j 8` でクリーンビルドが成功することを確認した

---

## T2: `MILESTONE.md` 更新

- [x] `MILESTONE.md` に `## v73.0.0 — Developer Experience 2.0（2026-08-13）` セクションを追記した
  - VS Code 拡張・AI アシスタント・`fav ai generate`・REPL 2.0・Playground 2.0・`fav init` テンプレート・`fav watch` 2.0・`fav learn` を列挙

---

## T3: `README.md` 更新

- [x] `README.md` のマイルストーン一覧に `v73.0 — Developer Experience 2.0` を追記した

---

## T3.5: `CHANGELOG.md` 更新（T6 部分テストより前に実施すること）

> **注意**: `changelog_has_v73_0_0` テストが T6 で pass するために、T4 実装より前に CHANGELOG を更新する必要がある。

- [x] `## [v73.0.0]` エントリを `CHANGELOG.md` 先頭に追加した
  - Milestone: Developer Experience 2.0 宣言
  - Changed: cargo clean・バージョン更新
  - Docs: MILESTONE.md・README.md 更新
  - Tests: 4 件、合計テスト数 3646（+4）

---

## T4: `v73000_tests` モジュール追加

- [x] `v729000_tests` モジュールの直後に `v73000_tests` モジュールを追加した
- [x] `use super::*` は不要（このテストモジュールは外部シンボルを使用しない）
- [x] `cargo_toml_version_is_73_0_0` テストを実装した
  - `include_str!("../Cargo.toml")` が `version = "73.0.0"` を含むことを assert
- [x] `changelog_has_v73_0_0` テストを実装した
  - `include_str!("../../CHANGELOG.md")` が `[v73.0.0]` を含むことを assert
- [x] `milestone_has_dev_exp2` テストを実装した
  - `include_str!("../../MILESTONE.md")` が `Developer Experience 2.0` を含むことを assert
- [x] `readme_mentions_dev_exp2` テストを実装した
  - `include_str!("../../README.md")` が `v73.0` または `Developer Experience 2.0` を含むことを assert
- [x] `cargo build` でエラーがないことを確認

---

## T5: バージョン更新

- [x] `fav/Cargo.toml` の `version = "72.9.0"` → `version = "73.0.0"` に変更した
- [x] `driver.rs` 内の `version = \"72.9.0\"` 文字列を `version = \"73.0.0\"` に replace_all した
- [x] `driver.rs` 内のエラーメッセージ `"Cargo.toml version should be 72.9.0"` を `"73.0.0"` に replace_all した
- [x] `driver.rs` 内のエラーメッセージ `"Cargo.toml should declare version 72.9.0"` を `"73.0.0"` に replace_all した
- [x] 残存 72.9.0 はコメント・セクションヘッダーのみで意図的保持を確認
- [x] `grep "72.9.0" driver.rs` で意図的保持分以外がゼロ件であることを確認

---

## T6: 部分テスト確認

- [x] `cargo test v73000` で 4 件 pass することを確認

---

## T7: 全体テスト確認

- [x] `cargo test` 全体で 3646 tests pass（0 failures）であることを確認

---

## T8: `versions/current.md` 更新

- [x] 「最終更新」を `2026-08-13 (v73.0.0)` に更新した
- [x] 「最終安定版」を `v73.0.0`（Developer Experience 2.0 宣言）に更新した
- [x] 「進行中バージョン」を `v73.1.0` に更新した
- [x] 「次に切る版」を `v73.2.0` に更新した
- [x] マイルストーン進捗表の `v73.0 — Developer Exp 2.0` 行を「完了」に更新した
- [x] `roadmap-v72.1-v73.0.md` の v73.0.0 バージョン一覧テーブルを「完了（実測 3646）」に更新した

---

## T9: 最終確認（T8 完了後）

- [x] `cargo test v73000` で 4 件 pass することを確認
- [x] `cargo test` 全体で 3646 tests pass（0 failures）であることを確認
- [x] `fav/Cargo.toml` のバージョンが `73.0.0` であることを確認
- [x] `MILESTONE.md` に `Developer Experience 2.0` が存在することを確認
- [x] `README.md` に `v73.0` が存在することを確認
- [x] `CHANGELOG.md` に `[v73.0.0]` エントリが存在することを確認
- [x] `versions/current.md` の「進行中バージョン」が `v73.1.0` であることを確認
- [x] `roadmap-v72.1-v73.0.md` の v73.0.0 行が「完了（実測 3646）」であることを確認

---

## コードレビュー指摘対応

| 優先度 | 指摘 | 対応 |
|---|---|---|
| [MED] | `main.rs` で `cmd_learn` が `crate::driver::cmd_learn()` 完全パスで呼ばれており、use インポートが未使用（Clippy 警告） | `cmd_learn()` 短縮名に統一（他の `cmd_*` と一貫性あり） |
| [LOW] | MILESTONE.md の "Developer Experience 2.0" 名が v61.0.0 と重複 | 機能上の問題なし・テスト PASS のため修正なし（許容範囲） |

---

## スコープ外（明示的除外）

- 新機能の追加（宣言バージョンのため）
- v73.1.0 以降の機能（Production Proven フェーズ）
