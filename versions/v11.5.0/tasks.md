# Favnir v11.5.0 Tasks

Date: 2026-06-06
Theme: `!Postgres` エフェクト + Fav ネイティブ Postgres 操作

---

## Phase A — Effect::Postgres 追加（8 ファイル + error_catalog）

- [x] A-1: `ast.rs` — `Effect::Postgres` 追加（Snowflake の直後）
- [x] A-2: `parser.rs` — `"Postgres"` → `Effect::Postgres` 解析追加
- [x] A-3: `fmt.rs` — `Effect::Postgres => "!Postgres"` 追加
- [x] A-4: `lineage.rs` — `Postgres => "!Postgres"` 追加
- [x] A-5: `driver.rs` — effect 表示 2 箇所（長表示 + 短縮名）追加
- [x] A-6: `ast_lower_checker.rs` — lowering 追加
- [x] A-7: `checker.rs` — builtin NS 2 箇所・effects 2 箇所・`require_postgres_effect`（E0315）・型シグネチャ 3 件追加
- [x] A-8: `reachability.rs` — `Effect::Postgres` ブランチ追加
- [x] A-9: `error_catalog.rs` — E0315 エントリ追加

---

## Phase B — Cargo.toml + vm.rs プリミティブ

- [x] B-1: `fav/Cargo.toml` に `tokio-postgres = "0.7"` 追加（native-only）
- [x] B-2: `vm.rs` に `Postgres.execute_raw(sql, params_json)` プリミティブ追加
- [x] B-3: `vm.rs` に `Postgres.query_raw(sql, params_json)` プリミティブ追加
- [x] B-4: `vm.rs` に `pg_conn_str_from_env` / `pg_execute` / `pg_query` ヘルパー追加（env var ベース）
- [x] B-5: `vm.rs` に `Postgres.infer_table_raw(table)` プリミティブ追加

---

## Phase C — compiler.rs builtin NS 追加

- [x] C-1: `compiler.rs` builtin namespace リスト 2 箇所に `"Postgres"` 追加

---

## Phase D — checker.fav 更新

- [x] D-1: `fav/self/checker.fav` に `postgres_fn` 関数追加
- [x] D-2: `builtin_ret_ty` に Postgres エントリ追加
- [x] D-3: `ns_to_effect` に `"Postgres"` エントリ追加

---

## Phase E — runes/postgres/postgres.fav Rune 実装

- [x] E-1: `runes/postgres/rune.toml` 作成
- [x] E-2: `runes/postgres/postgres.fav` + `client.fav` 作成（execute / query<T>）

---

## Phase F — fav.toml [postgres] セクション

- [x] F-1: `PostgresTomlConfig` 構造体追加（FavToml に postgres フィールド）
- [x] F-2: `parse_fav_toml` で `[postgres]` セクション読み込み + env var 展開

---

## Phase G — fav infer --from postgres

- [x] G-1: `cmd_infer_postgres` 追加、`main.rs` に `--from postgres` ブランチ追加
- [x] G-2: `information_schema.columns` → Fav 型定義生成（PG 型 → Fav 型マッピング）

---

## Phase H — テスト（5件）

- [x] H-1: `v11500_tests` モジュール追加
  - [x] `postgres_execute_requires_effect` — `!Postgres` なし → E0315
  - [x] `postgres_execute_with_effect_ok` — `!Postgres` あり → エラーなし
  - [x] `postgres_query_raw_type` — `query_raw` 戻り型が `Result<String, String>`
  - [x] `postgres_lineage_shows_effect` — lineage 出力に `!Postgres` 含む
  - [x] `fav_toml_postgres_section_parsed` — `[postgres]` セクションが FavToml に読み込まれる
- [x] H-2: `cargo test v11500` — 5 件通過
- [x] H-3: `cargo test --lib` — 705 件通過

---

## Phase I — バージョン更新 + コミット

- [x] I-1: `fav/Cargo.toml` version → `"11.5.0"`
- [x] I-2: `cargo build` で `Cargo.lock` 更新
- [ ] I-3: `git commit & push` — CI 確認

---

## 完了条件サマリー

| 確認項目 | 状態 |
|---|---|
| `!Postgres` エフェクトが 8 ファイルに追加される | ✅ |
| E0315 — `!Postgres` 未宣言エラー | ✅ |
| `Postgres.execute_raw` / `Postgres.query_raw` VM プリミティブ動作 | ✅ |
| `fav.toml [postgres]` セクション読み込み + env var 展開 | ✅ |
| `runes/postgres/postgres.fav` Rune 動作 | ✅ |
| `fav infer --from postgres --table <name>` 型定義生成 | ✅ |
| `cargo test v11500` 5 件通過 | ✅ |
| `cargo test --lib` 705 件通過 | ✅ |
