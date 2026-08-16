# v77.5.0 仕様書 — `fav verify` コマンド

Date: 2026-08-15
Status: 計画中

---

## Background

コントラクトの不変条件をサンプルデータに対して検証するコマンド基盤を提供する。`InvariantResult` 構造体・`VerificationReport` 構造体・`cmd_verify` 関数・`format_verification_report` 関数を追加する。v77.1.0〜v77.4.0 の型基盤の上に立つ「報告層」として機能し、将来の `fav verify` CLI への入口となる。

---

## Goals

1. `InvariantResult` 構造体（name: String, passed: bool, detail: String）を追加する
2. `VerificationReport` 構造体（pipeline: String, results: Vec<InvariantResult>, all_passed: bool）を追加する
3. `cmd_verify(pipeline_name: &str, invariants: &[PipelineInvariant]) -> VerificationReport` を追加する
4. `format_verification_report(report: &VerificationReport) -> String` を追加する
5. Rust テスト 2 件を追加し 3746 tests に到達する

---

## 型・関数仕様

### `InvariantResult` 構造体

```rust
#[derive(Debug, Clone)]
pub struct InvariantResult {
    pub name:   String,
    pub passed: bool,
    pub detail: String,
}
```

---

### `VerificationReport` 構造体

```rust
#[derive(Debug, Clone)]
pub struct VerificationReport {
    pub pipeline:   String,
    pub results:    Vec<InvariantResult>,
    pub all_passed: bool,
}
```

---

### `cmd_verify`

```rust
pub fn cmd_verify(
    pipeline_name: &str,
    invariants: &[PipelineInvariant],
) -> VerificationReport
```

**動作:**
- 各 `PipelineInvariant` に対して `InvariantResult { name: inv.name.clone(), passed: true, detail: format!("invariant '{}' declared for {:?}", inv.name, inv.check_point) }` を生成する
- `all_passed = results.iter().all(|r| r.passed)`
- `VerificationReport { pipeline: pipeline_name.to_string(), results, all_passed }` を返す

> **設計注記**: v77.5.0 では `cmd_verify` は PipelineInvariant の宣言情報から報告を生成する（実データの実行は将来の CLI 統合で行う）。違反シナリオのテストは `VerificationReport` を直接構築して `format_verification_report` を検証する。

---

### `format_verification_report`

```rust
pub fn format_verification_report(report: &VerificationReport) -> String
```

**出力フォーマット:**
```
Verifying <pipeline>...
  ✓ <name> (<detail>)
  ✗ <name> (<detail>)
Verification passed. N/N invariants checked.
```
または
```
Verification FAILED. M of N invariants violated.
```

**動作:**
- 1 行目: `"Verifying {pipeline_name}...\n"`
- 各 InvariantResult: passed なら `"  ✓ {name} ({detail})\n"`、failed なら `"  ✗ {name} ({detail})\n"`
- all_passed=true → 末尾に `"Verification passed. N/N invariants checked."`
- all_passed=false → 末尾に `"Verification FAILED. M of N invariants violated."`（M = 失敗件数）

---

## テスト仕様

### `verify_cmd_all_pass`

```rust
// Rust テスト（driver.rs 内）
let invariants = vec![
    PipelineInvariant {
        name:        "filter_reduces_rows".to_string(),
        expression:  "output.row_count < input.row_count".to_string(),
        check_point: InvariantCheckPoint::Post,
    },
    PipelineInvariant {
        name:        "total_amount_non_negative".to_string(),
        expression:  "SUM(output.amount) >= 0.0".to_string(),
        check_point: InvariantCheckPoint::Post,
    },
];
let report = cmd_verify("OrderPipeline", &invariants);
assert!(report.all_passed);
assert_eq!(report.pipeline, "OrderPipeline");
assert_eq!(report.results.len(), 2);

// format_verification_report が "Verifying OrderPipeline" と "passed" を含む
let formatted = format_verification_report(&report);
assert!(formatted.contains("Verifying OrderPipeline"));
assert!(formatted.contains("passed"));
```

### `verify_cmd_violation_reported`

```rust
// Rust テスト（driver.rs 内）
// VerificationReport を直接構築して format_verification_report の違反フォーマットを検証
let report = VerificationReport {
    pipeline:   "TestPipeline".to_string(),
    results:    vec![
        InvariantResult {
            name:   "ok_inv".to_string(),
            passed: true,
            detail: "ok".to_string(),
        },
        InvariantResult {
            name:   "fail_inv".to_string(),
            passed: false,
            detail: "row_count violated: expected <= 100, actual 150".to_string(),
        },
    ],
    all_passed: false,
};
assert!(!report.all_passed);
let formatted = format_verification_report(&report);
assert!(formatted.contains("FAILED"));
assert!(formatted.contains("fail_inv"));
```

---

## Success Criteria

- `InvariantResult` / `VerificationReport` 構造体が定義されている（Debug / Clone 付き）
- `cmd_verify` が各 PipelineInvariant から InvariantResult を生成し VerificationReport を返す
- `cmd_verify` が生成する各 `InvariantResult` の `passed` は常に `true`（v77.5.0 では実データ検証は行わない。違反シナリオは `VerificationReport` を直接構築してテストする）
- `format_verification_report` が passed 時に "passed"、failed 時に "FAILED" を含む文字列を返す
- `verify_cmd_all_pass` が pass
- `verify_cmd_violation_reported` が pass
- `cargo test` が 3746 tests all pass
- `driver.rs` 内の `77.4.0` バージョン文字列アサーションがすべて `77.5.0` に更新されている
- `CHANGELOG.md` の先頭に v77.5.0 エントリが存在する

---

## 変更ファイル

実装順序は plan.md 参照。

- `fav/src/driver.rs` — `InvariantResult`, `VerificationReport`, `cmd_verify`, `format_verification_report`, `v775000_tests` を追加
- `CHANGELOG.md` — v77.5.0 エントリを追加
- `versions/current.md` — 進行中バージョンを更新
- `fav/Cargo.toml` — バージョンを `77.4.0` → `77.5.0` に更新

---

## 依存

- v77.1.0 の `PipelineInvariant` / `InvariantCheckPoint` 構造体を再利用（`fav/src/driver.rs` 内 `// --- v77.1.0: PipelineInvariant 型基盤 ---` ブロック参照）
- `InvariantViolation`（v77.1.0 定義済み）は v77.5.0 では直接使用しない。`InvariantResult` が `passed: bool` と `detail: String` で違反情報を表現する

---

## 対象外

- ロードマップの `$ fav verify pipeline.fav --data data/sample.csv` CLI は将来の統合（v78.x 以降）。v77.5.0 では `driver.rs` への型・関数追加のみ
- `parser.rs` / `ast.rs` / `checker.rs` への変更は一切行わない
- 実データのロード・CSV パース（将来の CLI 統合で対応）
