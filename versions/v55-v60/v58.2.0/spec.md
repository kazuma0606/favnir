# Spec — v58.2.0 — カナリアリリース

## 概要

v58.1.0 で追加した `cmd_deploy_strategy` を拡張し、カナリアリリース機能を追加する。
`--strategy canary --canary-weight <pct>` でカナリアデプロイ、
`promote` / `abort` / `status` サブコマンドでカナリア管理を行う。
既存の main.rs ディスパッチにも `promote` / `abort` / `status` を追加する。
Rust テスト 2 件（`cmd_deploy_canary_weight` / `cmd_deploy_canary_promote`）で動作を検証する。

---

## ロードマップ参照

- `versions/roadmap/roadmap-v58.1-v59.0.md` — v58.2.0 セクション
- ベーステスト数: **3279**（v58.1.0 完了時点の実績値）
  - サブロードマップ記載値「3274 + 2 = 3276」は v58.0.0 実装前の予測に基づく誤値。
    v58.1.0 実績（3279）が正確なベース。
- 目標テスト数: **3281**（+2）、かつ `cargo test` failures=0
- rolling チェック更新: **5 件**（v56300 / v56900 / v57000 / v57900 / v58000）を `58.1.0` → `58.2.0`
  — 宣言バージョン以外でも Cargo.toml 版数変更に追随して全件更新が必要（v58.1.0 で確認済み）

---

## スコープ外項目

| 項目 | 備考 |
|---|---|
| カナリアの実際のトラフィック制御 | 出力文字列のモックで検証 |
| `--canary-weight` の範囲バリデーション（0-100） | v58.x で拡張予定 |
| `--canary-weight` 非数値パース失敗時の挙動 | `parse::<u32>().ok()` で失敗した場合はデフォルト 10 にサイレントフォールバック（意図的） |
| `fav deploy status` のヘルス取得（HTTP 通信） | 文字列出力のみ |
| `abort` と `rollback` の統合 | 別サブコマンドとして独立実装 |
| canary 出力にバージョン番号を埋め込む | ロードマップ例示（`Deploying v58.2.0 to canary`）より簡略化（テストは exit code のみ検証） |

---

## 実装内容

### 1. `fav/Cargo.toml` バージョン更新

```toml
version = "58.2.0"
```

### 2. `fav/src/driver.rs` — `cmd_deploy_strategy` 拡張

v58.1.0 で追加した `cmd_deploy_strategy` に canary / promote / abort / status ブランチを追加する。

```rust
pub fn cmd_deploy_strategy(args: &[String]) -> i32 {
    let sub = args.first().map(|a| a.as_str()).unwrap_or("");

    // rollback / promote / abort / status サブコマンド
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
}
```

### 3. `fav/src/main.rs` — ディスパッチ条件に `promote` / `abort` / `status` を追加

```rust
// 変更前
if sub == "rollback" || has_strategy {

// 変更後
let is_canary_sub = matches!(sub, "rollback" | "promote" | "abort" | "status");
if is_canary_sub || has_strategy {
```

### 4. `fav/src/driver.rs` — `v58200_tests` 追加

`v58100_tests` の直前に挿入する。

```rust
// -- v58200_tests (v58.2.0) -- カナリアリリース --
#[cfg(test)]
mod v58200_tests {
    use super::cmd_deploy_strategy;

    #[test]
    fn cmd_deploy_canary_weight() {
        // canary deploy with weight returns exit code 0
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
        // promote subcommand returns exit code 0
        let args = vec!["promote".to_string()];
        let code = cmd_deploy_strategy(&args);
        assert_eq!(code, 0, "canary promote should succeed");
    }
}
```

### 5. Rolling バージョンチェック更新（5 件）

```
v56300_tests::cargo_toml_version_is_56_3_0  : "58.1.0" → "58.2.0"
v56900_tests::cargo_toml_version_is_56_9_0  : "58.1.0" → "58.2.0"
v57000_tests::cargo_toml_version_is_57_0_0  : "58.1.0" → "58.2.0"
v57900_tests::cargo_toml_version_is_57_9_0  : "58.1.0" → "58.2.0"
v58000_tests::cargo_toml_version_is_58_0_0  : "58.1.0" → "58.2.0"
```

---

## テスト仕様

| テスト名 | 検証内容 |
|---|---|
| `cmd_deploy_canary_weight` | `cmd_deploy_strategy(["--strategy", "canary", "--canary-weight", "10"])` が 0 を返す |
| `cmd_deploy_canary_promote` | `cmd_deploy_strategy(["promote"])` が 0 を返す |

---

## 完了条件

- `cargo build` コンパイルエラーなし
- `cargo test` 全通過（**3281 tests passed, 0 failed**、ベース 3279 + 2）
- `cargo clippy -- -D warnings` クリーン
- `v58200_tests` 2 件全 pass
- `fav/src/main.rs` のディスパッチに `promote` / `abort` / `status` が追加されている
- rolling チェック 5 件が `"58.2.0"` になっている
- `v58100_tests::cmd_deploy_unknown_strategy` のテスト引数を `"canary"` → `"invalid-strategy"` に更新済み（canary が valid strategy になるため）

---

## 備考

- `cmd_deploy_strategy` の全サブコマンドは `args.first()` で判定する（v58.1.0 設計踏襲）
- `--canary-weight` が未指定の場合は `10` をデフォルト値とする
- rolling チェックは宣言バージョン以外でも全件更新が必要（v58.1.0 で確認した教訓）
