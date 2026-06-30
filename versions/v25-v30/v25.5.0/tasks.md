# v25.5.0 タスクリスト — mongodb Rune 実質化

**状態**: COMPLETE
**開始日**: 2026-06-25
**完了日**: 2026-06-25

---

## タスク

| ID | タスク | 状態 |
|---|---|---|
| T0 | `fav/Cargo.toml` を `version = "25.5.0"` に bump + `mongodb = { version = "3" }` crate 追加 | [x] |
| T1 | `fav/src/ast.rs` 更新（`Effect::MongoDB` 追加、`MySQL` の直後） | [x] |
| T2 | `fav/src/error_catalog.rs` 更新（E0322 追加） | [x] |
| T3 | `fav/src/fmt.rs` / `fav/src/lineage.rs` / `fav/src/emit_python.rs` / `fav/src/lint.rs` / `fav/src/middle/reachability.rs` / `fav/src/middle/ast_lower_checker.rs` 更新（`Effect::MongoDB` 対応・6 ファイル） | [x] |
| T4 | `fav/src/middle/checker.rs` 更新（`ns_to_inferred_effect` / `require_mongodb_effect` / Mongo builtin fns） | [x] |
| T5 | `fav/src/frontend/parser.rs` 更新（`"MongoDB" => Effect::MongoDB` アーム追加） | [x] |
| T6 | `fav/src/driver.rs` 更新（`format_effects` / `effect_json_name` に MongoDB アーム追加） | [x] |
| T7 | `fav/src/backend/vm.rs` 更新（`Mongo.*_raw` 8 件 + `extract_mongo_db_name` / `mongo_bson_to_json` / `mongo_json_to_bson` ヘルパー追加） | [x] |
| T8 | `runes/mongodb/mongodb.fav` 全面更新（type MongoConn + connect / find / find_one / insert_one / insert_many / update_one / delete_one / aggregate） | [x] |
| T9 | `examples/mongo_events_etl.fav` 新規作成（`import rune "mongodb"` 使用） | [x] |
| T10 | `site/content/docs/runes/mongodb.mdx` 新規作成（全 API 記載） | [x] |
| T11 | `CHANGELOG.md` 更新（`[v25.5.0]` エントリ追加） | [x] |
| T12 | `benchmarks/v25.5.0.json` 新規作成（test_count: 2007） | [x] |
| T13 | `fav/src/driver.rs` 更新（`v255000_tests` 7 件追加） | [x] |
| T14 | `cargo test v255000` — 7 件 PASS 確認 | [x] |
| T15 | `cargo test` 総テスト数 ≥ 2007 件 確認 | [x] |
| T16 | spec-reviewer レビュー実施 | [x] |

---

## チェックリスト（完了条件）

- [x] `Mongo.connect` が `runes/mongodb/mongodb.fav` に存在する
- [x] `Mongo.find` / `Mongo.find_one` / `Mongo.aggregate` が `runes/mongodb/mongodb.fav` に存在する
- [x] `Mongo.insert_one` / `Mongo.insert_many` / `Mongo.update_one` / `Mongo.delete_one` が存在する
- [x] `Mongo.*_raw` 8 件すべてが `fav/src/backend/vm.rs` に存在する
- [x] `Effect::MongoDB` が `fav/src/ast.rs` に存在する（`cargo build` で exhaustive match エラーなし確認済み）
- [x] E0322 が `fav/src/error_catalog.rs` に存在する（`checker.rs` の `require_mongodb_effect` が `"E0322"` を使用）
- [x] `examples/mongo_events_etl.fav` が存在し `import rune "mongodb"` / `find` / `insert_one` / `delete_one` を含む
- [x] `CHANGELOG.md` に `v25.5.0` が存在する
- [x] `site/content/docs/runes/mongodb.mdx` が存在し全 API を記載している
- [x] `v255000_tests` 7 件すべて PASS（`cargo test v255000` 実行済み）
- [x] 総テスト数 ≥ 2007 件（実測: 2007 件 ※ lsp 既存失敗 1 件は pre-existing で v25.5.0 と無関係）

---

## コードレビュー指摘（code-reviewer）

| 指摘 | 対応 |
|---|---|
| [HIGH] `find_one_raw` のエラー伝播パターンが他 primitive と非対称 | `?` パターンに統一（actual errors → Rust Err）、`None → err_vm("not_found")` のみ例外として残す |
| [MED] `update_one_raw` で `$set` 等演算子なし plain document を無検証で送信 | `update_doc.keys()` を確認し、`$` で始まらないキーがあればエラーを返す事前バリデーションを追加 |
| [MED] `extract_mongo_db_name` がパスワードに `/` を含む URL で誤動作 | `rfind('@')` で認証情報をスキップしてからパスを抽出するよう修正 |
| [MED] `mongo_bson_to_json` の `other` アームが無言で `Null` を返す | `DateTime / Timestamp / Binary / Decimal128 等は Extended JSON 形式で返る` というコメントを追記 |
| [LOW] 各 primitive で毎回 tokio runtime + 接続確立（設計意図は明記済みだが vm.rs に TODO 不在） | vm.rs MongoDB primitives ブロック先頭に `TODO(v26.x): コネクションプール` コメントを追加 |
| [LOW] ETL デモの名前空間が `Mongo.` で他 Rune の `MySQL.` パターンと不一致 | examples/mongo_events_etl.fav と mongodb.mdx に名前空間の注記（`Mongo` 優先、`MongoDB` も動作）を追加 |

## コードレビュー指摘（spec-reviewer）

spec-reviewer 8 件指摘 → spec.md / plan.md / tasks.md を修正後に実装。

| 指摘 | 対応 |
|---|---|
| [HIGH] mongodb v3 sync feature 廃止 | `mongodb = { version = "3" }` に変更（tokio-runtime はデフォルト）、block_on パターンで実装 |
| [HIGH] `classify_stage_node` → `classify_capability_kind` | spec.md / plan.md / tasks.md で関数名修正、実装では正しい `classify_capability_kind` を使用 |
| [HIGH] classify コードサンプル欠落 | plan.md に追記 |
| [MED] find_one Option→Result 理由未記載 | spec.md に設計判断ブロック追加 |
| [MED] insert_many_raw 件数取得未定義 | plan.md に `.inserted_ids.len()` コード追加、vm.rs で実装 |
| [MED] extract_mongo_db_name エッジケース | `find('/')` ベースの実装に修正（ポート番号誤検出防止） |
| [MED] E0322 テスト欠落 | `effect_mongodb_and_e0322_exist` テストで ast.rs + error_catalog.rs + checker.rs を両方確認 |
| [LOW] pipeline 型不整合 | examples で `Result<String, String>` / `Result<Int, String>` に修正 |
| [LOW] ObjectId JSON シリアライズ | `mongo_bson_to_json` に `oid.to_hex()` + `{"$oid": ...}` 変換を実装 |

---

## 実装時に発見した追加修正

- `mongodb = "3"` の features 指定: `tokio-runtime` feature は存在せず、デフォルトで tokio が含まれるため `features` 指定不要
- `aggregate` cursor の型: `collection.aggregate()` は `mongodb::Cursor<Document>` を返す（`futures::TryStreamExt::try_collect()` で収集）
- lsp テスト `write_message_emits_content_length_frame` は pre-existing 失敗（v25.5.0 変更前から lsp/mod.rs は変更済み）

---

## メモ

- `mongodb = "3"` の `sync` feature は v3 で廃止済み。**`features = ["tokio-runtime"]` が主方針**。`tokio::runtime::Builder::new_current_thread().enable_all().build()?.block_on(async { ... })` で非同期を同期化する（tokio は既に Cargo.toml に存在）
- `Mongo.*_raw` の namespace は `"Mongo"`（`"MongoDB"` ではない）— checker.rs の ns_to_inferred_effect は `"Mongo" | "MongoDB"` の両方に対応する
- `lineage.rs` の MongoDB 分類追加先は `classify_capability_kind` 関数（`classify_stage_node` は存在しない）
- `insert_many_raw` の挿入件数は `result.inserted_ids.len()` で取得（`InsertManyResult` に count フィールドなし）
- `mongo_bson_to_json`: ObjectId は `oid.to_hex()` + `{"$oid": "..."}` に変換（serde_json::to_value では Extended JSON 形式にならない）
- `extract_mongo_db_name`: `rsplit('/')` ではなく `find('/')` でスキーム除去後のパス部分を取得（ポート番号誤検出防止）
- examples の stage 型: `LoadActiveEvents: Unit -> Result<String, String>`, `ArchiveEvent: Result<String, String> -> Result<Int, String>`（pipeline 型整合）
- `effect_mongodb_and_e0322_exist` テストは ast.rs の `"MongoDB,"` + error_catalog.rs の `"E0322"` の両方を確認
- E0322 は E0321（MySQL）の次。E0316〜E0319 は未割当の空き番号。連番で E0322 を採用
- 目標テスト数 2007 件（v25.4.0 終了時実測 2000 件 + 7 件）
- `find_one` は見つからない場合 `Result.err("not_found")` を返す（`Option<T>` 返却はロードマップの記述だが VM 制約で `Result` に統一）
- BSON ↔ JSON 変換: `mongo_json_to_bson` は `serde_json::from_str` + `bson::to_document` で変換
