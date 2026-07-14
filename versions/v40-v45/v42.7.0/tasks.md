# v42.7.0 タスクリスト

**ステータス**: COMPLETE
**目標テスト数**: 2893（前バージョン 2891 + 2）
**実績テスト数**: 2893（v42700_tests 2/2 PASS）
※ ロードマップ記載の 2892 は旧 v42.6.0 基準（2890+2）の誤差。実績は 2893。

---

## T0 — 事前確認

- [x] `cargo test` が 2891 tests / 0 failures であることを確認
- [x] `fav/Cargo.toml` version が `42.6.0` であることを確認
- [x] `fav/src/main.rs` の `Some("profile")` アーム閉じ行番号を記録（line 1642）
- [x] `fav/src/driver.rs` の `cmd_profile_compare` 末尾行番号を記録（line 12183）
- [x] `fav/src/driver.rs` の `v42600_tests` 冒頭行番号を記録（line 44654）
- [x] `versions/roadmap/roadmap-v42.1-v43.0.md` に v42.7.0 エントリが存在することを確認

---

## T1 — `main.rs` — `Some("monitor")` アーム追加

- [x] `Some("profile")` アームの直後（`Some("infer")` の直前）に `Some("monitor") => cmd_monitor(&args),` を追加

---

## T2 — `main.rs` — `cmd_monitor` インポート追加・`driver.rs` に関数追加

- [x] `main.rs` のインポート行に `cmd_monitor` を追加
- [x] `driver.rs` の `cmd_profile_compare` の直後（`fn extract_profile_stages` の直前）に `pub fn cmd_monitor` を追加
- [x] stub 出力: `"fav monitor — pipeline metrics (stub)"` の旨を println!
- [x] 引数はすべて無視（`_args: &[String]`）

---

## T3 — `driver.rs` — `v42700_tests` モジュール追加

- [x] `v42600_tests` の `cargo_toml_version_is_42_6_0` をスタブ化（`assert!(true)`）
- [x] `v42700_tests` を `v42600_tests` の直前（降順）に挿入
- [x] `cargo_toml_version_is_42_7_0` テスト追加
- [x] `monitor_cmd_exists` テスト追加:
  - `main_src.contains("cmd_monitor(&args)")`
  - `main_src.contains("cmd_monitor")`

---

## T4 — `fav/Cargo.toml` バージョン bump

- [x] `version = "42.6.0"` → `"42.7.0"`

---

## T5 — `CHANGELOG.md` 更新

- [x] `[v42.7.0]` エントリを `[v42.6.0]` の直前に追加
- [x] `fav monitor` stub・v43.x 延期・未知引数無視の旨を記載

---

## T6 — テスト実行・確認

- [x] `cargo test` 実行
- [x] failures = 0 を確認
- [x] テスト数 = 2893 を確認（2891 + 2 件）
- [x] `v42700_tests` 2 件 pass を確認

---

## T7 — バージョン管理ドキュメント更新

- [x] `versions/current.md` を v42.7.0（最新安定版、2893 tests）・v42.8.0（次に切る版）に更新
- [x] `versions/roadmap/roadmap-v42.1-v43.0.md` の v42.7.0 を完了済みにマーク（`✅ COMPLETE（2026-07-12）`）
- [x] 同ロードマップの v42.7.0 推定テスト数を `2892` → 実績 `2893` に修正
- [x] `versions/v40-v45/v42.7.0/tasks.md` を COMPLETE ステータスに更新（全チェックボックス `[x]`）

---

## 最終ステータス

- [x] 全タスク完了

## spec-reviewer 指摘・対応記録

- [HIGH-1]: ロードマップテスト数が 2892（旧基準）で実態 2893 と不一致 → spec.md 影響範囲・tasks T7 に roadmap 実績値修正を明示
- [HIGH-2]: spec.md 影響範囲に `roadmap-v42.1-v43.0.md` が欠落 → spec.md 影響範囲テーブルに追加
- [HIGH-3]: `monitor_cmd_exists` の `contains("monitor")` が弱い → `contains("cmd_monitor(&args)")` に強化（spec/plan/tasks 更新）
- [MED-2]: MDX 非スコープ根拠の明示漏れ → spec §非スコープに `site/content/docs/tools/monitor.mdx` は v43.x 以降を追加
- [MED-3]: 不正引数挙動が spec に未定義 → spec §2 `cmd_monitor` コメントに「引数は全無視、v43.x で --interval 追加時に引数解析実装」を明記

## code-reviewer 指摘・対応記録

- [MED-1]: `include_str!("../src/main.rs")` がコンベンション（`"main.rs"`）と不一致 → `include_str!("main.rs")` に修正
- [LOW-1]: `monitor_cmd_exists` の第2アサート `contains("cmd_monitor")` が `contains("cmd_monitor(&args)")` のスーパーセットで冗長 → 削除
- [LOW-2]: `cmd_monitor` の println! が英語のみで CHANGELOG 日本語と乖離 → stub 出力は英語統一で許容（仕様上問題なし）
