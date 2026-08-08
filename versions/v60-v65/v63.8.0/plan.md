# v63.8.0 Plan — 標準 ETL ベンチマークスイート

Version: 63.8.0
Status: 未着手

---

## 変更ファイル一覧

| ファイル | 変更内容 |
|---|---|
| `fav/src/driver.rs` | `cmd_bench_suite` 追加 + `v63800_tests` 追加 |

---

## 実装ステップ

### Step 1: `cmd_bench_suite` 追加

`cmd_bench_aot_vm` の直後に追加。

```rust
pub fn cmd_bench_suite(suite: &str) -> String { ... }
```

- `"etl-standard"` 以外はエラーメッセージを返す
- 2 ケース（csv-to-postgres / kafka-window-aggregate）を `cmd_bench_aot_vm(src, 1, "")` で実行
- 結果を改行結合して返す

### Step 3: ビルド・テスト全件確認

`cargo test -j 8 -- --test-threads=8` で 3425 tests passed, 0 failed を確認。

---

### Step 2: `v63800_tests` 追加

`v63700_tests` の直前に挿入。

- `bench_suite_etl_standard` — 出力に Benchmark / csv-to-postgres / kafka-window-aggregate / suite: が含まれることを確認
- `bench_regression_check` — 出力に VM / AOT が含まれること、未知スイート名が "unknown" を含むことを確認

---

## テスト計画

| テスト名 | 確認内容 |
|---|---|
| `bench_suite_etl_standard` | Benchmark, csv-to-postgres, kafka-window-aggregate, suite: |
| `bench_regression_check` | VM, AOT timing + unknown suite error |

ベース: 3423 → 目標: 3425（+2）
