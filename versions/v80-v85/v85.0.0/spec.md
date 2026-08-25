# spec: v85.0.0 — Favnir 4.0 宣言 ★クリーンアップ

## Background

v84.1〜v84.9 で積み上げた 4 つの Quality 柱（Test-Driven Data / Data Quality 2.0 / Pipeline Contracts / Observability 2.0）を統合ショーケース・ドキュメント・OSS 公開強化で完成させた。
本バージョンは **Favnir 4.0** の正式宣言バージョンであり、クリーンアップ（cargo clean / バージョン番号更新）を行う。

> **テスト数注記**: ロードマップ計画値は 3919 だが、code-reviewer 指摘対応の累積により実際のベースは v84.9.0 完了時点で 3927。
> 本バージョン目標: 3927 + 4 = **3931**

## 宣言文

> 「テストが型となり、品質が型となり、契約が型となり、観測が型となった。
>
>  `fav test` がパイプラインの正しさを証明し、
>  `QualityGate` が品質基準を守り、
>  `IoContract` がチームを安全に繋ぎ、
>  `AlertRule` が壊れる前に教えてくれる。
>
>  Favnir 4.0 は、データパイプラインの品質を
>  コードと同じ言語で語れる、唯一の言語である。」

## Goals

1. `cargo clean` でビルドキャッシュを削除する（ディスク節約・クリーンビルド確認）
2. `Cargo.toml` の `version` を `85.0.0` に更新する
3. `CHANGELOG.md` に v85.0.0 エントリを追加する
4. `MILESTONE.md` に Favnir 4.0 宣言を反映する
5. `README.md` に Favnir 4.0 の言及を追加する
6. `versions/current.md` を v85.0.0 に更新する
7. `roadmap-v84.1-v85.0.md` の Sprint 5 バージョン一覧テーブルを全行「完了」に更新する
8. `fav/src/driver.rs` に `v85000_tests` を追加し、4 件のテストで全部 pass を確認する

## Success Criteria

- `cargo test` で 3931 tests, 0 failures
- `v85000_tests` 4 件すべて pass:
  - `cargo_toml_version_is_85_0_0`: `fav/Cargo.toml` に `version = "85.0.0"` が含まれること
  - `changelog_has_v85_0_0`: `CHANGELOG.md` に `v85.0.0` が含まれること
  - `milestone_has_favnir_4`: `MILESTONE.md` に `Favnir 4.0` が含まれること
  - `readme_mentions_favnir_4`: `README.md` に `Favnir 4.0` が含まれること

## Files to Modify

| ファイル | 変更内容 |
|---|---|
| `fav/Cargo.toml` | `version = "85.0.0"` に更新 |
| `CHANGELOG.md` | v85.0.0 エントリを先頭に追加 |
| `MILESTONE.md` | Favnir 4.0 宣言を追加 |
| `README.md` | Favnir 4.0 言及を追加 |
| `versions/current.md` | v85.0.0 に更新 |
| `versions/roadmap/roadmap-v84.1-v85.0.md` | Sprint 5 テーブル全行「完了」に更新、テスト数修正 |
| `fav/src/driver.rs` | `v85000_tests` モジュールを追加 |
