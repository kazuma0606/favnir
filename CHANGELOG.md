# Changelog

Favnir のバージョン履歴。形式は [Keep a Changelog](https://keepachangelog.com/ja/1.0.0/) に準拠。

---

## [v14.6.0] — 2026-06-12 — ドキュメント整備（README + CHANGELOG）

### Changed
- `README.md`: 「現在の状態」見出しを v14.6.0 に更新、ロードマップ表に v14.1.0〜v14.6.0 を追記
- `README.md`: 機能一覧表に Azure Blob Storage / Azure PostgreSQL 行を追加
- `README.md`: 旧 `!Effect` スタイルコード例に `--legacy` モード注記を追加
- `CHANGELOG.md`: v14.1.0〜v14.5.0 エントリを追加

### Notes
- コードの変更なし。純粋なドキュメント更新バージョン
- テスト: v146000_tests 3 件（version_is_14_6_0 / changelog_has_v14_5_0_entry / readme_mentions_azure_blob）

---

## [v14.5.0] — 2026-06-12 — Azure Blob Storage Rune

### New Features
- `azure_blob_sign` ヘルパー関数（`vm.rs`）: HMAC-SHA256 + base64 による Azure Shared Key 署名
  - 既存の `hmac 0.12` + `sha2 0.10` + `base64 0.22` + `chrono` を使用（新規 crate なし）
  - RFC 1123 日付フォーマット、x-ms-* ヘッダーのアルファベット順ソート
- `AzureBlob.put_raw(account, key, container, blob_name, body)` VM primitive（BlockBlob PUT）
- `AzureBlob.get_raw(account, key, container, blob_name)` VM primitive（GET → String）
- `AzureBlob.list_raw(account, key, container, prefix)` VM primitive（GET → JSON 配列文字列）
- `AzureBlob.delete_raw(account, key, container, blob_name)` VM primitive（DELETE）
- `checker.rs`: `require_azure_storage_effect` — `!AzureStorage` 未宣言時に E0317 を発生
- `checker.rs`: `("AzureBlob", "put_raw/get_raw/list_raw/delete_raw")` を `builtin_ret_ty` に追加
- `checker.rs`: `"AzureBlob"` を `BUILTIN_EFFECTS` に追加
- `runes/azure-blob/azure_blob.fav`: `put/get/list/delete` ctx-aware ラッパー（`ctx: String`）
- `runes/azure-blob/rune.toml`: rune メタデータ（version 14.5.0、effects !AzureStorage）

### Notes
- テスト: v145000_tests 4 件（version_is_14_5_0 / azure_blob_put_raw_registered / azure_storage_effect_required / azure_blob_rune_file_present）
- `let` 構文は rune ファイル内でパースエラーになるため引数はインライン化
- `import rune "ctx"` は使用不可（runes/ctx/ctx.fav 未存在）→ `ctx: String` で代替
- LIST の canonical_resource は query params をアルファベット順にソート: `comp:list\nprefix:...\nrestype:container`

---

## [v14.4.0] — 2026-06-12 — AWS Rune 正式パッケージング

### New Features
- `AWS.secrets_get_raw(region, secret_name)` VM primitive（SigV4 + ureq で Secrets Manager `GetSecretValue` API）
- `Ctx.aws_get_field_raw(ctx, field)` VM primitive — AwsCtx JSON 文字列からフィールドを取得
- `checker.rs`: `("AWS", "secrets_get_raw")` を `builtin_ret_ty` に追加（`require_aws_effect` 呼び出し）
- `checker.rs`: `("Ctx", "aws_get_field_raw")` → `Some(Type::String)` を `builtin_ret_ty` に追加
- `runes/aws/secrets.fav`: `secrets_get(ctx: String, secret_name: String)` ラッパー
- `runes/aws/s3.fav`: `s3_put/s3_get/s3_delete/s3_list` ctx-aware ラッパーを追加
- `runes/aws/rune.toml`: version `14.4.0`、description に Secrets Manager を追記

### Notes
- テスト: v144000_tests 4 件（version_is_14_4_0 / secrets_get_raw_registered / aws_ctx_field_raw_registered / aws_rune_s3_ctx_functions_present）
- LocalStack エンドポイント対応（`config.endpoint_url` がある場合は `/` に置換）
- `let` 構文パースエラーのため rune ファイルは全引数インライン化

---

## [v14.3.0] — 2026-06-12 — Azure lineage + !AzureStorage エフェクト

### New Features
- `ast::Effect::AzureStorage` 追加（parser / lineage / checker で認識）
- `lineage.rs`: `EffectKind::AzureDbRead` / `AzureDbWrite` / `AzureBlobRead` / `AzureBlobWrite` 追加
- `lineage.rs`: `collect_azure_blob_call_kinds` / `collect_azure_db_call_kinds` 追加
- `checker.rs`: `BUILTIN_EFFECTS` に `"AzureStorage"` を追加
- `fav explain --lineage` 出力に Azure エフェクトが表示されるよう更新

### Notes
- テスト: v143000_tests

---

## [v14.2.0] — 2026-06-12 — AzureCtx / AwsCtx + fav.toml [azure]

### New Features
- `Ctx.build_aws_raw(region, s3_bucket, db_url)` VM primitive — AwsCtx JSON を生成
- `Ctx.build_azure_raw(postgres_url, storage_account, storage_key, container)` VM primitive — AzureCtx JSON を生成
- `Ctx.aws_get_field_raw(ctx, field)` VM primitive — AwsCtx からフィールドを取得（v14.4.0 で checker に追加）
- `Ctx.azure_get_field_raw(ctx, field)` VM primitive — AzureCtx からフィールドを取得
- `fav.toml` に `[azure]` セクション追加（`postgres_url` / `storage_account` / `storage_key` / `container`）
- `inject_azure_config` — fav.toml の [azure] セクションを env var 展開して実行時 ctx に注入

### Notes
- テスト: v142000_tests

---

## [v14.1.0] — 2026-06-12 — Azure PostgreSQL Rune

### New Features
- `AzurePostgres.execute_raw(conn_str, sql, params)` VM primitive（tokio-postgres + tokio ランタイム）
- `AzurePostgres.query_raw(conn_str, sql, params)` VM primitive（JSON 配列文字列として返す）
- `checker.rs`: `AzurePostgres` namespace を `builtin_ret_ty` / `BUILTIN_EFFECTS` に追加
- `checker.rs`: `require_azure_db_effect` — `!AzureDb` 未宣言時に E0316 を発生
- `ast::Effect::AzureDb` 追加
- `lineage.rs`: `!AzureDb(read/write)` 区別追加
- `runes/azure-postgres/azure_postgres.fav`: `execute/query_rows` ctx-aware ラッパー
- `runes/azure-postgres/rune.toml`: rune メタデータ

### Notes
- テスト: v141000_tests
- SSL: `sslmode=require` を接続文字列に付加して Azure DB for PostgreSQL の SSL 必須要件に対応

---

## [v14.0.0] — 2026-06-11 — 能力型完成宣言

### Breaking Changes
- `!Effect` 記法は非 legacy モードで E0025 エラーになる（v13.10.0 から段階的導入、v14.0.0 で CI 確認完了）
- ambient effect 呼び出し（ctx なしの `IO.println` 等）は E0023 エラーになる（v13.8.0 から）

### New Features (v13.1.0〜v13.10.0 集約)
- `interface` 継承構文（`LoadCtx: CommonCtx`）のコンパイル時チェック
- `DbRead` / `DbWrite` / `StorageRead` / `StorageWrite` / `HttpClient` / `Io` / `Env` capability interface
- `LoadCtx` / `WriteCtx` / `MigrateCtx` コンテキスト interface（capability 充足チェック付き）
- `AppCtx` 具象型 + `Ctx.build` / `Ctx.mock` Rune
- `ctx.field.method()` フィールドアクセス構文
- `seq Pipeline(ctx)` — ctx 型推論
- E0024 型状態パターンチェック
- `Ctx { db: DbRead }` 糖衣構文（v13.10.0）
- `fav migrate --from-effects` 移行ツール（v13.10.0）

### Error Codes Added
- W008: ambient effect call（警告）
- E0020: capability interface has no such method
- E0021: capability not in context
- E0022: ctx-aware pipeline called with wrong number of arguments
- E0023: ambient effect call is not allowed（エラー）
- E0024: type state mismatch
- E0025: bang notation removed
- W009: direct Rune call deprecated
- W010: effect migration requires manual review

### Migration
`fav migrate --from-effects <file>` で旧 `!Effect` 記法を自動変換。
`--legacy` フラグで移行期間中も旧記法を許容（今後廃止予定）。

### Notes
- `self/compiler.fav` / `self/checker.fav` の E0025 件数がゼロであることを CI テストで保証
- テスト: 2207 件（v13.10.0 時点）

---

## [v13.0.0] — 2026-06-09

### Added
- 言語信頼性宣言: 型安全・エラー伝播・デバッグ可視性の三点における保証
- README.md に v13.0.0 宣言文を追記
- `versions/v13.0.0/` — spec / plan / tasks

### Notes
- テスト: 1415 件
- v12.1.0〜v12.10.0 で発覚した全問題（C-1〜C-4 / H-1〜H-2 / M-1 / A-1〜A-6）を解消

---

## [v12.10.0] — 2026-06-09

### Added
- `driver.rs` `get_help_text(code: &str) -> &'static [&'static str]` — 12 コード（E0001/E0007/E0008/E0009/E0013/E0014/E0015/E0018/W001/W004/W006/W007）に `help:` テキストを追加
- `fav check --strict` — W006 警告をエラーとして扱い exit 1（`-D warnings` 相当）
- `fav lint --deny-warnings` — 警告を exit 1 に昇格させる CI 用フラグ
- `fav.toml [lint]` セクション — `warn_as_error` / `allow` リストによる細粒度制御
- `toml.rs` `LintTomlConfig { warn_as_error: Option<Vec<String>>, allow: Option<Vec<String>> }`
- `driver.rs` `v121000_tests` — `help_text_e0001_present` / `help_text_w006_present` / `help_text_unknown_is_empty` / `version_is_12_10_0`
- `tests/integration.rs` — `check_strict_w006_exits_1` / `check_strict_no_warning_exits_0` / `lint_deny_warnings_exits_1`

### Changed
- `format_diagnostic` / `format_warning` — エラー・警告出力末尾に `= help:` 行を自動付与
- `cmd_lint` — `warn_only` に加え `deny_warnings` パラメータを追加; `[lint]` allow フィルタ・warn_as_error 昇格を適用
- `.github/workflows/ci.yml` Self-lint ステップに `--deny-warnings` を追加

### Notes
- テスト: 1415 unit + 8 integration

---

## [v12.9.0] — 2026-06-09

### Added
- `.github/workflows/ci.yml` `Self-test (fav test)` ステップ — `self/checker.fav` / `self/compiler.fav` / `self/codegen.fav` / `self/lexer.fav` / `self/parser.fav`
- `.github/workflows/ci.yml` `integration` ジョブ — `services: postgres:16` (POSTGRES_PASSWORD=test) + health check
- `fav/tests/integration.rs` — `fav_test_self_checker_runs` / `fav_test_self_lexer_runs` / `postgres_create_insert_select` / `postgres_error_table_not_found` / `postgres_ssl_disable_connects`
- `driver.rs` `pg_exec_for_test` / `pg_query_for_test` — 統合テスト用 pub ヘルパー
- `driver.rs` `v12900_tests` — `version_is_12_9_0`

### Notes
- テスト: 1415 件（統合テスト 8 件含む）

---

## [v12.8.0] — 2026-06-09

### Added
- `fav scaffold <template>` コマンド — stage / seq / postgres-etl / rune テンプレートを標準出力に生成
- `driver.rs` `cmd_scaffold(template: &str, name: Option<&str>)` 実装
- `main.rs` `Some("scaffold")` 分岐を追加
- `driver.rs` `v12800_tests` — `scaffold_stage_output_contains_stage` / `scaffold_seq_output_contains_seq` / `scaffold_postgres_etl_output_contains_stages` / `scaffold_rune_output_contains_rune` / `scaffold_stage_named_output_contains_name` / `version_is_12_8_0`（← comment out 済み）

### Notes
- テスト: 1411 件

---

## [v12.7.0] — 2026-06-08

### Added
- `fav doc --builtins [--format json|markdown] [--out <file>]` — 組み込み Primitive の型シグネチャ一覧（IO/Csv/Schema/Json/Gen/AWS/Postgres/Snowflake/Http/Llm）
- `fav explain <code>` — エラーコードの詳細説明（E0001〜E0018 / W001〜W007）
- `driver.rs` `builtin_primitives()` — 組み込み関数メタデータのリスト
- `driver.rs` `cmd_doc_builtins(format, out)` / `cmd_explain_code(code)`
- `driver.rs` `v12700_tests` — `doc_builtins_json_has_csv_parse_raw` / `doc_builtins_markdown_has_postgres` / `explain_e0001_output` / `explain_w006_output` / `doc_builtins_returns_result_field`

### Notes
- テスト: 1408 件

---

## [v12.6.0] — 2026-06-08

### Added
- `tokio-postgres-native-tls` / `native-tls` — Postgres TLS 対応
- `fav.toml [postgres]` `sslmode` キー（`disable` / `prefer` / `require`）
- `DATABASE_URL` の `sslmode` クエリパラメータ解析
- Postgres エラー詳細化 — `DbError.message()` / `code()` / `detail()` を連結（"db error" → "db error: SSL connection is required (SQLSTATE 08P01)"）
- `driver.rs` `v12600_tests` — `postgres_sslmode_disable` / `postgres_sslmode_parse` / `postgres_error_detail`

### Changed
- `pg_connect` — `sslmode` に応じて `NoTls` / `TlsConnector` を切り替え

### Notes
- テスト: 1402 件

---

## [v12.5.0] — 2026-06-08

### Added
- `fav run --verbose` — stage 入出力を stderr に出力（最大 200 文字トランケート）
- `fav run --trace` — stage 入出力をフル出力（トランケートなし）
- `fav.toml [run]` `verbose` / `trace` キー
- `fav check --json` — エラー・警告を JSON 形式で出力（AI フレンドリー）
- `fav check --show-types` — 各 `bind` / `chain` の型と W006 マーカーを表示
- `driver.rs` `CheckDiagnostic` / `BindingInfo` / `CheckOutput` 構造体（serde::Serialize）
- `driver.rs` `collect_binding_types(file)` — W006 検出（`bind _ <- NS.fn(...)` パターン）
- `driver.rs` `v12500_tests` — `verbose_stage_enter_exit` / `check_json_output_format` / `check_show_types_bind` / `check_show_types_w006_detected`

### Changed
- `VERBOSE_LEVEL` を `thread_local! { Cell<u8> }` に変更（並行テスト対応）

### Notes
- テスト: 1386 件

---

## [v12.4.0] — 2026-06-08

### Added
- `IRStmt::SeqChain` + `Opcode::SeqStageCheck = 0x36` — seq pipeline fail-fast
- `compile_flw_def` 修正: 2+ ステージを `SeqChain` stmts で構築
- `SeqStageCheck` VM ハンドラ: stage 名・番号付きエラーで短絡（`"pipeline stopped at stage N/M 'Name': error"`）
- `driver.rs` `v12400_tests` — `seq_stops_on_stage_err` / `seq_passes_ok_through` / `seq_error_includes_stage_name`

### Notes
- テスト: 1376 件

---

## [v12.3.0] — 2026-06-08

### Added
- `IRStmt::LegacyBind(u16, IRExpr)` + `Opcode::LegacyBindCheck = 0x35`
- `apply_legacy_bind_semantics(ir: IRProgram)` — `--legacy` モードで `Bind` → `LegacyBind` に変換
- `LegacyBindCheck` VM ハンドラ: `ok(v)`→unwrap, `err(e)`→escape, 非 Result→pass-through
- `driver.rs` `v12300_tests` — `legacy_bind_propagates_err` / `legacy_bind_ok_unwraps` / `legacy_bind_non_result_passthrough`

### Changed
- `--legacy` モードの `bind x <- expr` が `expr` の Result を unwrap して短絡するように修正（真の monadic bind）

### Notes
- テスト: 1370 件

---

## [v12.2.0] — 2026-06-07

### Added
- `is_result_returning_call(stmt)` — `bind _ <- NS.fn(...)` で Result を返す NS 呼び出しを AST 解析で検出
- W006 警告（`fav check --show-types`）: bind _ で Result を捨てると警告
- 対象 NS: Postgres / Snowflake / S3 / Sqs / Queue / Cache / Http / Grpc / Llm / IO
- `driver.rs` `v12200_tests` — `w006_detected_for_postgres_bind_underscore` / `w006_not_detected_for_named_bind`

### Notes
- テスト: 1357 件

---

## [v12.1.0] — 2026-06-07

### Added
- E0018 `bind` 再束縛禁止（checker.fav）— 同一スコープで同名変数への二重 `bind` を検出
- `check_rebind_ok(name, env)` ヘルパー — `Option<String>` → `Result<String, String>`
- `driver.rs` `v12100_tests` — `e0018_rebind_detected` / `e0018_underscore_allowed` / `e0018_help_message_shown`

### Changed
- `checker.fav` `infer_stmt` に bind 済みセット管理を追加

### Notes
- テスト: 1353 件

---

## [v12.0.0] — 2026-06-06

### Added
- `site/content/docs/transpile/python.mdx` — Python トランスパイラ公式ドキュメント（使用方法・エフェクト対応表・変換例・E2E デモリンク）
- Python トランスパイラ完成宣言（v11.1.0〜v11.9.0 の全機能が揃った）

### Changed
- README.md に `fav transpile --target python` 機能行を追記
- CHANGELOG に v11.1.0〜v11.9.0 の全履歴を追記

### Notes
- テスト: 707 件（v12000_tests 2 件追加）

---

## [v11.9.0] — 2026-06-06

### Added
- `infra/e2e-demo/fav2py/` — Fav ネイティブ vs Python トランスパイル E2E インフラ
  - `src/pipeline.fav` — LoadAndInsert |> Aggregate |> SaveResult（RDS Postgres）
  - `src/sample.csv` — 103 行サンプルデータ（region × category × amount）
  - `terraform/main.tf` — VPC / RDS PostgreSQL (t3.micro) / ECS Fargate x2 / ECR
  - `terraform/iam.tf` — ECS 実行ロール + タスクロール（S3 書き込み）
  - `terraform/variables.tf` / `terraform/outputs.tf`
  - `scripts/upload.sh` — Docker build + ECR push + S3 source upload
  - `scripts/run.sh` — terraform apply → ECS タスク x2 起動 → verify.sh 呼び出し
  - `scripts/verify.sh` — S3 最新 2 件 JSON 比較（native == python）
  - `Dockerfile` — Ubuntu 22.04 + uv + psycopg2-binary + fav binary
- `driver.rs` `v11900_tests` — `fav2py_e2e_demo_structure` / `fav2py_pipeline_fav_transpiles`

### Notes
- テスト: 705 件

---

## [v11.8.0] — 2026-06-06

### Added
- `fav transpile --no-check` オプション（型チェックスキップ）
- `fav transpile --lineage` オプション（生成 Python コードに lineage コメント付与）
- `emit_python.rs` `emit_python_with_lineage(prog, path, HashMap<String,String>) -> String`
- `emit_python.rs` `Emitter` に `lineage_comments: HashMap<String,String>` フィールド追加
- `driver.rs` `build_lineage_comments(report: &LineageReport) -> HashMap<String,String>`
- `driver.rs` `check_source_str_pub(src: &str) -> Vec<TypeError>`（pub ラッパー）
- `driver.rs` `v11800_tests` — 6 件（checker 統合 / lineage コメント検証）

### Changed
- `fav transpile` 実行前に `checker.fav` で型チェックを走らせる（型エラーで Python 生成をブロック）

### Notes
- テスト: 703 件

---

## [v11.7.0] — 2026-06-06

### Added
- `fav transpile --out-dir <dir>` — `main.py` + `pyproject.toml` + `README.md` を出力ディレクトリに生成
- `fav transpile --check` — `python -m py_compile` による構文検証
- `fav transpile --run` — 生成後に `uv run main.py` まで一括実行
- `driver.rs` `build_pyproject_content(py_src, name) -> String`（boto3 / psycopg2 依存を自動検出）
- `driver.rs` `build_readme_content(input_path, name) -> String`
- `driver.rs` `v11700_tests` — 6 件（pyproject 生成 / README 生成 / uv フラグ検証）

### Notes
- テスト: 697 件

---

## [v11.6.0] — 2026-06-06

### Added
- `emit_python.rs` `!Postgres` → psycopg2 変換
  - `_pg_connect()` — `DATABASE_URL` または `PGHOST`/`PGPORT`/etc. から接続
  - `_pg_execute(sql, params)` — INSERT/UPDATE/DELETE ヘルパー
  - `_pg_query(sql, params)` — SELECT → `RealDictCursor` ヘルパー
  - `Postgres.execute_raw` → `_pg_execute(sql, params)`
  - `Postgres.query_raw` → `_pg_query(sql, params)`
- `emit_python.rs` `needs_psycopg2` / `needs_pg_helpers` フラグ追加（2-pass 検出）
- `pyproject.toml` 生成時に `import psycopg2` 検出 → `psycopg2-binary>=2.9` 依存を自動追加
- `driver.rs` `v11600_tests` — 6 件

### Notes
- テスト: 691 件

---

## [v11.5.0] — 2026-06-06

### Added
- `Effect::Postgres` 追加（ast.rs / parser.rs / fmt.rs / lineage.rs / driver.rs / ast_lower_checker.rs / checker.rs / reachability.rs）
- `vm.rs` `Postgres.execute_raw(sql, params_json) -> Result<Unit, String>`（tokio-postgres ベース）
- `vm.rs` `Postgres.query_raw(sql, params_json) -> Result<String, String>`（JSON 文字列返却）
- `vm.rs` `Postgres.query_typed_raw(sql, params_json) -> Result<String, String>`（型付きクエリ）
- `toml.rs` `PostgresTomlConfig` — `fav.toml` `[postgres]` セクション解析
- `runes/postgres/postgres.fav` — `execute` / `query<T>` Rune 実装（`!Postgres` エフェクト）
- `checker.fav` `postgres_fn` / `builtin_ret_ty` / `ns_to_effect` に Postgres 追加
- `driver.rs` `v11500_tests` — 6 件

### Notes
- テスト: 685 件

---

## [v11.4.0] — 2026-06-06

### Added
- `emit_python.rs` `!AWS` → boto3 変換
  - `AWS.s3_put_object_raw(bucket, key, body)` → `boto3.client("s3").put_object(Bucket=..., Key=..., Body=...)`
  - `AWS.s3_get_object_raw(bucket, key)` → `boto3.client("s3").get_object(Bucket=..., Key=...)["Body"].read()`
- `emit_python.rs` `needs_boto3` フラグ追加（2-pass 検出）
- `pyproject.toml` 生成時に `import boto3` 検出 → `boto3>=1.34` 依存を自動追加
- `driver.rs` `v11400_tests` — 4 件

### Notes
- テスト: 679 件

---

## [v11.3.0] — 2026-06-06

### Added
- `emit_python.rs` `!IO` → Python 標準 I/O 変換
  - `IO.println(s)` → `print(s)`
  - `IO.read_file_raw(path)` → `open(path).read()`（try/except で `Result` を模倣）
  - `Csv.parse_raw(text, ",", true)` → `csv.DictReader` 変換ヘルパー生成
  - `Schema.adapt(raw, "T")` → dataclass 変換ヘルパー生成（`_adapt_T(d) -> T`）
  - `Schema.to_json_array(rows, "T")` → `json.dumps([asdict(r) for r in rows])`
- `driver.rs` `v11300_tests` — 4 件

### Notes
- テスト: 675 件

---

## [v11.2.0] — 2026-06-06

### Added
- `emit_python.rs` `stage` / `seq` → Python パイプライン変換
  - `stage Foo: A -> B !Eff = |x| { ... }` → `def foo(x: A) -> B: ...`（エフェクトはコメント）
  - `seq Pipeline = A |> B |> C` → `def pipeline(x): return c(b(a(x)))`
- `fn main()` → `if __name__ == "__main__": main()`
- `IO.argv()` → `sys.argv[1:]`
- `List.map` / `List.filter` / `List.length` → Python リスト内包表記 / `filter` / `len`
- `driver.rs` `v11200_tests` — 4 件

### Notes
- テスト: 671 件

---

## [v11.1.0] — 2026-06-06

### Added
- `src/emit_python.rs` 新規作成 — Favnir AST → Python コード生成基盤
  - 型定義（`type Foo = { ... }`）→ `@dataclass class Foo`
  - 基本式（Int / Float / String / Bool / List / if-else / binary ops）→ Python 式
  - `fn` → `def`（引数型・戻り型をコメントで保持）
  - `bind x <- expr` → `x = expr`（モナド脱糖）
  - `match` → `if/elif/else`（Option / Result パターン）
- `fav transpile --target python <file>` CLI エントリ（`cli.fav` の `CmdTranspile` + `driver.rs` の `cmd_transpile`）
- `driver.rs` `v11100_tests` — 4 件

### Notes
- テスト: 667 件

---

## [v11.0.0] — 2026-06-05

### Added
- `fav explain --lineage` で `!Snowflake(read)` / `!Snowflake(write)` を区別表示（`lineage.rs` `collect_snowflake_call_kinds`）
- `site/content/docs/runes/snowflake.mdx` — Snowflake Rune リファレンスページ

### Changed
- README.md の Rune エコシステム表に `snowflake`（`!Snowflake` エフェクト）を追加
- CHANGELOG に v10.1.0〜v10.9.0 の全履歴を追記

### Notes
- テスト: 1286 件（lineage Snowflake 区別テスト 3 件追加）

---

## [v10.9.0] — 2026-06-05

### Added
- `infra/e2e-demo/snowflake/` — Snowflake E2E デモ（demo.fav 4 ステージ・Terraform・scripts/run.sh・README）
- `driver.rs` `v10900_tests::snowflake_e2e_demo_structure` — ファイル存在確認テスト

### Notes
- テスト: 1283 件

---

## [v10.8.0] — 2026-06-04

### Added
- `fav infer --from snowflake --table <name>` — Snowflake INFORMATION_SCHEMA から Favnir 型定義を自動生成
- `Snowflake.infer_table_raw` VM primitive
- `cli.fav` `CmdInferSnowflake` / `parse_infer_cmd` / `run_infer_snowflake`
- Snowflake 型マッピング（NUMBER→Int / FLOAT→Float / VARCHAR→String / BOOLEAN→Bool / nullable→Option<T>）

### Notes
- テスト: 1282 件（型マッピングテスト 6 件追加）

---

## [v10.7.0] — 2026-06-04

### Added
- `toml.rs` `SnowflakeTomlConfig` — `fav.toml` `[snowflake]` セクション解析（account / user / warehouse / role / database / schema）
- `expand_env_vars` — `${VAR_NAME}` 形式の環境変数参照を展開
- `inject_snowflake_config` — 実行時に Snowflake 設定を環境変数に注入（上書きなし）
- `fav new` テンプレートに `[snowflake]` コメントアウト例を追加

### Notes
- テスト: 1276 件

---

## [v10.6.0] — 2026-06-04

### Added
- `runes/snowflake/` — Snowflake Rune 実装（`execute` / `query<T>`）
- `rune.toml` / `snowflake.fav` / `client.fav` / `snowflake.test.fav`

### Notes
- テスト: 1272 件

---

## [v10.5.0] — 2026-06-04

### Added
- `compiler.fav` builtin NS リストに `"Snowflake"` を追加（2 箇所）
- Favnir pipeline で `Snowflake.*` を含む stage がコンパイル可能になった

### Notes
- テスト: 1271 件

---

## [v10.4.0] — 2026-06-04

### Added
- `checker.fav` に `snowflake_fn` 追加（`execute_raw` / `query_raw` 型シグネチャ）
- `builtin_ret_ty` / `ns_to_effect` に Snowflake エントリ追加
- E0320 エラーコード（`!Snowflake` エフェクト未宣言）

### Notes
- テスト: 1269 件

---

## [v10.3.0] — 2026-06-04

### Added
- `Effect::Snowflake` を 8 ファイルに追加（ast / parser / fmt / lineage / driver / ast_lower_checker / checker / reachability）
- `require_snowflake_effect` (E0314) — `!Snowflake` 未宣言 stage での Snowflake.* 呼び出しを検出

### Notes
- テスト: 1267 件

---

## [v10.2.0] — 2026-06-04

### Added
- `Snowflake.execute_raw` / `Snowflake.query_raw` VM primitive（Snowflake SQL API v2 REST + JWT RS256 認証）
- `snowflake_read_env` / `snowflake_generate_jwt` / `snowflake_api_post` ヘルパー（`vm.rs`）

### Notes
- テスト: 1264 件

---

## [v10.1.0] — 2026-06-04

### Added
- `infra/snowflake/` — Snowflake Terraform インフラ（provider / warehouse / database / schema / role / RSA キー / SSM）
- `infra/snowflake/README.md`

### Notes
- テスト: 1261 件

---

## [v10.0.0] — 2026-06-03

### Added
- `fav new <name>` — プロジェクトスキャフォールディング（fav.toml / src/main.fav / .gitignore 生成）
- `IO.make_dir_raw` VM primitive（ディレクトリ作成）
- GitHub Actions CI に self-check ステップ追加（fav check / fav lint / fav fmt --check）
- `CONTRIBUTING.md` を現状に合わせて更新

### Notes
- テスト: 1260 件（fav_new 統合テスト 2 件追加）

---

## [v9.13.0] — 2026-06-03

### Added
- `par [A, B] |> Merge` — 並列 stage 実行（`std::thread::spawn` VM スレッド並列化）
- E0016（par ステップ入力型不一致）/ E0017（par 内未定義 stage）
- `compiler.fav` / `checker.fav` に `SeqStep` / `SeqDef` / `IStage` / `ISeq` 型追加
- `ast_lower_checker.rs` に `lower_trf_def` / `lower_flw_def` / `te_to_string` 追加

### Notes
- テスト: 1258 件

---

## [v9.12.0] — 2026-06-02

### Added
- `interface` / `impl ... for` / `type T with Iface` を `checker.fav` / `compiler.fav` でセルフホスト対応
- E0014（MissingImpl）/ E0015（ImplMethodNotFound）
- LSP: Rune 定義ジャンプ（`textDocument/definition`）

### Notes
- テスト: 1251 件

---

## [v9.11.0] — 2026-06-01

### Added
- LSP: フィールド補完・モジュール補完（`List.` / `String.` 等）・Rune 補完
- LSP: Signature help（関数呼び出し時の型シグネチャ表示）
- `textDocument/completion` / `textDocument/signatureHelp` ハンドラ

### Notes
- テスト: 1240 件

---

## [v9.10.0] — 2026-05-31

### Added
- `fav repl` — インタラクティブ REPL（式評価・定義累積・`:type` / `:reset` / `:env`）
- `cmd_repl` in `cli.fav`

### Notes
- テスト: 1220 件

---

## [v9.9.0] — 2026-05-31

### Added
- `fav profile` — stage 別実行時間計測（`--profile` フラグ）
- `fav watch` — ファイル監視 + 自動再実行（500ms ポーリング）

### Notes
- テスト: 1217 件

---

## [v9.8.0] — 2026-05-31

### Added
- `fav doc` — `///` ドキュメントコメント + 型シグネチャから Markdown 自動生成
- `cmd_doc` in `cli.fav`、`doc_item` / `doc_program` in `compiler.fav`

### Notes
- テスト: 1213 件

---

## [v9.7.5] — 2026-05-31

### Added
- `where` バリデーター（`type Email(String) where |v| String.contains(v, "@")`）
- E0013（`expr?` を非 Result 関数内で使用）

### Fixed
- Float シリアライズで整数値に小数点が付かないバグを修正

### Notes
- テスト: 1206 件

---

## [v9.7.0] — 2026-05-31

### Added
- 名目型ラッパー `type Name(Inner)` — コンストラクタ・パターンマッチ対応
- `T?` / `T!` / `??` / `expr?` を self-hosted pipeline で対応
- `with Eq, Show, Serialize, Deserialize` 自動合成

### Notes
- テスト: 1200 件

---

## [v9.6.0] — 2026-05-31

### Added
- `!Llm` エフェクト追加
- `llm` Rune — `llm.complete` / `llm.chat` / `llm.extract<T>`（Claude / OpenAI 対応）
- `LLM_PROVIDER` / `LLM_MODEL` 環境変数で切り替え

### Notes
- テスト: 1191 件

---

## [v9.5.0] — 2026-05-31

### Added
- `!Http` エフェクト追加
- `http` Rune 拡張（`get_text` / `get_json<T>` / `post_json_typed<T,R>`）
- `grpc` Rune 拡張・`graphql` Rune 新規作成

### Notes
- テスト: 1187 件

---

## [v9.4.0] — 2026-05-31

### Added
- `json` Rune — `encode<T>` / `decode<T>` / `pretty`
- `csv` Rune 拡張 — `read<T>` / `write_file<T>`
- `gen` Rune 拡張 — `uuid` / `uuid_v7` / `nano_id`
- W004 lint ルール（`fn` の引数が 4 個以上 → レコード型推奨）

### Notes
- テスト: 1182 件

---

## [v9.3.0] — 2026-05-31

### Added
- `fav lint` — W001〜W005 静的解析ルールエンジン（compiler.fav + cli.fav）
- W001（EffectlessSink）/ W002（NoWriteInSeq）/ W003（UnusedBinding）/ W005（WildcardOnlyMatch）

### Notes
- テスト: 1173 件

---

## [v9.2.0] — 2026-05-31

### Added
- `fav fmt` — コードフォーマッタ（compiler.fav の pretty printer、冪等性保証）
- `Compiler.fmt_source_raw` VM primitive
- `--check` フラグ（CI 向け）

### Notes
- テスト: 1167 件

---

## [v9.1.0] — 2026-05-31

### Added
- stdlib 大幅拡充（`List.chunk` / `flat_map` / `group_by` / `zip_with` / `unique` 等 30 関数超）
- `rvm` 独立バイナリ（`src/bin/rvm.rs`）
- マルチパラメータクロージャ `|x, y| x + y` 対応
- E0012（非ジェネリック関数引数数不一致）

### Notes
- テスト: 1162 件

---

## [v9.0.0] — 2026-05-31

### Changed
- **セルフホスト完成宣言**: `fav run` / `fav check` の全経路が Favnir pipeline 経由で動作
- `--legacy` フラグ非推奨化

### Notes
- テスト: 1136 件

---

## [v7.0.0] — 2026-05-27

### Added
- `Effect::DbRead` / `Effect::DbWrite` / `Effect::DbAdmin` を型システムに追加（`ast.rs`）
- `parser.rs`：`!DbRead` / `!DbWrite` / `!DbAdmin` のパースに対応
- `checker.rs`：BUILTIN_EFFECTS 更新、`require_db_write_effect` / `require_db_admin_effect` 追加
- `reachability.rs`：3 エフェクトのリーチャビリティ追跡に対応
- `fmt.rs` / `driver.rs`：3 エフェクトの表示・JSON 出力に対応
- `runes/db/query.fav`：query 系 → `!DbRead`、execute 系 → `!DbWrite` に更新
- `runes/db/transaction.fav`：`!Db` → `!DbWrite` に更新
- `runes/db/migration.fav`：`applied_migrations` → `!DbRead`、`mark_applied` / `ensure_migrations_table` → `!DbAdmin` に更新
- `site/content/docs/guides/schema-authority.mdx`：Schema Authority 全体ワークフローガイド新規作成
- `site/content/docs/runes/db.mdx`：エフェクト細分化テーブル追記

### Changed
- `require_db_effect`：後方互換化（`Db | DbRead | DbWrite | DbAdmin` をすべて受け入れる）

### Notes
- テスト: 1044 件（パーサーテスト +1）
- 後方互換: 既存の `!Db` を使ったコードは変更なしに動く

---

## [v6.9.0] — 2026-05-27

### Added
- `LICENSE`（MIT）をリポジトリルートに配置
- `CONTRIBUTING.md`：ビルド手順・テスト手順・PR ガイドライン・Rune 追加ガイド
- `CHANGELOG.md`（本ファイル）
- CI に `cargo clippy -- -D warnings` を追加

---

## [v6.8.0] — 2026-05-27

### Added
- `site/content/docs/runes/db.mdx`：db rune リファレンス（connect / query / paginate / batch_insert / with_transaction / savepoint）
- `site/content/docs/runes/http.mdx`：http rune リファレンス（GET/POST/PUT/DELETE/PATCH / retry / bearer/basic/api_key）
- `site/content/docs/runes/duckdb.mdx`：query_one / explain / Parquet・CSV IO セクション追記（read_parquet / read_csv / write_parquet / write_csv）

---

## [v6.6.0] — 2026-05-27

### Added
- `one_of` 制約：`schemas/*.yaml` で列挙値バリデーションが可能に
- `TypeName.validate(record)`：VM 動的 dispatch による型付きバリデーション
- `Validate.rows_raw(type_name, rows)`：複数行一括バリデーション builtin
- 統合テスト 10 件追加（`vm_stdlib_tests.rs`）、合計 1043 件

### Changed
- `schema.mdx`：preview Note を削除、`Order.validate` / `Validate.rows_raw` のコード例を追加

---

## [v6.5.0] — 2026-05-27

### Added
- `site/content/docs/language/pipeline.mdx`：stage / seq / `|>` / abstract stage・seq / fav explain ドキュメント
- `site/content/docs/language/schema.mdx`：schemas/*.yaml 構文・制約一覧・T.validate・fav build --schema ドキュメント
- `site/content/docs/stdlib/infer.mdx`：fav infer --csv / --db / --proto / --out ドキュメント
- `site/content/docs/rune-cli.mdx`：fav deploy（Lambda）/ fav build --schema セクション追記

---

## [v6.4.0] — 2026-05-27

### Added
- `scripts/build-wasm.sh`：wasm-pack build → `site/public/wasm/` 自動出力
- WASM バックエンドで `List` 型対応（list_singleton / list_length / list_is_empty）
- Playground サンプルを stage/seq パイプライン例に更新

### Fixed
- WASM メモリ設定（`minimum: 2` / 128KB）で heap が有効化

---

## [v6.3.0] — 2026-05-26

### Added
- `compiler.fav` に `stage` / `seq` / `|>` のパース・lowering を追加
- `bootstrap_stage_seq_self_host_executes_correctly` テスト追加

---

## [v6.2.0] — 2026-05-25

### Added
- Bootstrap 3 段階検証確立（Stage1→Stage2→Stage3、`bytecode_A == bytecode_B`）
- 新オペコード 5 種：`CallNamed` / `JumpIfNotVariantC` / `GetFieldC` / `BuildRecordC` / `MakeClosureN`
- `String.to_bytes`（`-> List<Int>`）

### Fixed
- self-host 成熟度ドキュメント整備（`semantic_gap_audit.md` 等）

---

## [v6.1.0] — 2026-05-24

### Added
- `compiler.fav`：lexer.fav / parser.fav / codegen.fav をフルインライン化
- `bootstrap_stage1_builds_and_serializes` テスト追加

### Fixed
- `codegen.rs`：`remap_string_operands` が `Swap`/`TrackLine` を未認識で中断するバグを修正

---

## [v6.0.0] — 2026-05-21

### Added
- セルフホストコンパイラ完成（`fav/self/compiler.fav`）
- Favnir 製レキサー（`lexer.fav`）・パーサー（`parser.fav`）・型チェッカー（`checker.fav`）・コード生成器（`codegen.fav`）
- `IO.argv` / `List.take_while` / `List.drop_while` / `List.singleton` を VM に追加
- Bootstrap Stage1 実行テスト群

### Fixed
- `JumpIfNotVariant`：`VMValue::VariantCtor`（引数なしバリアント）のパターンマッチが正しく動作しないバグを修正

---

## [v5.5.0] — 2026-05

### Added
- `Map.remove` / `Map.contains_key` / `String.from_chars`
- `Option.and_then` / `Result.and_then`

---

## [v5.0.0〜v5.4.0] — 2026-05

### Added
- AWS Lambda / S3 / SQS 本番稼働
- SigV4 認証（セッショントークン対応）
- CloudFront + S3 リファレンスサイト公開
- `fav deploy`（Lambda）実装
- Import 解決順序：`rune_modules/` → `runes/` → `~/.fav/registry/`

---

## [v4.12.0〜v4.1.0] — 2025〜2026

### Added
- Rune エコシステム構築：aws / duckdb / db / http / auth / log / env / gen / grpc / json / parquet / csv / incremental / stat / validate
- `fav test` / `fav bench` / `fav check` / `fav run` CLI
- `fav explain`（パイプライン可視化）/ `fav infer`（型推論）/ `fav build --schema`（DDL 生成）
- `stage` / `seq` / `|>` パイプライン構文
- `abstract stage` / `abstract seq`（依存注入）
- パターンマッチ（ネスト・ガード・バリアント）
- `collect` / `yield` / クロージャ
- ジェネリクス・インターフェース・エフェクト型チェッカー
- バイトコードコンパイラ + VM
- WASM バックエンド（`favnir-wasm`）
- LSP（hover・diagnostics）
- `schemas/*.yaml` によるスキーマ制約システム
- LocalStack 対応（`AWS_ENDPOINT_URL` 切り替え）

---

[v6.9.0]: https://github.com/kazuma0606/favnir/releases/tag/v6.9.0
[v6.8.0]: https://github.com/kazuma0606/favnir/releases/tag/v6.8.0
[v6.6.0]: https://github.com/kazuma0606/favnir/releases/tag/v6.6.0
[v6.5.0]: https://github.com/kazuma0606/favnir/releases/tag/v6.5.0
[v6.4.0]: https://github.com/kazuma0606/favnir/releases/tag/v6.4.0
[v6.3.0]: https://github.com/kazuma0606/favnir/releases/tag/v6.3.0
[v6.2.0]: https://github.com/kazuma0606/favnir/releases/tag/v6.2.0
[v6.1.0]: https://github.com/kazuma0606/favnir/releases/tag/v6.1.0
[v6.0.0]: https://github.com/kazuma0606/favnir/releases/tag/v6.0.0
