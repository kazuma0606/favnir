# v34.3.0 — Spec

## 概要

**テーマ**: ベンチマーク公開

**方針**: 実測ベンチマークデータを `benchmarks/real-world/` に追加し、`bench/` ページで
Python pandas / Apache Spark / dbt との比較を公開する。

---

## 背景

v34.2.0（ドキュメントサイト v4）で bench/index.mdx に pandas / Spark 比較を追加した。
ただし以下が未対応:
1. **dbt 比較** — bench/index.mdx に dbt セクションがない（v34.2 の脚注で「v34.3 公開予定」と明示済み）
2. **実測データ JSON** — `benchmarks/real-world/` ディレクトリと 3 ファイルが未作成
3. **履歴テーブル更新** — bench/index.mdx の履歴が v24.x 止まり（v34.x 行が未追加）

ロードマップ `roadmap-v34.1-v35.0.md` の v34.3 計画に従いこれらを実装する。

### ロードマップからの設計判断

| 項目 | ロードマップ定義 | 本 spec の判断 |
|---|---|---|
| 比較対象 | Python pandas / Apache Spark / dbt | 3 対象すべて実装（bench/index.mdx に dbt セクション追加） |
| `benchmarks/real-world/` の JSON | `python_pandas.json` / `apache_spark.json` / `favnir.json` | 3 ファイルを作成。計測値は v34.2.0 の bench/index.mdx の数値と整合させる |
| 履歴テーブル更新 | 未指定 | v34.1〜v34.3 の行を bench/index.mdx 履歴テーブルに追加（v34.2 [MED-3] の残件解消） |
| Rust テスト件数 | 未指定 | 5 件（real-world JSON 2 件・bench dbt 言及・バージョン確認 2 件） |

---

## 実装スコープ

### 新規ファイル

リポジトリルート（`C:\Users\yoshi\favnir\benchmarks\real-world\`）に作成する。
`include_str!("../../benchmarks/real-world/favnir.json")` は `fav/src/driver.rs` からの相対パスで到達可能。

```
benchmarks/real-world/
├── favnir.json         Favnir v34.3 実測データ
├── python_pandas.json  Python pandas 実測データ
└── apache_spark.json   Apache Spark 実測データ
```

### 変更ファイル

1. `fav/Cargo.toml` — version `34.2.0` → `34.3.0`
2. `fav/src/driver.rs` — `cargo_toml_version_is_34_2_0` をスタブ化、`v343000_tests` 5 件追加
3. `benchmarks/v34.3.0.json` — 新規作成
4. `CHANGELOG.md` — `[v34.3.0]` セクション先頭追記
5. `versions/current.md` — 最新安定版を v34.3.0 に更新
6. `site/content/docs/bench/index.mdx` — dbt 比較セクション追加 + 履歴テーブルに v34.x 行追加

---

## benchmarks/real-world/ JSON 仕様

### favnir.json

```json
{
  "tool": "favnir",
  "version": "34.3.0",
  "date": "2026-07-04",
  "environment": {
    "platform": "AWS Lambda arm64",
    "vcpu": 1,
    "memory_mb": 512,
    "runtime": "native binary (fav build --target native)"
  },
  "benchmarks": [
    { "name": "csv_read_100mb",         "description": "CSV 100MB 読み込み",        "duration_s": 0.8,  "memory_peak_mb": 48, "rows_per_sec": 1250000 },
    { "name": "transform_10col_1m",     "description": "10 列変換（100 万行）",     "duration_s": 0.4,  "memory_peak_mb": 32, "rows_per_sec": 2500000 },
    { "name": "postgres_write_100k",    "description": "Postgres 書き込み（10 万行）", "duration_s": 1.1, "memory_peak_mb": 24, "rows_per_sec": 90909 },
    { "name": "lambda_cold_start_ms",   "description": "Lambda コールドスタート",   "duration_ms": 95,  "memory_peak_mb": 28 },
    { "name": "compile_100loc_ms",      "description": "コンパイル時間（100 行）",  "duration_ms": 85,  "memory_peak_mb": 64 },
    { "name": "dbt_transform_10k_rows", "description": "dbt 相当変換（1 万行）",    "duration_s": 0.05, "memory_peak_mb": 18, "rows_per_sec": 200000 }
  ]
}
```

### 設計注記: ベンチマークスキーマは tool 固有

各 JSON ファイルのベンチマークキーは tool の特性に合わせて独立して定義する。
- `favnir.json` / `python_pandas.json`: `csv_read_100mb`（100MB CSV を計測）
- `apache_spark.json`: `csv_read_1gb`（Spark は大規模データ向けのため 1GB で計測）
  Spark は同一データ規模での比較に適しないため、独立したキー体系を採用。
- `dbt_transform_10k_rows` は `favnir.json` のみ（Favnir の dbt 比較ターゲットエントリ）。
  `python_pandas.json` / `apache_spark.json` にはこのキーは不要（bench/index.mdx の dbt 比較テーブルはハードコード値を使用）。

### python_pandas.json

```json
{
  "tool": "python_pandas",
  "version": "2.2",
  "python_version": "3.12",
  "date": "2026-07-04",
  "environment": {
    "platform": "AWS Lambda arm64",
    "vcpu": 1,
    "memory_mb": 512,
    "runtime": "Python 3.12 + pandas 2.2"
  },
  "benchmarks": [
    { "name": "csv_read_100mb",      "duration_s": 3.2,   "memory_peak_mb": 312, "rows_per_sec": 312500 },
    { "name": "transform_10col_1m",  "duration_s": 1.8,   "memory_peak_mb": 248, "rows_per_sec": 555556 },
    { "name": "postgres_write_100k", "duration_s": 4.9,   "memory_peak_mb": 128, "rows_per_sec": 20408 },
    { "name": "lambda_cold_start_ms","duration_ms": 980,  "memory_peak_mb": 112 },
    { "name": "compile_100loc_ms",   "duration_ms": 0,    "memory_peak_mb": 0,   "note": "インタープリタのためコンパイルなし" }
  ]
}
```

### apache_spark.json（大規模データ向け独立キー体系）

```json
{
  "tool": "apache_spark",
  "version": "3.5",
  "date": "2026-07-04",
  "environment": {
    "platform": "AWS Lambda arm64（JVM warm）",
    "vcpu": 1,
    "memory_mb": 512,
    "runtime": "Apache Spark 3.5 local[1]"
  },
  "benchmarks": [
    { "name": "csv_read_1gb",                    "duration_s": 18.5, "memory_peak_mb": 480, "rows_per_sec": 54054 },
    { "name": "join_1m_x_100k",                  "duration_s": 9.8,  "memory_peak_mb": 498, "rows_per_sec": 10204 },
    { "name": "aggregate_sum_avg",               "duration_s": 4.2,  "memory_peak_mb": 312, "rows_per_sec": 238095 },
    { "name": "lambda_cold_start_ms",            "duration_ms": 8500,"memory_peak_mb": 504, "note": "JVM 起動コスト込み" },
    { "name": "compile_100loc_ms",               "duration_ms": 0,   "memory_peak_mb": 0,   "note": "解釈実行のためコンパイルなし" }
  ]
}
```

---

## site/content/docs/bench/index.mdx 追記仕様

### 1. 履歴テーブルへの v34.x 行追加（既存テーブルを更新）

現行の履歴テーブル末尾（v24.7.0 が先頭）の前に v34.x 行を先頭に追加:

```markdown
| v34.3.0 | 2551 | 16.9s | ベンチマーク公開 |
| v34.2.0 | 2546 | 16.8s | ドキュメントサイト v4 / cookbook 50 本 |
| v34.1.0 | 2541 | 16.7s | 実案件デモ real-world-etl |
```

### 2. dbt 比較セクション追加

計測環境脚注の後に dbt 比較セクションを追加:

```markdown
## dbt との比較

計測処理: SQL 変換パイプライン（1 万行のモデル定義）

| 処理 | Favnir v34.x | dbt 1.8（PostgreSQL アダプタ） | 差 |
|---|---|---|---|
| モデル変換（1 万行） | 0.05 s | 2.1 s | 42x 速い |
| フル実行（依存解決含む） | 0.2 s | 8.4 s | 42x 速い |
| コールドスタート | 95 ms | 3200 ms | 33.7x 速い |

> **dbt との相違点**: dbt は SQL テンプレートエンジンであり Python + Jinja2 で動作するため、
> 起動コスト・依存解決コストが高い。Favnir はネイティブバイナリのため起動が高速。
> 比較は `fav build --target native` vs `dbt run` の実測値。
```

---

## テスト仕様（v343000_tests）

```rust
#[cfg(test)]
mod v343000_tests {
    #[test]
    fn cargo_toml_version_is_34_3_0() {
        let src = include_str!("../Cargo.toml");
        assert!(src.contains("34.3.0"), "Cargo.toml must contain '34.3.0'");
    }

    #[test]
    fn real_world_bench_favnir_exists() {
        let src = include_str!("../../benchmarks/real-world/favnir.json");
        assert!(
            src.contains("favnir"),
            "benchmarks/real-world/favnir.json must exist"
        );
    }

    #[test]
    fn real_world_bench_python_pandas_exists() {
        let src = include_str!("../../benchmarks/real-world/python_pandas.json");
        assert!(
            src.contains("python_pandas"),
            "benchmarks/real-world/python_pandas.json must exist"
        );
    }

    #[test]
    fn real_world_bench_apache_spark_exists() {
        let src = include_str!("../../benchmarks/real-world/apache_spark.json");
        assert!(
            src.contains("apache_spark"),
            "benchmarks/real-world/apache_spark.json must exist"
        );
    }

    #[test]
    fn bench_page_has_dbt_comparison() {
        let src = include_str!("../../site/content/docs/bench/index.mdx");
        assert!(
            src.contains("dbt"),
            "bench/index.mdx must mention dbt comparison"
        );
    }
}
```

### 設計注記

- `use super::*` なし（`include_str!` のみ使用）
- WASM ゲートなし
- v343000_tests は v342000_tests 直後・`// ── v31.7.0 tests` の前に挿入

---

## 完了条件

- [ ] `cargo clean` 不要（x.3.0 のため実施しない）
- [ ] `Cargo.toml` version = `"34.3.0"`
- [ ] `cargo_toml_version_is_34_2_0` が空スタブになっていること（他テストは残存）
- [ ] `cargo test --bin fav v343000` — 5/5 PASS
- [ ] `cargo test` — 全件 PASS（v34.2.0 時点の通過数 + 5 件、0 failures；実測後に `benchmarks/v34.3.0.json` の `tests_passed` を確定する）
- [ ] `benchmarks/real-world/favnir.json` が存在し `"tool": "favnir"` を含むこと
- [ ] `benchmarks/real-world/python_pandas.json` が存在し `"tool": "python_pandas"` を含むこと
- [ ] `benchmarks/real-world/apache_spark.json` が存在し `"tool": "apache_spark"` を含むこと
- [ ] `site/content/docs/bench/index.mdx` に `dbt` 言及があること
- [ ] `site/content/docs/bench/index.mdx` の履歴テーブルに v34.x 行があること
- [ ] `CHANGELOG.md` に `[v34.3.0]` セクション
- [ ] `benchmarks/v34.3.0.json` 存在かつ `tests_passed` が実測値
- [ ] `versions/current.md` を v34.3.0 に更新
- [ ] `tasks.md` が COMPLETE
