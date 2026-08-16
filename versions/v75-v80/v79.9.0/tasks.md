# v79.9.0 タスクリスト — 安定化・コードフリーズ（Favnir 3.0 前最終調整）

Date: 2026-08-16
Status: COMPLETE

---

## T0: 事前確認

- [x] `fav/Cargo.toml` のバージョンが `79.8.0` であることを確認
- [x] `cargo test` が全 pass（3803 tests = v79.8.0 完了後の実測ベース）であることを確認
- [x] `fav/tmp/hello.fav` が存在することを確認
- [x] `infra/e2e-demo/favnir3-showcase/pipeline.fav` が存在することを確認
- [x] `infra/e2e-demo/favnir3-showcase/contract.fav` が存在することを確認
- [x] `infra/e2e-demo/favnir3-showcase/fav.toml` が存在することを確認
- [x] `infra/e2e-demo/favnir3-showcase/README.md` が存在することを確認

---

## T1: E2E ショーケース統合確認（手動チェック）

- [x] `infra/e2e-demo/favnir3-showcase/pipeline.fav` に全 4 ステージが揃っていることを確認
  - `load_with_freshness`（v79.2 Temporal）
  - `load_with_provenance`（v79.3 Provenance）
  - `join_stage`（v79.5 Execution Effects）
  - `showcase_pipeline`（v79.1 基盤）
- [x] `infra/e2e-demo/favnir3-showcase/contract.fav` に `Favnir3ShowcaseContract` / `invariant` / `verifiable_enabled` が含まれることを確認（v79.4 Verifiable）
- [x] `infra/e2e-demo/favnir3-showcase/fav.toml` に `effects.cached` / `effects.adaptive` が含まれることを確認
- [x] `infra/e2e-demo/favnir3-showcase/README.md` に `Favnir 3.0` が含まれることを確認

---

## T2: CHANGELOG.md 更新（テスト追加より先）

- [x] `CHANGELOG.md` の先頭に v79.9.0 エントリを追加する（形式: `## [v79.9.0] — 2026-08-16 — 安定化・コードフリーズ（Favnir 3.0 前最終調整）`）
- [x] Stability セクション（全スプリント統合確認）を含める
- [x] Tests セクション（2 件）を含める

---

## T3: driver.rs — テストモジュール追加

- [x] `fav/src/driver.rs` の末尾に `// --- v79.9.0: 安定化・コードフリーズ ---` コメントを追加する
- [x] `v799000_tests` モジュールを追加する（`use super::*` 不要）
- [x] モジュール先頭に `const PIPELINE` / `const CONTRACT` / `const CONFIG` / `const README` を配置する
- [x] `favnir3_full_sprint_all_stable` テストを実装する
  - Temporal: `load_with_freshness` / `FreshnessPolicy` を assert
  - Provenance: `load_with_provenance` / `OpenLineage` を assert
  - Verifiable: `Favnir3ShowcaseContract` / `invariant` を assert
  - Execution Effects: `join_stage` / `!Adaptive` を assert
- [x] `favnir3_e2e_showcase_runs` テストを実装する
  - `showcase_pipeline` / `verifiable_enabled` / `favnir3-showcase` / `effects.cached` / `effects.adaptive` / `Favnir 3.0` を assert
  - `validate_showcase_contract` は v79.1.0 テスト済みのため使わない（重複防止）
- [x] `cargo test v799000` で 2 件が pass することを確認する

---

## T4: Cargo.toml バージョン更新

- [x] `fav/Cargo.toml` の `version` を `"79.8.0"` → `"79.9.0"` に変更する
- [x] driver.rs 内の escaped `\"79.8.0\"` を `\"79.9.0\"` に一括更新（sed）
- [x] driver.rs 内の unescaped エラーメッセージ `79.8.0` を `79.9.0` に更新する
- [x] **更新後に** `grep -c "79\.8\.0" /c/Users/yoshi/favnir/fav/src/driver.rs` を実行し **出力が 1** であることを確認する
  - 残るのは `// --- v79.8.0: ドキュメント完全化（v3 リファレンス）---` コメント行の 1 件のみ

---

## T5: versions/current.md 更新

- [x] `## 進行中バージョン` 欄を `**v79.9.0**（安定化・コードフリーズ）` に更新する
- [x] `## 次に切る版` 欄を `**v80.0.0**（Favnir 3.0 宣言 ★クリーンアップ）` に更新する

---

## T6: 最終確認

- [x] `cargo test` が全 pass であることを確認する（3805 tests）
- [x] `cargo test v799000` で 2 件が pass することを確認する
- [x] `fav/Cargo.toml` のバージョンが `79.9.0` であることを確認する
- [x] `fav/Cargo.lock` が cargo test 実行時に自動更新されていることを確認する
- [x] `CHANGELOG.md` の先頭が `[v79.9.0]` であることを確認する

---

## 完了チェックリスト

- [x] 全タスク（T0〜T6）が完了している
- [x] `favnir3_full_sprint_all_stable` が pass
- [x] `favnir3_e2e_showcase_runs` が pass
- [x] テスト総数: 3805（+2）
- [x] `CHANGELOG.md` の先頭が `[v79.9.0]` であることを確認済み
- [x] `fav/Cargo.toml` version = "79.9.0" に更新済み
- [x] `versions/current.md` が v79.9.0 に更新済み
- [x] `changelog_has_v79_9_0` テストの追加: 対象外（x.0.0 宣言バージョンのみに追加する慣例）
