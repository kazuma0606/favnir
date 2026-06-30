# v25.6.0 タスクリスト — dynamodb Rune 実質化

**状態**: COMPLETE
**開始日**: 2026-06-25
**完了日**: 2026-06-25

---

## タスク

| ID | タスク | 状態 |
|---|---|---|
| T0 | `fav/Cargo.toml` を `version = "25.6.0"` に bump（DynamoDB は既存 aws_post 再利用のため追加 crate 不要） | [x] |
| T1 | `fav/src/ast.rs` 更新（`Effect::DynamoDB` 追加、`MongoDB` の直後） | [x] |
| T2 | `fav/src/error_catalog.rs` 更新（E0323 追加） | [x] |
| T3 | `fav/src/fmt.rs` / `fav/src/emit_python.rs` / `fav/src/lint.rs` / `fav/src/middle/reachability.rs` / `fav/src/middle/ast_lower_checker.rs` / `fav/src/lineage.rs` 更新（`Effect::DynamoDB` 対応・6 ファイル） | [x] |
| T4 | `fav/src/middle/checker.rs` 更新（`ns_to_inferred_effect` / `require_dynamodb_effect` / DynamoDB builtin fns） | [x] |
| T5 | `fav/src/frontend/parser.rs` 更新（`"DynamoDB" => Effect::DynamoDB` アーム追加） | [x] |
| T6 | `fav/src/driver.rs` 更新（`format_effects` / `effect_json_name` に DynamoDB アーム追加） | [x] |
| T7 | `cargo build` で exhaustive match エラーなし確認 | [x] |
| T8 | `fav/src/backend/vm.rs` 更新（`get_dynamo_config` ヘルパー + `json_val_to_dynamo_attr` / `json_to_dynamo_item` / `dynamo_attr_to_json` / `dynamo_item_to_plain_json` ヘルパー + `DynamoDB.*_raw` 8 件） | [x] |
| T9 | `runes/dynamodb/dynamodb.fav` 全面更新（`type DynamoConn` + 8 関数） | [x] |
| T10 | `examples/dynamodb_session_store.fav` 新規作成（`import rune "dynamodb"` 使用） | [x] |
| T11 | `site/content/docs/runes/dynamodb.mdx` 新規作成（全 API 記載） | [x] |
| T12 | `CHANGELOG.md` 更新（`[v25.6.0]` エントリ追加） | [x] |
| T13 | `benchmarks/v25.6.0.json` 新規作成（test_count: 2014） | [x] |
| T14 | `fav/src/driver.rs` 更新（`v256000_tests` 7 件追加：`effect_dynamodb_and_e0323_exist` で ast.rs + error_catalog.rs + checker.rs を統合確認） | [x] |
| T15 | `cargo test v256000` — 7 件 PASS 確認 | [x] |
| T16 | `cargo test` 総テスト数 ≥ 2014 件 確認 | [x] |
| T17 | spec-reviewer レビュー実施 | [x] |

---

## チェックリスト（完了条件）

- [x] `DynamoDB.connect` が `runes/dynamodb/dynamodb.fav` に存在する
- [x] `DynamoDB.get_item` / `DynamoDB.query` / `DynamoDB.scan` が存在する（read 系）
- [x] `DynamoDB.put_item` / `DynamoDB.delete_item` / `DynamoDB.batch_write` / `DynamoDB.transact_write` が存在する（write 系）
- [x] `DynamoDB.*_raw` 8 件すべてが `fav/src/backend/vm.rs` に存在する
- [x] `Effect::DynamoDB` が `fav/src/ast.rs` に存在する（`cargo build` で exhaustive match エラーなし確認済み）
- [x] E0323 が `fav/src/error_catalog.rs` に存在する（`checker.rs` の `require_dynamodb_effect` が `"E0323"` を使用）
- [x] `examples/dynamodb_session_store.fav` が存在し `import rune "dynamodb"` / `put_item` / `get_item` / `delete_item` を含む
- [x] `CHANGELOG.md` に `v25.6.0` が存在する
- [x] `site/content/docs/runes/dynamodb.mdx` が存在し全 API を記載している
- [x] `v256000_tests` 7 件すべて PASS（`cargo test v256000` 実行済み）
- [x] 総テスト数 ≥ 2014 件

---

---

## コードレビュー指摘（code-reviewer）

| 指摘 | 対応 |
|---|---|
| [MED] `scan_raw` の `FilterExpression` に JSON Value を埋め込んでいた（DynamoDB は文字列のみ受け付ける） | `filter_json` をそのまま文字列として `FilterExpression` に設定するよう修正（JSON parse 不要）。dynamodb.mdx のサンプルも修正 |
| [MED] `batch_write_raw` の `collect::<Result<_,_>>()?` が Rust Err を伝播（VM クラッシュ相当） | `match ... { Ok(v) => v, Err(e) => return Ok(err_vm(...)) }` パターンに変換 |
| [MED] `connect_raw` の checker 型（`Result<String,String>`）と Rune 公開型（`Result<DynamoConn,String>`）の整合性未明示 | checker.rs に `DynamoConn(String)` は String として扱われる旨のコメントを追記（MongoConn と同パターン） |
| [SECURITY/LOW] デモコードで `session_id` を文字列結合で JSON に埋め込み（injection リスク） | `examples/dynamodb_session_store.fav` に本番コードでのエスケープ必須の警告コメントを追加 |
| [LOW] `dynamo_attr_to_json` が `SS`/`NS`/`BS`（Set 型）を未処理 | 既知制限として `TODO(v26.x): SS/NS/BS` コメントを追加 |

---

## メモ

- `aws_post` ヘルパーは既存（vm.rs）。引数シグネチャを確認してから使用する
- DynamoDB HTTP API の `X-Amz-Target` ヘッダ: `DynamoDB_20120810.<Action>`（例: `DynamoDB_20120810.GetItem`）
- `get_item_raw` の not_found: レスポンス JSON に `"Item"` フィールドがない → `err_vm("not_found")`（MongoDB の `find_one_raw` と同パターン）
- `batch_write_raw` の最大件数: 25 件制限チェックを primitive 内で実施（`puts.len() > 25` → Err）
- `transact_write_raw` の `ops_json`: ユーザーが DynamoDB TransactItems 形式の JSON を渡す（変換なし）
- E0323 は E0322（MongoDB）の次
- 目標テスト数 2014 件（v25.5.0 終了時実測 2007 件 + 7 件）
- `lineage.rs` の DynamoDB 分類: `classify_capability_kind` に `ast::Effect::DynamoDB => ("io", "KvStore")` 追加
- `cfg(not(target_arch = "wasm32"))` ガードを全 DynamoDB primitive と helper に付与
- 既存 `AWS.dynamo_*_raw`（VMValue::Record I/O）は変更しない。新規 `DynamoDB.*_raw`（JSON 文字列 I/O）と独立共存
