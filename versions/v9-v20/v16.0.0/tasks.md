# v16.0.0 Tasks — "Production Multi-Cloud" マイルストーン宣言

Date: 2026-06-14
Branch: master

---

## Phase A — Cargo バージョン更新

- [x] A-1: `fav/Cargo.toml` の `version` を `"16.0.0"` に変更

---

## Phase B — CHANGELOG.md 更新

- [x] B-1: `CHANGELOG.md` の先頭に v15.5.0 エントリ追加
- [x] B-2: `CHANGELOG.md` の先頭に v15.4.0 エントリ追加
- [x] B-3: `CHANGELOG.md` の先頭に v15.3.0 エントリ追加
- [x] B-4: `CHANGELOG.md` の先頭に v15.2.0 エントリ追加
- [x] B-5: `CHANGELOG.md` の先頭に v15.1.5 エントリ追加
- [x] B-6: `CHANGELOG.md` の先頭に v15.1.0 エントリ追加

---

## Phase C — README.md 更新

- [x] C-1: 「現在の状態」セクションに v16.0.0 宣言を追記
- [x] C-2: 対応クラウド一覧表（AWS/Azure/GCP/Snowflake + Kafka/MSK）を追加
- [x] C-3: 機能一覧に `fav deploy` を追加（Rune エコシステム表）
- [x] C-4: バージョン履歴表に v15.0.0〜v16.0.0 エントリを追加

---

## Phase D — サイトドキュメント追加

- [x] D-1: `site/content/docs/runes/bigquery.mdx` 新規作成
  - `BigQuery.*` 関数リファレンス
  - `fav.toml [gcp]` 設定例
  - `fav infer --from bigquery --table <name>` の使い方
  - `GOOGLE_APPLICATION_CREDENTIALS` 設定方法
- [x] D-2: `site/content/docs/runes/kafka.mdx` 新規作成
  - `Kafka.*` 関数リファレンス
  - `fav.toml [kafka]` 設定例
  - `!Stream` エフェクト・E0319 説明
  - AWS MSK 接続設定例

---

## Phase E — v160000_tests 追加（driver.rs）

- [x] E-1: `fav/src/driver.rs` に `v160000_tests` モジュール追加（5 テスト）
  - `version_is_16_0_0`
  - `changelog_has_v15_entries`
  - `readme_mentions_bigquery`
  - `readme_mentions_kafka`
  - `all_e2e_demo_dirs_exist`（airgap / fav2py / snowflake / crosscloud / bigquery / kafka）

---

## Phase F — テスト・コミット

- [x] F-1: `cargo test v160000` → 5/5 PASS
- [x] F-2: `cargo test` → 1574 PASS（リグレッションなし）
- [x] F-3: コミット `6f6336e` — feat: v16.0.0 — Production Multi-Cloud マイルストーン宣言

---

## 完了条件

| 確認項目 | 状態 |
|---|---|
| `Cargo.toml version == "16.0.0"` | [x] |
| `cargo test v160000` 全テストパス（5/5） | [x] |
| `cargo test` 全件パス（リグレッションなし） | [x] |
| CHANGELOG.md に v15.1.0〜v15.5.0 エントリが含まれる | [x] |
| README.md に BigQuery / Kafka が記載されている | [x] |
| `site/content/docs/runes/bigquery.mdx` が存在する | [x] |
| `site/content/docs/runes/kafka.mdx` が存在する | [x] |
| 全 E2E デモディレクトリが存在する（6 件） | [x] |
