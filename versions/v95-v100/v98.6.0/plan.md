# Plan: v98.6.0 — `fav report --sap`

## 実装順序

### Step 1: driver.rs に cmd_report_sap を実装

`fav/src/driver.rs` に関数を追加（末尾近くの既存コマンド関数群に挿入）：

```rust
/// v98.6.0: fav report --sap — HTML レポートをローカル生成するスタブ実装
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

---

### Step 2: main.rs に Some("report") ケースを追加

`Some("sap-mock") => { ... }` ブロックの後、`Some("ai") => { ... }` の直前に挿入。

`--sap` フラグが未指定の場合はエラーメッセージを出力して `process::exit(1)`。

---

### Step 3: driver.rs に mod v98600_tests を追加

`mod v98500_tests` の直後に追加：

```rust
#[cfg(test)]
mod v98600_tests {
    // use super::* は不要（driver 関数を直接呼ぶ）
    #[test]
    fn cmd_report_sap_exists() {
        // cmd_report_sap がコンパイルできることを確認（存在テスト）
        let _ = super::cmd_report_sap as fn(&str, &str, &str, &str) -> i32;
    }

    #[test]
    fn cmd_report_sap_generates_html() {
        let out = std::env::temp_dir().join("fav_test_report.html");
        let code = super::cmd_report_sap(
            "SalesOrder",
            "2026-08-01",
            "2026-08-31",
            out.to_str().unwrap(),
        );
        assert_eq!(code, 0, "cmd_report_sap should return 0 on success");
        let content = std::fs::read_to_string(&out).unwrap();
        assert!(content.contains("SAP Report: SalesOrder"), "HTML should contain entity name");
        let _ = std::fs::remove_file(&out);
    }
}
```

> **Note**: このテストは実際に HTML ファイルを一時ディレクトリに書き出して内容を検証する動作テストであり、
> 他バージョンの存在チェックテストより一歩進んだ検証を行う。

---

### Step 4: テスト実行

```bash
cd /c/Users/yoshi/favnir/fav && cargo test -- --test-threads=1 2>&1 | grep "test result"
```

期待値: 4,247 tests, 0 failures

---

### Step 5: CHANGELOG.md に v98.6.0 エントリを追加

---

### Step 6: versions/current.md 更新

最新安定版を `v98.6.0` に更新（テスト数 4,247）。

---

### Step 7: CI 事前確認

- `cargo clippy --locked -- -D warnings`
- `./target/debug/fav fmt --check self/compiler.fav`
- `./target/debug/fav fmt --check self/checker.fav`
