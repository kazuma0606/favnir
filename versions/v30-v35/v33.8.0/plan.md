# v33.8.0 — 実装プラン

## 方針

確認・記録パターン。v19.8.0 実装済みのプロファイリング強化（`parse_profile_json` / `to_folded_stacks`）を 4 テストで確認する。

---

## 実装ステップ

### Step 1: Cargo.toml バージョン更新
`fav/Cargo.toml` の version を `33.7.0` → `33.8.0` に変更。

### Step 2: benchmarks/v33.8.0.json 作成
```json
{
  "version": "33.8.0",
  "milestone": "Performance & Tooling",
  "date": "2026-07-04",
  "tests_passed": 2528,
  "tests_failed": 0,
  "notes": "プロファイリング強化確認（parse_profile_json / to_folded_stacks）。v338000_tests 4件追加。"
}
```
（`tests_passed` は `cargo test` 実測後に確定）

### Step 3: driver.rs 更新
1. `cargo_toml_version_is_33_7_0` を空スタブ化
2. `v337000_tests` 直後・`// ── v31.7.0 tests` の前に `v338000_tests` を挿入

```rust
// ── v33.8.0 tests ────────────────────────────────────────────────────────────
#[cfg(test)]
#[cfg(not(target_arch = "wasm32"))]
mod v338000_tests {
    use crate::profiler::collector::{StageRecord, parse_profile_json, to_folded_stacks};

    #[test]
    fn cargo_toml_version_is_33_8_0() {
        let src = include_str!("../Cargo.toml");
        assert!(src.contains("33.8.0"), "Cargo.toml must contain '33.8.0'");
    }

    #[test]
    fn benchmark_v33_8_0_exists() {
        let src = include_str!("../../benchmarks/v33.8.0.json");
        assert!(src.contains("33.8.0"), "benchmarks/v33.8.0.json must contain '33.8.0'");
    }

    #[test]
    fn profile_parse_json_valid_records() {
        // JSON キーは "name" / "ms"（StageRecord.name / elapsed_ms に対応）
        let json = r#"[{"name":"Load","ms":10},{"name":"Transform","ms":25}]"#;
        let records = parse_profile_json(json); // Returns Vec<StageRecord> directly
        assert_eq!(records.len(), 2, "expected 2 StageRecords");
        assert_eq!(records[0].name, "Load");
        assert_eq!(records[0].elapsed_ms, 10);
        assert_eq!(records[1].name, "Transform");
        assert_eq!(records[1].elapsed_ms, 25);
    }

    #[test]
    fn profile_folded_stacks_has_pipeline_prefix() {
        let records = vec![
            StageRecord { name: "Load".to_string(), elapsed_ms: 10 },
            StageRecord { name: "Transform".to_string(), elapsed_ms: 25 },
        ];
        let folded = to_folded_stacks(&records); // Returns Vec<String>
        assert!(
            folded.iter().all(|line| line.starts_with("pipeline;")),
            "all folded stack entries must start with 'pipeline;'"
        );
    }
}
```

### Step 4: CHANGELOG.md 更新
先頭に `[v33.8.0]` セクションを追加。

### Step 5: versions/current.md 更新
最新安定版を v33.8.0 に変更。

---

## テスト実行

```bash
cd /c/Users/yoshi/favnir/fav && cargo test --bin fav v338000 2>&1 | tail -8
cd /c/Users/yoshi/favnir/fav && cargo test -j 8 -- --test-threads=8 2>&1 | tail -20
```

---

## 完了処理

- `benchmarks/v33.8.0.json` の `tests_passed` を実測値で確定
- `tasks.md` を COMPLETE に更新（全チェックボックス `[x]`）
