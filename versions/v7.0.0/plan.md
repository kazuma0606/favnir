# Favnir v7.0.0 Plan — Schema Authority

作成日: 2026-05-27

## 変更ファイル一覧

| ファイル | 変更種別 | 内容 |
|---------|---------|------|
| `fav/src/ast.rs` | 変更 | `Effect` enum に `DbRead` / `DbWrite` / `DbAdmin` を追加 |
| `fav/src/frontend/parser.rs` | 変更 | `parse_effect_ann` に 3 つのアームを追加 |
| `fav/src/middle/checker.rs` | 変更 | BUILTIN_EFFECTS 更新、`require_db_effect` 後方互換化、`require_db_read_effect` 等を追加 |
| `runes/db/query.fav` | 変更 | `!Db` → `!DbRead` / `!DbWrite` に分離 |
| `runes/db/transaction.fav` | 変更 | `!Db` → `!DbWrite` |
| `runes/db/migration.fav` | 変更 | `!Db` → `!DbRead` / `!DbAdmin` に分離 |
| `site/content/docs/guides/schema-authority.mdx` | 新規 | Schema Authority ガイドドキュメント |
| `site/content/docs/runes/db.mdx` | 変更 | エフェクト細分化の説明を追記 |

---

## Phase A — エフェクト型追加

### A-1: `ast.rs` — Effect enum に 3 バリアントを追加

```rust
pub enum Effect {
    Pure,
    Io,
    Db,
    DbRead,    // 追加: SELECT 系
    DbWrite,   // 追加: INSERT/UPDATE/DELETE 系
    DbAdmin,   // 追加: DDL（CREATE/DROP/ALTER）系
    Network,
    // ...
}
```

### A-2: `parser.rs` — `parse_effect_ann` に 3 つのアームを追加

```rust
"DbRead"  => { self.advance(); Effect::DbRead }
"DbWrite" => { self.advance(); Effect::DbWrite }
"DbAdmin" => { self.advance(); Effect::DbAdmin }
```
`"Db"` のアームより前に配置（最長マッチ優先）。

### A-3: `checker.rs` — BUILTIN_EFFECTS を更新

```rust
const BUILTIN_EFFECTS: &[&str] = &[
    "Pure", "Io", "Db",
    "DbRead", "DbWrite", "DbAdmin",  // 追加
    "Network", "Rpc", "File", "Checkpoint", "Trace",
    "Emit", "Random", "Auth", "Env", "DuckDb", "AWS",
];
```

### A-4: `checker.rs` — `require_db_effect` を後方互換化

```rust
fn require_db_effect(&mut self, span: &Span) {
    if !self.has_effect(|e| matches!(e,
        Effect::Db | Effect::DbRead | Effect::DbWrite | Effect::DbAdmin))
    {
        self.type_error("E0107",
            "Db.* call requires `!Db`, `!DbRead`, `!DbWrite`, or `!DbAdmin` effect", span);
    }
}
```

### A-5: `checker.rs` — `require_db_write_effect` / `require_db_admin_effect` を追加

```rust
fn require_db_write_effect(&mut self, span: &Span) {
    if !self.has_effect(|e| matches!(e, Effect::Db | Effect::DbWrite | Effect::DbAdmin)) {
        self.type_error("E0108", "DB write call requires `!DbWrite`, `!DbAdmin`, or `!Db`", span);
    }
}

fn require_db_admin_effect(&mut self, span: &Span) {
    if !self.has_effect(|e| matches!(e, Effect::Db | Effect::DbAdmin)) {
        self.type_error("E0109", "DB admin call (DDL) requires `!DbAdmin` or `!Db`", span);
    }
}
```

---

## Phase B — `runes/db/` エフェクト更新

### B-1: `query.fav` — 読み取り系を `!DbRead` に

```favnir
public fn query(handle: DbHandle, sql: String) -> Result<...> !DbRead
public fn query_params(...) -> Result<...> !DbRead
public fn query_one(...) -> Result<...> !DbRead
public fn paginate(...) -> Result<...> !DbRead
```

### B-2: `query.fav` — 書き込み系を `!DbWrite` に

```favnir
public fn execute(handle: DbHandle, sql: String) -> Result<Int, DbError> !DbWrite
public fn execute_params(...) -> Result<Int, DbError> !DbWrite
public fn batch_insert(...) -> Result<Int, DbError> !DbWrite
```

### B-3: `transaction.fav` — `!DbWrite` に更新

```favnir
public fn with_transaction(handle: DbHandle, f: ...) -> Result<...> !DbWrite
public fn savepoint(handle: DbHandle, name: String) -> Result<Int, DbError> !DbWrite
public fn release_savepoint(...) -> Result<Int, DbError> !DbWrite
public fn rollback_to_savepoint(...) -> Result<Int, DbError> !DbWrite
```

### B-4: `migration.fav` — 役割別に分離

```favnir
// applied_migrations は SELECT のみ → !DbRead
public fn applied_migrations(handle: DbHandle) -> Result<...> !DbRead
// mark_applied は INSERT + CREATE TABLE IF NOT EXISTS → !DbAdmin
public fn mark_applied(handle: DbHandle, name: String) -> Result<Int, DbError> !DbAdmin
```

> `ensure_migrations_table`（private）は `!DbAdmin` に変更。

---

## Phase C — Schema Authority ガイドドキュメント

`site/content/docs/guides/schema-authority.mdx` を新規作成。

内容:
1. **全体ワークフロー図** — 外部データ → fav infer → schemas → fav check → T.validate
2. **ステップ 1**: `fav infer --csv` で型定義生成
3. **ステップ 2**: `schemas/*.yaml` で制約付与
4. **ステップ 3**: `T.validate` でランタイム検証
5. **ステップ 4**: エフェクト細分化（`!DbRead` / `!DbWrite`）で読み書きを型で分離
6. **パイプライン例**: stage/seq で全ステップを組み合わせた完全な例

---

## Phase D — テスト・最終確認

- `!DbRead` / `!DbWrite` / `!DbAdmin` パースのユニットテスト追加（`parser.rs` 既存テスト相当）
- `!DbRead` のみ宣言した fn が `!DbWrite` 呼び出しでエラーになるか確認
- 既存 1043 件テスト通過確認
- `db.mdx` にエフェクト細分化の説明を追記
