# v62.0.0 仕様書 — Language Polish 宣言 ★クリーンアップ

## 概要

v61.1〜v61.9 で実装した Language Polish 機能群（OR パターン・as-pattern・個別ガード・record update・f-string 強化・型エラー差分表示・`_` 型プレースホルダー・`--strict` モード・安定化）の完成を宣言する。

これは v61 サブスプリントの締めくくりであり、★クリーンアップ（`cargo clean`）を伴うメジャー宣言バージョン。

---

## 宣言文

> 「パターンは OR で分岐し、as で束縛される。
>  レコードは `{ base | field: value }` で一部だけ書き換えられる。
>  型注釈に `_` を置けば推論が答えを返す。
>  エラーは期待値と実際値の差分を語り、修正の道筋を示す。
>
>  Favnir の型システムはデータエンジニアの思考を助ける存在になった。
>
>  これが Favnir v62.0 — Language Polish の姿である。」

---

## 変更内容

### 1. `fav/Cargo.toml` バージョン更新

```toml
version = "62.0.0"
```

現行: `61.0.0` → `62.0.0` に変更。

### 2. `MILESTONE.md` — Language Polish 宣言エントリ追加

v62.0.0 の宣言文と v61.1〜v61.9 で実装した機能一覧を追記する。

### 3. `README.md` — v62.0 Language Polish 言及追加

バージョン履歴テーブルまたは「最新の変更」セクションに v62.0 Language Polish を追記する。

### 4. `CHANGELOG.md` — v62.0.0 エントリ追加

v61.1〜v61.9 の機能を集約したエントリを追記。

### 5. `driver.rs` — `v62000_tests` 追加（4 件）

`fav/src/driver.rs` からの `include_str!` 相対パス:
- `fav/Cargo.toml` → `"../../Cargo.toml"`
- `CHANGELOG.md` → `"../../../CHANGELOG.md"`
- `MILESTONE.md` → `"../../../MILESTONE.md"`
- `README.md` → `"../../../README.md"`

| テスト名 | 検証内容 | assert 条件 |
|---|---|---|
| `cargo_toml_version_is_62_0_0` | `Cargo.toml` バージョン確認 | `contains("version = \"62.0.0\"")` |
| `changelog_has_v62_0_0` | `CHANGELOG.md` エントリ確認 | `contains("v62.0.0")` |
| `milestone_has_language_polish` | `MILESTONE.md` v62.0 固有エントリ確認 | `contains("v62.0.0")` かつ `contains("Language Polish")` |
| `readme_mentions_language_polish` | `README.md` v62.0 言及確認 | `contains("v62.0")` かつ `contains("Language Polish")` |

**注意**: `MILESTONE.md` と `README.md` には v32.0.0 時点の既存 "Language Polish" 記述があるが、テストは `"v62.0.0"` / `"v62.0"` との組み合わせを確認するため、v62.0 固有の追記が必須となる。

### 6. `★クリーンアップ`

`cargo clean` を実行してビルド成果物をリセット。その後フル再ビルドで健全性確認。

---

## 完了条件

- `cargo test -j 8 -- --test-threads=8` で **3382 tests passed, 0 failed**
  - ベース 3378 + 4 = 3382
- `Cargo.toml version = "62.0.0"` に変更済み
- `MILESTONE.md` に v62.0.0 固有の Language Polish 宣言エントリあり（テストとは独立して追記必須）
- `README.md` に "v62.0" + "Language Polish" 言及あり（テストとは独立して追記必須）
- `CHANGELOG.md` に v62.0.0 エントリあり
- `cargo clean` 完了

---

## 参照

- ロードマップ: `versions/roadmap/roadmap-v61.1-v62.0.md`
- 前バージョン: v61.9.0（3378 tests、安定化完了）
