# Plan: v80.9.0 — 安定化・コードフリーズ

## Step 1: 前提確認

- `cargo test` を実行し、3835 tests, 0 failures を確認する
- `fav/src/test_framework.rs` に v80.8.0 の `TestReport` / `format_junit_xml` / `format_test_summary` が定義済みであることを確認する
- `fav/src/driver.rs` に `mod v80800_tests` が存在することを確認する

## Step 2: `fav/src/driver.rs` に統合テスト追加

`mod v80800_tests { ... }` の直後に `#[cfg(test)] mod v80900_tests { ... }` を追加する。

### test_framework_full_sprint_all_stable

```rust
#[test]
fn test_framework_full_sprint_all_stable() {
    use fav_core::test_framework::*;
    // v80.1: TestSuite / TestCase / run_test_suite
    let suite = TestSuite {
        name: "stable_check".to_string(),
        cases: vec![TestCase {
            name: "noop".to_string(),
            status: TestStatus::Pass,
            message: None,
        }],
    };
    let _ = run_test_suite(&suite);

    // v80.3: DataFactory / TestFixture
    let factory = DataFactory::from_seed(42);
    let fixture = TestFixture {
        name: "f".to_string(),
        schema: vec!["col".to_string()],
        rows: vec![vec![("col".to_string(), FieldSpec::Int(1))]],
    };
    let _ = factory.generate_rows(&fixture, 1);

    // v80.4: PropertyTest
    // v80.4.0 実装では PropertyTest のフィールド名は `kind: InvariantKind`
    let pt = PropertyTest {
        name: "nn".to_string(),
        kind: InvariantKind::NonNegative,
        samples: 1,
    };
    let _ = run_property_test(&pt, &[1.0]);

    // v80.5: StageTestCase / run_stage_test
    let input = StageInput { name: "in".to_string(), rows: vec![] };
    let expected = StageOutput { name: "out".to_string(), rows: vec![] };
    let stc = StageTestCase {
        stage_name: "s".to_string(),
        input,
        expected: expected.clone(),
    };
    let _ = run_stage_test(&stc, &expected);

    // v80.6: compute_test_coverage
    let _ = compute_test_coverage(&suite, &[]);

    // v80.7: SchemaSnapshot / compare_schema_snapshots
    let snap = SchemaSnapshot {
        pipeline_name: "p".to_string(),
        columns: vec![],
    };
    let _ = compare_schema_snapshots(&snap, &snap);

    // v80.8: TestReport / format_junit_xml / format_test_summary
    let report = TestReport {
        suite,
        duration_ms: 1,
        timestamp: "2026-08-19T00:00:00Z".to_string(),
    };
    let _ = format_junit_xml(&report);
    let _ = format_test_summary(&report);
}
```

### test_framework_e2e_pipeline_tested

```rust
#[test]
fn test_framework_e2e_pipeline_tested() {
    use fav_core::test_framework::*;
    let factory = DataFactory::from_seed(1);
    let fixture = TestFixture {
        name: "pipe_fixture".to_string(),
        schema: vec!["id".to_string()],
        rows: vec![vec![("id".to_string(), FieldSpec::Int(1))]],
    };
    let rows = factory.generate_rows(&fixture, 1);
    let actual = StageOutput { name: "out".to_string(), rows };
    let stc = StageTestCase {
        stage_name: "pipeline_tests".to_string(),
        input: StageInput { name: "in".to_string(), rows: vec![] },
        expected: actual.clone(),
    };
    let result = run_stage_test(&stc, &actual);
    let suite = TestSuite {
        name: "pipeline_tests".to_string(),
        cases: vec![result],
    };
    let report = TestReport {
        suite,
        duration_ms: 10,
        timestamp: "2026-08-19T00:00:00Z".to_string(),
    };
    let summary = format_test_summary(&report);
    assert!(summary.contains("pipeline_tests"));
}
```

## Step 3: `cargo test` で全 pass 確認

```
cargo test 2>&1 | tail -5
# 期待: 3837 tests, 0 failures
```

## Step 4: CHANGELOG 更新

`CHANGELOG.md` の先頭に v80.9.0 エントリを追加する。

## Step 5: CI 事前確認

以下はすべて `fav/` ディレクトリで実行する（`cargo clippy` も `fav/` で実行）。

```
# fav/ ディレクトリで実行
cargo clippy --locked -- -D warnings
./target/debug/fav fmt --check self/compiler.fav
./target/debug/fav fmt --check self/checker.fav
```
