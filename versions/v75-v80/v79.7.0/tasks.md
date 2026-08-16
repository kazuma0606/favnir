# v79.7.0 タスクリスト — OSS 公開強化・コミュニティ整備

Date: 2026-08-16
Status: COMPLETE

---

## T0: 事前確認

- [x] `fav/Cargo.toml` のバージョンが `79.6.0` であることを確認
- [x] `cargo test` が全 pass（3799 tests = v79.6.0 完了後の実測ベース）であることを確認
- [x] `fav/tmp/hello.fav` が存在することを確認
- [x] `CONTRIBUTING.md` が存在し、`Execution Effects` がまだ含まれていないことを確認（重複追加防止）
- [x] `COMMUNITY.md` がまだ存在しないことを確認（重複追加防止）

---

## T1: CONTRIBUTING.md 更新

- [x] `CONTRIBUTING.md` の末尾に `## Execution Effects の追加手順（v3 対応）` セクションを追加する
  - `Execution Effects` という文字列を含む
  - `fav verify` という文字列を含む（使い方セクション）
  - `## PipelineInvariant（invariant）の追加手順` セクションを追加する（`invariant` 文字列を含む）
- [x] `.github/CODEOWNERS` 更新・Rune validate ガイドはスコープ外（本バージョンでは対象としない）

---

## T2: COMMUNITY.md 新規作成

- [x] `COMMUNITY.md` を新規作成する
  - `RFC` という文字列を含む（RFC プロセスセクション）
  - `GitHub` という文字列を含む（ディスカッション場所）
  - `CODE_OF_CONDUCT.md` への参照を含む

---

## T3: CHANGELOG.md 更新（テスト追加より先）

- [x] `CHANGELOG.md` の先頭に v79.7.0 エントリを追加する（形式: `## [v79.7.0] — 2026-08-16 — OSS 公開強化・コミュニティ整備`）
- [x] Added セクション（CONTRIBUTING.md 更新・COMMUNITY.md 新規作成）を含める
- [x] Tests セクション（2 件）を含める

---

## T4: driver.rs — テストモジュール追加

- [x] `fav/src/driver.rs` の末尾に `// --- v79.7.0: OSS 公開強化・コミュニティ整備 ---` コメントを追加する
- [x] `v797000_tests` モジュールを追加する（`use super::*` 不要）
- [x] モジュール先頭に `const CONTRIBUTING` / `const COMMUNITY` を配置する（`include_str!` パス: `../../CONTRIBUTING.md` / `../../COMMUNITY.md`）
- [x] `oss_contributing_v2_exists` テストを実装する
  - `Execution Effects` / `fav verify` / `invariant` を assert
- [x] `oss_community_md_exists` テストを実装する
  - `RFC` / `GitHub` を assert
- [x] `cargo test v797000` で 2 件が pass することを確認する

---

## T5: Cargo.toml バージョン更新

- [x] `fav/Cargo.toml` の `version` を `"79.6.0"` → `"79.7.0"` に変更する
- [x] driver.rs 内の escaped `\"79.6.0\"` を `\"79.7.0\"` に一括更新（sed）
- [x] driver.rs 内の unescaped エラーメッセージ `79.6.0` を `79.7.0` に更新する
- [x] **更新後に** `grep -c "79\.6\.0" /c/Users/yoshi/favnir/fav/src/driver.rs` を実行し **出力が 1** であることを確認する
  - 残るのは `// --- v79.6.0: ドッグフーディング強化 ---` コメント行の 1 件のみ

---

## T6: versions/current.md 更新

- [x] `## 進行中バージョン` 欄を `**v79.7.0**（OSS 公開強化・コミュニティ整備）` に更新する
- [x] `## 次に切る版` 欄を `**v79.8.0**（ドキュメント完全化 v3 リファレンス）` に更新する

---

## T7: 最終確認

- [x] `cargo test` が全 pass であることを確認する（3801 tests）
- [x] `cargo test v797000` で 2 件が pass することを確認する
- [x] `fav/Cargo.toml` のバージョンが `79.7.0` であることを確認する
- [x] `fav/Cargo.lock` が cargo test 実行時に自動更新されていることを確認する
- [x] `CHANGELOG.md` の先頭が `[v79.7.0]` であることを確認する
- [x] `CONTRIBUTING.md` に `Execution Effects` / `fav verify` / `invariant` が含まれることを確認する
- [x] `COMMUNITY.md` に `RFC` / `GitHub` が含まれることを確認する

---

## 完了チェックリスト

- [x] 全タスク（T0〜T7）が完了している
- [x] `oss_contributing_v2_exists` が pass
- [x] `oss_community_md_exists` が pass
- [x] テスト総数: 3801（+2）
- [x] `CHANGELOG.md` の先頭が `[v79.7.0]` であることを確認済み
- [x] `fav/Cargo.toml` version = "79.7.0" に更新済み
- [x] `versions/current.md` が v79.7.0 に更新済み
- [x] `changelog_has_v79_7_0` テストの追加: 対象外（x.0.0 宣言バージョンのみに追加する慣例）
