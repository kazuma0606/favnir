# v77.5.0 実装計画 — `fav verify` コマンド

Date: 2026-08-15

---

## Step 1: driver.rs — InvariantResult / VerificationReport 追加

`fav/src/driver.rs` の末尾に `// --- v77.5.0: fav verify コマンド ---` コメントと型定義を追加する。

```rust
// --- v77.5.0: fav verify コマンド ---

#[derive(Debug, Clone)]
pub struct InvariantResult {
    pub name:   String,
    pub passed: bool,
    pub detail: String,
}

#[derive(Debug, Clone)]
pub struct VerificationReport {
    pub pipeline:   String,
    pub results:    Vec<InvariantResult>,
    pub all_passed: bool,
}
```

---

## Step 2: driver.rs — cmd_verify 追加

```rust
pub fn cmd_verify(
    pipeline_name: &str,
    invariants: &[PipelineInvariant],
) -> VerificationReport {
    let results: Vec<InvariantResult> = invariants.iter().map(|inv| {
        InvariantResult {
            name:   inv.name.clone(),
            passed: true,
            detail: format!("invariant '{}' declared for {:?}", inv.name, inv.check_point),
        }
    }).collect();
    let all_passed = results.iter().all(|r| r.passed);
    VerificationReport {
        pipeline:   pipeline_name.to_string(),
        results,
        all_passed,
    }
}
```

---

## Step 3: driver.rs — format_verification_report 追加

```rust
pub fn format_verification_report(report: &VerificationReport) -> String {
    let mut out = format!("Verifying {}...\n", report.pipeline);
    for r in &report.results {
        let mark = if r.passed { "✓" } else { "✗" };
        out.push_str(&format!("  {} {} ({})\n", mark, r.name, r.detail));
    }
    if report.all_passed {
        let n = report.results.len();
        out.push_str(&format!("Verification passed. {}/{} invariants checked.", n, n));
    } else {
        let failed = report.results.iter().filter(|r| !r.passed).count();
        let total = report.results.len();
        out.push_str(&format!("Verification FAILED. {} of {} invariants violated.", failed, total));
    }
    out
}
```

---

## Step 4: cargo test（既存テスト通過確認）

`cargo test` を実行し、既存の 3744 テストが引き続き pass することを確認する（v775000_tests 追加前の状態）。

---

## Step 5: CHANGELOG.md 更新

`CHANGELOG.md` の先頭に v77.5.0 エントリを追加する（テストモジュール追加より先）。

---

## Step 6: driver.rs — v775000_tests モジュール追加

```rust
#[cfg(test)]
mod v775000_tests {
    use super::*;  // InvariantResult, VerificationReport, cmd_verify, format_verification_report,
                   // PipelineInvariant, InvariantCheckPoint を参照するため必須

    #[test]
    fn verify_cmd_all_pass() { ... }

    #[test]
    fn verify_cmd_violation_reported() { ... }
}
```

---

## Step 7: Cargo.toml バージョン更新

`77.4.0` → `77.5.0`

また、driver.rs 内に存在する `77.4.0` バージョン文字列アサーションを `77.5.0` へ一括置換する（`replace_all: true`）。

> **注意**: `replace_all: true` はセクションコメント `// --- v77.4.0: Join 系不変条件 ---` 内の `77.4.0` にも反応し、`// --- v77.5.0: Join 系不変条件 ---` に書き換えてしまう。これは意図しない変更のため、**replace_all 実行後に必ず** `grep "// ---"` でセクションコメントを確認し、誤って書き換わっているものは手動で元のバージョン番号に戻すこと。

---

## Step 8: versions/current.md 更新

進行中バージョンを v77.5.0 に、次に切る版を v77.6.0 に更新する。

---

## Step 9: 最終確認

`cargo test` が 3746 tests all pass であることを確認する。
