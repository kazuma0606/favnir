# Favnir v11.0.0 仕様書

Date: 2026-06-05
Theme: Snowflake 統合完成 + リネージ可視化 + ドキュメント整備

---

## 概要

v10.1.0〜v10.9.0 で積み上げた Snowflake ネイティブ対応を統合し、
**Snowflake を Favnir の第一級データソース**として公式宣言する。

主な変更:
1. `fav explain --lineage` に `!Snowflake(read)` / `!Snowflake(write)` 区別表示を追加
2. CHANGELOG.md に v10.1.0〜v11.0.0 を追記
3. README.md の Rune エコシステム表に `snowflake` 追加
4. サイトドキュメント: `site/content/docs/runes/snowflake.mdx` 新規作成
5. バージョン 11.0.0

---

## Phase A: lineage.rs — `!Snowflake(read/write)` 区別

### 設計

現状: `!Snowflake` effect を持つ stage は一律 `"!Snowflake"` と表示。

v11.0.0 の目標出力:

```
Lineage: etl.fav

stage LoadCsv          !Io
stage TransformRows    Pure
stage SnowflakeInsert  !Snowflake(write)
stage QuerySummary     !Snowflake(read)

seq DemoPipeline  [LoadCsv → TransformRows → SnowflakeInsert → QuerySummary]
  sources: (SnowflakeInsert:snowflake-write)
  sinks:   (QuerySummary:snowflake-read)
```

### 実装アプローチ

`lineage.rs` に以下を追加:

1. `collect_snowflake_call_kinds(expr: &ast::Expr) -> (bool, bool)` — `(has_read, has_write)`
   - `snowflake.query(...)` / `snowflake.query_raw(...)` → `has_read = true`
   - `snowflake.execute(...)` / `snowflake.execute_raw(...)` → `has_write = true`
   - 判定: `FieldAccess(Ident("snowflake"), "query"` / `"execute"`)

2. `lineage_analysis` の TrfDef ループを更新:
   - `Effect::Snowflake` を持つ stage に対して `collect_snowflake_call_kinds` を呼ぶ
   - `has_read` → effects に `"!Snowflake(read)"` を追加
   - `has_write` → effects に `"!Snowflake(write)"` を追加
   - 区別できない場合（両方 false）→ `"!Snowflake"` のまま
   - `sources` / `sinks` にも `(stage:snowflake-read)` / `(stage:snowflake-write)` を追加

3. `format_effects` は変更不要（`Effect::Snowflake` 表示は lineage 以外で引き続き `"!Snowflake"` でよい）

### Rust テスト（+3）

`lineage.rs` の `#[cfg(test)] mod tests` に追加:

- `lineage_snowflake_write_stage_shows_write_label` — execute 呼び出しを持つ stage が `!Snowflake(write)` になる
- `lineage_snowflake_read_stage_shows_read_label` — query 呼び出しを持つ stage が `!Snowflake(read)` になる
- `lineage_snowflake_undistinguished_falls_back` — Snowflake 呼び出しなしは `!Snowflake` のまま

---

## Phase B: CHANGELOG.md 更新

v10.1.0〜v11.0.0 の全バージョンを追記。

---

## Phase C: README.md 更新

Rune エコシステム表に `snowflake`（`!Snowflake` エフェクト）を追加:

```markdown
| **Rune エコシステム** | AWS / DuckDB / SQL / DB / fs / Parquet | ✓ |
| | http / grpc / graphql（`!Http` エフェクト） | ✓ |
| | llm（`!Llm` エフェクト、Claude / OpenAI） | ✓ |
| | snowflake（`!Snowflake` エフェクト） | ✓ |
```

ロードマップ表にも v10.x / v11.0.0 を追記。

---

## Phase D: site/content/docs/runes/snowflake.mdx

既存 `aws.mdx` / `http.mdx` と同構造で Snowflake Rune リファレンスページを作成。

目次:
- 概要（Snowflake SQL API v2 via REST）
- インストール（`import rune "snowflake"`）
- `fav.toml` 設定（`[snowflake]` セクション）
- 環境変数一覧
- API リファレンス（`execute` / `query<T>`）
- `fav infer --from snowflake` の使い方
- `fav explain --lineage` でのエフェクト可視化
- コード例（完全な ETL パイプライン）

---

## バージョン更新

- `fav/Cargo.toml` version → `"11.0.0"`
- `fav/self/cli.fav` の `run_version` → `"11.0.0"`

---

## テスト

| テスト | 件数 |
|---|---|
| `lineage_snowflake_write_stage_shows_write_label` | +1 |
| `lineage_snowflake_read_stage_shows_read_label` | +1 |
| `lineage_snowflake_undistinguished_falls_back` | +1 |
| **合計** | **+3 → 1286 件** |

---

## 完了条件

| 条件 | 状態 |
|---|---|
| `fav explain --lineage` で `!Snowflake(read)` / `!Snowflake(write)` が区別表示される | - |
| `cargo test` 全件通過（1286 件） | - |
| CHANGELOG.md に v10.1.0〜v11.0.0 が記載されている | - |
| README.md の Rune 表に snowflake が含まれる | - |
| `site/content/docs/runes/snowflake.mdx` が存在する | - |
