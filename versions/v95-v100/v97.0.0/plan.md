# Plan: v97.0.0 — SAP Multi-system 1.0 宣言

## 実装ステップ

1. **`fav/Cargo.toml` バージョン更新**
   - `version = "96.0.0"` → `version = "97.0.0"`

2. **`CHANGELOG.md` 更新**
   - 先頭に `[v97.0.0]` エントリ追加
   - SAP Multi-system 1.0 宣言の内容を記載

3. **`MILESTONE.md` 更新**
   - v97.0 — SAP Multi-system 1.0 エントリを追加

4. **`README.md` 更新**
   - `## v97.0 — SAP Multi-system 1.0` セクションを追加

5. **`fav/src/driver.rs` に `mod v97000_tests` 追加**
   - `mod v96900_tests` の直後に追加
   - 4 テスト:
     - `cargo_toml_version_is_97_0_0`: `fav/Cargo.toml` に "97.0.0" が含まれる
     - `changelog_has_v97_0_0`: `CHANGELOG.md` に "[v97.0.0]" が含まれる
     - `milestone_has_sap_multi_system`: `MILESTONE.md` に "SAP Multi-system" が含まれる
     - `readme_mentions_sap_multi_system`: `README.md` に "SAP Multi-system" が含まれる

6. **`cargo test` で 4,213 tests, 0 failures を確認**

7. **`cargo clean` を実施（★クリーンアップ）**
   - `cd fav && cargo clean`

8. **`cargo clean` 後に再テスト**
   - `cargo test` で 4,213 tests, 0 failures を再確認

9. **`versions/current.md` 更新**
   - 最新安定版を v97.0.0 に更新（テスト数 4,213）
   - マイルストーン一覧に v97.0 追加

## 注意事項

- CHANGELOG の更新（ステップ 2）は `changelog_has_v97_0_0` テスト（ステップ 5）より先に行う
- `cargo_toml_version_is_97_0_0` テストが通るには Cargo.toml 更新（ステップ 1）が先
- `cargo clean` は target/ ディレクトリを削除するが `fav/tmp/hello.fav` は残る
- `cargo clean` 後は `./target/debug/fav` バイナリが消えるので fmt check は clean 前に実施
- driver.rs テストの `include_str!` / `read_to_string` パス:
  - `fav/Cargo.toml` → `include_str!("../Cargo.toml")`
  - `CHANGELOG.md` → `include_str!("../../CHANGELOG.md")`
  - `MILESTONE.md` → `include_str!("../../MILESTONE.md")`
  - `README.md` → `include_str!("../../README.md")`
