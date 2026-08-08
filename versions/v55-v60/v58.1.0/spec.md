# Spec — v58.1.0 — Blue/Green デプロイメントサポート

## 概要

`fav deploy --strategy blue-green` コマンドと `fav deploy rollback` コマンドを追加する。
2 スロット（blue / green）の切り替えロジックを `driver.rs` に実装し、
`infra/deploy/blue-green/` に Terraform テンプレートのスタブを追加する。
Rust テスト 2 件（`cmd_deploy_blue_green` / `cmd_deploy_rollback`）で動作を検証する。

---

## ロードマップ参照

- `versions/roadmap/roadmap-v58.1-v59.0.md` — v58.1.0 セクション
- ベーステスト数: **3276**（v58.0.0 完了時点の実績値）
  - サブロードマップ記載値「3272 + 2 = 3274」は v58.0.0 実装前の予測値であり、実際の v58.0.0 実績（3276）とは 4 件差がある。本バージョンのベースは **3276** が正確。
- 目標テスト数: **3278**（+2）、かつ `cargo test` failures=0

---

## スコープ外項目

| 項目 | 備考 |
|---|---|
| 実際の AWS/GCP デプロイ実行 | CLI コマンド dispatch とテストのみ |
| Blue/Green インフラの完全 Terraform 実装 | `infra/deploy/blue-green/main.tf` スタブのみ |
| ヘルスチェックの HTTP 通信 | 出力文字列のモックで検証 |
| AST / parser 変更 | `deploy` は CLI コマンド、言語機能ではない |
| Rolling バージョンチェック更新 | 宣言バージョン（x.0.0 / x.9.0）のみ更新する慣例 |
| `--env` フラグの解析・環境別設定適用 | v58.6.0 で対応予定。本バージョンでは引数として受け取るが無視する |
| rollback 出力の厳密なロードマップ一致 | ロードマップ例示（1 行）より `Rollback complete.` を追加出力する拡張実装。テストは exit code 0 のみ検証 |

---

## 実装内容

### 1. `fav/Cargo.toml` バージョン更新

```toml
version = "58.1.0"
```

### 2. `fav/src/driver.rs` — `cmd_deploy` 関数追加

`fav deploy` コマンドのディスパッチアームを追加する。
既存の `Some("doctor")` アームの近傍に `Some("deploy")` アームを追加する。

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

main.rs の `Some("doctor")` アームの直後（または直前）に `Some("deploy")` を追加する。

### 3. `infra/deploy/blue-green/main.tf` — Terraform スタブ

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

### 5. `fav/src/driver.rs` — `v58100_tests` 追加

`v58000_tests` の直前に挿入する。

```rust
// -- v58100_tests (v58.1.0) -- Blue/Green デプロイメントサポート --
#[cfg(test)]
mod v58100_tests {
    use super::cmd_deploy;

    #[test]
    fn cmd_deploy_blue_green() {
        // blue-green deploy returns exit code 0
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
        // rollback subcommand returns exit code 0
        let args = vec!["rollback".to_string()];
        let code = cmd_deploy(&args);
        assert_eq!(code, 0, "deploy rollback should succeed");
    }
}
```

---

## テスト仕様

| テスト名 | 検証内容 |
|---|---|
| `cmd_deploy_blue_green` | `cmd_deploy(["--strategy", "blue-green", ...])` が 0 を返すことを検証 |
| `cmd_deploy_rollback` | `cmd_deploy(["rollback"])` が 0 を返すことを検証 |

---

## 完了条件

- `cargo build` コンパイルエラーなし
- `cargo test` 全通過（**3278 tests passed, 0 failed**、ベース 3276 + 2）
- `cargo clippy -- -D warnings` クリーン
- `v58100_tests` 2 件全 pass
- `infra/deploy/blue-green/main.tf` が存在する
- `fav/src/main.rs` に `Some("deploy")` ディスパッチアームが追加されている

---

## 備考

- `cmd_deploy` は `pub fn` として定義し、テストから `super::cmd_deploy` でアクセスできるようにする
- rollback は `args[0] == "rollback"` で判定（`fav deploy rollback` の呼び出し形式）
- rolling バージョンチェック（v56300 / v56900 / v57000 / v57900）は宣言バージョンのみ更新する慣例のため、v58.1.0 では更新しない
