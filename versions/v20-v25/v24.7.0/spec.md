# v24.7.0 — ドキュメントサイト v2

## テーマ

`site/` を完全リニューアルし、学習・リファレンス・レシピ・パッケージの 4 軸構成にする。

---

## 動機

- v24.1（`fav spec`）・v24.3（ベンチマーク）・v24.5（Rune レジストリ 50+）の成果物が
  サイトに反映されていない
- チュートリアル・実用レシピが不在で、新規ユーザーが入門できない
- `packages/` ページが存在せず、`fav search` との対応が取れていない

---

## 目標サイト構成

```
favnir.dev/
  docs/          言語リファレンス（既存 site/content/docs/ を充実）
  learn/         チュートリアル（入門〜応用）← NEW
  cookbook/      レシピ集（実際のユースケース）← NEW
  spec/          形式的仕様書（v24.1 の fav spec 出力）← NEW
  bench/         ベンチマーク推移グラフ（v24.3 の出力）← NEW
  playground/    Playground v2（既存）
  packages/      Rune レジストリ（v24.5 連携）← NEW app page
```

---

## 成果物（tasks.md の T1〜T5 と対応）

### T1: `site/content/learn/` — チュートリアル（3 記事）

| ファイル | タイトル |
|---|---|
| `getting-started.mdx` | はじめての Favnir（インストール〜Hello World） |
| `pipeline-basics.mdx` | パイプラインの基礎（stage / seq / par） |
| `type-system.mdx` | 型システム入門（型推論・エフェクト・ジェネリクス） |

### T2: `site/content/cookbook/` — レシピ集（3 記事）

| ファイル | タイトル |
|---|---|
| `etl-csv-to-db.mdx` | CSV を読んでデータベースに投入する |
| `api-gateway.mdx` | HTTP API を Favnir で実装する |
| `parallel-pipeline.mdx` | par stage で並列 ETL パイプライン |

### T3: `site/app/packages/page.tsx` — Rune レジストリページ

- `OFFICIAL_CATALOG` の 50 パッケージを一覧表示する静的ページ
- `"rune"` キーワードを含む（テスト要件）

### T4: `site/content/docs/bench/index.mdx` — ベンチマーク推移ページ

- v20.3〜v24.6 のベンチマーク履歴リンク集
- `"benchmark"` キーワードを含む（テスト要件）

### T5: `site/content/docs/spec/index.mdx` — 形式的仕様書ページ

- `fav spec` コマンド説明 + SPEC.md ダウンロードリンク
- `"fav spec"` キーワードを含む（テスト要件）

---

## Rust テスト（v247000_tests、6 件）

| テスト名 | 内容 | 期待値 |
|---|---|---|
| `learn_getting_started_exists` | `site/content/learn/getting-started.mdx` に `"Hello"` が含まれる | assert |
| `cookbook_etl_recipe_exists` | `site/content/cookbook/etl-csv-to-db.mdx` に `"csv"` / `"db"` が含まれる | assert |
| `packages_page_has_rune_keyword` | `site/app/packages/page.tsx` に `"rune"` が含まれる | assert |
| `bench_page_exists` | `site/content/docs/bench/index.mdx` に `"benchmark"` が含まれる | assert |
| `spec_page_exists` | `site/content/docs/spec/index.mdx` に `"fav spec"` が含まれる | assert |
| `changelog_has_v24_7_0` | `CHANGELOG.md` に `[v24.7.0]` が含まれる | assert |

---

## テスト件数

- 削除: `v246000_tests::version_is_24_6_0`（1 件）
- 追加: `v247000_tests`（6 件）
- 合計: **1957 − 1 + 6 = 1962 件**

---

## 完了条件

- [ ] `site/content/learn/` に 3 記事（getting-started / pipeline-basics / type-system）
- [ ] `site/content/cookbook/` に 3 記事（etl-csv-to-db / api-gateway / parallel-pipeline）
- [ ] `site/app/packages/page.tsx` に `"rune"` を含む Rune レジストリページ
- [ ] `site/content/docs/bench/index.mdx` に `"benchmark"` を含むページ
- [ ] `site/content/docs/spec/index.mdx` に `"fav spec"` を含むページ
- [ ] `cargo test v247000 --bin fav` — 6/6 PASS
- [ ] `cargo test --bin fav` — リグレッションなし（1962 件合格）
- [ ] `CHANGELOG.md` に v24.7.0 エントリ
- [ ] `benchmarks/v24.7.0.json` 作成済み
