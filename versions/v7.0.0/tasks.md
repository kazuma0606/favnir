# Favnir v7.0.0 Tasks

Date: 2026-05-27

## Goal

Schema Authority 完成。
`!DbRead` / `!DbWrite` / `!DbAdmin` エフェクト細分化 + Schema Authority ガイドドキュメント。

---

## Phase A — エフェクト型追加（Rust コンパイラ）

- [x] A-1: `fav/src/ast.rs` — `Effect` enum に `DbRead` / `DbWrite` / `DbAdmin` を追加
- [x] A-2: `fav/src/frontend/parser.rs` — `parse_effect_ann` に `"DbRead"` / `"DbWrite"` / `"DbAdmin"` のアームを追加（`"Db"` より前に配置）
- [x] A-3: `fav/src/middle/checker.rs` — `BUILTIN_EFFECTS` に `"DbRead"` / `"DbWrite"` / `"DbAdmin"` を追加
- [x] A-4: `fav/src/middle/checker.rs` — `require_db_effect` を更新（`Db | DbRead | DbWrite | DbAdmin` の4つを受け入れる後方互換ロジック）
- [x] A-5: `fav/src/middle/checker.rs` — `require_db_write_effect` を追加（`Db | DbWrite | DbAdmin` を受け入れる）
- [x] A-6: `fav/src/middle/checker.rs` — `require_db_admin_effect` を追加（`Db | DbAdmin` を受け入れる）

## Phase B — `runes/db/` エフェクト更新

- [x] B-1: `runes/db/query.fav` — `query` / `query_params` / `query_one` / `paginate` の `!Db` → `!DbRead`
- [x] B-2: `runes/db/query.fav` — `execute` / `execute_params` / `batch_insert` の `!Db` → `!DbWrite`
- [x] B-3: `runes/db/transaction.fav` — `with_transaction` / savepoint 系 4 関数の `!Db` → `!DbWrite`
- [x] B-4: `runes/db/migration.fav` — `ensure_migrations_table`（private）→ `!DbAdmin`、`applied_migrations` → `!DbRead`、`mark_applied` → `!DbAdmin`
- [x] B-5: `cargo test` 通過確認（変更前後で 1043 件）

## Phase C — Schema Authority ガイドドキュメント

- [x] C-1: `site/content/docs/guides/` ディレクトリを作成
- [x] C-2: `site/content/docs/guides/schema-authority.mdx` を新規作成（全体ワークフロー図・5ステップ・パイプライン完全例）
- [x] C-3: `site/content/docs/runes/db.mdx` にエフェクト細分化テーブルを追記

## Phase D — テスト・最終確認

- [x] D-1: `parser.rs` のテスト — `!DbRead` / `!DbWrite` / `!DbAdmin` が正しくパースされることを確認
- [x] D-2: `checker.rs` のテスト — `!DbRead` のみ宣言の fn が `!DbWrite` 要求関数呼び出しでエラーになることを確認
- [x] D-3: `cargo test` 全件通過（1043 件以上）
- [x] D-4: このファイルを完了状態に更新

---

## 完了条件まとめ

- `!DbRead` / `!DbWrite` / `!DbAdmin` が型チェッカーで正式に追跡される ✓
- `runes/db/` の各関数が適切な細分化エフェクトを宣言している ✓
- 後方互換: 既存の `!Db` コードが変更なしに動く ✓
- Schema Authority ガイドドキュメントが公開されている ✓
- 既存テスト 1043 件が全件通る ✓
