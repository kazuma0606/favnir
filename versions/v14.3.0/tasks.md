# v14.3.0 Tasks — Azure lineage + fav explain 出力改善

Date: 2026-06-12
Branch: master

---

## Phase A — `fav/src/ast.rs`: AzureStorage effect 追加

- [x] A-1: `ast::Effect` enum に `AzureStorage` variant を追加（`AzureDb` の直後）
  ```rust
  AzureDb,
  AzureStorage,  // Azure Blob Storage (v14.3.0 infra, v14.5.0 primitives)
  ```

- [x] A-2: `cargo build` でコンパイルエラーなし確認
  （match 網羅性エラーが出た箇所を `_ => {}` または明示的アームで修正）

---

## Phase B — `fav/src/lineage.rs`: AzureBlob 基盤 + CrossCloud 出力

- [x] B-1: `format_effects` に `AzureStorage => "!AzureStorage".into()` 追加

- [x] B-2: `collect_azure_blob_call_kinds` + ヘルパー関数群を追加
  - `is_azure_blob_read_method` — `"get_raw" | "list_raw"`
  - `is_azure_blob_write_method` — `"put_raw" | "delete_raw"`
  - `collect_azure_blob_call_kinds(expr)` → `(bool, bool)`
  - `collect_azure_blob_kinds_inner` / `collect_azure_blob_kinds_stmt`

- [x] B-3: `combined_effects` を 6 引数に拡張
  - `az_blob_read: bool`, `az_blob_write: bool` 追加
  - `!AzureStorage(read)` / `!AzureStorage(write)` / `!AzureStorage` の出力処理

- [x] B-4: `lineage_analysis` 内の 2 箇所（trf ループ + fn def ループ）を更新
  - `has_azure_blob` 判定を追加
  - `collect_azure_blob_call_kinds` 呼び出しを追加
  - `combined_effects` の呼び出しを 6 引数に変更

- [x] B-5: `render_lineage_text` に CrossCloud Flow セクションを追加
  - `has_aws_db`（`!Postgres` / `!Db` / `!Snowflake` 含む変換が存在）
  - `has_azure_db`（`!AzureDb` 含む変換が存在）
  - 両方 true のとき `CrossCloud Flow:` セクションを `Pipelines:` の直前に出力
  - フォーマット: `  [AWS RDS] → StageA → StageB → [Azure Postgres]`

- [x] B-6: `cargo build` でコンパイルエラーなし確認

---

## Phase C — `fav/src/middle/checker.rs`: AzureStorage エフェクト登録

- [x] C-1: `BUILTIN_EFFECTS` / エフェクト認識リストに `"!AzureStorage"` を追加

- [x] C-2: `str_to_effect` または `parse_effect` match に `"AzureStorage"` を追加
  → `ast::Effect::AzureStorage`

- [x] C-3: `cargo build` でコンパイルエラーなし確認

---

## Phase D — `fav/src/frontend/parser.rs` または `lexer.rs`: パース対応

- [x] D-1: `"!AzureStorage"` を効果として認識するパースを追加
  （`"!AzureDb"` と同じ箇所に追加）

- [x] D-2: `cargo build` でコンパイルエラーなし確認

---

## Phase E — `fav/src/driver.rs`: v143000_tests + バージョンバンプ

- [x] E-1: `v143000_tests` モジュールを追加（`v142000_tests` の直後推奨）
  - [x] `version_is_14_3_0` — `CARGO_PKG_VERSION == "14.3.0"` 確認
  - [x] `azure_db_lineage_collected` — `!AzureDb` エフェクトがリネージに収集されることを確認
  - [x] `crosscloud_lineage_format` — `!Postgres + !AzureDb` 共存時に CrossCloud Flow が出力されることを確認

  テスト本文は `plan.md` の Phase E を参照。

- [x] E-2: `fav/Cargo.toml` バージョンを `"14.3.0"` にバンプ

- [x] E-3: `cargo test v143000` で 3 件全パス確認

---

## Phase F — 全テスト + コミット

- [x] F-1: `cargo test v143000` 全 3 件パス
- [x] F-2: `cargo test` 全件パス（リグレッションなし）
- [x] F-3: `git commit -m "feat: v14.3.0 — Azure lineage + fav explain CrossCloud format"`

---

## 完了条件

| 確認項目 | 状態 |
|---|---|
| `ast::Effect::AzureStorage` が存在する | [x] |
| `collect_azure_blob_call_kinds` が lineage.rs に存在する | [x] |
| `render_lineage_text` が CrossCloud Flow を出力する | [x] |
| `cargo test v143000` 全 3 件パス | [x] |
| `cargo test` 全件パス（リグレッションなし） | [x] |
| `CARGO_PKG_VERSION == "14.3.0"` | [x] |

---

## 参照ファイル

| ファイル | 目的 |
|---|---|
| `versions/v14.3.0/spec.md` | 仕様・ユーザー体験 |
| `versions/v14.3.0/plan.md` | 実装詳細・コードスニペット |
| `versions/v14.2.0/tasks.md` | 先行バージョンのパターン参照 |
| `versions/roadmap-v14.1-v15.0.md` | v14.3.0 の位置づけ・依存関係 |
