# Plan: v52.6.0 — `fav run --audit-log` データアクセスログ

Status: PLANNED
Date: 2026-07-21

---

## 実装順序

### Step 1 — `vm.rs` 更新

ファイル: `fav/src/backend/vm.rs`

**1a. `AUDIT_LOG_PATH` thread-local と `set_audit_log_path` を追加**

挿入位置: `set_strict_schema` 関数の直後（`STRICT_SCHEMA` thread-local と対称）。

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

**1b. `append_audit_event` ヘルパーを追加**

挿入位置: `set_audit_log_path` の直後。

```rust
/// v52.6.0: AUDIT_LOG_PATH が設定されている場合に JSONL 行を追記する。
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

**1c. `Kafka.produce_raw` アームにフックを挿入**

`topic` 変数取得後（`let topic = ...` の直後）:
```rust
#[cfg(not(target_arch = "wasm32"))]
{
    let ts = Utc::now().format("%Y-%m-%dT%H:%M:%SZ")  // use chrono::Utc; が vm.rs 行 13 で既存;
    append_audit_event(&format!(
        "{{\"ts\":\"{}\",\"op\":\"write\",\"effect\":\"Kafka\",\"topic\":\"{}\"}}",
        ts, topic
    ));
}
```

**1d. `Kafka.consume_one_raw` アームにフックを挿入**

`topic` 変数取得後:
```rust
#[cfg(not(target_arch = "wasm32"))]
{
    let ts = Utc::now().format("%Y-%m-%dT%H:%M:%SZ")  // use chrono::Utc; が vm.rs 行 13 で既存;
    append_audit_event(&format!(
        "{{\"ts\":\"{}\",\"op\":\"read\",\"effect\":\"Kafka\",\"topic\":\"{}\"}}",
        ts, topic
    ));
}
```

**1e. `Snowflake.execute_raw` アームにフックを挿入**

`sql` 変数取得後（sql の先頭 80 文字を記録）:
```rust
#[cfg(not(target_arch = "wasm32"))]
{
    let ts = Utc::now().format("%Y-%m-%dT%H:%M:%SZ")  // use chrono::Utc; が vm.rs 行 13 で既存;
    let sql_preview: String = sql.chars().take(80).collect();
    append_audit_event(&format!(
        "{{\"ts\":\"{}\",\"op\":\"write\",\"effect\":\"Snowflake\",\"sql\":\"{}\"}}",
        ts, sql_preview.replace('"', "\\\"")
    ));
}
```

`cargo build` → コンパイルエラーなし確認。

### Step 2+3 — `driver.rs` + `main.rs` 更新（同時実施必須）

`cmd_run` シグネチャ変更により `main.rs` および `cmd_run_self_hosted` が未更新だとコンパイルエラーになるため、すべて同時に変更してから `cargo build` する。

**driver.rs の変更**:

1. `cmd_run` シグネチャに `audit_log: Option<&str>` を末尾引数として追加
2. `set_strict_schema(strict_schema);` の直後に追加（`pub use` は不要、完全パスで呼び出す）:
   ```rust
   #[cfg(not(target_arch = "wasm32"))]
   crate::backend::vm::set_audit_log_path(audit_log.map(|s| s.to_string()));
   ```
3. `cmd_run_self_hosted`（行 1563〜1565）内の `cmd_run` 呼び出しを更新:
   ```rust
   cmd_run(file, db_url, false, false, false, false, false, false, None, None, false, None);
   ```
   （末尾に `None` を追加）

`rg "cmd_run(" fav/src/` で呼び出し箇所を確認。既知: `main.rs` 1 箇所 + `driver.rs` 内 `cmd_run_self_hosted` 1 箇所 = 計 2 箇所。

**main.rs の変更**:

1. `fav run` ブロックの変数宣言に追加（`strict_schema` の直後）:
   ```rust
   let mut audit_log: Option<String> = None;
   ```
2. `match` に追加（`"--strict-schema"` アームの直後）:
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
3. `cmd_run` 呼び出しを更新（末尾に `audit_log.as_deref()` を追加）

`cargo build` → コンパイルエラーなし確認。

### Step 4 — `driver.rs` にテスト追加 + バージョン更新

- `rg -n "v52500_tests" fav/src/driver.rs` で挿入位置を確認
- `v52600_tests` モジュールを `v52500_tests` の直前に追加（2 件）
- `fav/Cargo.toml` version → `"52.6.0"`
- `cargo test` → 3149 passed, 0 failed を確認
- `cargo clippy -- -D warnings` クリーンを確認

### Step 5 — 後処理

- `CHANGELOG.md` に v52.6.0 エントリ追加
- `versions/current.md` を v52.6.0（3149 tests）に更新
- `versions/roadmap/roadmap-v52.1-v53.0.md` の v52.6.0 実績欄を更新
  - ロードマップ推定値 3147 → 実績 3149 に修正
- `tasks.md` を COMPLETE に更新（T0〜T5 全 `[x]`）

---

## 注意事項

- `AUDIT_LOG_PATH` thread-local は `RefCell<Option<String>>` であり wasm32 ではコンパイル対象外。
- `STRICT_SCHEMA`（vm.rs 行 1276〜1278）はモジュールレベルの `thread_local!` ブロックとして定義されている。
  `AUDIT_LOG_PATH` も同様にモジュールレベルの `thread_local!` ブロックとして `STRICT_SCHEMA` の直後に追加する。
  `set_audit_log_path` 関数は `set_strict_schema` 関数（行 1473〜1476）の直後に追加する
  （thread-local ブロックと関数の挿入位置は異なる）。
- `Kafka.produce_raw` アームは `#[cfg(not(target_arch = "wasm32"))]` の外側にアームが存在する
  （wasm32 用の別アームが `#[cfg(target_arch = "wasm32")]` でフォールバックを提供）。
  フックは非 wasm32 側のアームのみに挿入する。
- `append_audit_event` でのファイル書き込み失敗は `let _ = writeln!(...)` で無視する
  （パイプライン実行を止めない best-effort ロギング）。
- SQL プレビューの `replace('"', "\\\"")` で JSON 文字列の破壊を防ぐ。
