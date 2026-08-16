# v77.6.0 仕様書 — 証明付き CI 統合

Date: 2026-08-16
Status: 計画中

---

## Background

`fav verify` コマンドの検証レポート（v77.5.0 で実装済み）を CI パイプライン（GitHub Actions 等）に統合するための型・関数基盤を提供する。`CiVerificationConfig` / `CiResult` 構造体と `run_ci_verification` / `format_ci_result_summary` 関数を追加し、不変条件の証明を CI ブロッカーとして機能させる土台を構築する。

---

## Goals

1. `CiVerificationConfig` 構造体（pipeline: String, fail_fast: bool, data_path: String）を追加する
2. `CiResult` 構造体（passed: bool, report: VerificationReport, exit_code: i32）を追加する
3. `run_ci_verification(config: &CiVerificationConfig, invariants: &[PipelineInvariant]) -> CiResult` を追加する
4. `format_ci_result_summary(result: &CiResult) -> String` を追加する
5. Rust テスト 2 件を追加し 3748 tests に到達する

---

## 型・関数仕様

### `CiVerificationConfig` 構造体

```rust
#[derive(Debug, Clone, PartialEq)]
pub struct CiVerificationConfig {
    pub pipeline:   String,
    pub fail_fast:  bool,
    pub data_path:  String,
}
```

> `PartialEq` を付与する（v77.1.0〜v77.5.0 の他の公開型と一貫した方針）。

| フィールド | 型 | 説明 |
|---|---|---|
| `pipeline` | String | 検証対象のパイプライン名 |
| `fail_fast` | bool | 最初の違反で即終了するか（将来の実データ評価で使用、v77.6.0 では構造上のみ） |
| `data_path` | String | サンプルデータのパス（将来の CLI 統合で使用、v77.6.0 では構造上のみ） |

---

### `CiResult` 構造体

```rust
#[derive(Debug, Clone, PartialEq)]
pub struct CiResult {
    pub passed:    bool,
    pub report:    VerificationReport,
    pub exit_code: i32,
}
```

> `PartialEq` を付与する（`VerificationReport` も `PartialEq` 付きのため `CiResult` も付与可能・一貫性のため必須）。

| フィールド | 型 | 説明 |
|---|---|---|
| `passed` | bool | 全不変条件が通過したか |
| `report` | VerificationReport | v77.5.0 の `VerificationReport`（詳細結果） |
| `exit_code` | i32 | CI 終了コード（passed=true → 0、passed=false → 1） |

---

### `run_ci_verification`

```rust
pub fn run_ci_verification(
    config: &CiVerificationConfig,
    invariants: &[PipelineInvariant],
) -> CiResult
```

**動作:**
- `cmd_verify(&config.pipeline, invariants)` を呼び出して `VerificationReport` を生成する
- `passed = report.all_passed`
- `exit_code = if passed { 0 } else { 1 }`
- `CiResult { passed, report, exit_code }` を返す

> **設計注記**: v77.6.0 では `cmd_verify` は v77.5.0 のスタブ実装（`passed=true` 固定）を使用する。`fail_fast` / `data_path` フィールドは将来の CLI 統合（v78.x 以降）で利用される。違反シナリオのテストは `CiResult` を直接構築して `format_ci_result_summary` を検証する。

---

### `format_ci_result_summary`

```rust
pub fn format_ci_result_summary(result: &CiResult) -> String
```

**出力フォーマット:**

passed=true の場合:
```
[CI] ✓ All invariants passed. Exit code: 0
```

passed=false の場合:
```
[CI] ✗ Invariant violations detected. M/N failed. Exit code: 1
```

**動作:**
- `result.passed == true` → `"[CI] ✓ All invariants passed. Exit code: {exit_code}"`
- `result.passed == false` → `"[CI] ✗ Invariant violations detected. {failed}/{total} failed. Exit code: {exit_code}"`
  - `failed` = `result.report.results.iter().filter(|r| !r.passed).count()`
  - `total` = `result.report.results.len()`

---

## テスト仕様

### `ci_verification_passes`

```rust
let config = CiVerificationConfig {
    pipeline:  "OrderPipeline".to_string(),
    fail_fast: false,
    data_path: "data/sample.csv".to_string(),
};
let invariants = vec![
    PipelineInvariant {
        name:        "row_count_reduced".to_string(),
        expression:  "output.row_count < input.row_count".to_string(),
        check_point: InvariantCheckPoint::Post,
    },
    PipelineInvariant {
        name:        "amount_non_negative".to_string(),
        expression:  "SUM(output.amount) >= 0.0".to_string(),
        check_point: InvariantCheckPoint::Post,
    },
];
let result = run_ci_verification(&config, &invariants);
assert!(result.passed);
assert_eq!(result.exit_code, 0);
assert_eq!(result.report.pipeline, "OrderPipeline");
let summary = format_ci_result_summary(&result);
assert!(summary.contains("[CI]"));
assert!(summary.contains("passed"));
```

### `ci_verification_fails_on_violation`

```rust
// CiResult を直接構築して format_ci_result_summary の違反フォーマットを検証
let report = VerificationReport {
    pipeline:   "FailPipeline".to_string(),
    results:    vec![
        InvariantResult {
            name:   "ok_inv".to_string(),
            passed: true,
            detail: "ok".to_string(),
        },
        InvariantResult {
            name:   "fail_inv".to_string(),
            passed: false,
            detail: "violated".to_string(),
        },
    ],
    all_passed: false,
};
let result = CiResult {
    passed:    false,
    report,
    exit_code: 1,
};
assert!(!result.passed);
assert_eq!(result.exit_code, 1);
let summary = format_ci_result_summary(&result);
assert!(summary.contains("[CI]"));
assert!(summary.contains("violations"));
assert!(summary.contains("Exit code: 1"));
```

---

## Success Criteria

- `CiVerificationConfig` / `CiResult` 構造体が定義されている（Debug / Clone 付き）
- `run_ci_verification` が `cmd_verify` を利用して `CiResult` を返す
- `run_ci_verification` が返す `exit_code` は `passed=true` → 0、`passed=false` → 1
- `format_ci_result_summary` が passed 時に `"passed"` を、failed 時に `"violations"` と `"Exit code: 1"` を含む文字列を返す
- `ci_verification_passes` が pass
- `ci_verification_fails_on_violation` が pass
- `cargo test` が 3748 tests all pass
- `driver.rs` 内の `cargo_toml_version_is_X` 系テストの `77.5.0` バージョン文字列アサーションがすべて `77.6.0` に更新されている（セクションコメント `// --- v77.5.0: fav verify コマンド ---` は変更しない）
- `CHANGELOG.md` の先頭に v77.6.0 エントリが存在する

---

## 変更ファイル

実装順序は plan.md 参照。

- `fav/src/driver.rs` — `CiVerificationConfig`, `CiResult`, `run_ci_verification`, `format_ci_result_summary`, `v776000_tests` を追加
- `CHANGELOG.md` — v77.6.0 エントリを追加
- `versions/current.md` — 進行中バージョンを更新
- `fav/Cargo.toml` — バージョンを `77.5.0` → `77.6.0` に更新

---

## 依存

- v77.5.0 の `VerificationReport` / `InvariantResult` / `cmd_verify` 構造体・関数を再利用
- v77.1.0 の `PipelineInvariant` / `InvariantCheckPoint` 構造体を再利用
- `InvariantViolation`（v77.1.0 定義済み）は v77.6.0 では直接使用しない

---

## 対象外

- 実際の CI YAML ファイル（`.github/workflows/verify.yml`）の生成: 将来の CLI 統合で対応
- `fail_fast` / `data_path` の実際の動作: フィールドは宣言のみ、将来の v78.x 以降で利用
- `parser.rs` / `ast.rs` / `checker.rs` への変更は一切行わない
- `format_ci_result_summary` 内の `✓`/`✗` Unicode 文字の Windows ターミナル表示: Rust テスト環境（UTF-8）では問題なし。CLI 出力としての表示互換性は将来の CLI 統合時に検討する
