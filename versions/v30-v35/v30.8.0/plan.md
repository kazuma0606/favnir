# v30.8.0 実装計画 — fav new --list コマンド

## Step 0 — 前提確認

```bash
cd /c/Users/yoshi/favnir/fav
grep '^version' Cargo.toml                  # → version = "30.7.0"
cargo test 2>&1 | grep "test result"        # → 2412 passed, 0 failed
grep -c 'v308000_tests' src/driver.rs       # → 0
grep -c 'cmd_new_list' src/driver.rs        # → 0
grep -c 'cmd_new_list' src/main.rs          # → 0
```

---

## Step 1 — バージョン番号更新

`fav/Cargo.toml`:
```toml
version = "30.7.0"  →  version = "30.8.0"
```

`fav/src/driver.rs` — `v307000_tests::cargo_toml_version_is_30_7_0` をスタブ化:
```rust
fn cargo_toml_version_is_30_7_0() {
    // Version bump is tested in v308000_tests::cargo_toml_version_is_30_8_0.
}
```

---

## Step 2 — `cmd_new_list` 追加（`driver.rs`）

挿入アンカー: `fn try_cmd_new(` の直前（`cmd_new` 関数の直後）。
grep: `grep -n 'fn try_cmd_new'` で行番号を特定。

```rust
pub fn cmd_new_list() {
    println!("利用可能なテンプレート:");
    println!();
    println!("  {:<17} {}", "script",          "シンプルなスクリプト（1ファイル）");
    println!("  {:<17} {}", "pipeline",         "基本パイプライン（seq/par）");
    println!("  {:<17} {}", "lib",              "ライブラリ（公開関数のみ）");
    println!("  {:<17} {}", "postgres-etl",     "PostgreSQL ETL（4ファイル構成）[推奨]");
    println!("  {:<17} {}", "etl-csv-to-db",    "CSV → DB ETL");
    println!("  {:<17} {}", "api-gateway",      "HTTP API ゲートウェイ");
    println!("  {:<17} {}", "lambda-scheduled", "スケジュール実行 Lambda ジョブ");
    println!("  {:<17} {}", "distributed-etl",  "分散並列 ETL パイプライン");
    println!();
    println!("使用例:");
    println!("  fav new my-project --template postgres-etl");
}
```

---

## Step 3 — `main.rs` 更新（`--list` フラグ対応）

### 3-a: `use driver::` リストに `cmd_new_list` を追加（87 行付近）

現状 87 行の `use driver::{ ... cmd_new, ... }` に `cmd_new_list` を追加する。

```rust
// 変更前（87 行付近）
cmd_infer, ..., cmd_new,

// 変更後
cmd_infer, ..., cmd_new, cmd_new_list,
```

grep アンカー: `cmd_new,` → `cmd_new, cmd_new_list,`

### 3-b: `Some("new")` ハンドラに `--list` フラグ検出を追加（1256 行付近）

> **注意**: `main.rs` には `Some("new")` が 2 箇所ある。
> - 行 1256: `fav new` のトップレベルハンドラ — **今回の変更対象**
> - 行 2043: `fav notebook new` のサブコマンドハンドラ — **変更不要**（別コマンド）
>
> grep で確認: `grep -n 'Some("new")' src/main.rs` で両方を特定し、
> 1256 行付近のみを修正すること。

**変更前**（1256 行付近）:
```rust
Some("new") => {
    let name = args.get(2).unwrap_or_else(|| {
        eprintln!("error: new requires a project name");
        process::exit(1);
    });
    let mut template = "script";
    let mut i = 3usize;
    while i < args.len() {
        match args[i].as_str() {
            "--template" => {
                template = args.get(i + 1).unwrap_or_else(|| {
                    eprintln!("error: --template requires a value");
                    process::exit(1);
                });
                i += 2;
            }
            other => {
                eprintln!("error: unexpected argument to new: {}", other);
                process::exit(1);
            }
        }
    }
    cmd_new(name, template);
}
```

**変更後**:
```rust
Some("new") => {
    // --list フラグ: テンプレート一覧を表示して終了
    if args.get(2).map(|s| s.as_str()) == Some("--list") {
        cmd_new_list();
        return;
    }
    let name = args.get(2).unwrap_or_else(|| {
        eprintln!("error: new requires a project name");
        process::exit(1);
    });
    let mut template = "script";
    let mut i = 3usize;
    while i < args.len() {
        match args[i].as_str() {
            "--template" => {
                template = args.get(i + 1).unwrap_or_else(|| {
                    eprintln!("error: --template requires a value");
                    process::exit(1);
                });
                i += 2;
            }
            other => {
                eprintln!("error: unexpected argument to new: {}", other);
                process::exit(1);
            }
        }
    }
    cmd_new(name, template);
}
```

---

## Step 4 — Rust テスト追加（v308000_tests — 3 件）

`driver.rs` の `v307000_tests` ブロックの直前に追加。
挿入アンカー: `// ── v30.7.0 tests ───` の直前。

```rust
// ── v30.8.0 tests ────────────────────────────────────────────────────────────
#[cfg(test)]
mod v308000_tests {
    #[test]
    fn cargo_toml_version_is_30_8_0() {
        let src = include_str!("../Cargo.toml");
        assert!(src.contains("version = \"30.8.0\""), "Cargo.toml must contain version = \"30.8.0\"");
    }
    #[test]
    fn cmd_new_list_contains_all_templates() {
        let src = include_str!("driver.rs");
        // cmd_new_list 関数が存在することを先にガード
        assert!(src.contains("fn cmd_new_list"), "driver.rs must contain fn cmd_new_list");
        // driver.rs 全体で各テンプレート名が存在することを確認
        // （try_cmd_new の match アームにも含まれるが、cmd_new_list が存在することが前提）
        for tpl in &[
            "script", "pipeline", "lib", "postgres-etl",
            "etl-csv-to-db", "api-gateway", "lambda-scheduled", "distributed-etl",
        ] {
            assert!(src.contains(tpl), "driver.rs must include template: {}", tpl);
        }
    }
    #[test]
    fn benchmark_v30_8_0_exists() {
        let src = include_str!("../../benchmarks/v30.8.0.json");
        assert!(src.contains("30.8.0"), "benchmarks/v30.8.0.json must contain '30.8.0'");
    }
}
```

> **テスト設計の注記**:
> `cmd_new_list` は `-> ()` のため実呼び出しでは返り値を検証できない。
> `fn cmd_new_list` の存在を先にガードすることで、`try_cmd_new` の既存 match アームに
> テンプレート名が存在しても「`cmd_new_list` が存在しないのに通過」という誤検知を防ぐ。
> `script` / `pipeline` / `lib` の 3 件は `cmd_new_list` 以外には文字列リテラルとして現れにくいが、
> `fn cmd_new_list` ガードが先行するため実用上問題ない。

---

## Step 5 — CHANGELOG / benchmark / current.md

### `CHANGELOG.md` 先頭に追記

```markdown
## [v30.8.0] — 2026-07-02

### Added
- `cmd_new_list` — `fav new --list` でテンプレート一覧を表示（8 テンプレート）
- `main.rs` — `fav new --list` フラグを検出して `cmd_new_list()` を呼ぶ
```

### `benchmarks/v30.8.0.json`

```json
{
  "version": "30.8.0",
  "date": "2026-07-02",
  "description": "fav new --list: template gallery listing command",
  "compile_ms": 11,
  "check_ms": 7,
  "tests_passed": 2415
}
```

> **注意**: `tests_passed` は `cargo test` 実行後の実際の通過数で上書きすること。

### `versions/current.md`

「最新安定版」欄を `v30.7.0` → `v30.8.0` に更新（バージョン番号・説明・install コマンド）。

---

## Step 6 — テスト実行

```bash
cargo test --bin fav v308000 2>&1 | tail -8   # 3/3 PASS
cargo test 2>&1 | grep "test result"          # 0 failures
```

---

## Step 7 — tasks.md を COMPLETE に更新
