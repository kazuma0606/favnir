# v77.6.0 実装計画 — 証明付き CI 統合

Date: 2026-08-16

---

## Step 1: driver.rs — CiVerificationConfig / CiResult 追加

`fav/src/driver.rs` の末尾に `// --- v77.6.0: 証明付き CI 統合 ---` コメントと型定義を追加する。

```rust
// --- v77.6.0: 証明付き CI 統合 ---

#[derive(Debug, Clone, PartialEq)]
pub struct CiVerificationConfig {
    pub pipeline:   String,
    pub fail_fast:  bool,
    pub data_path:  String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CiResult {
    pub passed:    bool,
    pub report:    VerificationReport,
    pub exit_code: i32,
}
```

---

## Step 2: driver.rs — run_ci_verification 追加

```rust
pub fn run_ci_verification(
    config: &CiVerificationConfig,
    invariants: &[PipelineInvariant],
) -> CiResult {
    // DESIGN: v77.6.0 では cmd_verify（v77.5.0 スタブ）を使用する。
    // fail_fast / data_path は将来の CLI 統合（v78.x 以降）で利用。
    let report = cmd_verify(&config.pipeline, invariants);
    let passed = report.all_passed;
    let exit_code = if passed { 0 } else { 1 };
    CiResult { passed, report, exit_code }
}
```

---

## Step 3: driver.rs — format_ci_result_summary 追加

```rust
pub fn format_ci_result_summary(result: &CiResult) -> String {
    if result.passed {
        format!("[CI] ✓ All invariants passed. Exit code: {}", result.exit_code)
    } else {
        let failed = result.report.results.iter().filter(|r| !r.passed).count();
        let total = result.report.results.len();
        format!(
            "[CI] ✗ Invariant violations detected. {}/{} failed. Exit code: {}",
            failed, total, result.exit_code
        )
    }
}
```

---

## Step 4: cargo test（既存テスト通過確認）

`cargo test` を実行し、既存の 3746 テストが引き続き pass することを確認する（v776000_tests 追加前の状態）。

---

## Step 5: CHANGELOG.md 更新

`CHANGELOG.md` の先頭に v77.6.0 エントリを追加する（テストモジュール追加より先）。

---

## Step 6: driver.rs — v776000_tests モジュール追加

```rust
#[cfg(test)]
mod v776000_tests {
    use super::*;

    #[test]
    fn ci_verification_passes() { ... }

    #[test]
    fn ci_verification_fails_on_violation() { ... }
}
```

---

## Step 7: Cargo.toml バージョン更新

`77.5.0` → `77.6.0`

また、driver.rs 内に存在する `77.5.0` バージョン文字列アサーションを `77.6.0` へ一括置換する（`replace_all: true`）。

> **注意**: `replace_all: true` はセクションコメント `// --- v77.5.0: fav verify コマンド ---` 内の `77.5.0` にも反応し、`// --- v77.6.0: fav verify コマンド ---` に書き換えてしまう。これは意図しない変更のため、**replace_all 実行後に必ず** `grep "// ---"` でセクションコメントを確認し、誤って書き換わっているものは手動で元のバージョン番号に戻すこと。

---

## Step 8: versions/current.md 更新

進行中バージョンを v77.6.0 に、次に切る版を v77.7.0 に更新する。

---

## Step 9: 最終確認

`cargo test` が 3748 tests all pass であることを確認する。
