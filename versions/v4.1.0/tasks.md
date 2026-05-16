# Favnir v4.1.0 Tasks

## Phase 0: バージョン更新

- [x] `fav/Cargo.toml` の version を `"4.1.0"` に更新
- [x] `fav/src/main.rs` のヘルプテキスト・バージョン文字列を `4.1.0` に更新

## Phase 1: `UseDecl` — AST / Lexer / Parser / Fmt

- [x] `ast.rs`: `RuneUseNames` enum を追加
  - `Specific(Vec<String>)` — `use X.{ a, b }`
  - `Wildcard` — `use X.*`
- [x] `ast.rs`: `UseDecl { module: String, names: UseNames, span: Span }` を追加
- [x] `ast.rs`: `Decl` enum に `Use(UseDecl)` バリアントを追加
- [x] `frontend/lexer.rs`: `use` を予約キーワード（`Token::Use`）として追加
  - `keyword_map` または `ident_or_keyword` ロジックに `"use" => Token::Use` を追加（既存）
- [x] `frontend/parser.rs`: `is_rune_use_pattern()` ヘルパーを追加
  - `pos+2 = Dot` かつ `pos+3 = LBrace or Star` を検出
- [x] `frontend/parser.rs`: `parse_program()` の `uses` ループを `!self.is_rune_use_pattern()` でガード
- [x] `frontend/parser.rs`: `parse_item()` に `TokenKind::Use` の分岐を追加
  - `use` を消費
  - モジュール名（Ident）をパース
  - `.` を消費
  - `*` → `RuneUseNames::Wildcard`
  - `{` name(`,` name)* `}` → `RuneUseNames::Specific(names)`
  - `RuneUse` を生成して `Item::RuneUse` を返す
- [x] `fmt.rs`: `Item::RuneUse { module, names, .. }` の pretty-print を追加
  - `Specific` → `use module.{ a, b }`
  - `Wildcard` → `use module.*`
- [x] `frontend/parser.rs` テスト追加
  - [x] `parse_rune_use_specific` — `use connection.{ connect, close }` → `Item::RuneUse`
  - [x] `parse_rune_use_wildcard` — `use query.*` → `RuneUseNames::Wildcard`
  - [x] `parse_rune_use_single_name` — `use migration.{ up }` → `RuneUseNames::Specific`
  - [x] `parse_rune_use_does_not_consume_namespace_use` — `use a.b` は従来の `program.uses` に

## Phase 2: ディレクトリ rune ロード（driver.rs）

- [x] `driver.rs`: `ImportDecl` の rune 解決でディレクトリ優先
  - `runes/<name>/` ディレクトリが存在する場合 → `runes/<name>/<name>.fav` をエントリポイント
  - 存在しない場合 → `runes/<name>.fav`（後方互換）
- [x] `driver.rs`: `load_rec` に `Item::RuneUse` ハンドラを追加
  - 現在ファイルの親ディレクトリから `<module>.fav` を解決してロード
  - `visited` で循環参照を自動防止
- [x] `driver.rs`: `all_items` の除外リストに `Item::RuneUse { .. }` を追加（両箇所）

## Phase 3: Checker 対応（型チェック + エクスポートスコープ収集）

- [x] `checker.rs`: `check_item()` の no-op リストに `Item::RuneUse` を追加
- [x] `checker.rs`: 第1パス（型収集）の skip リストに `Item::RuneUse` を追加
- [x] `checker.rs`: `process_import_decl` でディレクトリ rune のサイドファイルをマージ
  - `is_rune=true` かつエントリポイントに `RuneUse` アイテムがある場合
  - 各 `RuneUse.module` の `.fav` を読み込み items にマージ
  - マージ済み `Program` で `check_with_self` + `collect_export_scope` を実行

## Phase 4: Compiler / Fmt 対応

- [x] `compiler.rs`: `Item::RuneUse` は `_ => {}` で自動的に no-op
- [x] `fmt.rs`: `Item::RuneUse` の pretty-print 追加済み

## Phase 5: 既存 rune のマルチファイル化

- [x] `runes/db/` — 分割
  - [x] `connection.fav`: `connect`, `close`
  - [x] `query.fav`: `query`, `query_params`, `execute`, `execute_params`
  - [x] `db.fav`: `use connection.*` + `use query.*`（public API なし、委譲のみ）
- [x] `runes/http/` — 分割
  - [x] `request.fav`: `get`, `post`, `post_json`, `get_body`
  - [x] `response.fav`: `ok`, `error_response`
  - [x] `http.fav`: `use request.*` + `use response.*`
- [x] `runes/grpc/` — 分割
  - [x] `server.fav`: `serve`, `serve_stream`
  - [x] `client.fav`: `call`, `call_stream`
  - [x] `codec.fav`: `encode`, `decode`, `ok`, `err`
  - [x] `grpc.fav`: `use server.*` + `use client.*` + `use codec.*`
- [x] `runes/incremental/` — 分割
  - [x] `checkpoint.fav`: `last`, `save`, `reset`, `meta`
  - [x] `pipeline.fav`: `run_since`, `upsert`
  - [x] `incremental.fav`: `use checkpoint.*` + `use pipeline.*`
- [x] `runes/gen/` — 分割
  - [x] `primitives.fav`: `int_val`, `float_val`, `bool_val`, `string_val`, `choice`
  - [x] `structured.fav`: `one`, `list`, `simulate`, `profile`
  - [x] `gen.fav`: `use primitives.*` + `use structured.*`
- [x] `runes/parquet/` — 分割
  - [x] `rawio.fav`: `write`, `read`
  - [x] `parquet.fav`: `use rawio.*` + `append`, `row_count`（rawio の write/read を内部呼び出し）
- json, csv, stat, validate — 小規模のため現状維持

## Phase 6: テスト追加

- [x] `frontend/parser.rs` — パーサーテスト 4 件
- [x] `driver.rs` (rune_multifile_tests) — 統合テスト 5 件
  - [x] `rune_directory_load_basic` — ディレクトリ rune のロードと実行
  - [x] `rune_directory_wildcard_use` — `use X.*` でワイルドカードインポート
  - [x] `rune_single_file_backward_compat` — 単一ファイル rune の後方互換
  - [x] `rune_directory_takes_priority_over_single_file` — ディレクトリ優先
  - [x] `rune_use_missing_module_is_silent` — 存在しないモジュールは静かにスキップ
- [x] 全既存テスト（788 件）がパスすること → 797 件合格（新規 9 件）

## Phase 7: examples + docs

- [x] `fav/examples/rune_multifile_demo/src/main.fav` 作成
- [x] `versions/v4.1.0/spec.md` 作成
- [x] `versions/v4.1.0/plan.md` 作成
- [x] `versions/v4.1.0/tasks.md` 更新（このファイル）
- [x] `memory/MEMORY.md` を v4.1.0 完了状態に更新
