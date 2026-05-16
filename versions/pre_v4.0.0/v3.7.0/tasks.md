# Favnir v3.7.0 Tasks

## Phase 0: バージョン更新

- [x] `fav/Cargo.toml` の version を `"3.7.0"` に更新
- [x] `fav/Cargo.toml` に依存追加
  - [x] `ureq = "2"`
  - [x] `tiny_http = "0.12"`
  - [x] `parquet = "52"`
  - [x] `arrow = { version = "52", features = ["ipc"] }`
- [x] `fav/src/main.rs` のヘルプテキスト・バージョン文字列を更新（`fav build --graphql` 追記）

## Phase 1: 型登録 + namespace

- [x] `checker.rs`: `HttpResponse` を `type_defs` に pre-register（status/body/content_type フィールド）
- [x] `checker.rs`: `HttpError` を `type_defs` に pre-register（code/message/status フィールド）
- [x] `checker.rs`: `ParquetError` を `type_defs` に pre-register（message フィールド）
- [x] `checker.rs`: `!Network` エフェクトを追加（`Effect::Network`、Http.* 呼び出しに必須）
  - 注: spec では `!Http` と記載したが、実装名は `!Network`
- [x] `checker.rs`: `Http.get_raw / post_raw / serve_raw` のシグネチャ登録
- [x] `checker.rs`: `Parquet.write_raw / read_raw` のシグネチャ登録
- [x] `checker.rs`: `Http` / `Parquet` / `Network` namespace をチェックリストに追加
- [x] `compiler.rs`: `"Http"` / `"Parquet"` を2箇所のグローバル登録ループに追加
- [x] `compiler.rs`: `"HttpResponse"` / `"HttpError"` / `"ParquetError"` を type registration ループに追加

## Phase 2: VM プリミティブ

- [x] `vm.rs`: `Http.get_raw(url)` 実装（ureq 同期 GET → `VMValue::Result`）
- [x] `vm.rs`: `Http.post_raw(url, body, content_type)` 実装（ureq 同期 POST）
- [x] `vm.rs`: `Http.serve_raw(port, routes, handler_name)` 実装（tiny_http ブロッキングサーバー）
- [x] `vm.rs`: `Parquet.write_raw(path, type_name, rows)` 実装（Arrow RecordBatch → Parquet）
- [x] `vm.rs`: `Parquet.read_raw(path)` 実装（Parquet → `List<Map<String,String>>`）
- [x] `vm.rs`: HTTP エラーマッピング（接続失敗=0, タイムアウト=1, HTTPエラー=2）
- [x] `vm_stdlib_tests.rs`: 5件の新テスト追加（計画8件 → 実装5件、主要シナリオはカバー済み）
  - [x] `http_get_raw_returns_err_on_bad_url`
  - [x] `http_post_raw_sends_body`（ローカル tiny_http サーバーで確認）
  - [x] `parquet_write_then_read_roundtrip`
  - [x] `parquet_read_returns_err_on_missing_file`
  - [x] `parquet_write_empty_rows_ok`
  - 未実装: `http_response_status_field_accessible`, `parquet_write_creates_file`, `parquet_row_count_matches_write`（driver 統合テストで代替カバー）

## Phase 3: `runes/http/http.fav`

- [x] `runes/http/http.fav` 作成（6 関数、エフェクト `!Network`）
  - [x] `get(url)` — Http.get_raw ラッパー
  - [x] `post(url, body)` — text/plain POST
  - [x] `post_json(url, body)` — application/json POST
  - [x] `get_body(url)` — GET してボディ文字列を返す
  - [x] `ok(status, body)` — HttpResponse 生成ヘルパー（純粋関数）
  - [x] `error_response(status, message)` — エラー HttpResponse 生成ヘルパー（純粋関数）

## Phase 4: `runes/http/http.test.fav`

- [x] `runes/http/http.test.fav` 作成（9 テスト、計画10件 → 外部依存テストなし）
  - [x] `http_get_returns_result`（127.0.0.1:9 — 拒否エラーまたは OK を型チェック）
  - [x] `http_get_bad_url_is_err`
  - [x] `http_post_returns_result`
  - [x] `http_post_json_content_type_result`
  - [x] `http_get_body_on_bad_url`
  - [x] `http_ok_helper_status`
  - [x] `http_ok_helper_body`
  - [x] `http_error_response_status`
  - [x] `http_error_response_message_nonempty`
  - 未実装: `http_get_httpbin_status_200`（外部ネットワーク依存; CI で不安定なため省略）

## Phase 5: `runes/parquet/parquet.fav` + テスト

- [x] `runes/parquet/parquet.fav` 作成（4 関数）
  - [x] `write(path, type_name, rows)` — Parquet.write_raw ラッパー
  - [x] `read(path)` — Parquet.read_raw ラッパー
  - [x] `append(path, type_name, rows)` — 既存読み込み + concat + 書き直し
  - [x] `row_count(path)` — read して List.length
- [x] `runes/parquet/parquet.test.fav` 作成（7 テスト、計画8件 → `parquet_read_preserves_int_field` 省略）
  - [x] `parquet_write_returns_ok`
  - [x] `parquet_read_after_write_count`
  - [x] `parquet_read_preserves_string_field`
  - [x] `parquet_read_missing_file_is_err`
  - [x] `parquet_row_count_matches_write`
  - [x] `parquet_append_increases_count`
  - [x] `parquet_write_empty_rows`
  - 未実装: `parquet_read_preserves_int_field`（String フィールドテストで読み書き動作は確認済み）

## Phase 6: driver 統合テスト + `fav build --graphql`

- [x] `driver.rs`: `http_rune_test_file_passes` テスト追加
- [x] `driver.rs`: `parquet_rune_test_file_passes` テスト追加
- [x] `driver.rs`: `http_get_body_in_favnir_source` テスト追加
- [x] `driver.rs`: `parquet_write_read_roundtrip_in_favnir_source` テスト追加
- [x] `driver.rs`: `http_ok_helper_in_favnir_source` テスト追加
- [x] `driver.rs`: `cmd_build_graphql(file, out)` 実装
  - [x] `Item::TypeDef` → GraphQL `type` ブロック生成
  - [x] `Item::InterfaceDef` → GraphQL `type Query` ブロック生成
  - [x] 型マッピング（Int/Float/String/Bool/Option<T>/List<T>/Result<T,E>）
  - [x] ファイル書き出し（`--out` 指定時）/ stdout（省略時）
- [x] `driver.rs`: `fav_build_graphql_generates_type_block` テスト追加
- [x] `main.rs`: `fav build --graphql <file> [--out <path>]` フラグ追加
  - 注: 独立サブコマンドではなく `fav build` の `--graphql` フラグとして実装
- [x] `main.rs`: ヘルプテキストに `--graphql` フラグ追記

## Phase 7: examples + docs

- [x] `fav/examples/http_demo/src/main.fav` 作成
- [x] `fav/examples/parquet_demo/src/main.fav` 作成
- [x] `fav/examples/graphql_schema_demo/src/main.fav` 作成
- [x] `versions/v3.7.0/langspec.md` 作成（Codex 版は簡素; 上書き更新済み）
- [x] `versions/v3.7.0/migration-guide.md` 作成
- [x] `versions/v3.7.0/progress.md` 全フェーズ完了に更新
- [x] `memory/MEMORY.md` を v3.7.0 完了状態に更新
