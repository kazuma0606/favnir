# Plan — v58.2.0 — カナリアリリース

## 実装方針

v58.1.0 で追加した `cmd_deploy_strategy` を差分更新する。
`match sub` パターンに `promote` / `abort` / `status` を追加し、
`match strategy` に `"canary"` アームを追加する。
main.rs のディスパッチ条件を拡張してカナリアサブコマンドを振り分ける。

---

## ファイル変更一覧

| ファイル | 変更内容 |
|---|---|
| `fav/Cargo.toml` | version `58.1.0` → `58.2.0` |
| `fav/src/driver.rs` | `cmd_deploy_strategy` 拡張（canary / promote / abort / status）+ `v58200_tests` 追加 + rolling 5 件更新 |
| `fav/src/main.rs` | ディスパッチ条件に `"promote"` / `"abort"` / `"status"` を追加 |

---

## 詳細手順

### Step 1: `fav/Cargo.toml` version 更新

```
58.1.0 → 58.2.0
```

### Step 2: `fav/src/driver.rs` — `cmd_deploy_strategy` 全体を差分更新

既存の `cmd_deploy_strategy` 関数本体を以下のように更新する（関数シグネチャは変えない）。

現在の実装:
```rust
let is_rollback = args.first().map(|a| a == "rollback").unwrap_or(false);
if is_rollback {
    println!("Traffic switch: green → blue [100%]");
    println!("Rollback complete.");
    return 0;
}
let strategy = args.iter()
    .position(|a| a == "--strategy")
    .and_then(|i| args.get(i + 1))
    .map(|s| s.as_str())
    .unwrap_or("rolling"); // fallback: caught by `_` arm (unsupported until v58.x)
match strategy { ... }
```

更新後の実装（`is_rollback` を `match sub` パターンに統合）:
```rust
let sub = args.first().map(|a| a.as_str()).unwrap_or("");

match sub {
    "rollback" => {
        println!("Traffic switch: green → blue [100%]");
        println!("Rollback complete.");
        return 0;
    }
    "promote" => {
        println!("Canary promoted to 100% traffic.");
        return 0;
    }
    "abort" => {
        println!("Canary aborted. Traffic reverted to stable.");
        return 0;
    }
    "status" => {
        println!("Canary status: error_rate=0.1% latency_p99=120ms");
        return 0;
    }
    _ => {}
}

let strategy = args.iter()
    .position(|a| a == "--strategy")
    .and_then(|i| args.get(i + 1))
    .map(|s| s.as_str())
    .unwrap_or("rolling"); // fallback: caught by `_` arm (unsupported until v58.x)

let weight = args.iter()
    .position(|a| a == "--canary-weight")
    .and_then(|i| args.get(i + 1))
    .and_then(|s| s.parse::<u32>().ok())
    .unwrap_or(10);

match strategy {
    "blue-green" => {
        println!("Deploying to: green slot (current: blue)");
        println!("Health check: OK (green)");
        println!("Traffic switch: blue → green [100%]");
        println!("Old slot (blue): kept for 10 minutes (rollback window)");
        0
    }
    "canary" => {
        println!("Deploying to canary ({weight}% traffic)");
        println!("Canary health: OK");
        0
    }
    _ => {
        eprintln!("Unknown deploy strategy: {strategy}");
        1
    }
}
```

### Step 3: `fav/src/main.rs` — ディスパッチ条件拡張

現在:
```rust
let sub = args.get(2).map(|s| s.as_str()).unwrap_or("");
let has_strategy = args.iter().any(|a| a == "--strategy");
if sub == "rollback" || has_strategy {
```

更新後:
```rust
let sub = args.get(2).map(|s| s.as_str()).unwrap_or("");
let has_strategy = args.iter().any(|a| a == "--strategy");
let is_canary_sub = matches!(sub, "rollback" | "promote" | "abort" | "status");
if is_canary_sub || has_strategy {
```

### Step 4: `fav/src/driver.rs` — `v58200_tests` 追加

`v58100_tests` の直前に挿入する。

```rust
// -- v58200_tests (v58.2.0) -- カナリアリリース --
#[cfg(test)]
mod v58200_tests {
    use super::cmd_deploy_strategy;

    #[test]
    fn cmd_deploy_canary_weight() {
        let args = vec![
            "--strategy".to_string(),
            "canary".to_string(),
            "--canary-weight".to_string(),
            "10".to_string(),
        ];
        let code = cmd_deploy_strategy(&args);
        assert_eq!(code, 0, "canary deploy should succeed");
    }

    #[test]
    fn cmd_deploy_canary_promote() {
        let args = vec!["promote".to_string()];
        let code = cmd_deploy_strategy(&args);
        assert_eq!(code, 0, "canary promote should succeed");
    }
}
```

### Step 5: Rolling バージョンチェック更新（5 件）

| テスト | 変更前 | 変更後 |
|---|---|---|
| `v56300_tests::cargo_toml_version_is_56_3_0` | `"58.1.0"` | `"58.2.0"` |
| `v56900_tests::cargo_toml_version_is_56_9_0` | `"58.1.0"` | `"58.2.0"` |
| `v57000_tests::cargo_toml_version_is_57_0_0` | `"58.1.0"` | `"58.2.0"` |
| `v57900_tests::cargo_toml_version_is_57_9_0` | `"58.1.0"` | `"58.2.0"` |
| `v58000_tests::cargo_toml_version_is_58_0_0` | `"58.1.0"` | `"58.2.0"` |

---

## `cmd_deploy_strategy` リファクタリングの注意点

`is_rollback` フラグを廃止して `match sub` に統合する。
既存の `v58100_tests::cmd_deploy_rollback` / `cmd_deploy_blue_green` テストは引き続きパスする。

**重要: `cmd_deploy_unknown_strategy` テスト引数の更新（Step 2 と同時に実施）**

`"canary"` が v58.2.0 で valid strategy になるため、`cmd_deploy_unknown_strategy` のテスト引数を変更する必要がある:

```rust
// 変更前（v58.1.0）
let args = vec!["--strategy".to_string(), "canary".to_string()];
assert_eq!(code, 1, ...);

// 変更後（v58.2.0）
let args = vec!["--strategy".to_string(), "invalid-strategy".to_string()];
assert_eq!(code, 1, "unknown deploy strategy should return exit code 1");
```

## main.rs ディスパッチの位置について

`cmd_deploy_strategy` には `args.get(2..).unwrap_or(&[])` のスライスを渡すため、
driver.rs 側の `args.first()` と main.rs 側の `args.get(2)` は同じトークンを指す。
位置ずれは発生しない。

---

## テスト戦略

```bash
cd /c/Users/yoshi/favnir/fav && cargo test -j 8 -- --test-threads=8 2>&1 | tail -20
```

期待: `3281 tests passed, 0 failed`（ベース 3279 + 2）

```bash
cargo clippy -- -D warnings
```

期待: 警告ゼロ

---

## 実装順序

```
Step 1 (Cargo.toml)
→ Step 2 (cmd_deploy_strategy 拡張)
→ Step 3 (main.rs ディスパッチ更新)
→ Step 4 (v58200_tests)
→ Step 5 (rolling 更新)
→ cargo build → cargo test → cargo clippy
```
