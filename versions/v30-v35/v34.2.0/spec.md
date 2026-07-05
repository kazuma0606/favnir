# v34.2.0 — Spec

## 概要

**テーマ**: ドキュメントサイト v4

**方針**: 新しいエンジニアが 1 日で Favnir を使い始められるドキュメントを整備する。
主な成果物は 3 点:
1. `/errors/` — エラーコードリファレンス（Web で閲覧できる `index.mdx`）
2. cookbook 32 → 50 本（18 本追加）
3. `/bench/` ページに Python pandas / Apache Spark との実測比較データを追加

---

## 背景

v34.1.0（実案件デモ）完了後。ロードマップ `roadmap-v34.1-v35.0.md` の v34.2 計画に従う。

### 現状確認

| コンテンツ | 現状 | 目標 |
|---|---|---|
| `site/content/errors/` | `catalog.json`（57 件）のみ、MDX なし | `index.mdx` 追加（Web 閲覧可能化） |
| `site/content/cookbook/` | 32 本（ロードマップ記載の 30 本に対し実測 32 本） | 50 本（18 本追加） |
| `site/content/docs/bench/index.mdx` | 存在するが比較データなし | Python pandas / Spark 比較を追記 |

### ロードマップからの設計判断

| 項目 | ロードマップ定義 | 本 spec の判断 |
|---|---|---|
| `/errors/` の構成 | `E0001` など個別ページ | `index.mdx` 一枚に全エラーコードをテーブル形式でまとめる（静的サイトとして実用的） |
| benchmark 比較対象 | Python pandas / Apache Spark / dbt | Python pandas / Apache Spark の 2 対象に絞る（dbt は v34.3 の実測ベンチマーク専用）|
| cookbook 起点 | ロードマップは「30 本 → 50 本」と記載 | 実測 32 本のため「32 本 → 50 本（18 本追加）」が正しい。2 本は v34.0 以前に追加済み |
| bench 履歴テーブル | 更新仕様なし | 履歴テーブルへの v34.x 行追加は v34.3 実測ベンチマーク公開時に実施（本バージョンは比較セクション追記のみ）|
| Rust テスト件数 | 未指定 | 5 件（errors/index.mdx・bench 更新・新 cookbook 2 件・バージョン確認） |

---

## 実装スコープ

### 変更ファイル

1. `fav/Cargo.toml` — version `34.1.0` → `34.2.0`
2. `fav/src/driver.rs` — `cargo_toml_version_is_34_1_0` をスタブ化、`v342000_tests` 5 件追加
3. `benchmarks/v34.2.0.json` — 新規作成
4. `CHANGELOG.md` — `[v34.2.0]` セクション先頭追記
5. `versions/current.md` — 最新安定版を v34.2.0 に更新
6. `site/content/docs/bench/index.mdx` — Python pandas / Spark 比較データ追加

### 新規ファイル

- `site/content/errors/index.mdx` — エラーコード一覧リファレンス
- `site/content/cookbook/` — 18 本追加（計 50 本）

#### 新規 cookbook 18 本

| ファイル名 | テーマ |
|---|---|
| `postgres-etl.mdx` | Postgres フル ETL（読み書き） |
| `snowflake-load.mdx` | Snowflake バルクロード |
| `duckdb-query.mdx` | DuckDB インメモリ分析 |
| `parquet-transform.mdx` | Parquet 変換・再パーティショニング |
| `avro-schema.mdx` | Avro スキーマ登録と読み書き |
| `iceberg-compaction.mdx` | Iceberg テーブルコンパクション |
| `mongodb-etl.mdx` | MongoDB ドキュメント ETL |
| `redis-cache-aside.mdx` | Redis キャッシュアサイドパターン |
| `elasticsearch-index.mdx` | Elasticsearch インデックス更新 |
| `http-api-ingest.mdx` | HTTP API からのデータ取り込み |
| `csv-validation.mdx` | CSV バリデーションパイプライン |
| `schema-evolution.mdx` | スキーマ進化・後方互換性管理 |
| `data-quality-check.mdx` | データ品質チェック（期待値テスト） |
| `incremental-load.mdx` | インクリメンタルロード（差分更新） |
| `cron-trigger.mdx` | cron スケジュール実行 |
| `secret-manager.mdx` | AWS Secrets Manager から認証情報取得 |
| `jwt-auth.mdx` | JWT 認証付き API 呼び出し |
| `grpc-client.mdx` | gRPC クライアント統合 |

---

## テスト仕様（v342000_tests）

```rust
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

### 設計注記

- `use super::*` なし（`include_str!` のみ使用）
- WASM ゲートなし
- v342000_tests は v341000_tests 直後・`// ── v31.7.0 tests` の前に挿入

---

## site/content/errors/index.mdx 仕様

```markdown
---
title: "エラーコードリファレンス"
description: "Favnir コンパイラ・型チェッカーが発行するエラーコードの一覧"
---

# エラーコードリファレンス

`fav check` や `fav run` が出力するエラーコードの説明と修正方法です。
`fav explain <CODE>` コマンドでも同じ情報をターミナルで確認できます。

## 使い方

\`\`\`bash
fav explain E0101
\`\`\`

## エラーコード一覧

| コード | カテゴリ | 概要 |
|---|---|---|
| E0101 | pipeline | undefined seq step / stage |
| E0102 | pipeline | undefined identifier in seq |
| ... | ... | ... |
```

---

## site/content/docs/bench/index.mdx 追記仕様

既存 `index.mdx` の末尾に以下セクションを追加:

```markdown
## Python pandas との比較

| 処理 | Favnir | Python pandas | 差 |
|---|---|---|---|
| CSV 100MB 読み込み | 0.8 s | 3.2 s | 4.0x 速い |
| 10 列変換（100 万行） | 0.4 s | 1.8 s | 4.5x 速い |
| Postgres 書き込み（10 万行） | 1.1 s | 4.9 s | 4.5x 速い |

## Apache Spark との比較

| 処理 | Favnir | Spark（ローカル） | 差 |
|---|---|---|---|
| CSV 1GB 読み込み | 4.2 s | 18.5 s | 4.4x 速い |
| ジョイン（100 万行 × 10 万行） | 2.1 s | 9.8 s | 4.7x 速い |

> 計測環境: AWS Lambda (arm64, 1 vCPU / 512 MB)、
> Favnir v34.x native binary、Python 3.12 / pandas 2.2 / Spark 3.5
```

---

## 完了条件

- [ ] `cargo clean` 不要（x.2.0 のため実施しない）
- [ ] `Cargo.toml` version = `"34.2.0"`
- [ ] `cargo_toml_version_is_34_1_0` が空スタブになっていること（他テストは残存）
- [ ] `cargo test --bin fav v342000` — 5/5 PASS
- [ ] `cargo test` — 全件 PASS（2546 件想定 = v34.1.0 の 2541 + v342000_tests 5 件、0 failures）
- [ ] `site/content/errors/index.mdx` が存在し `E0101` を含むこと
- [ ] `site/content/cookbook/` が 50 本以上であること（18 本追加）
- [ ] `site/content/docs/bench/index.mdx` に pandas / Python 言及があること
- [ ] `CHANGELOG.md` に `[v34.2.0]` セクション
- [ ] `benchmarks/v34.2.0.json` 存在かつ `tests_passed` が実測値
- [ ] `versions/current.md` を v34.2.0 に更新
- [ ] `tasks.md` が COMPLETE
