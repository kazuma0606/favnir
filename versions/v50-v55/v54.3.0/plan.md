# Plan: v54.3.0 — パフォーマンスリグレッションスイート CI 統合

---

## ステップ 1: 事前確認

```bash
cargo test -j 8 -- --test-threads=8 2>&1 | grep "test result"
# → 3189 passed, 0 failed を確認

cargo clippy -- -D warnings
# → warnings なしであることを確認

# v54300_tests が未存在を確認
rg -n "v54300_tests" fav/src/driver.rs  # → 0 件

# v54200_tests の行番号を確認（挿入位置）
rg -n "v54200_tests" fav/src/driver.rs  # → 行番号を特定

# Cargo.toml が 54.2.0 であることを確認
grep "^version" fav/Cargo.toml  # → version = "54.2.0"

# benchmarks/baseline.json が未存在を確認
ls benchmarks/baseline.json  # → No such file

# bench.yml に --fail-on-regression が未存在を確認
grep "fail-on-regression" .github/workflows/bench.yml  # → 0 件

# fav bench --all が未実装を確認
grep '"--all"' fav/src/main.rs | grep -A2 "bench"  # → 0 件
```

---

## ステップ 2: `benchmarks/baseline.json` 新規作成

```json
{
  "version": "54.3.0",
  "date": "2026-07-22",
  "milestone": "Production 3.0 Sprint",
  "tests_passed": 3191,
  "tests_failed": 0,
  "metrics": {
    "checker_ms": 12,
    "compiler_ms": 8,
    "total_pipeline_ms": 25
  },
  "regression": false,
  "notes": "v54.3.0 CI リグレッションスイート統合後の baseline。PR ごとに fav bench --compare benchmarks/baseline.json --fail-on-regression で自動比較される。metrics の値はプレースホルダー — 初回 CI 実行後に fav bench --json の実測値で更新すること（CONTRIBUTING.md 参照）。"
}
```

必須フィールド: `"version"`, `"metrics"`, `"regression"`

---

## ステップ 3: `.github/workflows/bench.yml` 拡張

### 3a: `continue-on-error: true` をジョブ全体からステップ限定に移動

**削除**: job レベルの `continue-on-error: true`

**追加**: `Run benchmarks` ステップに `continue-on-error: true` を移動

理由: job レベルに残すと `--fail-on-regression` の exit 1 が CI 全体で無視されてしまう。

### 3b: 末尾に 2 ステップを追加

```yaml
# ── v54.3.0: パフォーマンスリグレッションスイート CI 統合 ────────────────
- name: Run perf regression unit tests
  run: cargo test bench_ -- --nocapture
  working-directory: fav

- name: Regression check against baseline
  env:
    FAV: ./fav/target/release/fav
  run: |
    $FAV bench --all --compare benchmarks/baseline.json --fail-on-regression || exit 1
```

---

## ステップ 4: `main.rs` — `fav bench --all` フラグ追加

bench コマンドの match アームに `"--fail-on-regression"` の直後に追加:

```rust
"--all" => {
    // v54.3.0: --all は file 省略と等価（プロジェクト全体を走査）
    i += 1;
}
```

`cargo build` → コンパイルエラーなし確認。

---

## ステップ 5: `driver.rs` — `v54300_tests` 追加

`v54200_tests` の直前に追加:

```rust
// -- v54300_tests (v54.3.0) -- パフォーマンスリグレッションスイート CI 統合 --
#[cfg(test)]
mod v54300_tests {
    #[test]
    fn ci_perf_regression_suite() {
        let bench_yml = include_str!("../../.github/workflows/bench.yml");
        assert!(bench_yml.contains("--fail-on-regression"), ...);
        assert!(bench_yml.contains("baseline.json"), ...);
        assert!(bench_yml.contains("cargo test bench_"), ...);
        assert!(bench_yml.contains("--all"), ...);
    }

    #[test]
    fn ci_perf_baseline_recorded() {
        let baseline = include_str!("../../benchmarks/baseline.json");
        assert!(baseline.contains("\"version\""), ...);
        assert!(baseline.contains("\"metrics\""), ...);
        assert!(baseline.contains("\"regression\""), ...);
    }
}
```

---

## ステップ 6: `fav/Cargo.toml` バージョン更新

`version = "54.2.0"` → `version = "54.3.0"`

---

## ステップ 7: テスト実行・確認

```bash
cd /c/Users/yoshi/favnir/fav && cargo test -j 8 -- --test-threads=8 2>&1 | grep "test result"
```

期待値: 3191 passed, 0 failed

```bash
cargo clippy -- -D warnings
```

---

## ステップ 8: 後処理

- `CHANGELOG.md`: v54.3.0 エントリ追加（v54.2.0 の直上）
- `versions/current.md` を v54.3.0（3191 tests）に更新
- `roadmap-v54.1-v55.0.md` の v54.3.0 実績欄を COMPLETE に更新
- `tasks.md` を COMPLETE に更新（T0〜T8 全 `[x]`）

コードレビュー対応（実施済み）:
- [HIGH] `--all` フラグ未実装 → main.rs に `"--all" => { i += 1; }` 追加
- [MED] `continue-on-error` ジョブ全体 → `Run benchmarks` ステップ限定に移動
- [MED] baseline.json notes に実測値更新手順を明記
- [LOW] bench.yml 末尾改行追加
- [LOW] テストに `bench_yml.contains("--all")` アサーションを追加
