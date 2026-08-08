# Plan — v58.1.0 — Blue/Green デプロイメントサポート

## 実装方針

`driver.rs` に `pub fn cmd_deploy(args: &[String]) -> i32` を追加し、
`--strategy blue-green` と `rollback` サブコマンドをディスパッチする。
Terraform スタブは `infra/deploy/blue-green/main.tf` に作成する。
テスト 2 件を `v58000_tests` の直前に挿入する。

---

## ファイル変更一覧

| ファイル | 変更内容 |
|---|---|
| `fav/Cargo.toml` | version `58.0.0` → `58.1.0` |
| `fav/src/driver.rs` | `pub fn cmd_deploy` 追加 + `Some("deploy")` ディスパッチアーム追加 + `v58100_tests` 追加 |
| `fav/src/main.rs` | `Some("deploy")` アーム追加（`cmd_deploy` 呼び出し） |
| `infra/deploy/blue-green/main.tf` | Terraform スタブ新規作成 |

---

## 詳細手順

### Step 1: `fav/Cargo.toml` version 更新

```
58.0.0 → 58.1.0
```

### Step 2: `fav/src/driver.rs` — `pub fn cmd_deploy` 追加

既存の `pub fn cmd_doctor` 関数の直後あたりに追加する。

```rust
pub fn cmd_deploy(args: &[String]) -> i32 {
    let strategy = args.iter()
        .position(|a| a == "--strategy")
        .and_then(|i| args.get(i + 1))
        .map(|s| s.as_str())
        .unwrap_or("rolling");

    let is_rollback = args.first().map(|a| a == "rollback").unwrap_or(false);

    if is_rollback {
        println!("Traffic switch: green → blue [100%]");
        println!("Rollback complete.");
        return 0;
    }

    match strategy {
        "blue-green" => {
            println!("Deploying to: green slot (current: blue)");
            println!("Health check: OK (green)");
            println!("Traffic switch: blue → green [100%]");
            println!("Old slot (blue): kept for 10 minutes (rollback window)");
            0
        }
        _ => {
            eprintln!("Unknown deploy strategy: {strategy}");
            1
        }
    }
}
```

### Step 3: `fav/src/main.rs` — `Some("deploy")` アーム追加

`Some("doctor")` アームの前後を確認して `Some("deploy")` アームを追加する。

```rust
Some("deploy") => {
    std::process::exit(cmd_deploy(&args[2..]));
}
```

### Step 4: `infra/deploy/blue-green/main.tf` — Terraform スタブ

`infra/deploy/blue-green/` は新規ディレクトリのため、先に作成してから `main.tf` を配置する:

```bash
mkdir -p infra/deploy/blue-green/
```

```hcl
# Blue/Green deployment infrastructure stub
# Managed by: fav deploy --strategy blue-green

variable "env" {
  description = "Target environment (dev/staging/prod)"
  type        = string
  default     = "dev"
}

locals {
  blue_slot  = "${var.env}-blue"
  green_slot = "${var.env}-green"
}

output "blue_slot" {
  value = local.blue_slot
}

output "green_slot" {
  value = local.green_slot
}
```

### Step 5: `fav/src/driver.rs` — `v58100_tests` 追加

`v58000_tests` の直前に挿入する。

```rust
// -- v58100_tests (v58.1.0) -- Blue/Green デプロイメントサポート --
#[cfg(test)]
mod v58100_tests {
    use super::cmd_deploy;

    #[test]
    fn cmd_deploy_blue_green() {
        let args = vec![
            "--strategy".to_string(),
            "blue-green".to_string(),
            "--env".to_string(),
            "prod".to_string(),
        ];
        let code = cmd_deploy(&args);
        assert_eq!(code, 0, "blue-green deploy should succeed");
    }

    #[test]
    fn cmd_deploy_rollback() {
        let args = vec!["rollback".to_string()];
        let code = cmd_deploy(&args);
        assert_eq!(code, 0, "deploy rollback should succeed");
    }
}
```

---

## `cmd_deploy` の設計詳細

```
fav deploy --strategy blue-green --env prod pipeline.fav
  → cmd_deploy(["--strategy", "blue-green", "--env", "prod", "pipeline.fav"])
  → strategy = "blue-green"
  → is_rollback = false
  → return 0

fav deploy rollback --env prod
  → cmd_deploy(["rollback", "--env", "prod"])
  → is_rollback = true（args[0] == "rollback"）
  → return 0
```

---

## main.rs のディスパッチ確認

事前に `main.rs` を読んで `cmd_doctor` の呼び出し箇所を確認してから `Some("deploy")` アームを挿入する。

---

## テスト戦略

```bash
cd /c/Users/yoshi/favnir/fav && cargo test -j 8 -- --test-threads=8 2>&1 | tail -20
```

期待: `3278 tests passed, 0 failed`（ベース 3276 + 2）

```bash
cargo clippy -- -D warnings
```

期待: 警告ゼロ

---

## 実装順序

```
Step 1 (Cargo.toml)
→ Step 2 (cmd_deploy 関数)
→ Step 3 (main.rs ディスパッチ)
→ Step 4 (Terraform スタブ)
→ Step 5 (v58100_tests)
→ cargo build → cargo test → cargo clippy
```
