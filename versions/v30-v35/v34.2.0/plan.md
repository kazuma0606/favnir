# v34.2.0 — 実装プラン

## 方針

ドキュメントサイト v4 パターン。新規 MDX ファイル作成が主体。
`cargo clean` は x.2.0 のため不要。

---

## 実装ステップ

### Step 1: Cargo.toml バージョン更新

`fav/Cargo.toml` の version を `34.1.0` → `34.2.0` に変更。

---

### Step 2: site/content/errors/index.mdx 作成

`catalog.json` の 57 エントリを参照したエラーコードリファレンスページ。
上位カテゴリ（pipeline / type / effect / lint）でグループ化した MDX を作成する。

内容:
- フロントマター（title / description）
- `fav explain <CODE>` の使い方説明
- 全エラーコードをカテゴリ別テーブルで列挙（catalog.json の主要エントリを抜粋）

---

### Step 3: site/content/docs/bench/index.mdx 更新

既存ファイルを Read してから末尾に Python pandas / Spark 比較セクションを追加する。

追加内容:
- `## Python pandas との比較` — 3 指標の比較テーブル
- `## Apache Spark との比較` — 2 指標の比較テーブル
- 計測環境の脚注

---

### Step 4: cookbook 18 本追加

以下の順で新規 MDX を作成する（各ファイルは最小構成: フロントマター + 概要 + コードサンプル + 解説）。

```
postgres-etl.mdx
snowflake-load.mdx
duckdb-query.mdx
parquet-transform.mdx
avro-schema.mdx
iceberg-compaction.mdx
mongodb-etl.mdx
redis-cache-aside.mdx
elasticsearch-index.mdx
http-api-ingest.mdx
csv-validation.mdx
schema-evolution.mdx
data-quality-check.mdx
incremental-load.mdx
cron-trigger.mdx
secret-manager.mdx
jwt-auth.mdx
grpc-client.mdx
```

各ファイルの最小構成:

```mdx
---
title: "<タイトル>"
description: "<一行説明>"
---

# <タイトル>

<概要 2〜3 文>

## コード例

\`\`\`favnir
<サンプルコード>
\`\`\`

## ポイント

<解説 2〜3 行>
```

---

### Step 5: driver.rs 更新

1. `cargo_toml_version_is_34_1_0` を空スタブ化（コメント付き）
2. `v341000_tests` 直後・`// ── v31.7.0 tests` の前に `v342000_tests` を挿入

挿入位置の確認コマンド:

```bash
grep -n "v341000_tests\|// ── v31\.7\.0 tests" fav/src/driver.rs
# v341000_tests の閉じ括弧と // ── v31.7.0 tests の行番号を確認
```

```rust
// ── v34.2.0 tests ────────────────────────────────────────────────────────────
#[cfg(test)]
mod v342000_tests {
    #[test]
    fn cargo_toml_version_is_34_2_0() {
        let src = include_str!("../Cargo.toml");
        assert!(src.contains("34.2.0"), "Cargo.toml must contain '34.2.0'");
    }

    #[test]
    fn errors_index_mdx_exists() {
        let src = include_str!("../../site/content/errors/index.mdx");
        assert!(
            src.contains("E0101"),
            "site/content/errors/index.mdx must exist and reference error code E0101"
        );
    }

    #[test]
    fn bench_page_has_python_comparison() {
        let src = include_str!("../../site/content/docs/bench/index.mdx");
        assert!(
            src.contains("pandas") || src.contains("Python"),
            "bench/index.mdx must mention Python pandas comparison"
        );
    }

    #[test]
    fn cookbook_postgres_etl_exists() {
        let src = include_str!("../../site/content/cookbook/postgres-etl.mdx");
        assert!(
            src.contains("postgres") || src.contains("Postgres"),
            "site/content/cookbook/postgres-etl.mdx must exist"
        );
    }

    #[test]
    fn cookbook_snowflake_load_exists() {
        let src = include_str!("../../site/content/cookbook/snowflake-load.mdx");
        assert!(
            src.contains("Snowflake") || src.contains("snowflake"),
            "site/content/cookbook/snowflake-load.mdx must exist"
        );
    }
}
```

---

### Step 6: CHANGELOG.md 更新

先頭に `[v34.2.0]` セクションを追加:

```markdown
## [v34.2.0] — 2026-07-04

### Added
- `site/content/errors/index.mdx` — エラーコードリファレンスページ（57 コード、カテゴリ別）
- `site/content/cookbook/` — 18 本追加（計 50 本）
  postgres-etl / snowflake-load / duckdb-query / parquet-transform / avro-schema /
  iceberg-compaction / mongodb-etl / redis-cache-aside / elasticsearch-index /
  http-api-ingest / csv-validation / schema-evolution / data-quality-check /
  incremental-load / cron-trigger / secret-manager / jwt-auth / grpc-client

### Changed
- `site/content/docs/bench/index.mdx` — Python pandas / Apache Spark との比較データを追加
```

---

### Step 7: benchmarks/v34.2.0.json 作成

```json
{
  "version": "34.2.0",
  "milestone": "Production Ready",
  "date": "2026-07-04",
  "tests_passed": 2546,
  "tests_failed": 0,
  "notes": "ドキュメントサイト v4: errors/index.mdx 追加・cookbook 50 本達成・bench 比較データ追加。v342000_tests 5 件追加。"
}
```

（`tests_passed` は `cargo test` 実測後に確定）

---

### Step 8: versions/current.md 更新

最新安定版を v34.2.0 に変更。

---

## テスト実行

```bash
cd /c/Users/yoshi/favnir/fav && cargo test --bin fav v342000 2>&1 | tail -8
cd /c/Users/yoshi/favnir/fav && cargo test -j 8 -- --test-threads=8 2>&1 | grep "test result"
```

---

## 完了処理

- `benchmarks/v34.2.0.json` の `tests_passed` を実測値で確定
- `tasks.md` を COMPLETE に更新
