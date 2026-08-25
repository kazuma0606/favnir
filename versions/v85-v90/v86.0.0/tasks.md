# Tasks: v86.0.0 — SAP Foundation 1.0 宣言 ★クリーンアップ

Status: COMPLETE

## T0: 着手前チェックリスト

- [x] `cargo test` を実行し、3,949 tests, 0 failures を確認する
- [x] `fav/src/driver.rs` に `mod v85900_tests` が存在することを確認する（v85.9.0 完了済みの証拠）

## T1: CHANGELOG.md に v86.0.0 エントリを追加

- [x] `CHANGELOG.md` の先頭に v86.0.0 エントリを追加する（テストモジュール追加より先に実施）

## T2: MILESTONE.md に SAP Foundation 1.0 エントリを追加

- [x] `MILESTONE.md` の先頭（v85.0.0 エントリの前）に v86.0.0 エントリを追加する
  - 宣言文を含む（「SAP に、型安全に接続できるようになった。...」）
  - SAP Foundation 1.0 達成内容（Rust 基盤 / Favnir 型 / Rune / インフラ / テンプレート）

## T3: README.md を更新

- [x] `README.md` に v86.0 SAP Foundation 1.0 セクションを追加する（SAP Integration 言及）

## T4: `versions/current.md` を更新

- [x] 「最終更新」を `2026-08-23 (v86.0.0)` に変更
- [x] 「最新安定版」を `v86.0.0 — SAP Foundation 1.0 宣言 — 3953 tests` に変更
- [x] 「進行中バージョン」を `v86.1.0〜v90.0.0` に変更
- [x] 「次に切る版」を `v86.1.0` に変更
- [x] マイルストーン進捗に `v86.0 — SAP Foundation 1.0 | **完了**` を追加

## T5: `fav/Cargo.toml` バージョンを更新

- [x] `version = "85.0.0"` → `version = "86.0.0"` に変更する

## T6: `fav/src/driver.rs` アサーションを一括更新

- [x] `version = \"85.0.0\"` → `version = \"86.0.0\"` を `replace_all: true` で一括置換する（35 件）

## T7: `mod v86000_tests` を追加

- [x] `mod v85900_tests { ... }` の直後に `#[cfg(test)] mod v86000_tests { ... }` を追加する
- [x] `cargo_toml_version_is_86_0_0` テストを実装する
- [x] `changelog_has_v86_0_0` テストを実装する
- [x] `milestone_has_sap_foundation` テストを実装する
- [x] `readme_mentions_sap_integration` テストを実装する

## T8: `cargo test` で全 pass 確認

- [x] `cargo test 2>&1 | grep "test result"` を実行し、3,953 tests, 0 failures であることを確認する

## T9: `cargo clean` 実施

- [x] `cargo clean` を実行する
- [x] `fav/tmp/hello.fav` が残っていることを確認する（target/ のみ削除）

## T-last: CI 事前確認（`cargo clean` 後はリビルドが必要）

- [x] `cargo build` を実行してリビルドする（`cargo clean` 後に実行）
- [x] `cargo clippy --locked -- -D warnings` が pass することを確認する（CI と同じフラグ）
- [x] `./target/debug/fav fmt --check self/compiler.fav` が pass することを確認する
- [x] `./target/debug/fav fmt --check self/checker.fav` が pass することを確認する

## 修正事項（code-reviewer 指摘対応）

- [MED] `versions/current.md` の「次に切る版」セクションが 2 重複（古い「未定」ブロックが残存）→ 削除
- [MED] `cargo_toml_version_is_85_0_0` の assert が `86.0.0` を検索している件 → プロジェクト慣習上の既知パターン（全バージョン共通）のため変更なし
- [LOW] 旧バージョン参照の assert メッセージ → 本バージョンで導入した問題でないため対応外
