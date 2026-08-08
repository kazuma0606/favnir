# v64.3.0 Plan — パフォーマンスガイド（`site/content/docs/runtime/performance.mdx`）

Version: 64.3.0
Status: 未着手

---

## 変更ファイル一覧

| ファイル | 変更内容 |
|---|---|
| `site/content/docs/runtime/performance.mdx` | 新規作成（パフォーマンスチューニングガイド） |
| `fav/src/driver.rs` | `v64300_tests` 追加（2テスト） |

---

## 実装ステップ

### Step 1: `site/content/docs/runtime/performance.mdx` 新規作成

以下のセクションを含むパフォーマンスチューニングガイドを作成:
- frontmatter（title / description）
- `## AOT Compilation` — `fav build --link` / `fav bench` の使い方
- `## Incremental Compilation` — `.fav-cache/` / cache hit の説明
- `## Parallel Execution` — `par` / `--opt-stats` / `fav parallel-stats`
- `## DAG Optimization` — dead stage 削除 / pure stage fusion
- `## Backpressure Control` — `fav.toml` `[backpressure]` セクション
- `## Bottleneck Identification Workflow` — 4ステップの手順
- `## Regression Detection` — `[bench]` `regression_threshold_pct`

### Step 2: `driver.rs` — `v64300_tests` 追加

`v64200_tests` の直前に挿入:
- `docs_performance_guide_exists`: `include_str!` でファイル読み込み、空でないこと・`"Performance"` 含むことを確認
- `docs_performance_has_aot_section`: `"AOT"` / `"fav bench"` / `"fav profile"` を含むことを確認

### Step 3: ビルド・テスト全件確認

- `cargo build` エラーなし
- `cargo test --bin fav v64300_tests` で 2 件 PASS
- `cargo test -j 8 -- --test-threads=8` で 3437 tests passed, 0 failed

---

## テスト計画

| テスト名 | 確認内容 |
|---|---|
| `docs_performance_guide_exists` | ファイル存在・非空・`"Performance"` 含む |
| `docs_performance_has_aot_section` | `"AOT"` / `"fav bench"` / `"fav profile"` 含む |

ベース: 3435 → 目標: 3437（+2）
