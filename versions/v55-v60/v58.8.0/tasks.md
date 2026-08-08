# v58.8.0 Tasks — ドキュメントサイト Governance & Deployment 記事

Date: 2026-07-29
Status: COMPLETE（2026-07-29）— 3302 tests passed, 0 failed

---

## T0: 事前確認

- [x] `cargo test` でベースラインが 3300 tests passed, 0 failed であることを確認
- [x] `fav/Cargo.toml` のバージョンが `"58.7.0"` であることを確認
- [x] `site/content/docs/enterprise/` ディレクトリが存在することを確認
- [x] `site/content/cookbook/` ディレクトリが存在することを確認
- [x] `site/content/docs/enterprise/deployment.mdx` がまだ存在しないことを確認
- [x] `site/content/docs/enterprise/governance.mdx` がまだ存在しないことを確認
- [x] `site/content/cookbook/multi-env-pipeline.mdx` がまだ存在しないことを確認
- [x] `grep -c '58\.7\.0' fav/src/driver.rs` でローリング文字列件数を確認

---

## T1: Cargo.toml バージョン更新

- [x] `fav/Cargo.toml`: `version = "58.7.0"` → `"58.8.0"`

---

## T2: roadmap 更新

- [x] v58.9.0 のベース数を `3297 → 3302`、目標を `3299 → 3304` に修正

---

## T3: deployment.mdx 作成

- [x] `site/content/docs/enterprise/deployment.mdx` を新規作成
  - `"Blue/Green"` を含む（テストで検証）
  - `"canary"` を含む
  - `"HA"` / `"--ha"` を含む
  - `--strategy blue-green` の bash 例を含む

---

## T4: governance.mdx 作成

- [x] `site/content/docs/enterprise/governance.mdx` を新規作成
  - `"Policy-as-Code"` を含む（テストで検証）
  - `"Schema Migration"` を含む
  - `"Data Catalog"` を含む
  - `"E0426"` を含む

---

## T5: multi-env-pipeline.mdx 作成

- [x] `site/content/cookbook/multi-env-pipeline.mdx` を新規作成
  - `"--env"` / `"dev"` / `"staging"` / `"prod"` を含む

---

## T6: driver.rs テストモジュール追加

- [x] `v58800_tests` モジュールを v58700_tests の直前に挿入
  - **注意**: MDX ファイル（T3〜T5）を先に作成してから追加する（`include_str!` はコンパイル時解決）
  - [x] `docs_deployment_page_exists`: `include_str!("../../site/content/docs/enterprise/deployment.mdx")` が `"Blue/Green"` を含む
  - [x] `docs_governance_page_exists`: `include_str!("../../site/content/docs/enterprise/governance.mdx")` が `"Policy-as-Code"` を含む
  - [x] `use super::*` は不要（`include_str!` のみ使用）

---

## T7: driver.rs ローリングチェック更新

- [x] `version = \"58.7.0\"` → `\"58.8.0\"` に一括更新（5 件、`replace_all`）
- [x] failure メッセージ `"Cargo.toml version should be 58.7.0"` → `"58.8.0"` に更新（5 件）

---

## T8: テスト実行・確認

- [x] `cargo test -j 8 -- --test-threads=8` を実行
- [x] `docs_deployment_page_exists` pass を確認
- [x] `docs_governance_page_exists` pass を確認
- [x] 総テスト数 **3302** tests passed, 0 failed を確認

---

## T9: 事後処理

- [x] `CHANGELOG.md` に v58.8.0 エントリを追加
- [x] `versions/current.md` を v58.8.0 / 3302 tests に更新
- [x] `versions/roadmap/roadmap-v58.1-v59.0.md` の v58.8.0 実績欄を更新
- [x] v58.9.0 ベース数を実績値に合わせて再確認・修正（code-review でテスト数が増加した場合、roadmap の v58.9.0 ベース数・目標数を更新する）
- [x] このファイル（tasks.md）を COMPLETE ステータスに更新

---

## コードレビュー記録

実装完了 — code-review 前（3302 tests passed, 0 failed）

---

Status: COMPLETE（2026-07-29）— 3302 tests passed, 0 failed
