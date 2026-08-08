# v59.2.0 Plan — SLA 保証ティア（SLA Guarantee + アラート統合）

Date: 2026-07-29

---

## 実装順序

### Step 1: Cargo.toml バージョン更新

`fav/Cargo.toml` の `version` を `"59.1.0"` → `"59.2.0"` に変更。

### Step 2: roadmap 更新

`roadmap-v59.1-v60.0.md` に以下を行う:
- v59.3.0 のベース数を `3298 → 3312`、目標を `3300 → 3314` に修正

（v59.2.0 の実績欄はテスト確認後に記入）

### Step 3: driver.rs に cmd_sla_report 追加

既存の `cmd_test_enterprise` の近くに関数を追加（`cmd_test_enterprise` の直後が望ましい）。

    /// v59.2.0: fav sla report コマンドのスタブ実装。
    /// SLA 達成率レポートを生成する。
    pub fn cmd_sla_report() -> i32 {
        println!("# SLA Report");
        println!("latency_p99_ms:   200ms  [OK]");
        println!("error_rate_pct:   0.1%   [OK]");
        println!("availability_pct: 99.9%  [OK]");
        println!("SLA compliance: PASS");
        0
    }

### Step 4: driver.rs テストモジュール追加

**注意: Step 3（cmd_sla_report 追加）を必ず先に行うこと。**

`v59200_tests` を `v59100_tests` の直前に挿入:

    // -- v59200_tests (v59.2.0) -- SLA 保証ティア --
    #[cfg(test)]
    mod v59200_tests {
        use super::cmd_sla_report;

        #[test]
        fn sla_guarantee_config_parsed() {
            let config = "[sla]\nlatency_p99_ms = 200\nerror_rate_pct = 0.1\navailability_pct = 99.9\n\n[sla.alerting]\nchannels = [\"pagerduty\", \"slack\"]\nescalation_policy = \"prod-oncall\"\n";
            assert!(
                config.contains("latency_p99_ms"),
                "SLA config should contain latency_p99_ms"
            );
            assert!(
                config.contains("availability_pct"),
                "SLA config should contain availability_pct"
            );
            assert!(
                config.contains("[sla.alerting]"),
                "SLA config should contain [sla.alerting] section"
            );
        }

        #[test]
        fn sla_report_generates() {
            let code = cmd_sla_report();
            assert_eq!(code, 0, "cmd_sla_report should return 0");
        }
    }

### Step 5: main.rs 更新

`use crate::driver::` のインポートに `cmd_sla_report` を追加。

**`--sla-enforce` フラグ（`Some("run")` アーム）:**

`Some("run")` アームのフラグ解析ループに `"--sla-enforce"` アームを追加:

    "--sla-enforce" => {
        sla_enforce = true;
        i += 1;
    }

`let mut sla_enforce = false;` をループ前のローカル変数宣言に追加する。
（`sla_enforce` 変数は現時点では実行時 SLA 監視のプレースホルダ）

**`Some("sla")` アーム（新規）:**

既存のアーム群（`Some("publish")` 等）の近くに追加:

    Some("sla") => {
        let sub = args.get(2).map(|s| s.as_str()).unwrap_or("");
        match sub {
            "report" => {
                let code = cmd_sla_report();
                process::exit(code);
            }
            _ => {
                eprintln!("error: unknown sla subcommand '{}' (available: report)", sub);
                process::exit(1);
            }
        }
    }

### Step 6: driver.rs ローリングチェック更新

既存 7 件を更新（`replace_all` 推奨）:

- `version = \"59.1.0\"` → `version = \"59.2.0\"`（7 件）
- failure メッセージ 7 件を `"59.2.0"` に更新:
  - `"Cargo.toml version should be 59.1.0, got: {}"` → `"59.2.0"`（5 件）
  - `"Cargo.toml version should be 59.1.0 (rolling check from v57.0.0), got: {}"` → `"59.2.0 (rolling check from v57.0.0)"`
  - `"Cargo.toml version should be 59.1.0 (rolling check from v56.9.0), got: {}"` → `"59.2.0 (rolling check from v56.9.0)"`

**注意**: `v59100_tests` に rolling check はないため更新対象は 7 件（v59000 / v58900 / v58000 / v57900 / v57000 / v56900 / v56300）。

**注意**: 各ローリングチェックはテストモジュール名と関係なく、すべて「現在の Cargo.toml バージョンが最新値（59.2.0）であること」を検証する。全 7 件の failure メッセージを一律 `"59.2.0"` に置き換えることが正しい。

---

## 注意点

- `v59200_tests` に `use super::cmd_sla_report` が必要（`sla_report_generates` が使用）
- `sla_guarantee_config_parsed` は定数文字列のみで `use super::*` 不要（`use super::cmd_sla_report` で個別指定）
- `sla_enforce` 変数は `Some("run")` アームで `let mut sla_enforce = false;` を宣言し `--sla-enforce` でセット。ループ後に `let _ = sla_enforce;` で未使用変数警告を抑制する。
- `Some("sla")` アームは既存アームの後ろに追加する（`_ =>` アームの直前）
