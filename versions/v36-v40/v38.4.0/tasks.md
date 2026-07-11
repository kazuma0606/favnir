# v38.4.0 タスクリスト — LSP AI 補完（オプション）

## ステータス: COMPLETE

> ロードマップ整合: `roadmap-v38.1-v39.0.md` の v38.4.0（「LSP AI 補完（オプション）」）に沿ったバージョン。

## T0: 事前確認

- [x] `cargo test` の実測通過数を確認（目安: 2754（v38.3.0 完了時点の実績値））し、実測値をここに記録: 2754
- [x] Cargo.toml バージョンが `38.3.0` であることを確認
- [x] `v38300_tests::cargo_toml_version_is_38_3_0` がライブアサーション（`assert!(cargo.contains("38.3.0"), ...)`）であることを確認し、行番号を記録: 43711
- [x] `driver.rs` に `v38400_tests` モジュールが存在しないことを確認（今回新規作成）
- [x] `v38300_tests` の閉じ `}` の行番号を確認し、ここに記録: 43745
- [x] `CHANGELOG.md` に `[v38.4.0]` エントリが存在しないことを確認（今回新規作成）
- [x] `toml.rs` に `parse_lsp_ai_config` が存在しないことを確認（今回追加）
- [x] `versions/current.md` の最新安定版が `v38.3.0`・次バージョンが `v38.4.0` であることを確認
- [x] `versions/roadmap/roadmap-v38.1-v39.0.md` の v38.4.0 が未完了（✅ なし）であることを確認（T9 で更新）

## T1: CHANGELOG.md に [v38.4.0] エントリを追加

- [x] `## [v38.3.0]` の直前に `## [v38.4.0]` エントリを挿入
- [x] 日付を `2026-07-10` に設定
- [x] セパレータが `—`（全角ダッシュ U+2014）形式であることを確認

## T2: `fav/src/toml.rs` — `LspAiConfig` + `parse_lsp_ai_config` 追加

- [x] Read で `toml.rs` の末尾を確認（`parse_fav_toml_pub` の行番号を確認）
- [x] spec.md §1 の内容で `LspAiConfig` struct + `parse_lsp_ai_config` + `parse_lsp_ai_enabled` を末尾に追加
- [x] `pub struct LspAiConfig { pub enabled: bool }` を含む
- [x] `pub fn parse_lsp_ai_config(toml: &str) -> LspAiConfig` を含む
- [x] `[lsp.ai]` セクション検出 + 別セクションに入ったら `in_lsp_ai = false` にリセット

## T3: `driver.rs` — `v38300_tests::cargo_toml_version_is_38_3_0` をスタブ化

- [x] Read で `cargo_toml_version_is_38_3_0` の行番号を確認
- [x] ライブアサーション → `// Stubbed: version bumped to 38.4.0 — assertion intentionally removed` に変更
- [x] **注意:** `changelog_has_v38_3_0` / `generate_csv_fn_exists` / `csv_to_favnir_basic` はスタブ化しない
- [x] スタブ形式が前バージョンのスタブと一致していることを確認

## T4: `driver.rs` — `v38400_tests` モジュールを新規追加（T1・T2 完了後に実施）

- [x] T1（CHANGELOG 追加）と T2（toml.rs 追加）が完了していることを確認してから着手
- [x] `v38300_tests` の閉じ `}` の行番号（T0 で記録済み）を Read で特定してから Edit を実行
- [x] `v38300_tests` の閉じ `}` の後に `v38400_tests` モジュールを追加（4 テスト）
  - [x] `cargo_toml_version_is_38_4_0`
  - [x] `changelog_has_v38_4_0`
  - [x] `lsp_ai_enabled_when_configured`（`[lsp.ai]\nenabled = true` → `cfg.enabled == true`）
  - [x] `lsp_ai_disabled_by_default`（`[lsp.ai]` なし → `cfg.enabled == false`）

## T5: バージョン更新（T1〜T4 すべて完了後）

- [x] `fav/Cargo.toml` バージョンを `38.4.0` に更新

## T6: テスト実行

- [x] T5（Cargo.toml 更新）が完了していることを確認してから着手
- [x] `cargo test` 全通過 — ≥ 2758 passed; 0 failed — 実測: 2758 passed, 0 failed
- [x] `v38400_tests` の 4 テストがすべて pass
- [x] `cargo_toml_version_is_38_4_0` が pass
- [x] `changelog_has_v38_4_0` が pass
- [x] `lsp_ai_enabled_when_configured` が pass
- [x] `lsp_ai_disabled_by_default` が pass

## T7: ドキュメント更新（T6 完了後）

- [x] `versions/v36-v40/v38.4.0/tasks.md` を COMPLETE ステータスに更新（T0〜T7 全チェックボックスを `[x]` に）
- [x] `versions/current.md` を v38.4.0（最新安定版）・v38.5.0（次バージョン）に更新
- [x] `versions/roadmap/roadmap-v38.1-v39.0.md` の v38.4.0 を完了済みにマーク（✅）し、テスト件数を「設定解析テスト 4 件（meta 2 件 + 機能 2 件）」に更新（実装前に「2 件」→「4 件」修正済みのため ✅ マークのみ追加でよい）
- [x] roadmap の v38.4.0 行を Read で確認し ✅ が含まれることをここに記録: ✅ 確認: ### v38.4.0 — LSP AI 補完（オプション）✅
- [x] roadmap の v38.4.0 行を Read で確認し「4 件」が含まれることをここに記録: テスト件数 4 件確認: **完了条件**: 設定解析テスト 4 件（meta 2 件 + 機能 2 件）（2758 tests passed, 0 failed）

---

## 完了条件チェックリスト（spec.md 対応）

| # | spec.md 完了条件 | 確認方法 |
|---|---|---|
| 1 | `toml.rs` に `pub fn parse_lsp_ai_config` が含まれる | `lsp_ai_enabled_when_configured` テスト ✅ |
| 2 | `[lsp.ai] enabled = true` で `cfg.enabled == true` | `lsp_ai_enabled_when_configured` テスト ✅ |
| 3 | `[lsp.ai]` 非設定時に `cfg.enabled == false` | `lsp_ai_disabled_by_default` テスト ✅ |
| 4 | `CHANGELOG.md` に `[v38.4.0]` が含まれる | `changelog_has_v38_4_0` テスト ✅ |
| 5 | `Cargo.toml` バージョンが `38.4.0` | `cargo_toml_version_is_38_4_0` テスト ✅ |
| 6 | `cargo test` 全通過（failures=0 かつテスト数 ≥ 2758） | 実測: 2758 passed, 0 failed ✅ |
| 7 | `roadmap-v38.1-v39.0.md` の v38.4.0 が ✅ かつテスト件数が 4 件 | 更新済み ✅ |
| 8 | `versions/current.md` が v38.4.0（最新安定版）に更新されている | 更新済み ✅ |
