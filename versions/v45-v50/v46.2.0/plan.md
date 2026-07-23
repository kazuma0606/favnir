# Plan: v46.2.0 — `fav test` コマンド: `#[test]` fn 対応

Date: 2026-07-16
Status: TODO

---

## ステップ

### Step 1 — 事前確認

```bash
cd /c/Users/yoshi/favnir/fav && cargo test -j 8 -- --test-threads=8 2>&1 | grep "test result"
```

2994 tests passed, 0 failed を確認。

---

### Step 2 — `driver.rs`: `collect_test_cases` に `is_test` アーム追加

`collect_test_cases` 内の `_ => {}` の直前に以下を追加（行番号は参考値、grep で確認すること）:

```rust
ast::Item::FnDef(fd) if fd.is_test => {
    total_discovered += 1;
    if let Some(f) = filter {
        if !fd.name.contains(f) {
            continue;
        }
    }
    tests_to_run.push((
        path.clone(),
        fd.name.clone(),   // display_name
        fd.name.clone(),   // fn_name（アーティファクト上、プレフィックスなし）
        prog.clone(),
    ));
}
```

挿入前に以下でコンテキストを確認:

```bash
grep -n "_ => {}" src/driver.rs
```

`collect_test_cases` 関数内の `_ => {}` を特定する。

---

### Step 3 — `driver.rs`: v462000_tests 追加

`grep -n "v461000_tests" src/driver.rs` でモジュール終端行を確認し、
直後に `v462000_tests` モジュールを追加（2件）:
- `fav_test_discovers_tests`
- `fav_test_reports_results`

---

### Step 4 — テスト実行

```bash
cd /c/Users/yoshi/favnir/fav && cargo test -j 8 -- --test-threads=8 2>&1 | grep "test result"
```

2996 passed（2994 + 2件）, 0 failed を確認。

---

### Step 5 — Clippy

```bash
cd /c/Users/yoshi/favnir/fav && cargo clippy -- -D warnings 2>&1 | tail -5
```

---

### Step 6 — バージョン・ドキュメント更新

1. `fav/Cargo.toml`: `version = "46.2.0"`
2. `CHANGELOG.md`: v46.2.0 エントリ追加
3. `versions/current.md`: v46.2.0（2996 tests）に更新
4. `versions/v45-v50/v46.2.0/tasks.md`: COMPLETE に更新

---

## 実装順序まとめ

```
Step 1: cargo test（事前確認: 2994 tests）
Step 2: driver.rs — collect_test_cases に is_test アーム追加
Step 3: driver.rs — v462000_tests 追加（2件）
Step 4: cargo test（2996 pass 確認）
Step 5: cargo clippy
Step 6: バージョン・ドキュメント更新
```
