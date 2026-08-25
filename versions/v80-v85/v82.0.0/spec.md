# v82.0.0 — Data Quality 2.0 宣言 ★クリーンアップ

Date: 2026-08-20
Status: 計画中

---

## Background

v81.1〜v81.9 で実装した **Data Quality 2.0 スプリント**の全機能が完成した。
本バージョンは宣言バージョンとして以下のクリーンアップ作業を行い、
`v82000_tests` 4 件で完成を検証する。

**宣言文**:
> 「品質が型になった。外れ値はコンパイル時に検出され、
>  スキーマドリフトはパイプライン起動前に止まる。
>  Favnir のデータは今、品質を型で保証する。」

---

## Goals

1. `cargo clean` でビルドキャッシュをクリア
2. `Cargo.toml` バージョンを `82.0.0` に更新
3. `CHANGELOG.md` に v82.0.0 エントリを追加
4. `MILESTONE.md` に Data Quality 2.0 宣言エントリを追加
5. `README.md` の最新バージョン記述を v82.0 に更新（`QualityGate` に言及）
6. `versions/current.md` の状態を v82.0.0 に更新
7. `roadmap-v80.1-v85.0.md` の Sprint 2 テーブルを全行「完了」に更新
8. `roadmap-v81.1-v82.0.md` の全バージョン行を「完了」に更新
9. `v82000_tests` モジュール 4 件追加

---

## Success Criteria: `v82000_tests` 4 件

### `cargo_toml_version_is_82_0_0`
`fav/Cargo.toml` の `version` フィールドが `"82.0.0"` であること。

### `changelog_has_v82_0_0`
`CHANGELOG.md` に `"v82.0.0"` という文字列が含まれること。

### `milestone_has_data_quality_2`
`MILESTONE.md` に `"Data Quality 2.0"` という文字列が含まれること。

### `readme_mentions_quality_gate`
`README.md` に `"QualityGate"` という文字列が含まれること。

### テスト総数
`cargo test` 全テスト通過（3,865 tests pass、0 failures）。
- ベース（v81.9.0 完了時点）: 3,861
- 本バージョン追加: +4
- 合計: 3,865

---

## Files to Modify

| ファイル | 変更内容 |
|---|---|
| `fav/Cargo.toml` | `version = "82.0.0"` に更新 |
| `CHANGELOG.md` | v82.0.0 エントリを先頭に追加 |
| `MILESTONE.md` | Data Quality 2.0 宣言エントリを先頭に追加 |
| `README.md` | v82.0 セクションを先頭に追加（`QualityGate` 言及） |
| `versions/current.md` | 最新バージョン・進行中スプリントを更新 |
| `versions/roadmap/roadmap-v80.1-v85.0.md` | Sprint 2 テーブルを全行「完了」に更新 |
| `versions/roadmap/roadmap-v81.1-v82.0.md` | 全バージョン行を「完了」に更新 |
| `fav/src/driver.rs` | `#[cfg(test)] mod v82000_tests` を追加（4 件） |
