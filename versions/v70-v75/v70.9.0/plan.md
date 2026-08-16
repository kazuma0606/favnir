# v70.9.0 Plan — 安定化・コードフリーズ

Date: 2026-08-09
Status: 計画中

---

## 実装ステップ（依存順）

### Step 1: bench.yml の Compare ステップを strict mode に変更

`.github/workflows/bench.yml` の `Compare with baseline` ステップ:

```yaml
# 変更前（line 42-46 付近）
      - name: Compare with baseline
        env:
          FAV: ./fav/target/release/fav
        run: |
          $FAV run benchmarks/compare.fav \
            -- --baseline benchmarks/v24.2.0.json \
               --current  benchmarks/latest.json \
               --threshold 5 \
               --emit-md || true

# 変更後
      - name: Compare with baseline
        env:
          FAV: ./fav/target/release/fav
        run: |
          $FAV run benchmarks/compare.fav \
            -- --baseline benchmarks/v24.2.0.json \
               --current  benchmarks/latest.json \
               --threshold 5 \
               --emit-md
```

---

### Step 2: driver.rs に `v709000_tests` モジュールを追加

`v708000_tests` の直後に追加:

```rust
#[cfg(test)]
mod v709000_tests {
    #[test]
    fn language_complete_all_stable() {
        let src = include_str!("driver.rs");
        let required = [
            "backlog_compiler_fav_ctx_multiparams",   // v70.1
            "migrate_effect_annotation_to_ctx",        // v70.2
            "bench_subcommand_all_outputs_json",       // v70.3
            "diagnostic_e0374_shows_migration_hint",   // v70.4
            "pattern_match_if_guard",                  // v70.5
            "bind_destructure_record",                 // v70.6
            "self_coverage_compiler_fav_above_95pct",  // v70.7
            "doctor_detects_paper_rune",               // v70.8
        ];
        for name in &required {
            assert!(src.contains(name), "missing test: {name}");
        }
    }

    #[test]
    fn bench_ci_no_continue_on_error() {
        let yml = include_str!("../../.github/workflows/bench.yml");
        let compare_block = yml
            .split("Compare with baseline")
            .nth(1)
            .expect("Compare ステップが bench.yml に存在しない");
        let compare_step = compare_block
            .split("- name:")
            .next()
            .unwrap_or(compare_block);
        assert!(
            !compare_step.contains("|| true"),
            "Compare ステップに || true が残存している"
        );
    }
}
```

確認: `cargo test v709000` で 2 件 pass することを確認。

---

### Step 3: Cargo.toml バージョン更新

- `fav/Cargo.toml` の `version = "70.8.0"` → `"70.9.0"`
- driver.rs 内の `cargo_toml_version_is_70_8_0` 関数本体内のバージョン文字列リテラル `"70.8.0"` を `"70.9.0"` に書き換える（関数名自体はリネームしない）

---

### Step 4: CHANGELOG.md 更新

v70.9.0 エントリを v70.8.0 の直前に追加。

---

### Step 5: 最終確認

- `cargo test v709000` で 2 件 pass
- `cargo test` 全体で 3580 tests pass（0 failures）
- `versions/current.md` を v70.9.0 に更新
