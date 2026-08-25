# Spec: v81.0.0 — Test-Driven Data 1.0 宣言 ★クリーンアップ

## Background

v80.1.0〜v80.9.0 で Test-Driven Data 1.0 フレームワーク（TestSuite / DataFactory /
PropertyTest / StageTestCase / TestCoverageReport / SchemaSnapshot / TestReport）を完成させた。
本バージョンでは宣言クリーンアップを実施し、**Test-Driven Data 1.0 完成** を正式に宣言する。

ロードマップ: `versions/roadmap/roadmap-v80.1-v81.0.md`（v81.0.0 セクション）

> **テスト数補足**: ロードマップは 3827 + 4 = 3831 と記載しているが、
> v80.2.0〜v80.9.0 の code-reviewer 対応で累積 10 件追加されたため実際のベースは **3837**。
> （内訳: v80.9.0 完了時 3837 tests）
> 本バージョンの完了条件は **3837 + 4 = 3841**。

> **CHANGELOG テスト注意**: `v81000_tests` に `changelog_has_v81_0_0` テストが含まれるため、
> T4（CHANGELOG 更新）を T3（`cargo test`）より **前** に実施すること。

## Goals

- `Cargo.toml` バージョンを `81.0.0` に更新する
- `CHANGELOG.md` / `MILESTONE.md` / `README.md` / `versions/current.md` を更新する
- `roadmap-v80.1-v85.0.md` の Sprint 1 バージョン一覧テーブルを全行「完了」に更新する
- `cargo clean` を実施してビルドキャッシュをリセットする
- テスト 4 件を追加して **3841 tests** を達成する

## 宣言文

> 「テストが型になり、カバレッジが数値になり、スキーマ変更が検出される。
>  Favnir のパイプラインは今、その正しさを `fav test` で証明できる。」

## API / Type Definitions

新規型・関数なし。

## Success Criteria

- `Cargo.toml` の `version` が `"81.0.0"` である
- `CHANGELOG.md` に `v81.0.0` エントリが存在する
- `MILESTONE.md` に `Test-Driven Data` が記載されている
- `README.md` に `fav test` が記載されている
- `cargo test` が **3841 tests**, 0 failures
- `cargo_toml_version_is_81_0_0`: `Cargo.toml` の version 文字列が `"81.0.0"` を含む
- `changelog_has_v81_0_0`: `CHANGELOG.md` の先頭付近に `"v81.0.0"` が含まれる
- `milestone_has_test_driven_data`: `MILESTONE.md` に `"Test-Driven Data"` が含まれる
- `readme_mentions_fav_test`: `README.md` に `"fav test"` が含まれる

## Files to Modify

| ファイル | 操作 | 内容 |
|---|---|---|
| `fav/Cargo.toml` | 更新 | version `"80.0.0"` → `"81.0.0"`（v80.x マイナーは更新しない慣例、宣言バージョンで一括更新） |
| `CHANGELOG.md` | 先頭追記 | v81.0.0 エントリ |
| `MILESTONE.md` | 更新 | Test-Driven Data 1.0 達成宣言追記 |
| `README.md` | 更新 | `fav test` コマンド言及を追加 |
| `versions/current.md` | 更新 | 現在バージョンを v81.0.0 に更新 |
| `versions/roadmap/roadmap-v80.1-v85.0.md` | 更新 | Sprint 1 テーブルを全行「完了」に |
| `fav/src/driver.rs` | 追記 | `mod v81000_tests`（テスト 4 件） |

## Error Codes

新規エラーコードなし。

## 注記

- `cargo clean` は `v81000_tests` の `cargo test` 実行前に行う。
- `site/content/docs/` の更新は任意（スプリント完了後に別途実施）。
- `cargo_toml_version_is_81_0_0` テストは `Cargo.toml` の実バージョンを `fs::read_to_string` で読み取る。
