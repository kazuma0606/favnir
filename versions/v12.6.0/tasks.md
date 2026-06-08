# Favnir v12.6.0 Tasks

Date: 2026-06-08
Theme: Postgres Rune TLS 対応 + エラー詳細化

---

## Phase A — 依存クレート追加

- [x] A-1: `fav/Cargo.toml` に `tokio-postgres-rustls` / `rustls` / `webpki-roots` を追加
- [x] A-2: `cargo build` が通ることを確認（バージョン整合チェック）

---

## Phase B — エラー詳細化

- [x] B-1: `format_pg_error(e: &tokio_postgres::Error) -> String` を vm.rs に追加
  - `as_db_error()` → `message()` / `code()` / `detail()` を連結
  - 非 DB エラーは `format!("db error: {}", e)` にフォールバック
- [x] B-2: `pg_execute` の `.map_err(|e| e.to_string())` → `format_pg_error` に置換
- [x] B-3: `pg_query` の `.map_err(|e| e.to_string())` → `format_pg_error` に置換

---

## Phase C — sslmode 実装

- [x] C-1: `PostgresTomlConfig.sslmode` は既存（v11.5.0 時点で実装済み）
- [x] C-2: `FavToml.postgres: Option<PostgresTomlConfig>` は既存
- [x] C-3: `resolve_sslmode(conn_str) -> String` を vm.rs に実装
  - 優先順位: `DATABASE_URL ?sslmode=` > `PGSSLMODE` env > `"prefer"`
- [x] C-4: `pg_connect_inner(conn_str, sslmode) -> Result<Client, String>` を実装
  - `"disable"` → `NoTls`
  - それ以外 → `MakeRustlsConnect`（webpki-roots の CA bundle）
- [x] C-5: `pg_execute` を `pg_connect_inner` 使用に変更
- [x] C-6: `pg_query` を `pg_connect_inner` 使用に変更

---

## Phase D — fav.toml パース更新

- [x] D-1: `inject_postgres_config` を driver.rs に追加（`PGSSLMODE` を env var に設定）
- [x] D-2: `run_project_with_toml` と legacy run path で `inject_postgres_config` を呼ぶ

---

## Phase E — テスト追加

- [x] E-1: `postgres_sslmode_from_url` — `?sslmode=require` を URL から読む
- [x] E-2: `postgres_sslmode_from_url_with_extra_params` — `&` 以降は無視される
- [x] E-3: `postgres_sslmode_default_is_prefer` — デフォルトが "prefer"
- [x] E-4: `inject_postgres_config_sets_pgsslmode` — toml の sslmode が PGSSLMODE に設定される
- [x] E-5: `postgres_error_format_non_db` — 非 DB エラーが "db error: ..." 形式
- [x] E-6: `version_is_12_6_0`
- [x] E-7: `cargo test v12600` — 6 件通過確認

---

## Phase F — バージョン更新・コミット

- [x] F-1: `fav/Cargo.toml` version → `"12.6.0"`
- [x] F-2: `cargo test` — 全通過（1392 件）
- [ ] F-3: `git commit -m "feat: v12.6.0 — Postgres TLS (rustls) + error detail"`
- [ ] F-4: `git push` → CI 通過確認

---

## 完了条件サマリー

| 確認項目 | 状態 |
|---|---|
| `sslmode=require` で TLS 接続パスを通る | ✅ |
| `sslmode=disable` で既存 NoTls 接続がリグレッションなし | ✅ |
| エラー文字列に `message()` + `SQLSTATE` が含まれる | ✅ |
| `DATABASE_URL?sslmode=` パースが通る | ✅ |
| `fav.toml [postgres] sslmode` が PGSSLMODE に inject される | ✅ |
| `cargo test v12600` 6 件通過 | ✅ |
| `cargo test` 全通過 | ✅ 1392 件 |
