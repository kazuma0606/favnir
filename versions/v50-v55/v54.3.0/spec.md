# Spec: v54.3.0 — パフォーマンスリグレッションスイート CI 統合

Status: COMPLETE
Date: 2026-07-22

---

## 概要

既存の `.github/workflows/bench.yml` に `fav bench --fail-on-regression` ステップを追加し、
`benchmarks/baseline.json` をリポジトリ管理することで PR ごとの自動リグレッション比較を実現する。
新ワークフローを新設するのではなく、既存の `bench` ジョブを拡張する。

`fav bench --all` フラグ（`--fail-on-regression` と組み合わせて使用）を `main.rs` に追加する。

---

## 実装スコープ

### 1. `benchmarks/baseline.json` 新規作成

PR ごとの比較基準値としてリポジトリ管理する:

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
  "notes": "... 初回 CI 実行後に fav bench --json 実測値で更新すること（CONTRIBUTING.md 参照）。"
}
```

必須フィールド: `"version"`, `"metrics"`, `"regression"`（`cmd_bench_compare` が参照）

### 2. `.github/workflows/bench.yml` — 拡張

2a. `continue-on-error: true` をジョブ全体（job レベル）から `Run benchmarks` ステップのみに移動:

```yaml
- name: Run benchmarks
  continue-on-error: true   # libduckdb-sys v1.x フラグバグのため暫定
  ...
```

2b. 末尾に 2 ステップを追加:

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

設計上の注意: `continue-on-error: true` をジョブ全体に残すと `--fail-on-regression` が
CI を落とせなくなる（リグレッション検出が無効化）。ステップ限定に移動することで解消する。

### 3. `main.rs` — `fav bench --all` フラグ追加

bench コマンドの `match` アームに追加:

```rust
"--all" => {
    // v54.3.0: --all は file 省略と等価（プロジェクト全体を走査）
    i += 1;
}
```

`file` を省略した場合と `--all` は機能的に同一（`cmd_bench` がプロジェクト全体を走査）。
`--all` を unknown とした場合 `opts.file = Some("--all")` になりコマンドエラーとなるため必須。

### 4. `driver.rs` — `v54300_tests` 追加

`v54200_tests` の直前に追加（2 テスト）:

```rust
mod v54300_tests {
    fn ci_perf_regression_suite() {
        // bench.yml に --fail-on-regression / baseline.json / cargo test bench_ / --all が含まれること
    }
    fn ci_perf_baseline_recorded() {
        // baseline.json に "version" / "metrics" / "regression" フィールドが含まれること
    }
}
```

---

## テスト仕様

| テスト名 | 検証内容 |
|---|---|
| `ci_perf_regression_suite` | bench.yml が `--fail-on-regression` / `baseline.json` / `cargo test bench_` / `--all` を含む |
| `ci_perf_baseline_recorded` | `benchmarks/baseline.json` が `"version"` / `"metrics"` / `"regression"` フィールドを含む |

パス確認:
- `include_str!("../../.github/workflows/bench.yml")` : `fav/src/` → `../../` = `favnir/.github/workflows/bench.yml` ✓
- `include_str!("../../benchmarks/baseline.json")` : `fav/src/` → `../../` = `favnir/benchmarks/baseline.json` ✓

---

## バージョン更新

- `fav/Cargo.toml`: `"54.2.0"` → `"54.3.0"`

---

## 完了条件

- `cargo test` 3191 passed, 0 failed（ベース 3189 + 2 件追加）
- `v54300_tests` 2 件 pass:
  - `ci_perf_regression_suite`
  - `ci_perf_baseline_recorded`
- `cargo clippy -- -D warnings` クリーン

---

## 影響範囲

| ファイル | 変更種別 |
|---|---|
| `benchmarks/baseline.json` | 新規作成 — CI 自動比較の基準値 |
| `.github/workflows/bench.yml` | `continue-on-error` スコープ修正 + 2 ステップ追加 |
| `fav/src/main.rs` | `fav bench --all` フラグ追加 |
| `fav/src/driver.rs` | `v54300_tests` 追加 |
| `fav/Cargo.toml` | version 更新 |
| `fav/Cargo.lock` | version 更新に伴い自動更新 |
| `CHANGELOG.md` | v54.3.0 エントリ追加 |
| `versions/current.md` | v54.3.0 / 3191 tests に更新 |
| `versions/roadmap/roadmap-v54.1-v55.0.md` | v54.3.0 実績欄を COMPLETE に更新 |

---

## 設計上の注意

- `cmd_bench_compare` は `baseline_json` と現在の実行結果（`bench_stats_to_compare_json` 出力）の
  `"metrics"` オブジェクトを比較する。`baseline.json` の `metrics` 値はプレースホルダーであり、
  初回 CI 実行後に `fav bench --json` の実測値で更新する必要がある（CONTRIBUTING.md 参照）。
- `fav bench --all` は `file` 省略と等価な no-op フラグとして実装。機能拡張は将来バージョン。
- `continue-on-error: true` は `Run benchmarks` ステップ専用（duckdb-sys Linux フラグバグ対策）。
  リグレッション検出ステップには適用しないことで、CI が確実にリグレッションを検出できる。
- `CONTRIBUTING.md` は本バージョンでは更新しない。baseline.json の notes に「CONTRIBUTING.md 参照」と
  記載してあるのは将来の更新への指示であり、v54.3.0 のスコープ外。影響範囲テーブルへの記載なし。
