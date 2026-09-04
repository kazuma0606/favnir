# Spec: v93.9.0 — 安定化・コードフリーズ

## Background

v93.1.0〜v93.8.0 で実装した SAP Metadata Infer 機能を通しで確認する最終安定化スプリント。
v94.0.0 宣言に向けたコードフリーズを行い、バグ修正のみを受け入れる。

## Goals

1. v93.1〜v93.8 の全機能が 4,136 tests, 0 failures で pass することを確認する。
2. `parse_edmx` → `entity_type_to_favnir` → `apply_fmt_to_generated`（`fmt_source_raw` バックエンド）の全パスが存在することをスモークテストで確認する。
3. `driver.rs` に `mod v93900_tests`（2 件）を追加し、4,138 tests を達成する。
4. バグ修正のみ受け入れ（新機能追加なし）。

## Files to Modify

| ファイル | 変更内容 |
|---|---|
| `fav/src/driver.rs` | `mod v93900_tests` を追加（2 テスト） |
| `CHANGELOG.md` | v93.9.0 エントリを追加 |
| `versions/roadmap/roadmap-v93.1-v94.0.md` | v93.9.0 本文のテスト数確認（既に `4136 + 2 = 4138` に更新済み） |

## Success Criteria

- `cargo test 2>&1 | grep "test result"` → `4138 tests, 0 failures`
- `cargo clippy --locked -- -D warnings` → pass
- `./target/debug/fav fmt --check self/compiler.fav` → pass
- `./target/debug/fav fmt --check self/checker.fav` → pass
- `sap_metadata_smoke_url_and_file_cli`: `cli.fav` に `from sap` と `metadata-file` の両方が含まれる
- `sap_metadata_parser_handles_entity_and_enum`: `sap_metadata.rs` に `entity_type_to_favnir` と `enum_type_to_favnir` の両方が含まれる

## Notes

- 安定化スプリントのため新規 Rust 関数・Favnir 関数の追加は行わない。
- テストは既存コードのシンボル存在確認（contains チェック）のみ。
- v94.0.0 クリーンアップ（`cargo clean`、バージョン番号更新等）は次バージョンで実施する。
