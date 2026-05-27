# Favnir v7.0.0 Spec — Schema Authority

作成日: 2026-05-27

## テーマ

「外部データを型で守る」Favnir のコアユースケースを完成させる。

## 設計哲学

```
外部データ（CSV / DB / S3 / API）
    ↓  fav infer    → Favnir 型定義を自動生成
    ↓  schemas/*.yaml → 制約を付与
    ↓  fav check    → コンパイル時検査
    ↓  T.validate   → ランタイム検査
型安全なデータパイプライン
```

v6.6.0 で `T.validate` は完成した。
v7.0.0 では「どこで DB が使われ、何が読まれ何が書かれるか」を型で表現・追跡する
**エフェクト細分化**を完成させる。

---

## 現状分析

| 項目 | 現状 | v7.0.0 目標 |
|------|------|------------|
| DB エフェクト | `!Db`（単一） | `!DbRead` / `!DbWrite` / `!DbAdmin`（3段階） |
| `ast.rs` Effect enum | `Db` のみ | `DbRead` / `DbWrite` / `DbAdmin` 追加 |
| `parser.rs` | `"Db"` → `Effect::Db`（それ以外は Unknown） | 3 つを正式に parse |
| `checker.rs` BUILTIN_EFFECTS | `"Db"` のみ | 3 つを追加 |
| `require_db_effect` | `Effect::Db` のみ受け入れ | `Db \| DbRead \| DbWrite \| DbAdmin` 後方互換 |
| `runes/db/query.fav` | `!Db` | `!DbRead` |
| `runes/db/query.fav`（execute 系） | `!Db` | `!DbWrite` |
| `runes/db/transaction.fav` | `!Db` | `!DbWrite` |
| `runes/db/migration.fav` | `!Db` | `!DbAdmin` |
| Schema Authority ガイド | なし | 新規作成 |

---

## スコープ

### Phase A — エフェクト型追加（ast.rs + parser.rs + checker.rs）

`!DbRead` / `!DbWrite` / `!DbAdmin` をコンパイラが正式に認識・追跡できるようにする。

**後方互換性ルール:**
- `!Db` を宣言した関数は、`!DbRead` / `!DbWrite` / `!DbAdmin` のどれを呼び出しても許可される
- `!DbRead` のみ宣言した関数は、`!DbWrite`/`!DbAdmin` 関数の呼び出しを禁止
- 既存の `!Db` を使ったコードは変更なしに動く

### Phase B — `runes/db/` エフェクト更新

```
query.fav   : query / query_one / query_params → !DbRead
query.fav   : execute / execute_params / batch_insert → !DbWrite
transaction.fav : with_transaction / savepoint 系 → !DbWrite
migration.fav   : applied_migrations → !DbRead、mark_applied → !DbAdmin
connection.fav  : connect / close → !Db（変更なし）
```

### Phase C — Schema Authority ガイドドキュメント

`site/content/docs/guides/schema-authority.mdx` を新規作成。
CSV → `fav infer` → 型定義 → `schemas/*.yaml` → `T.validate` → 型安全な読み込み
の一貫したワークフローを示す「データパイプラインを型で守る」ガイド。

---

## スコープ外（v7.x 以降）

| 項目 | 理由 |
|------|------|
| `T.validate` の Favnir 実装（Rust から compiler.fav への委譲） | セルフホストコンパイラへの統合が大規模 |
| `fav build --schema` DDL 差分検出 | DB 接続 + DDL 比較ロジックが複雑 |
| `fav explain --lineage` データリネージ | v7.1.0 テーマ |

---

## 完了条件

- `ast.rs` に `Effect::DbRead` / `Effect::DbWrite` / `Effect::DbAdmin` が存在する
- `parser.rs` で `!DbRead` / `!DbWrite` / `!DbAdmin` が正しくパースされる
- `checker.rs` の BUILTIN_EFFECTS に 3 つが追加されている
- `require_db_effect` が `Db | DbRead | DbWrite | DbAdmin` すべてを受け入れる（後方互換）
- `runes/db/` の各関数が適切な細分化エフェクトを宣言している
- Schema Authority ガイドドキュメントが公開されている
- 既存テストが全件通る（1043 件）
