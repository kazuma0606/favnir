# v35.8.0 (v35.0C) — !Effect 廃止完結（LSP / エラーカタログ / MCP / help テキスト）

## バージョン概要

| 項目 | 内容 |
|---|---|
| バージョン | v35.8.0 |
| コードネーム | v35.0C |
| 目的 | 残存する `!Effect` 表記をすべて除去し、廃止を完結させる |
| 前提 | v35.7.0（v35.0B）完了済み — `docs_server.rs` 修正完了 |

## 背景

v35.7.0 でコードベースの主要な `!Effect` 参照は除去されたが、以下の 4 ファイルに残存が確認された。
いずれもユーザーが直接目にする（IDE 補完・エラーメッセージ・CLI help・MCP ドキュメント）。

## 残存箇所と修正方針

### 1. `fav/src/lsp/completion.rs`（最重要）

**問題**: VSCode 等の IDE の補完ポップアップに表示されるシグネチャに `!Effect` が残存。

**対象関数グループ**:
| 関数 | 修正前 | 修正後 |
|---|---|---|
| `IO.println` | `"(s: String) -> Unit !Io"` | `"(s: String) -> Unit"` |
| `IO.print` | `"(s: String) -> Unit !Io"` | `"(s: String) -> Unit"` |
| `IO.read_line` | `"() -> String !Io"` | `"() -> String"` |
| `IO.read_file` | `"(path: String) -> Result<String, String> !Io"` | `"(path: String) -> Result<String, String>"` |
| `IO.write_file` | `"(path: String, content: String) -> Result<Unit, String> !Io"` | `"(path: String, content: String) -> Result<Unit, String>"` |
| `IO.append_file` | `"(path: String, content: String) -> Result<Unit, String> !Io"` | `"(path: String, content: String) -> Result<Unit, String>"` |
| `IO.file_exists` | `"(path: String) -> Bool !Io"` | `"(path: String) -> Bool"` |
| `IO.now_ms` | `"() -> Int !Io"` | `"() -> Int"` |
| `Csv.read` | `"(path: String) -> Result<List<'a>, String> !Io"` | `"(path: String) -> Result<List<'a>, String>"` |
| `Csv.write_file` | `"(path: String, rows: List<'a>) -> Result<Unit, String> !Io"` | `"(path: String, rows: List<'a>) -> Result<Unit, String>"` |
| `Gen.uuid` | `"() -> String !Gen"` | `"() -> String"` |
| `Gen.uuid_v7` | `"() -> String !Gen"` | `"() -> String"` |
| `Gen.nano_id` | `"(n: Int) -> String !Gen"` | `"(n: Int) -> String"` |
| `Http.*` (4関数) | `... !Http` | `!Http` 除去 |
| `Llm.*` (3関数) | `... !Llm` | `!Llm` 除去 |
| `Db.*` (3関数) | `... !Db` | `!Db` 除去 |
| `Sys.sleep` | `"(ms: Int) -> Unit !Io"` | `"(ms: Int) -> Unit"` |

**Rune 説明文（配列リテラル）**:
```
("cache", "Cache operations !Cache")  →  ("cache", "Cache operations")
("csv",   "CSV read/write !Io")       →  ("csv",   "CSV read/write")
("db",    "SQL database !Db")         →  ("db",    "SQL database")
("email", "Email sending !Io")        →  ("email", "Email sending")
("fs",    "Filesystem operations !Io") →  ("fs",   "Filesystem operations")
("graphql","GraphQL client !Http")    →  ("graphql","GraphQL client")
("grpc",  "gRPC client !Http")        →  ("grpc",  "gRPC client")
("http",  "HTTP client !Http")        →  ("http",  "HTTP client")
("llm",   "LLM (Claude/OpenAI) !Llm") →  ("llm",  "LLM (Claude/OpenAI)")
("queue", "Message queue !Queue")     →  ("queue", "Message queue")
("slack", "Slack messaging !Io")      →  ("slack", "Slack messaging")
("sql",   "SQL query builder !Db")    →  ("sql",   "SQL query builder")
```

### 2. `fav/src/error_catalog.rs`（重要）

**問題**: E0310〜E0319 エラーの `fix:` フィールドが廃止済みの `!Effect` 構文を修正提案している。

**修正方針**: `fix:` を「`ctx: AppCtx` パラメータを追加してください」に書き換える。

| エラー | 修正前 | 修正後 |
|---|---|---|
| E0310 `!Db` | `"Add \`!Db\` to the function signature: ..."` | `"Pass a capability context parameter: \`fn query(ctx: AppCtx) -> String\`."` |
| E0311 `!Auth` | 同様 | `"Pass a capability context parameter: \`fn verify(ctx: AppCtx, token: String) -> Bool\`."` |
| E0312 `!Env` | 同様 | `"Pass a capability context parameter: \`fn cfg(ctx: AppCtx) -> String\`."` |
| E0313 `!AWS` | 同様 | `"Pass a capability context parameter: \`fn upload(ctx: AppCtx, key: String) -> Unit\`."` |
| E0314 `!Snowflake` | 同様 | `"Pass a capability context parameter: \`fn run(ctx: AppCtx, sql: String) -> Result<String, String>\`."` |
| E0315 `!Postgres` | 同様 | `"Pass a capability context parameter: \`fn run(ctx: AppCtx, sql: String) -> Result<String, String>\`."` |
| E0319 `!Stream` | 同様 | `"Pass a capability context parameter: \`fn run(ctx: AppCtx, topic: String) -> Result<Unit, String>\`."` |

### 3. `fav/src/mcp/mod.rs`（中）

**問題**: MCP プロトコル向けの Rune ドキュメント文字列（`db` / `http` / `log` rune の関数シグネチャ）に `!Io` が残存。

**修正**: 各関数シグネチャの末尾から `!Io` を除去。

### 4. `fav/src/main.rs`（低）

**問題**: `--help` テキストのコメント行 1 行に `!DbRead/!DbWrite effects` が残存。

**修正**: `!DbRead/!DbWrite effects` → `DbRead/DbWrite lineage tags` に書き換え。

## テスト方針

`v35800_tests` モジュールを `driver.rs` に追加（5 テスト）：

1. `cargo_toml_version_is_35_8_0` — Cargo.toml バージョン確認
2. `lsp_completion_signatures_no_effect` — `completion.rs` の全 signature 文字列に `!` + 大文字が含まれないこと
3. `error_catalog_fix_no_effect_syntax` — `error_catalog.rs` の `fix:` フィールドに `!Db`/`!Auth`/`!Env` 等が含まれないこと
4. `mcp_docs_no_effect_annotation` — `mcp/mod.rs` の文字列に `!Io` が含まれないこと
5. `changelog_has_v35_8_0` — CHANGELOG エントリ確認

## 影響範囲

| ファイル | 変更内容 |
|---|---|
| `fav/src/lsp/completion.rs` | ~25 行の signature / description 文字列から `!Effect` 除去 |
| `fav/src/error_catalog.rs` | E0310〜E0319 の `fix:` フィールド 7 件書き換え |
| `fav/src/mcp/mod.rs` | db/http/log rune ドキュメント ~12 行から `!Io` 除去 |
| `fav/src/main.rs` | help テキスト 1 行修正 |
| `fav/Cargo.toml` | バージョンを `35.8.0` に更新 |
| `CHANGELOG.md` | `## [35.8.0]` エントリ追加 |
| `fav/src/driver.rs` | `v35800_tests` モジュール追加（5 テスト） |

## 完了条件

- [ ] `lsp/completion.rs` に `!` + 大文字で始まるエフェクト表記が存在しない
- [ ] `error_catalog.rs` の `fix:` フィールドが `ctx: AppCtx` ベースの修正提案になっている
- [ ] `mcp/mod.rs` のドキュメント文字列に `!Io` が存在しない
- [ ] 全テスト pass（0 failures）
- [ ] Cargo.toml バージョン = `35.8.0`
