# Plan: v96.0.0 — SAP Real-time 1.0 宣言

## Step 1: `fav/Cargo.toml` のバージョンを更新

- `version = "95.0.0"` → `version = "96.0.0"` に変更する

## Step 2: `fav/src/driver.rs` に `mod v96000_tests` を追加

1. `mod v95900_tests` の直後に `#[cfg(test)] mod v96000_tests { ... }` を追加する
2. `cargo_toml_version_is_96_0_0` テスト: `Cargo.toml` に `version = "96.0.0"` が含まれる
   （`std::fs::read_to_string("Cargo.toml")` — テスト実行ディレクトリ `fav/` からのパス）
3. `changelog_has_v96_0_0` テスト: `../CHANGELOG.md` に `v96.0.0` が含まれる
4. `milestone_has_sap_realtime` テスト: `../MILESTONE.md` に `SAP Real-time` が含まれる
5. `readme_mentions_sap_realtime` テスト: `../README.md` に `SAP Real-time` が含まれる

## Step 3: `CHANGELOG.md` に v96.0.0 エントリを追加

- `CHANGELOG.md` の先頭に `[v96.0.0]` エントリを追加する

## Step 4: `MILESTONE.md` に v96.0.0 エントリを追加

- `MILESTONE.md` の先頭に v96.0.0 SAP Real-time 1.0 宣言エントリを追加する

## Step 5: `README.md` に v96.0 セクションを追加

- `## v95.0 — SAP Advanced 1.0` セクションの直前に `## v96.0 — SAP Real-time 1.0` セクションを追加する

## Step 6: `cargo test` で全 pass 確認

- `cargo test 2>&1 | grep "test result"` を実行し、4,188 tests, 0 failures を確認する

## Step 7: `cargo clean` ★クリーンアップ

- `cargo clean` を実行する（target/ ディレクトリを削除）
- `cargo test 2>&1 | grep "test result"` を再実行し、4,188 tests, 0 failures を再確認する

## Step 8: `versions/current.md` 更新

- 最新安定版を `v96.0.0` に更新する
- 「進行中バージョン」欄を更新する

## Step 9: tasks.md 更新

- 本バージョンの `tasks.md` を COMPLETE ステータスに更新する
