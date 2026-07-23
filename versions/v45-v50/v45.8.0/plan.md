# Plan: v45.8.0 — examples 更新 Phase 1

Date: 2026-07-16
Status: TODO

---

## ステップ

### Step 1 — 事前確認

```bash
cd /c/Users/yoshi/favnir/fav && cargo test -j 8 -- --test-threads=8 2>&1 | grep "test result"
```

2985 tests passed, 0 failed を確認。

---

### Step 2 — `examples/pipeline/pipeline.fav` 更新

`examples/pipeline/pipeline.fav` を読み込み、末尾に `return` ガード節パターンの
関数例を追加する。

追加内容（W017 lint を避けるため `main` から呼び出す形式で追加）:
```favnir
// return guard pattern (v45.8.0) — validates amount using early return
fn validate_amount(amount: Float) -> Result<Float, String> {
    if amount <= 0.0 { return Err("amount must be positive") }
    if amount > 1_000_000.0 { return Err("amount exceeds maximum") }
    Ok(amount)
}
```
`validate_amount` は既存の `main` 関数内から呼び出す行を追加し、未使用警告を防ぐ。

---

### Step 3 — `driver.rs`: v458000_tests 追加

`v457000_tests` の直後に `v458000_tests` モジュールを追加（1件）:

```rust
#[cfg(test)]
mod v458000_tests {
    use walkdir::WalkDir;
    use std::path::Path;

    #[test]
    fn examples_no_legacy_effect_syntax() { ... }

    fn regex_like_match(content: &str) -> bool { ... }
}
```

`walkdir` は `[target.'cfg(not(target_arch = "wasm32"))'.dependencies]` に登録済み。
`[dev-dependencies]` には未登録だが、テストモジュール全体に
`#[cfg(not(target_arch = "wasm32"))]` を付与することで WASM ビルド時の衝突を回避する。

`CARGO_MANIFEST_DIR` マクロは `fav/` ディレクトリを指すため、
`examples/` パスは `Path::new(env!("CARGO_MANIFEST_DIR")).join("examples")` で取得できる。

---

### Step 4 — テスト実行

```bash
cd /c/Users/yoshi/favnir/fav && cargo test -j 8 -- --test-threads=8 2>&1 | grep "test result"
```

2986 tests passed, 0 failed を確認。

---

### Step 5 — Clippy

```bash
cd /c/Users/yoshi/favnir/fav && cargo clippy -- -D warnings 2>&1 | tail -5
```

---

### Step 6 — バージョン更新・ドキュメント

1. `fav/Cargo.toml`: `version = "45.8.0"`
2. `CHANGELOG.md`: v45.8.0 エントリ追加
3. `versions/current.md`: v45.8.0（2986 tests）に更新
4. `versions/v45-v50/v45.8.0/tasks.md`: COMPLETE に更新

---

## 実装順序まとめ

```
Step 1: cargo test（事前確認: 2985 tests）
Step 2: examples/pipeline/pipeline.fav に return ガード節追加
Step 3: driver.rs — v458000_tests 追加（walkdir でスキャン）
Step 4: cargo test（全通過確認: 2986 tests）
Step 5: cargo clippy
Step 6: バージョン・ドキュメント更新
```
