# Tasks: v52.6.0 — `fav run --audit-log` データアクセスログ

Status: COMPLETE
Date: 2026-07-21

---

## T0 — 事前確認

- [x] `cargo test` 3147 passed, 0 failed を確認（ベース確認）
- [x] `cargo clippy -- -D warnings` クリーンであることを確認
- [x] `rg "cmd_run(" fav/src/` で呼び出し箇所が 2 箇所であることを確認:
  - [x] `main.rs` 行 557 付近（1 箇所）
  - [x] `driver.rs` 行 1564（`cmd_run_self_hosted` 内、1 箇所）
- [x] `vm.rs` に `AUDIT_LOG_PATH` が**存在しない**ことを確認（新規追加対象）
- [x] `vm.rs` に `append_audit_event` が**存在しない**ことを確認（新規追加対象）
- [x] `main.rs` の `fav run` ブロックに `--audit-log` が**存在しない**ことを確認（新規追加対象）
- [x] `v52500_tests` に `cargo_toml_version_is_52_5_0` が**存在しない**ことを確認（削除対象なし）
- [x] `include_str!` パス確認（`fav/src/driver.rs` 起点）:
  - [x] `include_str!("backend/vm.rs")` → `fav/src/backend/vm.rs` ✓
  - [x] `include_str!("main.rs")` → `fav/src/main.rs` ✓
- [x] `set_strict_schema` の位置を確認（`AUDIT_LOG_PATH` の挿入位置の基準）:
  - [x] `rg -n "set_strict_schema" fav/src/backend/vm.rs` → 挿入位置を特定
- [x] `Kafka.produce_raw` の `topic` 変数取得行を確認（フック挿入位置の基準）:
  - [x] `rg -n '"Kafka.produce_raw"' fav/src/backend/vm.rs` → 行番号を確認
- [x] `Snowflake.execute_raw` の `sql` 変数取得行を確認:
  - [x] `rg -n '"Snowflake.execute_raw"' fav/src/backend/vm.rs` → 行番号を確認

## T1 — `vm.rs` 更新（thread-local + ヘルパー + フック）

- [x] `set_strict_schema` 直後に `AUDIT_LOG_PATH` thread-local を追加:
  - [x] `#[cfg(not(target_arch = "wasm32"))]` で保護
  - [x] `std::cell::RefCell<Option<String>>` を使用
  - [x] `const { RefCell::new(None) }` で初期化
- [x] `set_audit_log_path(path: Option<String>)` 関数を追加:
  - [x] `pub` かつ `#[cfg(not(target_arch = "wasm32"))]`
  - [x] `AUDIT_LOG_PATH.with(|p| *p.borrow_mut() = path)` の実装
- [x] `append_audit_event(json_line: &str)` ヘルパーを追加:
  - [x] `fn`（非 `pub`）かつ `#[cfg(not(target_arch = "wasm32"))]`
  - [x] `OpenOptions::new().append(true).create(true).open(path)` パターン
  - [x] `writeln!` の失敗は `let _ = ...` で無視
- [x] `Kafka.produce_raw` アームにフックを挿入（`topic` 変数取得後）:
  - [x] `#[cfg(not(target_arch = "wasm32"))]` ブロックで囲む
  - [x] `op` = `"write"`, `effect` = `"Kafka"`, `topic` = topic 変数
  - [x] `chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ")` でタイムスタンプ生成
- [x] `Kafka.consume_one_raw` アームにフックを挿入（`topic` 変数取得後）:
  - [x] `op` = `"read"`, `effect` = `"Kafka"`
- [x] `Snowflake.execute_raw` アームにフックを挿入（`sql` 変数取得後）:
  - [x] `op` = `"write"`, `effect` = `"Snowflake"`
  - [x] `sql.chars().take(80).collect::<String>()` で先頭 80 文字に切り詰め
  - [x] SQL 内のダブルクォートを `replace('"', "\\\"")` でエスケープ
- [x] `cargo build` → コンパイルエラーなし確認

## T2+T3 — `driver.rs` + `main.rs` 更新（同時実施必須）

> **注意**: T2 でシグネチャを変更すると T3 が完了するまで `cargo build` が通らない。
> T2 と T3 は一気通貫で実施し、両方完了後に `cargo build` を確認すること。

### driver.rs の変更（T2）

- [x] `cmd_run` シグネチャに `audit_log: Option<&str>` を末尾に追加
- [x] `set_strict_schema(strict_schema);` の直後に追加（`pub use` は不要、完全パスで呼び出す）:
  - [x] `#[cfg(not(target_arch = "wasm32"))]` で保護
  - [x] `crate::backend::vm::set_audit_log_path(audit_log.map(|s| s.to_string()));`
- [x] `cmd_run_self_hosted`（行 1563〜1565）内の `cmd_run` 呼び出しを更新:
  - [x] 末尾引数に `None` を追加（`cmd_run(file, db_url, false, ..., false, None)`）
- [x] `rg "cmd_run(" fav/src/` で呼び出し箇所を確認（既知: `main.rs` + `cmd_run_self_hosted` の 2 箇所）

### main.rs の変更（T3）

- [x] `fav run` ブロックの変数宣言に `let mut audit_log: Option<String> = None;` を追加（`strict_schema` の直後）
- [x] `match` に `"--audit-log"` アームを追加（`"--strict-schema"` アームの直後、`other =>` catch-all の前）:
  - [x] `args.get(i + 1)` でパスを取得
  - [x] `i += 2`
  - [x] パスなし時は `eprintln!("error: --audit-log requires a file path")` + `process::exit(1)`
- [x] `cmd_run` 呼び出しを更新（末尾に `audit_log.as_deref()` を追加）
- [x] `cargo build` → コンパイルエラーなし確認

## T4 — テスト追加 + バージョン更新

- [x] `rg -n "v52500_tests" fav/src/driver.rs` で挿入位置を確認
- [x] `v52600_tests` モジュールを `v52500_tests` の直前に追加（2 件）:
  - [x] `audit_log_read_event`:
    - [x] `include_str!("backend/vm.rs")` に `AUDIT_LOG_PATH` が含まれることを assert
    - [x] `include_str!("backend/vm.rs")` に `append_audit_event` が含まれることを assert
    - [x] フォーマット文字列のリテラル `op\\\":\\\"read\\\"` が含まれることを assert
  - [x] `audit_log_write_event`:
    - [x] `include_str!("backend/vm.rs")` にフォーマット文字列のリテラル `op\\\":\\\"write\\\"` が含まれることを assert
    - [x] `include_str!("main.rs")` に `--audit-log` が含まれることを assert
- [x] `v52500_tests` に version テストなし → 削除対象なし（確認済み）
- [x] `fav/Cargo.toml` version → `"52.6.0"`
- [x] `cargo test` 実行 → 3149 passed, 0 failed を確認
- [x] `cargo clippy -- -D warnings` クリーンを確認

## T5 — 後処理

- [x] `CHANGELOG.md` に v52.6.0 エントリ追加
- [x] `versions/current.md` を v52.6.0（3149 tests）に更新
- [x] `roadmap-v52.1-v53.0.md` の v52.6.0 実績欄を更新:
  - [x] ロードマップ推定値 3147 → 実績 3149 に修正（v52.5.0 実績 3147 + 追加 2 件）
  - [x] `!S3` が vm.rs に存在しないため本バージョンのスコープ外である旨を注釈追加
  - [x] v52.7.0 の推定値（現在 3149）が変わらないことを確認する
- [x] tasks.md を COMPLETE に更新（T0〜T5 全 `[x]`）
