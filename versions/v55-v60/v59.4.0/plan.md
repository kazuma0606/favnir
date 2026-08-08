# v59.4.0 Plan — Rune マーケットプレイス Phase 1（`fav marketplace`）

Date: 2026-07-29

---

## 実装順序

### Step 1: Cargo.toml バージョン更新

`fav/Cargo.toml` の `version` を `"59.3.0"` → `"59.4.0"` に変更。

### Step 2: roadmap 更新

`roadmap-v59.1-v60.0.md` に以下を行う:
- v59.5.0 のベース数を `3302 → 3316`、目標を `3304 → 3318` に修正

（v59.4.0 の実績欄はテスト確認後に記入）
（ベース数 3316 はテスト完了後に確定する暫定値。T7 テスト後に T8 で最終確認・修正する。）
（v59.6.0 以降のベース数は各バージョン着手時に都度修正する運用とする）

### Step 3: driver.rs に cmd_marketplace_list / cmd_marketplace_publish 追加

`cmd_cost_estimate` の直後に追加:

    /// v59.4.0: fav marketplace list コマンドのスタブ実装。
    pub fn cmd_marketplace_list() -> i32 {
        println!("Name          Author          Downloads  License");
        println!("kafka         favnir-official  12,450    MIT");
        println!("snowflake     favnir-official   8,320    MIT");
        println!("salesforce    acme-corp           920    Apache-2.0");
        0
    }

    /// v59.4.0: fav marketplace publish コマンドのスタブ実装。
    pub fn cmd_marketplace_publish(rune: &str) -> i32 {
        println!("Publishing rune '{}' to Favnir Marketplace...", rune);
        println!("[OK] Rune '{}' published successfully.", rune);
        0
    }

### Step 4: driver.rs テストモジュール追加

**注意: Step 3（関数追加）を必ず先に行うこと。**

`v59400_tests` を `v59300_tests` の直前に挿入:

    // -- v59400_tests (v59.4.0) -- Rune マーケットプレイス Phase 1 --
    #[cfg(test)]
    mod v59400_tests {
        use super::cmd_marketplace_list;
        use super::cmd_marketplace_publish;

        #[test]
        fn cmd_marketplace_list() {
            let code = super::cmd_marketplace_list();
            assert_eq!(code, 0, "cmd_marketplace_list should return 0");
        }

        #[test]
        fn cmd_marketplace_publish() {
            let code = super::cmd_marketplace_publish("my-rune");
            assert_eq!(code, 0, "cmd_marketplace_publish should return 0");
        }
    }

**注意**: テスト関数名が `cmd_marketplace_list` / `cmd_marketplace_publish` と driver.rs の pub fn と同名になる。`mod v59400_tests` の内側では `use super::cmd_marketplace_list` でインポートしているため、テスト関数内では `super::cmd_marketplace_list()` と明示する必要がある（関数名の衝突を避けるため）。もし `use super::...` と同名の `fn ...()` の共存でコンパイルエラーが発生する場合は、`use` 宣言を削除して `super::` 修飾のみに統一する。

### Step 5: main.rs 更新

`use crate::driver::` のインポートに `cmd_marketplace_list`・`cmd_marketplace_publish` を追加。

`Some("marketplace")` アームを `Some("cost-estimate")` の直前に追加:

    Some("marketplace") => {
        let sub = args.get(2).map(|s| s.as_str()).unwrap_or("");
        match sub {
            "list" => {
                let code = cmd_marketplace_list();
                process::exit(code);
            }
            "search" => {
                let query = args.get(3).map(|s| s.as_str()).unwrap_or("");
                println!("Searching marketplace for '{}'...", query);
                println!("kafka         favnir-official  12,450    MIT");
                process::exit(0);
            }
            "publish" => {
                let mut rune_name: &str = "";
                let mut i = 3usize;
                while i < args.len() {
                    match args[i].as_str() {
                        "--rune" => {
                            rune_name = args.get(i + 1).map(|s| s.as_str()).unwrap_or_else(|| {
                                eprintln!("error: --rune requires a value");
                                process::exit(1);
                            });
                            i += 2;
                        }
                        _ => { i += 1; }
                    }
                }
                if rune_name.is_empty() {
                    eprintln!("error: marketplace publish requires --rune <name>");
                    process::exit(1);
                }
                let code = cmd_marketplace_publish(rune_name);
                process::exit(code);
            }
            _ => {
                eprintln!("error: unknown marketplace subcommand '{}' (available: list, search, publish)", sub);
                process::exit(1);
            }
        }
    }

HELP テキストにも `marketplace` を追加（`sla report` の直前あたりに挿入）:

    marketplace list|search <q>|publish --rune <n>
                  Browse, search, and publish Runes on the Favnir Marketplace (v59.4.0+).

### Step 6: driver.rs ローリングチェック更新

既存 7 件を更新（`replace_all` 推奨）:

- `version = \"59.3.0\"` → `version = \"59.4.0\"`（7 件）
- failure メッセージ 7 件を `"59.4.0"` に更新:
  - `"Cargo.toml version should be 59.3.0, got: {}"` → `"59.4.0"`（5 件）
  - `"Cargo.toml version should be 59.3.0 (rolling check from v57.0.0), got: {}"` → `"59.4.0 (rolling check from v57.0.0)"`
  - `"Cargo.toml version should be 59.3.0 (rolling check from v56.9.0), got: {}"` → `"59.4.0 (rolling check from v56.9.0)"`

**注意**: `v59100_tests`〜`v59300_tests` に rolling check はないため更新対象は 7 件。

---

## 注意点

- テスト関数名が pub fn と同名（`cmd_marketplace_list` / `cmd_marketplace_publish`）になるため、テスト内では `super::cmd_marketplace_list()` と明示する
- `use super::cmd_marketplace_list` と `use super::cmd_marketplace_publish` の両方が必要。もし同名の `fn ...()` との共存でコンパイルエラーになる場合は `use` を削除して `super::` 修飾のみに統一する
- `rune_name: &str` のライフタイム: `let mut rune_name: &str = "";` と型注釈を明示する（`let mut provider: &str = "aws";` と同様）
- `Some("marketplace")` アームは `Some("cost-estimate")` の直前に配置する
