# plan: v84.2.0 — テスト統合ショーケース（`fav test` E2E）

## 実装ステップ（依存順）

### Step 1: 事前確認

- `cargo test` を実行し、3,911 tests, 0 failures を確認する（前提: v84.1.0 完了済み）
- `Cargo.toml` バージョンが `84.0.0` であることを確認する
  （v84.x マイナーバージョンは Cargo.toml を更新しない慣例。v85.0.0 宣言時に一括更新する。
   この慣例は v84.0.0 宣言時から適用されており、v85.0.0 の tasks.md T2 で Cargo.toml 更新が行われる）
- `fav/src/driver.rs` に `mod v84100_tests` が存在することを確認する

### Step 2: pipeline.fav にテスト統合セクションを追加

現在の `pipeline.fav`（4 ステージ骨格）の末尾に、TestSuite / GoldenDataset /
SchemaSnapshot を使ったテスト関数 3 本を追加する。

追加する関数:

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

### Step 3: driver.rs に v84200_tests を追加

`mod v84100_tests` の直後に `#[cfg(test)] mod v84200_tests` を追加する。

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

### Step 4: cargo test で全 pass 確認

`cargo test 2>&1 | grep "test result"` を実行し、3,913 tests, 0 failures を確認する。

### Step 5: CHANGELOG 更新

`CHANGELOG.md` の先頭に v84.2.0 エントリを追加する。

> 注意: `v84200_tests` には `changelog_has_v84_2_0` テストが含まれないため、
> CHANGELOG 更新は Step 4 の後でよい。

### Step 6: CI 事前確認

- `cargo clippy --locked -- -D warnings` が pass することを確認する
- `./target/debug/fav fmt --check self/compiler.fav` が pass することを確認する
- `./target/debug/fav fmt --check self/checker.fav` が pass することを確認する
