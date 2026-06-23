# v24.5.0 — Rune レジストリ成熟（公式パッケージ 50+）タスク

## ステータス: COMPLETE（2026-06-23）

---

## タスク一覧

### T0: 事前確認 + `driver.rs` OFFICIAL_CATALOG + cmd_search 追加

- [x] `grep -n "version = " fav/Cargo.toml` — `"24.4.0"` であること
- [x] `grep -n "mod v245000_tests" fav/src/driver.rs | head -3` — 未存在
- [x] `grep -n "cmd_search\|OFFICIAL_CATALOG" fav/src/driver.rs | head -5` — 全 0 件
- [x] **T0-1**: `fav/src/driver.rs` に `pub const OFFICIAL_CATALOG: &[(&str, &str, &str)]` 定数を追加（50 エントリ）
- [x] **T0-2**: `pub fn cmd_search(query: &str)` を `OFFICIAL_CATALOG` の直後に追加
- [x] **事後確認**: `cargo check --bin fav` — エラー 0

---

### T1: `fav/src/main.rs` — `Some("search")` アーム追加

- [x] **T1-1**: `main.rs` の import 行（`cmd_install` がある行）に `cmd_search,` を追加
- [x] **T1-2**: `Some("install")` の前に `Some("search")` アームを追加:
  ```rust
  Some("search") => {
      let query = args.get(2).map(|s| s.as_str()).unwrap_or("");
      cmd_search(query);
  }
  ```
- [x] **事後確認**: `cargo check --bin fav` — エラー 0

---

### T2: 15 新規 Rune スタブ作成（`runes/` 配下）

各ディレクトリに `rune.toml` と `<name>.fav` を作成する。

- [x] `runes/avro/rune.toml` + `runes/avro/avro.fav`
- [x] `runes/orc/rune.toml` + `runes/orc/orc.fav`
- [x] `runes/excel/rune.toml` + `runes/excel/excel.fav`
- [x] `runes/xml/rune.toml` + `runes/xml/xml.fav`
- [x] `runes/huggingface/rune.toml` + `runes/huggingface/huggingface.fav`
- [x] `runes/scikit/rune.toml` + `runes/scikit/scikit.fav`
- [x] `runes/gcs/rune.toml` + `runes/gcs/gcs.fav`
- [x] `runes/pubsub/rune.toml` + `runes/pubsub/pubsub.fav`
- [x] `runes/redis/rune.toml` + `runes/redis/redis.fav`
- [x] `runes/mysql/rune.toml` + `runes/mysql/mysql.fav`
- [x] `runes/mongodb/rune.toml` + `runes/mongodb/mongodb.fav`
- [x] `runes/s3/rune.toml` + `runes/s3/s3.fav`
- [x] `runes/sqs/rune.toml` + `runes/sqs/sqs.fav`
- [x] `runes/dynamodb/rune.toml` + `runes/dynamodb/dynamodb.fav`
- [x] `runes/azure-servicebus/rune.toml` + `runes/azure-servicebus/azure-servicebus.fav`
- [x] **事後確認**: `ls -d runes/*/` ディレクトリ数が 50 以上

---

### T3: `fav/src/driver.rs` — v245000_tests 追加

- [x] **事前確認**: `grep -n "fn version_is_24_4_0" fav/src/driver.rs | head -3`
- [x] **T3-1（T5-1 より前に必須）**: `v244000_tests::version_is_24_4_0` テスト関数を**削除**（モジュール自体と他5件のテストは保持すること）
- [x] **T3-2**: `v245000_tests` モジュールを `v244000_tests` の直後に追加（5 件）
  - `version_is_24_5_0`
  - `fav_search_command_exists`
  - `official_catalog_50_plus`
  - `catalog_covers_cloud_formats_ml`
  - `changelog_has_v24_5_0`
- [x] `cargo test v245000 --bin fav` — 5/5 PASS を確認
- [x] `cargo test --bin fav` — リグレッションなし（1953 件合格）を確認
  > 件数計算: 1949 (現在) - 1 (version_is_24_4_0 削除) + 5 (v245000_tests) = 1953

---

### T4: ドキュメントサイト更新

- [x] `site/content/docs/runes/catalog.mdx` を新規作成（全 50 Rune の一覧表）

---

### T5: Cargo.toml + CHANGELOG + benchmarks

- [x] `fav/Cargo.toml` の `version = "24.4.0"` → `"24.5.0"` に変更（T3-1 完了後）
- [x] `CHANGELOG.md` 先頭に v24.5.0 エントリを追加
- [x] `benchmarks/v24.5.0.json` を新規作成（test_count: 1954、duration_ms は実測値に更新）
- [x] `cargo test v245000 --bin fav` — 最終確認 5/5 PASS
- [x] `cargo test --bin fav` — リグレッションなし（1953 件合格）

---

## テスト一覧（v245000_tests、5 件）

| テスト名 | 内容 | 期待値 |
|---|---|---|
| `version_is_24_5_0` | Cargo.toml に `version = "24.5.0"` | — |
| `fav_search_command_exists` | `main.rs` に `Some("search")` / `driver.rs` に `pub fn cmd_search` | — |
| `official_catalog_50_plus` | `OFFICIAL_CATALOG.len() >= 50` | `>= 50` |
| `catalog_covers_cloud_formats_ml` | avro / orc / excel / xml / huggingface / scikit / gcs / redis がカタログに存在 | 全件 `true` |
| `changelog_has_v24_5_0` | `CHANGELOG.md` に `[v24.5.0]` | — |

---

## 完了条件チェックリスト

- [x] `OFFICIAL_CATALOG` 定数（50 エントリ）追加済み（`driver.rs`）
- [x] `pub fn cmd_search(query: &str)` 実装済み（`driver.rs`）
- [x] `Some("search")` アーム追加済み（`main.rs`）
- [x] 15 新規 Rune スタブ作成済み（各 `rune.toml` + `<name>.fav`）
- [x] `v244000_tests::version_is_24_4_0` が削除済み（T5-1 より前）
- [x] `cargo test v245000 --bin fav` — 5/5 PASS
- [x] `cargo test --bin fav` — リグレッションなし（1953 件合格）
- [x] `CHANGELOG.md` に v24.5.0 エントリ
- [x] `benchmarks/v24.5.0.json` 作成済み（test_count: 1953）
- [x] `site/content/docs/runes/catalog.mdx` 作成済み

---

## 実装時の修正メモ

- `include_str!("../main.rs")` / `include_str!("../driver.rs")` は誤り（ファイルが見つからないエラー）。
  `driver.rs` は `fav/src/` にあるため、同ディレクトリの `main.rs` / `driver.rs` へは `include_str!("main.rs")` / `include_str!("driver.rs")` が正しい。
  plan.md の擬似コードも合わせて修正済み。

## コードレビュー指摘と対応（2026-06-23）

| 優先度 | 指摘 | 対応 |
|---|---|---|
| MED | `Some("dap")` アームが catch-all `Some(cmd)` より後に配置されて到達不能 | `Some("dap")` を `Some(cmd)` の直前に移動（main.rs）|
