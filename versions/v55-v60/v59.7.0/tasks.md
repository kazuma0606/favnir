# v59.7.0 Tasks — README / MILESTONE Enterprise 1.0 整備

Date: 2026-07-30
Status: COMPLETE（2026-07-30）— 3322 tests passed, 0 failed

---

## T0: 事前確認

- [x] `cargo test` でベースラインが 3320 tests passed, 0 failed であることを確認
- [x] `fav/Cargo.toml` のバージョンが `"59.6.0"` であることを確認
- [x] `fav/src/driver.rs` に `v59700_tests` がまだ存在しないことを確認
- [x] `README.md` に `"Enterprise 1.0"` がまだ含まれていないことを確認
- [x] `MILESTONE.md` に `"v60.0.0（予定）"` がまだ含まれていないことを確認
- [x] `site/content/docs/enterprise/enterprise1-overview.mdx` がまだ存在しないことを確認
- [x] `grep -o '59\.6\.0' fav/src/driver.rs | wc -l` でローリング文字列件数を確認（14 件: assertion 7 件 + failure メッセージ 7 件）

---

## T1: Cargo.toml バージョン更新

- [x] `fav/Cargo.toml`: `version = "59.6.0"` → `"59.7.0"`

---

## T2: README.md — Enterprise 1.0 言及追加

- [x] 既存の v59.0 宣言ブロックの直後に Enterprise 1.0 スプリント言及を追加
  - `"Enterprise 1.0"` が含まれる文を追加（`readme_has_enterprise1_mention` テストの要件）
  - v56〜v60 機能サマリー（RBAC / Secrets / TLS / Audit / Compliance /
    Blue-Green Deploy / Cost Visibility / SLA Guarantee / Migration Toolkit / Enterprise Certify）を記載

---

## T3: MILESTONE.md — v60.0.0（予定）エントリ追加

- [x] `## v59.0.0` エントリの直前に `## v60.0.0（予定）— Enterprise 1.0` エントリを挿入
  - `"Enterprise 1.0"` を含む説明文を記載
- [x] **注意**: MILESTONE.md の内容は v60.0.0 の `milestone_has_enterprise1` テストで検証するため、v59700_tests には MILESTONE.md チェックを追加しない

---

## T4: enterprise1-overview.mdx 作成

- [x] `site/content/docs/enterprise/enterprise1-overview.mdx` を新規作成
  - `"Enterprise 1.0"` を含む（`docs_enterprise1_overview_exists` テストの要件）
  - Enterprise 1.0 機能一覧テーブル（10 機能）を記載
  - `fav certify --level enterprise` への言及を記載

---

## T5: driver.rs — v59700_tests 追加

- [x] **注意**: T2〜T4（README / MILESTONE / MDX 作成）を先に行うこと（`include_str!` はコンパイル時に読み込む）
- [x] `v59700_tests` モジュールを `v59600_tests` の直前（セパレータ行の後ろ、`// -- v59600_tests` コメントの前）に挿入
  - [x] `readme_has_enterprise1_mention` テスト: `include_str!("../../README.md").contains("Enterprise 1.0")` を検証
  - [x] `docs_enterprise1_overview_exists` テスト: `include_str!("../../site/content/docs/enterprise/enterprise1-overview.mdx").contains("Enterprise 1.0")` を検証
  - [x] `use super::*;` は不要（`include_str!` のみ使用）

---

## T6: driver.rs — ローリングチェック更新

- [x] `version = \"59.6.0\"` → `\"59.7.0\"` に一括更新（7 件）
- [x] failure メッセージ 7 件を `"59.7.0"` に更新（全 7 件とも同一パターン）:
  - `"Cargo.toml version should be 59.6.0"` → `"Cargo.toml version should be 59.7.0"`
  - 対象: `cargo_toml_version_is_59_0_0` / `v58900` / `v58000` / `v57900` / `v57000` / `v56900` / `v56300`
  - **注意**: `rolling check from` サフィックスは driver.rs に存在しない（特殊書式なし）
  - **注意**: `v59100_tests`〜`v59600_tests` は rolling check なし（対象外）

---

## T7: テスト実行・確認

- [x] `cargo test -j 8 -- --test-threads=8` を実行
- [x] `v59700_tests::readme_has_enterprise1_mention` pass を確認
- [x] `v59700_tests::docs_enterprise1_overview_exists` pass を確認
- [x] 総テスト数 **3322** tests passed, 0 failed を確認
- [x] failures=0 であることを確認（全既存テストが通過）

---

## T8: 事後処理

- [x] `CHANGELOG.md` に v59.7.0 エントリを追加
- [x] `versions/current.md` を v59.7.0 / 3322 tests に更新
- [x] `versions/roadmap/roadmap-v59.1-v60.0.md` の v59.7.0 実績欄を更新
- [x] `roadmap-v59.1-v60.0.md` の v59.8.0 ベース数を「着手時に更新」→ `3322` に確定（T7 でテスト数確認後に実施）
- [x] このファイル（tasks.md）を COMPLETE ステータスに更新

---

Status: COMPLETE（2026-07-30）— 3322 tests passed, 0 failed
