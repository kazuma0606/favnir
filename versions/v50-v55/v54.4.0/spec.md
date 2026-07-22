# Spec: v54.4.0 — fav dq-report データ品質レポートコマンド

Status: COMPLETE
Date: 2026-07-22

---

## 概要

`fav dq-report --audit-log <path>` コマンドを追加する。
`fav run --audit-log` が出力する JSONL ログを解析し、スキーマ統計・SLA 違反を集計して
Markdown レポートを生成する。

---

## 実装スコープ

### 1. `driver.rs` — `cmd_dq_report_collect` 関数

```rust
/// Generate a data-quality Markdown report from an audit-log JSONL string.
///
/// Each line of `audit_log` is expected to be a JSON object with at least
/// `"ts"` and `"op"` fields (written by `fav run --audit-log`).
/// Lines with `"op":"schema_error"` count as validation errors.
/// Lines with `"latency_ms"` > 200 count as SLA violations.
///
/// Note: a single audit log line may contribute to **both** schema statistics
/// and SLA statistics (e.g. a `schema_check` line with `latency_ms` is counted
/// in both the row total and the SLA violation list).
pub fn cmd_dq_report_collect(audit_log: &str) -> String { ... }
```

JSONL 解析ロジック:
- `op == "schema_check"` または `op == "schema_error"` → `total_rows` を +1、スキーマ別カウント更新
- `op == "schema_error"` → さらに `error_rows` を +1
- `latency_ms > 200.0` → SLA 違反リストに追加

出力形式（Markdown）:
```
# Data Quality Report

Schema validation:  N rows checked, E errors (P%)
  SchemaName:  ok / total OK|ERRORS

SLA violations:
  latency >Xms at TS (stage: S)
```

設計上の注意:
- `schema` を `schema_counts.entry()` に渡す際は **move**（`.clone()` 不要）
- `schema_counts` の値型 `(u64, u64)`: `.0` = total、`.1` = errors（行コメントで明示）
- スキーマ名はキーでソートして出力（安定した順序）

### 2. `main.rs` — `fav dq-report` コマンド

`Some("bench")` ブロックの直後（`Some("watch")` の直前）に追加:

```rust
Some("dq-report") => {
    let audit_log_path = args.iter().position(|a| a == "--audit-log")
        .and_then(|i| args.get(i + 1))
        .map(|s| s.as_str())
        .unwrap_or_else(|| {
            eprintln!("error: fav dq-report requires --audit-log <path>");
            process::exit(1);
        });
    let audit_log_content = std::fs::read_to_string(audit_log_path).unwrap_or_else(|e| {
        eprintln!("error: cannot read {audit_log_path}: {e}");
        process::exit(1);
    });
    let report = driver::cmd_dq_report_collect(&audit_log_content);
    println!("{report}");
}
```

### 3. `driver.rs` — `v54400_tests` 追加

`v54300_tests` の直前に追加（2 テスト）:

```rust
const SAMPLE_AUDIT_LOG: &str = r#"...6 行の JSONL（schema_check×4, schema_error×1, write×1, latency_ms:250×1）..."#;

fn cmd_dq_report_generates()     { /* 非空 + "Data Quality Report" ヘッダー */ }
fn cmd_dq_report_has_schema_stats() { /* "Schema validation" / "rows checked" /
                                         "OrderRow" / "SLA violations" /
                                         !contains("SLA violations:  none") */ }
```

---

## テスト仕様

| テスト名 | 検証内容 |
|---|---|
| `cmd_dq_report_generates` | レポートが非空かつ `"Data Quality Report"` ヘッダーを含む |
| `cmd_dq_report_has_schema_stats` | `"Schema validation"` / `"rows checked"` / `"OrderRow"` / `"SLA violations"` を含み、`"SLA violations:  none"` を含まない（SLA 違反検出確認） |

サンプルデータ（`SAMPLE_AUDIT_LOG`）:
- `op:schema_check, schema:OrderRow` × 2
- `op:schema_error, schema:OrderRow` × 1
- `op:schema_check, schema:PaymentRow` × 1
- `op:write` × 1（スキーマ集計対象外）
- `op:schema_check, schema:OrderRow, latency_ms:250` × 1（SLA 違反）

期待集計: total=5、errors=1 (20.00%)、SLA violations=1

---

## バージョン更新

- `fav/Cargo.toml`: `"54.3.0"` → `"54.4.0"`

---

## 完了条件

- `cargo test` 3193 passed, 0 failed（ベース 3191 + 2 件追加）
- `v54400_tests` 2 件 pass:
  - `cmd_dq_report_generates`
  - `cmd_dq_report_has_schema_stats`
- `cargo clippy -- -D warnings` クリーン

---

## 影響範囲

| ファイル | 変更種別 |
|---|---|
| `fav/src/driver.rs` | `cmd_dq_report_collect` 追加 / `v54400_tests` 追加 |
| `fav/src/main.rs` | `fav dq-report` コマンド追加 |
| `fav/Cargo.toml` | version 更新 |
| `fav/Cargo.lock` | version 更新に伴い自動更新 |
| `CHANGELOG.md` | v54.4.0 エントリ追加 |
| `versions/current.md` | v54.4.0 / 3193 tests に更新 |
| `versions/roadmap/roadmap-v54.1-v55.0.md` | v54.4.0 実績欄を COMPLETE に更新 |

---

## 設計上の注意

- `--schemas <dir>` フラグはロードマップで言及されているが、
  スキーマ名は audit log の `"schema"` フィールドから直接取得するため
  v54.4.0 では外部スキーマディレクトリ参照は不要（`cmd_dq_report_collect` は
  audit log のみを引数に取る）。
- `(u64, u64)` のタプルは `.0`=total / `.1`=errors の意味をコメントで明示。
  将来バージョンで named struct に移行する際はここを変更する。
- コードレビュー [LOW] 指摘の `(u64, u64)` 構造体化は将来バージョンに委ねる。
