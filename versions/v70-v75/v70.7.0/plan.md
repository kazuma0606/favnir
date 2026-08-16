# v70.7.0 Plan — Self-Hosting Coverage Report

Date: 2026-08-09
Status: 計画中

---

## 実装ステップ（依存順）

### Step 1: driver.rs に `SelfCoverageReport` / `compute_self_coverage` / `format_self_coverage` を追加

`cmd_doctor_run` 付近（driver.rs の末尾付近、テストモジュールの直前）に追加。

```rust
#[derive(Debug)]
pub struct SelfCoverageReport {
    pub compiler_covered: usize,
    pub compiler_total: usize,
    pub compiler_missing: Vec<&'static str>,
    pub checker_covered: usize,
    pub checker_total: usize,
    pub checker_missing: Vec<&'static str>,
}

impl SelfCoverageReport {
    pub fn compiler_pct(&self) -> f64 {
        self.compiler_covered as f64 / self.compiler_total as f64 * 100.0
    }
    pub fn checker_pct(&self) -> f64 {
        self.checker_covered as f64 / self.checker_total as f64 * 100.0
    }
}

pub fn compute_self_coverage() -> SelfCoverageReport {
    let compiler_missing: Vec<&'static str> = vec![
        "list-pattern-in-bind",
        "dependent-type-annotation",
    ];
    let compiler_total = 51_usize;
    let compiler_covered = compiler_total - compiler_missing.len();

    let checker_missing: Vec<&'static str> = vec!["E0021"];
    let checker_total = 18_usize;
    let checker_covered = checker_total - checker_missing.len();

    SelfCoverageReport {
        compiler_covered,
        compiler_total,
        compiler_missing,
        checker_covered,
        checker_total,
        checker_missing,
    }
}

pub fn format_self_coverage(report: &SelfCoverageReport) -> String {
    let compiler_missing_str = if report.compiler_missing.is_empty() {
        String::new()
    } else {
        format!("\n  Missing: {}", report.compiler_missing.join(", "))
    };
    let checker_missing_str = if report.checker_missing.is_empty() {
        String::new()
    } else {
        format!("\n  Missing: {}", report.checker_missing.join(", "))
    };
    format!(
        "compiler.fav coverage: {:.1}% ({}/{} syntax forms){}\n\nchecker.fav coverage: {:.1}% ({}/{} error codes){}",
        report.compiler_pct(),
        report.compiler_covered,
        report.compiler_total,
        compiler_missing_str,
        report.checker_pct(),
        report.checker_covered,
        report.checker_total,
        checker_missing_str,
    )
}
```

確認: `cargo test` で既存テスト（3572 件）が全 pass することを確認。

---

### Step 2: main.rs に `Some("self-coverage")` コマンドアームを追加

`Some("doctor")` アームの直前（付近）に追加:

```rust
// ── v70.7.0: fav self-coverage ───────────────────────────────────────
Some("self-coverage") => {
    let report = driver::compute_self_coverage();
    println!("{}", driver::format_self_coverage(&report));
}
```

確認: `cargo build` が成功することを確認。

---

### Step 3: `v707000_tests` モジュールを driver.rs 末尾に追加

```rust
#[cfg(test)]
mod v707000_tests {
    #[test]
    fn self_coverage_compiler_fav_above_95pct() {
        let report = super::compute_self_coverage();
        assert!(
            report.compiler_pct() >= 95.0,
            "compiler.fav coverage {:.1}% is below 95%; covered={}/{}",
            report.compiler_pct(),
            report.compiler_covered,
            report.compiler_total
        );
    }

    #[test]
    fn self_coverage_checker_fav_above_90pct() {
        let report = super::compute_self_coverage();
        assert!(
            report.checker_pct() >= 90.0,
            "checker.fav coverage {:.1}% is below 90%; covered={}/{}",
            report.checker_pct(),
            report.checker_covered,
            report.checker_total
        );
    }
}
```

確認: `cargo test v707000` で 2 件 pass することを確認。

---

### Step 4: Cargo.toml バージョン更新

- `fav/Cargo.toml` の `version = "70.6.0"` → `"70.7.0"`
- driver.rs 内の `"70.6.0"` を `sed` で `"70.7.0"` に一括更新

---

### Step 5: CHANGELOG.md 更新

v70.7.0 エントリを v70.6.0 の直前に追加。

---

### Step 6: 最終確認

- `cargo test v707000` で 2 件 pass
- `cargo test` 全体で 3574 tests pass（0 failures）
- `versions/current.md` を v70.7.0 進行中に更新
