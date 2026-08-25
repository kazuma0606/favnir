# spec: v84.2.0 — テスト統合ショーケース（`fav test` E2E）

## Background

> **テスト数注記**: ロードマップ計画値は 3,899/3,901 だったが、code-reviewer 対応の
> 累積により実際のベースは **3,911 tests**（v84.1.0 完了時点）。
> v84.2.0 完了目標は **3,913 tests**（+2）。

v84.1.0 で `infra/e2e-demo/favnir4-showcase/` の骨格（pipeline.fav / fav.toml /
contract.fav / README.md）を配置した。v84.2.0 では Sprint 1「Test-Driven Data 1.0」
の機能（TestSuite / StageTestCase / GoldenDataset / SchemaSnapshot）を
`pipeline.fav` に統合し、ショーケースが型付きテストを示すことを確認する。

## Goals

1. `infra/e2e-demo/favnir4-showcase/pipeline.fav` に TestSuite セクションを追加する
   - `StageTestCase` を使った単体テスト関数
   - `GoldenDataset` 比較テスト関数
   - `SchemaSnapshot` 比較テスト関数
2. Rust テスト 2 件でショーケースの内容を検証する
   - `showcase_test_suite_passes` — TestSuite / StageTestCase の存在確認
   - `showcase_golden_dataset_comparison` — GoldenDataset / SchemaSnapshot の存在確認

## Syntax / API Examples

### pipeline.fav への追加セクション

```favnir
-- ── テスト統合セクション（Sprint 1: Test-Driven Data 1.0）────────────

fn showcase_stage_test(ctx: AppCtx) -> Result<List<TestResult>, String> {
    bind suite <- TestSuite.new("showcase")
    bind suite <- suite.add(StageTestCase {
        name: "load_stage_returns_rows",
        run: fn() -> Bool { True },
    })
    Result.ok(TestSuite.run(suite))
}

fn showcase_golden_dataset(ctx: AppCtx) -> Result<Bool, String> {
    bind expected <- GoldenDataset.load("data/golden.csv")
    bind actual   <- load_stage(ctx)
    bind result   <- compare_golden_dataset(expected, actual)
    Result.ok(result.passed)
}

fn showcase_schema_snapshot(ctx: AppCtx) -> Result<Bool, String> {
    bind snapshot <- SchemaSnapshot.load("snapshots/schema.json")
    bind rows     <- load_stage(ctx)
    bind result   <- SchemaSnapshot.compare(snapshot, rows)
    Result.ok(result.matches)
}
```

### v84200_tests（Rust テスト）

```rust
#[cfg(test)]
mod v84200_tests {
    #[test]
    fn showcase_test_suite_passes() {
        let content = include_str!("../../infra/e2e-demo/favnir4-showcase/pipeline.fav");
        assert!(content.contains("TestSuite"), "pipeline.fav should include TestSuite");
        assert!(content.contains("StageTestCase"), "pipeline.fav should include StageTestCase");
    }

    #[test]
    fn showcase_golden_dataset_comparison() {
        let content = include_str!("../../infra/e2e-demo/favnir4-showcase/pipeline.fav");
        assert!(content.contains("GoldenDataset"), "pipeline.fav should include GoldenDataset");
        assert!(content.contains("SchemaSnapshot"), "pipeline.fav should include SchemaSnapshot");
    }
}
```

## Success Criteria

- `infra/e2e-demo/favnir4-showcase/pipeline.fav` に `TestSuite`・`StageTestCase`・
  `GoldenDataset`・`SchemaSnapshot` の各識別子が含まれること
- `cargo test` が 3,913 tests pass（+2）、0 failures であること

## Error Codes

なし（本バージョンはファイル更新のみ。構文エラーは fav build で確認）

## Files to Modify / Create

### 更新
- `infra/e2e-demo/favnir4-showcase/pipeline.fav` — TestSuite セクションを末尾に追加

### 追記
- `fav/src/driver.rs` — `v84200_tests` モジュール追加（2 テスト）
- `CHANGELOG.md` — v84.2.0 エントリ追加

### パス起点の違いについて（v84.1.0 から踏襲）

| マクロ / 関数 | パス起点 | 理由 |
|---|---|---|
| `std::path::Path::new("../infra/...")` | `fav/`（cargo test CWD） | ランタイム解決 |
| `include_str!("../../infra/...")` | `fav/src/`（ソースファイル位置） | コンパイル時マクロ |

v84200_tests は `include_str!` のみを使うため `"../../infra/..."` 形式を使用する。

> **注意**: `include_str!` のパスは `fav/src/driver.rs` の位置を起点とする。
> `driver.rs` を別ディレクトリへ移動した場合はパスを更新すること。
