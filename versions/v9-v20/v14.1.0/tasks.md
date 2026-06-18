# v14.1.0 Tasks — Azure PostgreSQL Rune

Date: 2026-06-11

---

## Phase A — Cargo.toml 依存追加

- [x] A-1: 依存追加不要 — `tokio-postgres-rustls` が既存（openssl 不要）。`azure_pg_execute` は既存 rustls TLS を流用
  ```toml
  openssl = { version = "0.10", features = ["v102", "v110"] }
  tokio-postgres-openssl = "0.5"
  ```
- [x] A-2: ビルド確認済み（依存追加なしでコンパイル成功）

---

## Phase B — VM プリミティブ追加

- [x] B-1: `fav/src/backend/vm.rs` に `"AzurePostgres.execute_raw"` / `"AzurePostgres.query_raw"` ハンドラを追加
- [x] B-2: `azure_pg_execute` ヘルパー追加（既存 `pg_connect_inner` / `pg_params_from_json` を流用、i64 を返す）
- [x] B-3: `cargo build` コンパイルエラーなし確認

---

## Phase C — checker.rs: `AzurePostgres` namespace 追加

- [x] C-1: `builtin_ret_ty` に `"AzurePostgres"` ブロックを追加（execute_raw → Result<Int,String>、query_raw → Result<String,String>）
- [x] C-2: `BUILTIN_EFFECTS` に `"AzureDb"` を追加（E0252 なし）
- [x] C-3: `require_azure_db_effect` 関数を追加（E0316）
- [x] C-4: NS env def に `"AzurePostgres"` を追加

---

## Phase D — lineage.rs + ast.rs + parser.rs + 関連ファイル: `AzureDb` エフェクト追加

- [x] D-1: `ast.rs` `Effect` enum に `AzureDb` 追加
- [x] D-2: `parser.rs` `parse_effect_ann` に `"AzureDb"` ケース追加
- [x] D-3: `lineage.rs` に `collect_azure_call_kinds` + `azure_db_effects` + `combined_effects` 追加
- [x] D-4: `lineage_analysis` の TrfDef/FnDef 両方に AzureDb 分類を追加
- [x] D-5: `format_effects` (`lineage.rs`, `driver.rs`, `fmt.rs`, `lint.rs`, `ast_lower_checker.rs`, `reachability.rs`) に `AzureDb` 追加

---

## Phase E — `runes/azure-postgres/` 新規作成

- [x] E-1: `runes/azure-postgres/client.fav` を作成（AzureDbCtx 型 + execute / query）
- [x] E-2: `runes/azure-postgres/azure_postgres.fav` を作成（re-export）

---

## Phase F — テスト追加

- [x] F-1: `fav/src/driver.rs` に `v141000_tests` モジュールを追加
  - [x] `version_is_14_1_0` — CARGO_PKG_VERSION == "14.1.0" 確認
  - [x] `azure_postgres_primitives_registered` — E0007 なし確認
  - [x] `azure_db_effect_in_checker` — E0252 なし確認
  - [x] `azure_db_lineage_tracked` — !AzureDb(write) が lineage に収集される確認
- [x] F-2: `cargo test v141000` 全件パス（4/4）

---

## Phase G — バージョンバンプ + 全テスト + コミット

- [x] G-1: `fav/Cargo.toml` → `version = "14.1.0"`
- [x] G-2: `cargo test v141000` 全件パス（4/4）
- [x] G-3: `cargo test` 全件パス（1515/705/8/47/8 — リグレッションなし）
- [x] G-4: `git commit -m "feat: v14.1.0 — Azure PostgreSQL Rune"`

---

## 完了条件

| 確認項目 | 状態 |
|---|---|
| `AzurePostgres.execute_raw` / `query_raw` が VM に登録されている | ✅ |
| `!AzureDb` 宣言が E0252 を出さない | ✅ |
| `fav explain --lineage` で `!AzureDb(read/write)` が表示される | ✅ |
| `runes/azure-postgres/client.fav` が `fav check` でエラーなし | ✅ |
| `cargo test v141000` 全件パス（4/4） | ✅ |
| `cargo test` 全件パス（リグレッションなし） | ✅ |
| `CARGO_PKG_VERSION == "14.1.0"` | ✅ |

---

## 実装ノート

- **Phase A → B の順守**: `openssl` の依存がないと B がコンパイルできない。
- **TLS 検証スキップ（`SslVerifyMode::NONE`）**: 開発・デモ用。本番接続は CA 証明書検証を追加すること。
- **テストヘルパー**: `check_source_raw` / `collect_lineage_raw` は既存の v141000_tests 用ヘルパーを参照。実際の関数名は `fav/src/driver.rs` 内の既存テストを確認して合わせる。
- **`parse_params_json` の `serde_json::Value` → `ToSql`**: `tokio-postgres` の feature `with-serde_json-1` が必要。`Cargo.toml` の `tokio-postgres` 行で有効になっているか確認。
