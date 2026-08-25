# v85.0.0 タスクリスト

Status: COMPLETE

---

## T0: 着手前チェックリスト

- [x] `cargo test` を実行し、3,927 tests, 0 failures を確認する（前提: v84.9.0 完了済み）
- [x] `grep -m1 '^version' fav/Cargo.toml` の出力が `version = "84.0.0"` であることを確認する
- [x] `fav/src/driver.rs` に `mod v84900_tests` が存在することを確認する（v84.9.0 完了済みの証拠）

## T1: cargo clean

- [x] `fav/` ディレクトリで `cargo clean` を実行する（12.5 GiB 解放）
- [x] `fav/tmp/hello.fav` が存在することを確認する（存在確認済み）

## T2: Cargo.toml バージョン更新

- [x] `fav/Cargo.toml` の `version = "84.0.0"` を `version = "85.0.0"` に更新する
- [x] `driver.rs` 内の旧 `cargo_toml_version` テスト（33 件）の assert 文字列を `85.0.0` に一括更新する

## T3: CHANGELOG 更新

- [x] `CHANGELOG.md` の先頭に v85.0.0 エントリを追加する

## T4: MILESTONE.md 更新

- [x] `MILESTONE.md` に v85.0.0 / Favnir 4.0 宣言を追加する（`"Favnir 4.0"` 文字列必須）

## T5: README.md 更新

- [x] `README.md` の Latest 欄を v85.0.0 / Favnir 4.0 宣言に更新する（`"Favnir 4.0"` 文字列必須）

## T6: versions/current.md 更新

- [x] `versions/current.md` の現行マスターロードマップが `roadmap-v80.1-v85.0.md` を指していることを確認する
- [x] `versions/current.md` の現行バージョンを `v85.0.0` に更新する

## T7: ロードマップ更新

- [x] `versions/roadmap/roadmap-v84.1-v85.0.md` の Sprint 5 バージョン一覧テーブルを全行「完了」に更新する
- [x] `roadmap-v84.1-v85.0.md` 完了条件のテスト数を `3927 + 4 = 3931` に修正する
- [x] `roadmap-v84.1-v85.0.md` テスト数推移テーブルの v85.0.0 行を `3,931` に修正する
- [x] `versions/roadmap/roadmap-v80.1-v85.0.md` の Sprint 5 テーブル v85.0.0 行を「完了」に更新する
- [x] `roadmap-v80.1-v85.0.md` の v85.0.0 完了条件テスト数を `3915 + 4 = 3919` → `3927 + 4 = 3931` に修正する

## T8: `fav/src/driver.rs` に `v85000_tests` を追加

- [x] `mod v84900_tests { ... }` の直後に `#[cfg(test)] mod v85000_tests { ... }` を追加する
  - `Cargo.toml` パス: `../Cargo.toml`（`fav/src/` → `fav/`）
  - ルートファイルパス: `../../CHANGELOG.md`, `../../MILESTONE.md`, `../../README.md`
- [x] `cargo_toml_version_is_85_0_0` テストを実装する（`include_str!("../Cargo.toml")` を使用）
- [x] `changelog_has_v85_0_0` テストを実装する
- [x] `milestone_has_favnir_4` テストを実装する
- [x] `readme_mentions_favnir_4` テストを実装する

## T9: テスト通過確認

- [x] `cargo test 2>&1 | grep "test result"` を実行し、3,931 tests, 0 failures（+4）であることを確認する

## T-last: CI 事前確認

- [x] `cargo clippy --locked -- -D warnings` が pass することを確認する（CI と同じフラグ）
- [x] `./target/debug/fav fmt --check self/compiler.fav` が pass することを確認する
- [x] `./target/debug/fav fmt --check self/checker.fav` が pass することを確認する
