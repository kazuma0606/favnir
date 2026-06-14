# v16.0.0 Tasks — "Production Multi-Cloud" マイルストーン宣言

Date: 2026-06-14
Branch: master

---

## Phase A — Cargo バージョン更新

- [ ] A-1: `fav/Cargo.toml` の `version` を `"16.0.0"` に変更

---

## Phase B — CHANGELOG.md 更新

- [ ] B-1: `CHANGELOG.md` の先頭に v15.5.0 エントリ追加
- [ ] B-2: `CHANGELOG.md` の先頭に v15.4.0 エントリ追加
- [ ] B-3: `CHANGELOG.md` の先頭に v15.3.0 エントリ追加
- [ ] B-4: `CHANGELOG.md` の先頭に v15.2.0 エントリ追加
- [ ] B-5: `CHANGELOG.md` の先頭に v15.1.5 エントリ追加
- [ ] B-6: `CHANGELOG.md` の先頭に v15.1.0 エントリ追加

---

## Phase C — README.md 更新

- [ ] C-1: 「現在の状態」セクションに v16.0.0 宣言を追記
- [ ] C-2: 対応クラウド一覧表（AWS/Azure/GCP/Snowflake + Kafka/MSK）を追加
- [ ] C-3: 機能一覧に `fav test` / `fav deploy` を追加

---

## Phase D — サイトドキュメント追加

- [ ] D-1: `site/content/docs/runes/bigquery.mdx` 新規作成
  - `BigQuery.*` 関数リファレンス
  - `fav.toml [gcp]` 設定例
  - `GOOGLE_APPLICATION_CREDENTIALS` 設定方法
- [ ] D-2: `site/content/docs/runes/kafka.mdx` 新規作成
  - `Kafka.*` 関数リファレンス
  - `fav.toml [kafka]` 設定例
  - AWS MSK 接続設定例

---

## Phase E — v160000_tests 追加（driver.rs）

- [ ] E-1: `fav/src/driver.rs` に `v160000_tests` モジュール追加（5 テスト）
  - `version_is_16_0_0`
  - `changelog_has_v15_entries`
  - `readme_mentions_bigquery`
  - `readme_mentions_kafka`
  - `all_e2e_demo_dirs_exist`（airgap / fav2py / snowflake / crosscloud / bigquery / kafka）

---

## Phase F — テスト・コミット

- [ ] F-1: `cargo test v160000` → 5/5 PASS
- [ ] F-2: `cargo test` → リグレッションなし
- [ ] F-3: コミット — feat: v16.0.0 — Production Multi-Cloud マイルストーン宣言

---

## 完了条件

| 確認項目 | 状態 |
|---|---|
| `Cargo.toml version == "16.0.0"` | [ ] |
| `cargo test v160000` 全テストパス（5/5） | [ ] |
| `cargo test` 全件パス（リグレッションなし） | [ ] |
| CHANGELOG.md に v15.1.0〜v15.5.0 エントリが含まれる | [ ] |
| README.md に BigQuery / Kafka が記載されている | [ ] |
| `site/content/docs/runes/bigquery.mdx` が存在する | [ ] |
| `site/content/docs/runes/kafka.mdx` が存在する | [ ] |
| 全 E2E デモディレクトリが存在する（6 件） | [ ] |
