# Spec: v52.6.0 — `fav run --audit-log` データアクセスログ

Status: PLANNED
Date: 2026-07-21

---

## 目的

v52.5.0 で SLA 監視 Rune を追加した。
v52.6.0 では `fav run --audit-log <output.jsonl>` オプションを追加し、
`!Kafka` / `!Snowflake` のアクセスイベントを JSONL 形式で記録する。

**注意**: 既存の `fav audit`（`main.rs` `Some("audit")` / `fav_audit::cmd_audit`）は
Enterprise Governance 用であり独立して継続する。本機能は `fav run` への拡張。

---

## 使用例

```bash
$ fav run pipeline.fav --audit-log audit.jsonl
```

`audit.jsonl` の内容（各行が JSON オブジェクト）:
```json
{"ts":"2026-07-21T10:00:00Z","op":"write","effect":"Kafka","topic":"orders"}
{"ts":"2026-07-21T10:00:01Z","op":"write","effect":"Snowflake","sql":"INSERT INTO orders_v2..."}
{"ts":"2026-07-21T10:00:02Z","op":"read","effect":"Kafka","topic":"orders"}
```

---

## 変更ファイル一覧

| ファイル | 変更内容 |
|---|---|
| `fav/src/backend/vm.rs` | `AUDIT_LOG_PATH` thread-local 追加、`set_audit_log_path` / `append_audit_event` 追加、Kafka/Snowflake 各アームにフック挿入 |
| `fav/src/driver.rs` | `cmd_run` に `audit_log: Option<&str>` 引数追加、`set_audit_log_path` 呼び出し追加 |
| `fav/src/main.rs` | `fav run` ブロックに `--audit-log <path>` フラグ解析追加 |
| `fav/Cargo.toml` | version → `"52.6.0"` |
| `CHANGELOG.md` | v52.6.0 エントリ追加 |
| `versions/current.md` | v52.6.0（3149 tests）に更新 |
| `versions/roadmap/roadmap-v52.1-v53.0.md` | v52.6.0 実績欄を更新 |

---

## 詳細仕様

### 1. `vm.rs` — スレッドローカル + ヘルパー追加

#### 1a. `AUDIT_LOG_PATH` thread-local

```rust
#[cfg(not(target_arch = "wasm32"))]
thread_local! {
    static AUDIT_LOG_PATH: std::cell::RefCell<Option<String>> = const { std::cell::RefCell::new(None) };
}

/// v52.6.0: `fav run --audit-log <path>` でアクセスログ出力先を設定する。
#[cfg(not(target_arch = "wasm32"))]
pub fn set_audit_log_path(path: Option<String>) {
    AUDIT_LOG_PATH.with(|p| *p.borrow_mut() = path);
}
```

挿入位置: `set_strict_schema` の直後（`STRICT_SCHEMA` と対称的な位置）。

#### 1b. `append_audit_event` ヘルパー

```rust
/// v52.6.0: AUDIT_LOG_PATH が設定されている場合に JSONL 行を追記する。
/// `ts` は `chrono::Utc::now()` で生成する（既に `use chrono::Utc;` が存在する）。
#[cfg(not(target_arch = "wasm32"))]
fn append_audit_event(json_line: &str) {
    AUDIT_LOG_PATH.with(|p| {
        if let Some(ref path) = *p.borrow() {
            use std::io::Write;
            if let Ok(mut f) = std::fs::OpenOptions::new()
                .append(true)
                .create(true)
                .open(path)
            {
                let _ = writeln!(f, "{}", json_line);
            }
        }
    });
}
```

挿入位置: `set_audit_log_path` の直後。

#### 1c. call_builtin フック

各 `_raw` アームの **先頭**（ビジネスロジックの前）に `append_audit_event` 呼び出しを挿入する。

| アーム | op | effect | 追加フィールド |
|---|---|---|---|
| `"Kafka.produce_raw"` | `write` | `Kafka` | `"topic":"<topic>"` |
| `"Kafka.consume_one_raw"` | `read` | `Kafka` | `"topic":"<topic>"` |
| `"Snowflake.execute_raw"` | `write` | `Snowflake` | `"sql":"<sql の先頭 80 文字>"` |

**挿入コードパターン** (`Kafka.produce_raw` の例、`wasm32` は skip):
```rust
#[cfg(not(target_arch = "wasm32"))]
{
    let ts = chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ");
    append_audit_event(&format!(
        "{{\"ts\":\"{}\",\"op\":\"write\",\"effect\":\"Kafka\",\"topic\":\"{}\"}}",
        ts, topic
    ));
}
```

- `Kafka.produce_raw`: `topic` 変数が取得された後に挿入
- `Kafka.consume_one_raw`: `topic` 変数が取得された後に挿入
- `Snowflake.execute_raw`: `sql` 変数が取得された後に挿入（`sql.chars().take(80).collect::<String>()` で先頭 80 文字）

**WASM ガード**: `#[cfg(not(target_arch = "wasm32"))]` ブロックで囲む
（`append_audit_event` 自体が `#[cfg(not(...))]` のため、ブロックで囲めばコンパイラが除去する）。

### 2. `driver.rs` — `cmd_run` シグネチャ更新

#### 2a. `cmd_run` シグネチャ

```rust
pub fn cmd_run(
    file: Option<&str>,
    db_url: Option<&str>,
    legacy: bool,
    verbose: bool,
    trace: bool,
    no_tap: bool,
    legacy_value_repr: bool,
    explain_pushdown: bool,
    checkpoint_dir: Option<&str>,
    resume_dir: Option<&str>,
    strict_schema: bool,
    audit_log: Option<&str>,   // ← 追加（末尾）
)
```

#### 2c. `cmd_run` 本体

`set_strict_schema(strict_schema);` の直後に追加:
```rust
// Set global audit_log_path flag (v52.6.0)
#[cfg(not(target_arch = "wasm32"))]
crate::backend::vm::set_audit_log_path(audit_log.map(|s| s.to_string()));
```

#### 2b. 呼び出し箇所（2 箇所）

- `main.rs` 行 557 付近: `cmd_run(file, ..., strict_schema)` → `cmd_run(file, ..., strict_schema, audit_log.as_deref())`
- `driver.rs` 行 1564（`cmd_run_self_hosted` 内）: `cmd_run(file, db_url, false, ..., false)` → 末尾に `None` を追加
- `rg "cmd_run(" fav/src/` で他の呼び出し箇所がないか確認すること

### 3. `main.rs` — `--audit-log <path>` フラグ解析

`fav run` ブロックの変数宣言に追加（`strict_schema` の直後）:
```rust
let mut audit_log: Option<String> = None;
```

`while i < args.len()` の `match` に追加（`"--strict-schema"` アームの直後）:
```rust
"--audit-log" => {
    audit_log = Some(
        args.get(i + 1)
            .unwrap_or_else(|| {
                eprintln!("error: --audit-log requires a file path");
                process::exit(1);
            })
            .clone(),
    );
    i += 2;
}
```

`cmd_run` 呼び出しを更新:
```rust
cmd_run(file, db_path.as_deref(), legacy, verbose, trace, no_tap, legacy_value_repr,
        explain_pushdown, checkpoint_dir.as_deref(), resume_dir.as_deref(),
        strict_schema, audit_log.as_deref());
```

---

## テスト（2 件）

追加先: `driver.rs` の `v52600_tests` モジュール（`v52500_tests` の直前）

### `audit_log_read_event`

```rust
#[test]
fn audit_log_read_event() {
    let src = include_str!("backend/vm.rs");
    assert!(src.contains("AUDIT_LOG_PATH"), "vm.rs must have AUDIT_LOG_PATH");
    assert!(src.contains("append_audit_event"), "vm.rs must have append_audit_event");
    // フォーマット文字列内のリテラル表現（include_str! はファイルの生テキストを読む）
    assert!(src.contains("op\\\":\\\"read\\\""), "vm.rs must emit read events");
}
```

### `audit_log_write_event`

```rust
#[test]
fn audit_log_write_event() {
    let src = include_str!("backend/vm.rs");
    // フォーマット文字列内のリテラル表現
    assert!(src.contains("op\\\":\\\"write\\\""), "vm.rs must emit write events");
    let main = include_str!("main.rs");
    assert!(main.contains("--audit-log"), "main.rs must support --audit-log flag");
}
```

---

## テスト数

- ベース: **3147** tests（v52.5.0 完了時点）
- `v52500_tests` に version テストなし → 削除 0 件
- 追加: `v52600_tests` 2 件（`audit_log_read_event` + `audit_log_write_event`）
- **合計: 3149 tests**（ロードマップ記載の 3147 から +2 補正: v52.5.0 実績が 3147 だったため）

---

## 完了条件

- `cargo test` 3149 passed, 0 failed
- `cargo clippy -- -D warnings` クリーン
- `fav run pipeline.fav --audit-log audit.jsonl` で `audit.jsonl` に JSONL が書き出される
- `Kafka.produce_raw` / `Kafka.consume_one_raw` / `Snowflake.execute_raw` 呼び出し時にイベントが記録される
- WASM ビルドに影響しない（`#[cfg(not(target_arch = "wasm32"))]` で保護）
- `--audit-log` 未指定時は従来通りログ出力なし

---

## 注意事項

- **`!S3` は本バージョンのスコープ外**: vm.rs に `"S3.*_raw"` アームが存在しないため実装不可。ロードマップの `!S3` 記載は将来バージョン（vm に S3 プリミティブが追加された時点）で対応する。
- **`site/content/docs/tools/audit-log.mdx` は v52.8.0 で追加予定**。本バージョンでは作成しない。
- `set_audit_log_path` / `append_audit_event` は `#[cfg(not(target_arch = "wasm32"))]` で保護する
- `driver.rs` の `set_audit_log_path` 呼び出しも `#[cfg(not(target_arch = "wasm32"))]` で保護する
- `cmd_run` シグネチャ変更後 `main.rs` が未更新だとコンパイルエラー → T2 と T3 は同時実施必須
- `AUDIT_LOG_PATH` は thread-local のため、複数スレッドから同一ファイルへの追記競合は起きない
  （`fav run` は単一スレッドで VM を実行するため）
- `append_audit_event` でファイルオープン失敗は無視（ログ記録は best-effort、パイプライン停止しない）
