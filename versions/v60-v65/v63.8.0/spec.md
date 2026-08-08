# v63.8.0 Spec — 標準 ETL ベンチマークスイート

Version: 63.8.0
Status: 未着手

---

## 概要

`driver.rs` に `cmd_bench_suite(suite: &str) -> String` を追加する。
`"etl-standard"` スイートに対して、代表的な ETL ワークロード（CSV 変換・Kafka 処理）の
コンパイル時間（VM ビルド / AOT）を計測し、フォーマットされた結果を返す。

出力例（ロードマップ準拠）:
```
Benchmark suite: etl-standard
Benchmark: csv-to-postgres (1M rows)
Mode     | Mean (ms) | P99 (ms)
...
Benchmark: kafka-window-aggregate (10M events)
...
suite: 2 benchmarks complete
```

---

## 前提確認（T0 で実施）

- `cargo test -j 8 -- --test-threads=8` でベース 3423 tests passed, 0 failed を確認
- `driver.rs` に `cmd_bench_aot_vm` が存在することを確認（`cmd_bench_suite` 内部で再利用）
- `driver.rs` に `cmd_bench_suite` が存在しないことを確認（新規追加）
- `driver.rs` に `v63700_tests` が存在することを確認（`v63800_tests` の挿入位置確認）

**ロードマップとの差異（重要）**:
- ロードマップ原案は `fav/benchmarks/` ディレクトリへの `.fav` ファイル追加と `fav bench --suite` CLI フラグを記載しているが、本バージョンでは `driver.rs` への `cmd_bench_suite` 関数追加のみ実装する。
- 実際のファイルシステム I/O・CLI フラグ統合・リアルな行数スループット計測は非スコープ（後送り）。
- ロードマップのベーステスト数（3420）は v63.6.0 code-reviewer 対応 +3 件および v63.7.0 code-reviewer 対応により実際は 3423 に変更。

---

## 実装スコープ

### 1. `driver.rs` — `cmd_bench_suite` 追加

`cmd_bench_aot_vm` の直後に追加:

```rust
/// v63.8.0: 標準 ETL ベンチマークスイートを実行し結果を返す。
/// `suite` に "etl-standard" を指定すると csv-to-postgres / kafka-window-aggregate の
/// コンパイルベンチ（VM build + AOT lower）を `cmd_bench_aot_vm` で計測して返す。
pub fn cmd_bench_suite(suite: &str) -> String {
    if suite != "etl-standard" {
        return format!("unknown benchmark suite: {}. Available: etl-standard", suite);
    }

    let cases: &[(&str, &str, &str)] = &[
        (
            "csv-to-postgres",
            "1M rows",
            "public stage LoadCsv: Int -> Int = |x| { x }\n\
             public stage Transform: Int -> Int = |x| { x + 1 }\n\
             public stage Insert: Int -> Int = |x| { x }",
        ),
        (
            "kafka-window-aggregate",
            "10M events",
            "public stage Consume: Int -> Int = |x| { x }\n\
             public stage WindowAgg: Int -> Int = |x| { x * 2 }",
        ),
    ];

    let mut lines = vec![format!("Benchmark suite: {suite}")];
    for &(name, scale, src) in cases {
        let stats = cmd_bench_aot_vm(src, 1, "");
        lines.push(format!("Benchmark: {name} ({scale})"));
        lines.push(stats);
    }
    lines.push(format!("suite: {} benchmarks complete", cases.len()));
    lines.join("\n")
}
```

### 2. `driver.rs` — `v63800_tests` 追加

`v63700_tests` の直前に挿入:

```rust
// -- v63800_tests (v63.8.0) -- 標準 ETL ベンチマークスイート --
#[cfg(test)]
mod v63800_tests {
    #[test]
    fn bench_suite_etl_standard() {
        let out = crate::driver::cmd_bench_suite("etl-standard");
        assert!(out.contains("Benchmark"), "should contain Benchmark: {}", out);
        assert!(out.contains("csv-to-postgres"), "should include csv-to-postgres: {}", out);
        assert!(out.contains("kafka-window-aggregate"), "should include kafka-window-aggregate: {}", out);
        assert!(out.contains("suite:"), "should include completion summary: {}", out);
    }

    #[test]
    fn bench_regression_check() {
        let out = crate::driver::cmd_bench_suite("etl-standard");
        assert!(out.contains("VM"), "should contain VM timing: {}", out);
        assert!(out.contains("AOT"), "should contain AOT timing: {}", out);
        // 未知のスイート名はエラーメッセージを返す
        let err = crate::driver::cmd_bench_suite("nonexistent-suite");
        assert!(err.contains("unknown"), "unknown suite should return error: {}", err);
    }
}
```

---

## 完了条件

- `cargo build` エラーなし
- `cargo test --bin fav v63800_tests` で 2 件 PASS
  - `bench_suite_etl_standard` PASS（csv-to-postgres / kafka-window-aggregate が出力に含まれる）
  - `bench_regression_check` PASS（VM/AOT タイミング + 未知スイートエラーを確認）
- `cargo test -j 8 -- --test-threads=8` で 3425 tests passed, 0 failed

---

## 非スコープ

以下は後送り（v64.0 以降でロードマップ更新の上対応）:

- `fav/benchmarks/` ディレクトリへの新規 `.fav` ファイル追加
- `fav bench --suite` CLI フラグ（`main.rs` 更新）
- リアルなスループット計測（行数/秒、events/秒）
- `bench-results.json` へのファイル出力
- `compiler.rs` DAG 最適化との連携ベンチ

---

## 技術ノート

### `cmd_bench_aot_vm` の再利用

既存の `cmd_bench_aot_vm(src, runs, json_out)` は:
- `src`: Favnir ソース文字列
- `runs`: 計測繰り返し回数
- `json_out`: JSON 出力パス（空文字列 = 出力なし）

`cmd_bench_suite` 内では `runs=1`・`json_out=""` で呼び出し、返り値のテーブル文字列をそのまま出力に組み込む。
これにより、実際のコンパイル計測（VM bytecode build / AOT cranelift lower）が実行される。

### 挿入位置

`cmd_bench_aot_vm` は `driver.rs` 行 2322 付近に存在する。
`cmd_bench_suite` はその直後（`cmd_bench_graphql` 等より前）に追加する。

### ベーステスト数の差異

ロードマップ記載: base=3420、target=3422。
実際: v63.6.0 code-reviewer 対応 +3 件 + v63.7.0 code-reviewer 対応により base=3423、target=3425。
