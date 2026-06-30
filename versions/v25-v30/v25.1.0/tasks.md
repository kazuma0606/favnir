# v25.1.0 タスクリスト — postgres Rune 実質化

**状態**: COMPLETE
**開始日**: 2026-06-24
**完了日**: 2026-06-24

---

## タスク

| ID | タスク | 状態 |
|---|---|---|
| T0 | ブランチ作成（`feat/v25.1-postgres-rune`） | [x] |
| T0.5 | `fav/Cargo.toml` を `version = "25.1.0"` に bump | [x] |
| T1 | `runes/postgres/db_conn.fav` 新規作成（`DbConn` interface） | [x] |
| T2 | `runes/postgres/types.fav` 新規作成（`PgConfig` / `PgConn` / `PoolConfig`） | [x] |
| T3 | `runes/postgres/client.fav` 更新（`connect` / `execute_many` / `transaction` / `Pool.create` / `Pool.get` / `Pool.release` 追加） | [x] |
| T4 | `fav/src/backend/vm.rs` 更新（VM primitive 5 件追加） | [x] |
| T5 | `runes/postgres/postgres.fav` 更新（re-export 追加） | [x] |
| T6 | `examples/postgres_etl.fav` 新規作成（`type User` 定義を含む E2E デモ） | [x] |
| T7 | `site/content/docs/runes/postgres.mdx` 更新（新規 API セクション追記） | [x] |
| T8 | `CHANGELOG.md` 更新（`[v25.1.0]` エントリ追加） | [x] |
| T9 | `benchmarks/v25.1.0.json` 新規作成 | [x] |
| T10 | `fav/src/driver.rs` 更新（`v251000_tests` 6 件追加） | [x] |
| T11 | `cargo test v251000` — 6 件 PASS 確認 | [x] |
| T12 | `cargo test` 総テスト数 ≥ 1980 件 確認 | [x] |
| T13 | spec-reviewer レビュー実施 | [x] |

---

## チェックリスト（完了条件）

- [x] `Postgres.connect` が `runes/postgres/client.fav` に存在する
- [x] `Postgres.execute_many` が `runes/postgres/client.fav` に存在する
- [x] `Postgres.transaction` が `runes/postgres/client.fav` に存在する
- [x] `Postgres.Pool.create` が `runes/postgres/client.fav` に存在する
- [x] `Postgres.Pool.get` が `runes/postgres/client.fav` に存在する
- [x] `Postgres.Pool.release` が `runes/postgres/client.fav` に存在する
- [x] `DbConn` interface が `runes/postgres/db_conn.fav` に存在する
- [x] `PgConfig` / `PgConn` / `PoolConfig` が `runes/postgres/types.fav` に存在する
- [x] `examples/postgres_etl.fav` が存在し `type User` を含む
- [x] `CHANGELOG.md` に `v25.1.0` が存在する
- [x] `site/content/docs/runes/postgres.mdx` に新規 API セクションがある
- [x] `v251000_tests` 6 件すべて PASS
- [x] 総テスト数 = 1980 件（目標達成）

---

## コードレビュー指摘（spec-reviewer 対応済み）

| 優先度 | 指摘内容 | 対応 |
|---|---|---|
| HIGH | E0314 → E0315 | spec.md / plan.md で E0315 に修正 |
| HIGH | VM primitive 命名不一致（pool_create_raw） | `pool_create_with_config_raw` として新規追加、既存 `Pool.create` と競合回避 |
| HIGH | SSL 矛盾（roadmap vs spec） | PgConfig に ssl フィールド概念を追加、「カスタム証明書のみスコープ外」に限定 |
| HIGH | テスト件数乖離（5件 vs 6件） | spec.md に「≥ ロードマップ最小 5 件」注釈追加 |
| MED | Pool.get / Pool.release 欠落 | client.fav に追加、tasks チェックリストに追記 |
| MED | User 型未定義 | examples/postgres_etl.fav に `type User` 追加 |
| MED | transaction[T] 型変数欠落 | db_conn.fav / client.fav で `<T>` 表記に統一 |
| LOW | postgres.mdx 更新欠落 | T7 実施済み |
| LOW | Cargo.toml bump 欠落 | T0.5 実施済み（25.0.0 → 25.1.0） |

---

## メモ

- `Pool.create` の既存 primitive（`Postgres.Pool.create`）との競合を避けるため、`Postgres.pool_create_with_config_raw` として新規 primitive を追加した
- `PgConn` / `PgConfig` は `type Foo(String)` 形式の名目型ラッパーとして実装（Favnir の既存パターンに準拠）
- `transaction_raw` は現バージョンで BEGIN/COMMIT のみ実装。クロージャの VM 内実行は将来バージョン（v25.x）で対応予定
- 既存の環境変数ベース `execute` / `query<T>` は後方互換として維持
