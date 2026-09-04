# Spec: v98.6.0 — `fav report --sap`（ローカル HTML レポート生成コマンド）

## Background

v98.1.0〜v98.5.0 で SAP Analytics の型定義・pipeline・アラートを実装した。
v98.6.0 では、SAP データからローカル HTML レポートを生成する CLI コマンド `fav report --sap` を追加する。
このコマンドはスタブ実装（モックデータを使用）であり、ファイル出力の形式確認に重点を置く。

## Goals

1. `fav/src/main.rs` — `Some("report")` ケースを追加し `--sap` フラグを処理する
2. `fav/src/driver.rs` — `cmd_report_sap(entity, from, to, output)` 実装 + `mod v98600_tests`（2テスト）

## Syntax / API Examples

### CLI 使用例

```
$ fav report --sap --entity SalesOrder --from 2026-08-01 --to 2026-08-31 --output report.html
Fetching SalesOrder from SAP... 1,234 records
Generating report...
Saved: report.html
```

フラグ一覧:

| フラグ | 必須 | デフォルト | 説明 |
|---|---|---|---|
| `--sap` | 必須 | — | SAP モードを指定 |
| `--entity` | 任意 | `"SalesOrder"` | エンティティセット名 |
| `--from` | 任意 | `""` | 開始日（YYYY-MM-DD） |
| `--to` | 任意 | `""` | 終了日（YYYY-MM-DD） |
| `--output` | 任意 | `"report.html"` | 出力ファイルパス |

### cmd_report_sap の動作（スタブ実装）

```rust
pub fn cmd_report_sap(entity: &str, from: &str, to: &str, output: &str) -> i32 {
    println!("Fetching {} from SAP... 1,234 records", entity);
    println!("Generating report...");
    let html = format!(
        "<html><body><h1>SAP Report: {entity}</h1>\
         <p>From: {from} To: {to}</p>\
         <p>Records: 1,234</p></body></html>"
    );
    match std::fs::write(output, html) {
        Ok(_) => {
            println!("Saved: {}", output);
            0
        }
        Err(e) => {
            eprintln!("error: failed to write report: {}", e);
            1
        }
    }
}
```

### main.rs — report コマンド追加箇所

`Some("ai") => { ... }` ブロックの直前（`Some("sap-mock")` ブロックの後）に挿入：

```rust
// ── v98.6.0: fav report ──────────────────────────────────────────────
Some("report") => {
    let sap = args.iter().any(|a| a == "--sap");
    if !sap {
        eprintln!("error: fav report requires --sap flag");
        eprintln!("  fav report --sap [--entity <name>] [--from <date>] [--to <date>] [--output <file>]");
        process::exit(1);
    }
    let entity = args.iter()
        .position(|a| a == "--entity")
        .and_then(|i| args.get(i + 1))
        .map(|s| s.as_str())
        .unwrap_or("SalesOrder");
    let from = args.iter()
        .position(|a| a == "--from")
        .and_then(|i| args.get(i + 1))
        .map(|s| s.as_str())
        .unwrap_or("");
    let to = args.iter()
        .position(|a| a == "--to")
        .and_then(|i| args.get(i + 1))
        .map(|s| s.as_str())
        .unwrap_or("");
    let output = args.iter()
        .position(|a| a == "--output")
        .and_then(|i| args.get(i + 1))
        .map(|s| s.as_str())
        .unwrap_or("report.html");
    let code = driver::cmd_report_sap(entity, from, to, output);
    process::exit(code);
}
```

## Success Criteria

- `fav report --sap --entity SalesOrder --from 2026-08-01 --to 2026-08-31 --output /tmp/r.html` を実行すると
  `Fetching SalesOrder from SAP... 1,234 records` / `Generating report...` / `Saved: /tmp/r.html` が出力される
- 出力 HTML ファイルに `"SAP Report: SalesOrder"` が含まれる
- `cargo test -- --test-threads=1` が 4,247 tests, 0 failures で通過する

## Error Codes

新規エラーコードなし。

## Files to Modify

| ファイル | 変更種別 |
|---|---|
| `fav/src/main.rs` | 追記（`Some("report")` ケース） |
| `fav/src/driver.rs` | 追記（`cmd_report_sap` 関数 + `mod v98600_tests`） |
| `CHANGELOG.md` | 追記 |
| `versions/current.md` | 更新 |
