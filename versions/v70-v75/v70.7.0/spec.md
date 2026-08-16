# v70.7.0 Spec — Self-Hosting Coverage Report

Date: 2026-08-09
Status: 計画中

---

## Background

v70.5.0・v70.6.0 で compiler.fav の Or-パターン対応（TkIf guard）・Record 分割束縛（TkLBrace）を追加した。
これらの変更がセルフホスティング網羅率に与えた影響を定量化する機能として `fav self-coverage` コマンドを実装する。

### 計測対象

| カテゴリ | 定義 | 計測方法 |
|---|---|---|
| compiler.fav syntax forms | Favnir が持つ構文形式（51 種） | 静的リストと compiler.fav の実装有無を照合 |
| checker.fav error codes | checker.fav が実装すべき基本エラーコード（18 種） | checker.fav の emit 一覧と対照 |

### 現状（実装前）

- compiler.fav: **49/51** 構文形式対応（96.1%）
  - Missing: `list-pattern-in-bind`（compiler.fav の TkLBracket 未対応）、`dependent-type-annotation`
- checker.fav: **17/18** エラーコード対応（94.4%）
  - Missing: `E0021`（context interface error）

---

## Goals

1. `compute_self_coverage()` 関数を driver.rs に追加（`SelfCoverageReport` 返却）
2. `format_self_coverage(report: &SelfCoverageReport) -> String` を driver.rs に追加
3. `fav self-coverage` コマンドを main.rs に登録
4. `v707000_tests` モジュールを driver.rs 末尾に追加（2 テスト）
5. テスト 2 件追加 → 3574 tests

---

## Syntax / API Examples

```bash
$ fav self-coverage
compiler.fav coverage: 96.1% (49/51 syntax forms)
  Missing: list-pattern-in-bind, dependent-type-annotation

checker.fav coverage: 94.4% (17/18 error codes)
  Missing: E0021
```

---

## Implementation Details

### `SelfCoverageReport` 構造体

```rust
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
```

### `compute_self_coverage()` の定義

compiler.fav の 51 構文形式（静的リスト）と missing 2 件を hardcode。
checker.fav の 18 エラーコード（基本セット）と missing 1 件（E0021）を hardcode。

### `format_self_coverage()` の出力形式

```
compiler.fav coverage: {pct:.1}% ({covered}/{total} syntax forms)
  Missing: {missing...}

checker.fav coverage: {pct:.1}% ({covered}/{total} error codes)
  Missing: {missing...}
```

---

## Success Criteria

- [ ] `self_coverage_compiler_fav_above_95pct`: compiler_pct() >= 95.0
- [ ] `self_coverage_checker_fav_above_90pct`: checker_pct() >= 90.0
- [ ] `fav self-coverage` がパニックせず出力する
- [ ] `cargo test v707000` で 2 件 pass
- [ ] `cargo test` 全体で 3574 tests pass（0 failures）

---

## Error Codes

新規エラーコードなし

---

## Files to Modify

| ファイル | 変更内容 |
|---|---|
| `fav/src/driver.rs` | `SelfCoverageReport` + `compute_self_coverage` + `format_self_coverage` + `v707000_tests` |
| `fav/src/main.rs` | `Some("self-coverage")` コマンドアーム追加 |
| `fav/Cargo.toml` | `version` を `"70.6.0"` → `"70.7.0"` |
| `CHANGELOG.md` | v70.7.0 エントリ追加 |
| `versions/current.md` | 進行中バージョンを v70.7.0 に更新 |
