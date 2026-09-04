# Tasks: v95.0.0 — SAP Advanced 1.0 宣言 ★クリーンアップ

Status: COMPLETE

## T0: 着手前チェックリスト

- [x] `cargo test` を実行し、4,160 tests, 0 failures を確認する（着手前ベースライン）
- [x] `fav/src/driver.rs` に `mod v94900_tests` が存在することを確認する（v94.9.0 完了済みの証拠）
- [x] `fav/Cargo.toml` の version が `94.0.0` であることを確認する

## T1: `CHANGELOG.md` に v95.0.0 エントリを追加する

- [x] `CHANGELOG.md` の先頭に v95.0.0 エントリを追加する
- [x] 宣言文（SAP Advanced 1.0）を含める
- [x] v95000_tests の 4 テスト名を列挙する
- [x] テスト数 **4,164**（+4）を記載する

## T2: `MILESTONE.md` に v95.0.0 エントリを追加する

- [x] `MILESTONE.md` の先頭（`## v94.0.0` の前）に v95.0.0 エントリを追加する
- [x] 宣言文・達成内容（$batch / SnapStart / ベンチマーク / E2E デモ / ドキュメント）を記載する
- [x] `SAP Advanced` の文字列が含まれることを確認する（テスト要件: `milestone_has_sap_advanced`）

## T3: `README.md` に v95.0「SAP Advanced 1.0」セクションを追加する

- [x] `## v94.0 —` の前に `## v95.0 — SAP Advanced 1.0 宣言` セクションを追加する
- [x] コード例（`cleanup_partners` 関数）を含める
- [x] `SAP Advanced` の文字列が含まれることを確認する（テスト要件: `readme_mentions_sap_advanced`）
- [x] Favnir コード例は `bind` を使う（`let` は禁止）

## T4: `versions/current.md` を v95.0.0 に更新する

- [x] `最終更新` 行を `v95.0.0` に変更する
- [x] 「最新安定版」を `**v95.0.0** — SAP Advanced 1.0 宣言 — 4,164 tests` に更新する
- [x] マイルストーン表の `v94.0 — SAP Metadata Infer 1.0 | 計画中` → `完了` に更新する
- [x] マイルストーン表の `v95.0 — SAP Advanced 1.0 | 計画中` → `完了` に更新する

## T5: SAP Advanced Era ロードマップを「完了」マークする

- [x] `versions/roadmap/roadmap-v90.1-v91.0.md` の Status を完了に更新する
- [x] `versions/roadmap/roadmap-v91.1-v92.0.md` の Status を完了に更新する
- [x] `versions/roadmap/roadmap-v92.1-v93.0.md` の Status を完了に更新する
- [x] `versions/roadmap/roadmap-v93.1-v94.0.md` の Status を完了に更新する
- [x] `versions/roadmap/roadmap-v94.1-v95.0.md` の Status を完了に更新する

## T6: `driver.rs` の `cargo_toml_version_is_94_0_0` テストを stub 化する

- [x] `v94000_tests::cargo_toml_version_is_94_0_0` のアサーション本体を `// stubbed: version has advanced to 95.0.0` に置き換える

## T7: `driver.rs` に `mod v95000_tests` を追加する

- [x] `mod v94900_tests { ... }` の直後に `#[cfg(test)] mod v95000_tests { ... }` を追加する（4 テスト）
- [x] `cargo_toml_version_is_95_0_0`: `Cargo.toml` に `"version = \"95.0.0\""` が含まれる
- [x] `changelog_has_v95_0_0`: `../CHANGELOG.md` に `"v95.0.0"` が含まれる
- [x] `milestone_has_sap_advanced`: `../MILESTONE.md` に `"SAP Advanced"` が含まれる
- [x] `readme_mentions_sap_advanced`: `../README.md` に `"SAP Advanced"` が含まれる

## T8: `Cargo.toml` バージョンを `95.0.0` に更新する

- [x] `fav/Cargo.toml` の `version = "94.0.0"` を `version = "95.0.0"` に変更する

## T9: `cargo clean` を実施する

- [x] `cargo clean` を実行し、`target/` ディレクトリを削除する

## T10: `cargo test` で全 pass 確認

- [x] `cargo test 2>&1 | grep "test result"` を実行し、4,164 tests, 0 failures であることを確認する

## T11: tasks.md を COMPLETE に更新する

- [x] 本ファイルの Status を `COMPLETE` に変更する
- [x] T0〜T-last の全チェックボックスを `[x]` にする

## T-last: CI 事前確認（T10 の `cargo test` 全 pass 確認後に実施すること）

- [x] `cargo clippy --locked -- -D warnings` が pass することを確認する（CI と同じフラグ）
- [x] `./target/debug/fav fmt --check self/compiler.fav` が pass することを確認する
- [x] `./target/debug/fav fmt --check self/checker.fav` が pass することを確認する
