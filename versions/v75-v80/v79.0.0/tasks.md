# v79.0.0 タスクリスト — Execution Effects 1.0 宣言 ★クリーンアップ

Date: 2026-08-16
Status: COMPLETE

---

## T0: 事前確認

- [x] `fav/Cargo.toml` のバージョンが `78.9.0` であることを確認
- [x] `cargo test` が全 pass（3783 tests）であることを確認
- [x] `fav/tmp/hello.fav` が存在することを確認

---

## T1: cargo clean

- [x] `cd /c/Users/yoshi/favnir/fav && cargo clean` を実行する
- [x] `fav/tmp/hello.fav` が依然として存在することを確認する（target/ 外なので影響なし）

---

## T2: CHANGELOG.md 更新（テスト追加より先）

- [x] `CHANGELOG.md` の先頭に v79.0.0 エントリを追加する（形式: `## [v79.0.0] — 2026-08-16 — Execution Effects 1.0 宣言 ★クリーンアップ`）
- [x] Added セクション:「Execution Effects 1.0 宣言（v78.1〜v78.9 の全 Execution Effects 基盤の完成を宣言）」を追加する
- [x] Tests セクション（4 件）を含める

---

## T3: MILESTONE.md 更新

- [x] `## v78.0.0` 節の直前に `## v79.0.0（2026-08-16）— Execution Effects 1.0 宣言` 節を追加する
- [x] 宣言文・達成内容（v78.1〜v78.9 の 9 項目）を含める

---

## T4: README.md 更新

- [x] `## v78.0 — Verifiable Pipelines 宣言` の直前に `## v79.0 — Execution Effects 1.0 宣言（2026-08-16）` 節を追加する

---

## T5: driver.rs — テストモジュール追加

- [x] `fav/src/driver.rs` の末尾に `// --- v79.0.0: Execution Effects 1.0 宣言 ★クリーンアップ ---` コメントを追加する
- [x] `v79000_tests` モジュールを追加する（`use super::*` は不要 — 宣言バージョンにつき外部シンボル未使用）
- [x] `cargo_toml_version_is_79_0_0` テストを実装する（`include_str!("../Cargo.toml")` で `version = "79.0.0"` を assert）
- [x] `changelog_has_v79_0_0` テストを実装する（`include_str!("../../CHANGELOG.md")` で `[v79.0.0]` を assert）
- [x] `milestone_has_execution_effects` テストを実装する（`include_str!("../../MILESTONE.md")` で `Execution Effects 1.0` を assert）
- [x] `readme_mentions_execution_effects` テストを実装する（`include_str!("../../README.md")` で `Execution Effects` を assert）
- [x] `cargo test v79000` で 4 件が pass することを確認する

---

## T6: Cargo.toml バージョン更新

- [x] `fav/Cargo.toml` の `version` を `"78.9.0"` → `"79.0.0"` に変更する
- [x] driver.rs 内の `78.9.0` バージョン文字列アサーションを `79.0.0` に一括更新（`replace_all: true`）
- [x] **replace_all 後に** `grep -c "78.9.0" /c/Users/yoshi/favnir/fav/src/driver.rs` を実行し **出力が 1** であることを確認する（Git Bash で実行）
  - 残るのは `// --- v78.9.0: 安定化・コードフリーズ ---` の 1 件のみ

---

## T7: versions/current.md 更新

- [x] `## 進行中バージョン` 欄を `**v79.0.0**（Execution Effects 1.0 宣言）` に更新する
- [x] `## 次に切る版` 欄を `**v79.1.0**（次スプリント開始予定）` に更新する

---

## T8: 最終確認

- [x] `cargo test` が全 pass であることを確認する（3787 tests）
- [x] `cargo test v79000` で 4 件が pass することを確認する
- [x] `fav/Cargo.toml` のバージョンが `79.0.0` であることを確認する
- [x] `CHANGELOG.md` の先頭が `[v79.0.0]` であることを確認する
- [x] `MILESTONE.md` に「Execution Effects 1.0」が含まれることを確認する
- [x] `README.md` に「Execution Effects」が含まれることを確認する
- [x] `versions/current.md` の「進行中バージョン」が v79.0.0 であることを確認する

---

## 完了チェックリスト

- [x] 全タスク（T0〜T8）が完了している
- [x] `cargo_toml_version_is_79_0_0` が pass
- [x] `changelog_has_v79_0_0` が pass
- [x] `milestone_has_execution_effects` が pass
- [x] `readme_mentions_execution_effects` が pass
- [x] テスト総数: 3787（+4）
- [x] `cargo clean` 実施済み
- [x] site/ MDX 追加: 対象外（宣言バージョンのみ `changelog_has_vXX` テスト追加）
