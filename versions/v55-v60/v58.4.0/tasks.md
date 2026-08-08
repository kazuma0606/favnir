# Tasks — v58.4.0 — Data Catalog 統合（`fav catalog`）

## ステータス: COMPLETE

---

## 事前確認（T0）

- [x] `versions/roadmap/roadmap-v58.1-v59.0.md` の v58.4.0 セクションを確認
- [x] ベーステスト数 3286（v58.3.0 完了時点の実績値）を確認 — `cargo test 2>&1 | grep 'tests passed'` で 3286 であることを数値確認
- [x] `fav/Cargo.toml` が `58.3.0` であることを確認（更新前）
- [x] `v58400_tests` が `driver.rs` に存在しないことを確認（新規追加対象）
- [x] `v58300_tests` が `driver.rs` に存在することを確認（`v58400_tests` の挿入位置として使用）
- [x] `cmd_catalog_push` / `cmd_catalog_search` が `driver.rs` に存在しないことを確認（新規追加対象）
- [x] `cmd_schema_migrate` が `driver.rs` に存在することを確認（挿入位置として使用）
- [x] `Some("catalog")` arm が `main.rs` に存在しないことを確認（新規追加対象）
- [x] rolling チェック 5 件（v56300 / v56900 / v57000 / v57900 / v58000）が `"58.3.0"` を期待していることを確認

---

## 実装タスク（順序厳守）

- [x] T1: `fav/Cargo.toml` version を `58.4.0` に更新
- [x] T2: `fav/src/driver.rs` — `cmd_catalog_push` 追加
  - [x] 関数シグネチャ: `pub fn cmd_catalog_push(catalog_url: &str) -> i32`
  - [x] `println!("Registering pipeline: OrderIngestion")` 出力
  - [x] `println!("  stage Parse:    RawOrder → Order")` 出力
  - [x] `println!("  stage Validate: Order → Result<ValidOrder>")` 出力
  - [x] `println!("  stage Store:    ValidOrder → Unit  (Snowflake: orders_v2)")` 出力（固定文字列、ロードマップ例示に準拠）
  - [x] `println!("Catalog push: OK  [catalog: {}]", catalog_url)` 出力
  - [x] `0` を返す
  - [x] `cmd_schema_migrate` の直後に追加（`// ── fav catalog (v58.4.0)` コメント付き）
- [x] T3: `fav/src/driver.rs` — `cmd_catalog_search` 追加
  - [x] 関数シグネチャ: `pub fn cmd_catalog_search(query: &str) -> i32`
  - [x] `println!("Catalog search: \"{}\"", query)` 出力
  - [x] `println!("OrderIngestion  pipeline  last_run: 2026-07-23T10:00:00Z")` 出力
  - [x] `0` を返す
  - [x] `cmd_catalog_push` の直後に追加
- [x] T4: `fav/src/main.rs` — `Some("catalog")` arm 追加
  - [x] `Some("schema")` arm の後（`Some("doc")` arm の前）に追加
  - [x] `Some("push")` サブコマンド: `--catalog` フラグをパース、値欠落時は eprintln + exit(1)
  - [x] `Some("push")` サブコマンド: `driver::cmd_catalog_push(catalog_url)` を呼び exit
  - [x] `Some("search")` サブコマンド: `args.get(3)` でクエリを取得（デフォルト `""`）
  - [x] `Some("search")` サブコマンド: `driver::cmd_catalog_search(query)` を呼び exit
  - [x] `sub =>` アーム: usage を eprintln して exit(1)
- [x] T5: `fav/src/driver.rs` — `v58400_tests` モジュールを `v58300_tests` の直前に追加
  - [x] `use super::{cmd_catalog_push, cmd_catalog_search};` をモジュール内に追加
  - [x] `cmd_catalog_push_test`: `cmd_catalog_push("datahub://localhost:8080")` が 0 を返すことを assert
  - [x] `cmd_catalog_search_test`: `cmd_catalog_search("order")` が 0 を返すことを assert
  - [x] テスト関数名は `cmd_catalog_push_test` / `cmd_catalog_search_test`（関数名との衝突を避けるため）
- [x] T6: rolling バージョンチェック 5 件を `"58.3.0"` → `"58.4.0"` に更新（replace_all）
  - [x] v56300_tests
  - [x] v56900_tests
  - [x] v57000_tests
  - [x] v57900_tests
  - [x] v58000_tests

---

## テスト・検証

- [x] T7: `cargo build` でコンパイルエラーがないことを確認
- [x] T8: `cargo test` 全通過（**3288 tests passed, 0 failed**）
  - [x] `v58400_tests::cmd_catalog_push_test` ok
  - [x] `v58400_tests::cmd_catalog_search_test` ok
  - [x] `v58300_tests` 全件引き続き通過
  - [x] `v58200_tests` 全件引き続き通過
  - [x] 既存 3286 件全通過
- [x] T9: `cargo clippy -- -D warnings` クリーン

---

## ポスト処理

- [x] T10: `CHANGELOG.md` に v58.4.0 エントリを追加（形式: `## [v58.4.0] — 2026-07-28 — Data Catalog 統合`）
- [x] T11: `versions/current.md` を v58.4.0 / 3288 tests に更新
- [x] T12: `versions/roadmap/roadmap-v58.1-v59.0.md` の v58.4.0 実績を COMPLETE に更新
  - [x] `3286 + 2 = 3288 tests passed, 0 failed（2026-07-28）` を追記
- [x] T13: `versions/v55-v60/v58.4.0/tasks.md` を COMPLETE に更新

---

## 完了確認

- [x] `cmd_catalog_push_test` pass
- [x] `cmd_catalog_search_test` pass
- [x] **3288 tests passed, 0 failed**（ベース 3286 + 2）
- [x] `cargo clippy -- -D warnings` クリーン
- [x] `fav/src/main.rs` に `Some("catalog")` arm が追加されている
- [x] rolling チェック 5 件が `"58.4.0"` になっている
- [x] `CHANGELOG.md` に `[v58.4.0]` エントリが追加されている
- [x] `versions/current.md` が v58.4.0 / 3288 tests を反映
- [x] `versions/roadmap/roadmap-v58.1-v59.0.md` の v58.4.0 実績が COMPLETE に更新されている

---

## 実装メモ

- `cmd_catalog_push` / `cmd_catalog_search` はスタブ実装（実 HTTP 通信なし）
- テスト名は `cmd_catalog_push_test` / `cmd_catalog_search_test`（v58.3.0 慣例）
- `Some("catalog")` arm は `driver::` プレフィックス経由（use インポート不要）
- `!Catalog` エフェクトの AST/IR 統合はスコープ外（v58.x パターン踏襲）
- rolling チェックは全バージョンで 5 件全件更新が必要（v56300 / v56900 / v57000 / v57900 / v58000）
- push 出力の `stage Store` 行は `(Snowflake: orders_v2)` 固定文字列（ロードマップ例示に準拠）、catalog_url は `Catalog push: OK` 行に表示
