# v84.0.0 仕様書 — Observability 2.0 宣言 ★クリーンアップ

## Background

v83.1〜v83.9 で Observability 2.0 スプリントの全実装を完了した。
v84.0.0 は宣言バージョン。`Cargo.toml` バージョンを `84.0.0` に更新し、
ドキュメントを整備し、宣言テストを追加してスプリントの完成を宣言する。

ロードマップ参照: `versions/roadmap/roadmap-v83.1-v84.0.md` — v84.0.0 セクション

## 宣言文

> 「メトリクスが型になり、アラートが型になり、SLO が型になった。
>  Favnir のパイプラインは壊れる前に教えてくれる。」

## Goals

1. `cargo clean` を実施する
2. `Cargo.toml` バージョンを `84.0.0` に更新する
3. `CHANGELOG.md` に v84.0.0 エントリを追加する
4. `MILESTONE.md` に Observability 2.0 達成内容を追記する
5. `README.md` に `fav observe` の言及を追加する
6. `versions/current.md` を v84.0.0 に更新する
7. `roadmap-v80.1-v85.0.md` の Sprint 4 バージョン一覧テーブルを全行「完了」に更新する（テスト数も drift 補正後の実際値に修正）
8. `v84000_tests` モジュール（4件）を `driver.rs` に追加する

**新規型・関数の追加なし。宣言・ドキュメント・クリーンアップのみ。**

## テスト（4件）

```rust
fn cargo_toml_version_is_84_0_0()
fn changelog_has_v84_0_0()
fn milestone_has_observability_2()
fn readme_mentions_fav_observe()
```

各テストは `include_str!()` で対象ファイルを読み込み、期待文字列の存在を確認する。

## Success Criteria

- `cargo test` が 3909 tests pass（+4）、0 failures
- `Cargo.toml` バージョンが `"84.0.0"` である
- `CHANGELOG.md` に `"v84.0.0"` が含まれる
- `MILESTONE.md` に `"Observability 2.0"` が含まれる
- `README.md` に `"fav observe"` が含まれる

## Files to Modify

- `fav/Cargo.toml` — バージョン更新
- `CHANGELOG.md` — v84.0.0 エントリ追加
- `MILESTONE.md` — Observability 2.0 達成内容追記
- `README.md` — `fav observe` 言及追加
- `fav/src/driver.rs` — `v84000_tests` モジュール追加
- `versions/current.md` — v84.0.0 に更新
- `versions/roadmap/roadmap-v80.1-v85.0.md` — Sprint 4 テーブル更新
