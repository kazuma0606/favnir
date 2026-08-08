# v58.9.0 Tasks — 安定化・コードフリーズ（Governance & Deployment 2.0 前調整）

Date: 2026-07-29
Status: COMPLETE（2026-07-29）— 3304 tests passed, 0 failed

---

## T0: 事前確認

- [x] `cargo test` でベースラインが 3302 tests passed, 0 failed であることを確認
- [x] `fav/Cargo.toml` のバージョンが `"58.8.0"` であることを確認
- [x] `site/content/docs/governance-overview.mdx` がまだ存在しないことを確認
- [x] `cargo clippy -- -D warnings` でエラーがないことを確認
- [x] `grep -c '58\.8\.0' fav/src/driver.rs` でローリング文字列件数を確認

---

## T1: Cargo.toml バージョン更新

- [x] `fav/Cargo.toml`: `version = "58.8.0"` → `"58.9.0"`

---

## T2: roadmap 更新

- [x] v59.0.0 の `ベース 3299 + 4 = 3303` → `ベース 3304 + 4 = 3308` に修正
- [x] v59.0.0 の `テスト数 ≥ 3303` → `テスト数 ≥ 3308` の記述も更新（見落とし注意）

---

## T3: governance-overview.mdx 作成

- [x] `site/content/docs/governance-overview.mdx` を新規作成
  - `"Governance & Deployment"` を含む（テストで検証）
  - `# Governance & Deployment Overview` 見出し
  - v58.1〜v58.8 の機能一覧（Blue/Green・カナリア・HA・Schema Migration・Data Catalog・Policy-as-Code・マルチ環境設定）

---

## T4: driver.rs テストモジュール追加

- [x] main.rs を変更していないことを確認（安定化専用バージョン — main.rs 変更は不要）
- [x] `v58900_tests` モジュールを v58800_tests の直前に挿入
  - **注意**: MDX ファイル（T3）を先に作成してから追加する（`include_str!` はコンパイル時解決）
  - [x] `cargo_toml_version_is_58_9_0`: `include_str!("../Cargo.toml")` が `"version = \"58.9.0\""` を含む
  - [x] `governance_overview_exists`: `include_str!("../../site/content/docs/governance-overview.mdx")` が `"Governance & Deployment"` を含む
  - [x] `use super::*` は不要（`include_str!` のみ使用）

---

## T5: driver.rs ローリングチェック更新

- [x] `version = \"58.8.0\"` → `\"58.9.0\"` に一括更新（5 件、`replace_all`）
- [x] failure メッセージ `"Cargo.toml version should be 58.8.0"` → `"58.9.0"` に更新（5 件）

---

## T6: テスト実行・確認

- [x] `cargo test -j 8 -- --test-threads=8` を実行
- [x] `cargo_toml_version_is_58_9_0` pass を確認
- [x] `governance_overview_exists` pass を確認
- [x] 総テスト数 **3304** tests passed, 0 failed を確認
- [x] `failures=0` であることを確認（v58100〜v58800 を含む全既存テストが通過）

---

## T7: 事後処理

- [x] `CHANGELOG.md` に v58.9.0 エントリを追加
- [x] `versions/current.md` を v58.9.0 / 3304 tests に更新
- [x] `versions/roadmap/roadmap-v58.1-v59.0.md` の v58.9.0 実績欄を更新
- [x] v59.0.0 ベース数を実績値に合わせて再確認・修正（code-review でテスト数が増加した場合、roadmap の v59.0.0 ベース数・目標数を更新する）
- [x] このファイル（tasks.md）を COMPLETE ステータスに更新

---

## コードレビュー記録

- [LOW] `governance-overview.mdx` の `/cookbook/multi-env-pipeline` リンク先が CI テストで未保証
  → `multi-env-pipeline.mdx` は v58.8.0 spec で「Rust テスト対象外（人手確認のみ）」と設計済み。ファイルの存在を手動確認（EXISTS）。テスト追加はスコープ外のため対応なし。

最終テスト数: 3304 tests passed, 0 failed（code-review 対応なし）

---

Status: COMPLETE（2026-07-29）— 3304 tests passed, 0 failed
