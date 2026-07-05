# v35.7.0 (v35.0B) — docs_server.rs !Effect 完全除去

## バージョン概要

| 項目 | 内容 |
|---|---|
| バージョン | v35.7.0 |
| コードネーム | v35.0B |
| 目的 | `docs_server.rs` から `!Effect` 表記を完全除去し、!Effect 廃止を完結させる |
| 前提 | v35.6.0（v35.0A）完了済み — サイト MDX 128 ファイルから `!Effect` 除去完了 |

## 背景

v35.5.0〜v35.6.0 で `!Effect` 廃止作業を実施し、以下を達成した：

- `ast.rs` の `Effect` enum および `effects` フィールド削除
- `checker.rs`, `compiler.rs`, `lineage.rs`, `ir.rs`, `reachability.rs` の Effect 参照削除
- `lint.rs` の W007/W021 を no-op 化
- `site/content/` 128 MDX ファイルからの `!Effect` アノテーション除去
- `README.md`, `MILESTONE.md` の更新

残存する唯一の `!Effect` 参照は `fav/src/docs_server.rs` の stdlib 関数表示文字列である。

## 残存箇所の特定

**ファイル**: `fav/src/docs_server.rs`

**対象**: `IO_FUNCTIONS` 定数内の 3 関数エントリ（計 6 行）

| 関数 | 修正前 | 修正後 |
|---|---|---|
| `IO.println` — signature | `"String -> Unit !Io"` | `"String -> Unit"` |
| `IO.println` — effects | `&["Io"]` | `&[]` |
| `IO.print` — signature | `"String -> Unit !Io"` | `"String -> Unit"` |
| `IO.print` — effects | `&["Io"]` | `&[]` |
| `IO.read_line` — signature | `"() -> String !Io"` | `"() -> String"` |
| `IO.read_line` — effects | `&["Io"]` | `&[]` |

## 設計方針

- `StdlibFunction` 構造体の `effects` フィールド自体は **残す**
  - JSON API スキーマ（`/api/stdlib`）の後方互換性のため
  - 値のみを空配列 `&[]` にする
- `signature` 文字列から `!Io` サフィックスを除去
- `schema_version` は `"3.1"` のまま変更しない（フィールド削除なし）

## テスト方針

`v35700_tests` モジュールを `driver.rs` に追加。テスト内容：

1. **`cargo_toml_version_is_35_7_0`** — Cargo.toml バージョンが `35.7.0`
2. **`docs_server_io_signatures_no_effect`** — `IO_FUNCTIONS` の signature に `!` が含まれないことを確認
3. **`docs_server_io_effects_empty`** — `IO_FUNCTIONS` の全 effects が空配列であることを確認
4. **`changelog_has_v35_7_0`** — CHANGELOG.md に `## [35.7.0]` エントリが存在する
5. **`effect_annotation_fully_purged`** — `docs_server.rs` ソース全体に `!Io` / `!Http` 等の `!Effect` 文字列が存在しないことを grep で確認

## 影響範囲

| ファイル | 変更内容 |
|---|---|
| `fav/src/docs_server.rs` | IO_FUNCTIONS 6 行修正 |
| `fav/Cargo.toml` | バージョンを `35.7.0` に更新 |
| `CHANGELOG.md` | `## [35.7.0]` エントリ追加 |
| `fav/src/driver.rs` | `v35700_tests` モジュール追加（5 テスト） |

## 完了条件

- [ ] `docs_server.rs` に `!` + 大文字英字で始まるエフェクト表記が存在しない
- [ ] 全テスト pass（0 failures）
- [ ] Cargo.toml バージョン = `35.7.0`
- [ ] CHANGELOG に `## [35.7.0]` エントリ
