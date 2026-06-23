# v24.7.0 — ドキュメントサイト v2 タスク

## ステータス: COMPLETE

---

## タスク一覧

### T0: 事前確認

- [x] `grep -n "version = " fav/Cargo.toml` — `"24.6.0"` であること
- [x] `grep -n "mod v247000_tests" fav/src/driver.rs | head -3` — 未存在
- [x] `ls site/content/learn/ 2>/dev/null` — 未存在
- [x] `ls site/content/cookbook/ 2>/dev/null` — 未存在
- [x] `ls site/app/packages/ 2>/dev/null` — 未存在

---

### T1: `site/content/learn/` — チュートリアル 3 記事

- [x] **T1-1**: `site/content/learn/getting-started.mdx` 作成
  - フロントマター（title / description）を含む
  - `"Hello"` を含む（テスト要件）
  - インストール手順 + Hello World コード例
- [x] **T1-2**: `site/content/learn/pipeline-basics.mdx` 作成
  - `stage` / `seq` / `par` / `|>` の基礎解説
- [x] **T1-3**: `site/content/learn/type-system.mdx` 作成
  - 型推論・エフェクトシステム・ジェネリクスの入門

---

### T2: `site/content/cookbook/` — レシピ 3 記事

- [x] **T2-1**: `site/content/cookbook/etl-csv-to-db.mdx` 作成
  - `"csv"` と `"db"` を両方含む（テスト要件）
  - CSV 読込 → DB 投入のコード例（runes/csv + runes/postgres）
- [x] **T2-2**: `site/content/cookbook/api-gateway.mdx` 作成
  - HTTP API 実装レシピ（`!Http` エフェクト使用例）
- [x] **T2-3**: `site/content/cookbook/parallel-pipeline.mdx` 作成
  - `par [StageA, StageB] |> Merge` パターンの解説

---

### T3: `site/app/packages/page.tsx` 作成

- [x] `site/app/packages/` ディレクトリ作成
- [x] `site/app/packages/page.tsx` 作成
  - `"rune"` を含む（テスト要件）
  - Rune パッケージ一覧の静的ページ（Next.js App Router コンポーネント）

---

### T4: `site/content/docs/bench/index.mdx` 作成

- [x] `site/content/docs/bench/` ディレクトリ作成
- [x] `site/content/docs/bench/index.mdx` 作成
  - `"benchmark"` を含む（テスト要件）
  - v20.3〜v24.7 のベンチマーク履歴リンク集 + `fav bench` 解説

---

### T5: `site/content/docs/spec/index.mdx` 作成

- [x] `site/content/docs/spec/` ディレクトリ作成
- [x] `site/content/docs/spec/index.mdx` 作成
  - `"fav spec"` を含む（テスト要件）
  - 形式的仕様書の概要 + `fav spec --format markdown` の使い方

---

### T6: `fav/src/driver.rs` — v247000_tests 追加

- [x] **事前確認**: `grep -n "fn version_is_24_6_0" fav/src/driver.rs | head -3`
- [x] **T6-1（必須）**: `v246000_tests::version_is_24_6_0` テスト関数を**削除**（モジュール自体と他 4 件は保持）
- [x] **T6-2**: `v247000_tests` モジュールを `v246000_tests` の直後に追加（6 件）
  - `learn_getting_started_exists`
  - `cookbook_etl_recipe_exists`
  - `packages_page_has_rune_keyword`
  - `bench_page_exists`
  - `spec_page_exists`
  - `changelog_has_v24_7_0`
- [x] `cargo test v247000 --bin fav` — 6/6 PASS を確認
- [x] `cargo test --bin fav` — リグレッションなし（1962 件合格）
  > 件数計算: 1957 (現在) - 1 (version_is_24_6_0 削除) + 6 (v247000_tests) = 1962

---

### T7: Cargo.toml + CHANGELOG + benchmarks

- [x] `fav/Cargo.toml` の `version = "24.6.0"` → `"24.7.0"` に変更（T6-1 完了後）
- [x] `CHANGELOG.md` 先頭に v24.7.0 エントリを追加
- [x] `benchmarks/v24.7.0.json` を新規作成（test_count: 1962、duration_ms: 17200）
- [x] `cargo test v247000 --bin fav` — 最終確認 6/6 PASS
- [x] `cargo test --bin fav` — リグレッションなし（1962 件合格）

---

## テスト一覧（v247000_tests、6 件）

| テスト名 | 内容 | 期待値 |
|---|---|---|
| `learn_getting_started_exists` | `site/content/learn/getting-started.mdx` に `"Hello"` が含まれる | assert |
| `cookbook_etl_recipe_exists` | `site/content/cookbook/etl-csv-to-db.mdx` に `"csv"` / `"db"` が含まれる | assert |
| `packages_page_has_rune_keyword` | `site/app/packages/page.tsx` に `"rune"` が含まれる | assert |
| `bench_page_exists` | `site/content/docs/bench/index.mdx` に `"benchmark"` が含まれる | assert |
| `spec_page_exists` | `site/content/docs/spec/index.mdx` に `"fav spec"` が含まれる | assert |
| `changelog_has_v24_7_0` | `CHANGELOG.md` に `[v24.7.0]` が含まれる | assert |

---

## 完了条件チェックリスト

- [x] `site/content/learn/getting-started.mdx` — `"Hello"` 含む
- [x] `site/content/learn/pipeline-basics.mdx` — 作成済み
- [x] `site/content/learn/type-system.mdx` — 作成済み
- [x] `site/content/cookbook/etl-csv-to-db.mdx` — `"csv"` / `"db"` 含む
- [x] `site/content/cookbook/api-gateway.mdx` — 作成済み
- [x] `site/content/cookbook/parallel-pipeline.mdx` — 作成済み
- [x] `site/app/packages/page.tsx` — `"rune"` 含む
- [x] `site/content/docs/bench/index.mdx` — `"benchmark"` 含む
- [x] `site/content/docs/spec/index.mdx` — `"fav spec"` 含む
- [x] `v246000_tests::version_is_24_6_0` 削除済み
- [x] `cargo test v247000 --bin fav` — 6/6 PASS
- [x] `cargo test --bin fav` — リグレッションなし（1962 件合格）
- [x] `CHANGELOG.md` に v24.7.0 エントリ
- [x] `benchmarks/v24.7.0.json` 作成済み（test_count: 1962）
