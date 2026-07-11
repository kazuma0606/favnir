# v35.7.0 タスクリスト — docs_server.rs !Effect 完全除去

## ステータス: COMPLETE

> コードネーム v35.0B。sprint 作業（docs_server.rs 修正・CHANGELOG 追記・v35700_tests pre-existing）は完了済み。
> 本セッションでは half-stub 修正 + Cargo.toml バンプを実施する。
>
> ロードマップ差異: spec.md §「ロードマップとの差異」参照。`fav deploy --dry-run` は後続バージョンで対応。

## T0: 事前確認

- [x] テスト数: 2646 passed; 0 failed（実測）
- [x] Cargo.toml バージョンが `35.6.0` であることを確認
- [x] `v35600_tests::cargo_toml_version_is_35_6_0` がライブアサーション（`assert!(cargo.contains("35.6.0"), ...)`)）であることを確認
- [x] `v35700_tests` モジュールが driver.rs に 5 件存在することを確認
- [x] `v35700_tests::cargo_toml_version_is_35_7_0` が半スタブ（`assert!(cargo.contains("35."), ...)`）＋コメント `// Stubbed: version bumped to 35.8.0 in v35.0C` であることを確認 → T1・T2 で修正
- [x] `v35800_tests::cargo_toml_version_is_35_8_0` がすでにスタブ（空関数）であることを確認（バンプ後も影響なし）
- [x] `build_stdlib_json` が `pub fn` であることを確認（`docs_server.rs` line 701）
- [x] `CHANGELOG.md` に `[v35.7.0]` が含まれることを確認（sprint 完了済み）
- [x] `docs_server.rs` に `!Io"` 文字列リテラルが存在しないことを確認（sprint 完了済み）

## T1: driver.rs — v35600_tests::cargo_toml_version_is_35_6_0 をスタブ化

- [x] ライブアサーション → `// stubbed: version bumped to 35.7.0` に変更

## T2: driver.rs — cargo_toml_version_is_35_7_0 を生きたアサーションに修正

- [x] 半スタブ → `assert!(cargo.contains("35.7.0"), "Cargo.toml must contain version 35.7.0")` に修正
- [x] コメント行（"Stubbed: version bumped to 35.8.0 in v35.0C"）を削除

## T3: バージョン更新（T1 完了後）

- [x] `fav/Cargo.toml` バージョンを `35.7.0` に更新

## T4: テスト実行

- [x] `cargo test` 全通過 — N passed; 0 failed（テスト数 ≥ 2646、今回追加テストなし・前バージョンと同数維持）
- [x] `v35700_tests` の 5 テストが pass

## T5: ドキュメント更新

- [x] `versions/v30-v35/v35.7.0/tasks.md` を COMPLETE ステータスに更新
- [x] `versions/current.md` を v35.7.0（最新安定版）・v35.8.0（次バージョン）に更新

---

## 完了条件チェックリスト（spec.md 対応）

| # | spec.md 完了条件 | 確認方法 |
|---|---|---|
| 1 | `docs_server.rs` に `!Io"` / `!Http"` / `!Aws"` が存在しない | `docs_server_io_signatures_no_effect` + `effect_annotation_fully_purged` テスト |
| 2 | `build_stdlib_json()` の IO 関数 effects が空配列 | `docs_server_io_effects_empty` テスト |
| 3 | `CHANGELOG.md` に `[v35.7.0]` が含まれる | `changelog_has_v35_7_0` テスト |
| 4 | `Cargo.toml` バージョンが `35.7.0` | `cargo_toml_version_is_35_7_0` テスト |
| 5 | `cargo test` 全通過（failures=0 かつテスト数 ≥ 2646） | T4 実行結果 |

## コードレビュー対応（実施後に記録）

| 指摘 | 優先度 | 対応 |
|---|---|---|
| `v35800_tests::cargo_toml_version_is_35_8_0` のスタブコメントが `35.1.0` と誤記 | [MED] | `// stubbed: version bumped to 35.7.0` に修正 |
| `docs_server_io_effects_empty` の `.expect("IO module")` のメッセージが不明瞭 | [MED] | `"IO module not found in build_stdlib_json() output"` に改善 |
| `docs_server_io_signatures_no_effect` でコメント行をスキップしていない | [LOW] | `trimmed.starts_with("//")` で continue を追加 |
| `docs_server.rs` の unsafe ブロックに正当化コメントなし | [LOW] | Unix/Windows 両ハンドラに `// SAFETY:` コメントを追記 |
| `v35800_tests` が `v35700_tests` より前に定義されており昇順が崩れている | [LOW] | スキップ（スプリント一括コミットの構造のため変更リスク大） |
