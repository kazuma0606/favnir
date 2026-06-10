# v13.10.0 Spec — `!` 記法廃止 + 糖衣構文追加

Date: 2026-06-11

---

## 概要

v13.8.0 で ambient effect が禁止（E0023）され、v13.6.0 以降では
E2E デモも ctx ベースで動作している。
この版で旧 `!Effect` 記法を言語仕様から正式に削除し、
`capability-context.md` の「後回し」糖衣構文を追加する。

---

## 1. `!` 記法廃止

### 1-1. 非 legacy モード

`!Effect` トークンを含む関数宣言はコンパイルエラー（E0025）となる:

```
E0025: `!` effect notation is no longer supported
  --> pipeline.fav:3:40
   |
 3 | fn load() -> Result<Loaded, String> !Postgres
   |                                     ^^^^^^^^^
   |
   = help: migrate to `fn load(ctx: LoadCtx) -> Result<Loaded, String>`
   = note: run `fav migrate --from-effects pipeline.fav` to auto-migrate
   = note: use `--legacy` flag to suppress this error during migration
```

### 1-2. legacy モード

`--legacy` フラグ付きでは E0025 を発生させず、旧 `!` 記法を従来通り解析・実行する。
移行期間中のフォールバックとして残存。

### 1-3. 対象トークン

以下の `!Effect` 記法がすべて E0025 対象:

| 旧記法 | 移行先 |
|---|---|
| `!Postgres` | `ctx: LoadCtx` / `ctx: WriteCtx` / `ctx: MigrateCtx` |
| `!AWS` | `ctx: WriteCtx`（S3 write）/ `ctx: LoadCtx`（S3 read） |
| `!Snowflake` | `ctx: LoadCtx` / `ctx: WriteCtx` |
| `!Io` | `ctx.io.println(...)` |
| `!Http` | `ctx.http.get(...)` |
| `!Llm` | `ctx.llm.chat(...)` |
| `!Grpc` | `ctx.rpc.call(...)` |
| `!Queue` | 将来の `ctx.queue.*` |
| `!Cache` | 将来の `ctx.cache.*` |

---

## 2. `Ctx { db: DbRead }` 糖衣構文

### 2-1. 構文

関数引数位置で `Ctx { field: CapType, ... }` と書くと、
対応する ctx interface 型に自動脱糖される:

```
// 糖衣構文
fn Load(Ctx { db: DbRead, io }, page: Int) -> Result<Loaded, String>

// 脱糖後（コンパイラ内部では下記に変換）
fn Load(ctx: LoadCtx, page: Int) -> Result<Loaded, String>
```

### 2-2. 脱糖ルール

| 糖衣フィールドセット | 脱糖後 ctx 型 |
|---|---|
| `{ db: DbRead }` | `LoadCtx` |
| `{ db: DbWrite }` または `{ db: DbWrite, storage: StorageWrite }` | `WriteCtx` |
| `{ db: DbWrite, db_migrate: DbWrite }` | `MigrateCtx` |
| その他（任意組み合わせ） | 構造的に満たせる最小 ctx 型、または `AppCtx` |

フィールドが `io` のみの場合は `CommonCtx` に脱糖。

### 2-3. 型チェック

脱糖後の型に対して通常の capability 充足チェック（E0021）が適用される。

### 2-4. `fav fmt` による正規化

`fav fmt` は糖衣構文を保持する（非展開）。
`fav fmt --expand-sugar` で脱糖後の形式に展開することも可能。

---

## 3. `fav fmt --migrate` 自動変換

### 3-1. 変換対象

旧 `!Effect` 記法を含むファイルを、ctx ベースシグネチャへ自動変換する:

```
$ fav fmt --migrate pipeline.fav
```

変換規則:
- `fn f() -> T !Postgres` → `fn f(ctx: LoadCtx) -> T`（読み取り操作が主体の場合）
- `fn f() -> T !Postgres !AWS` → `fn f(ctx: WriteCtx) -> T`（書き込み操作が主体の場合）
- 判断できない場合は `fn f(ctx: AppCtx) -> T` に変換し W010 で警告

### 3-2. W010: 手動確認が必要

```
W010: effect migration requires manual review
  --> pipeline.fav:8:1
   |
 8 | fn process() -> T !Postgres !AWS !Io
   |
   = note: auto-migrated to `fn process(ctx: AppCtx) -> T`
   = help: consider using a more specific context type (LoadCtx / WriteCtx)
```

---

## 4. `fav migrate --from-effects` ツール

`fav fmt --migrate` の単独コマンド版。複数ファイルをまとめて変換する:

```
$ fav migrate --from-effects src/
```

- `src/` 以下の `.fav` ファイルを再帰スキャン
- `!Effect` 記法を含むファイルを変換（バックアップは `.fav.bak` として保存）
- 変換サマリーを出力（変換件数・W010 件数）

---

## 5. エラーコード一覧（新規）

| コード | タイトル | 条件 |
|---|---|---|
| E0025 | bang notation removed | 非 legacy モードで `!Effect` が使用された |
| W010 | effect migration requires manual review | `fav migrate` で自動変換できない `!Effect` が検出された |

---

## 6. 後方互換性

- `--legacy` フラグで E0025 は完全に抑制される
- `fav migrate` 実行後も `--legacy` フラグなしで既存コードが動作する想定
- `fav check --legacy` / `fav run --legacy` は v14.0.0 まで有効

---

## 7. 影響範囲

| ファイル | 変更内容 |
|---|---|
| `fav/src/parser.rs` | `!Effect` トークン → E0025（非 legacy）または従来通り解析（legacy）|
| `fav/src/error_catalog.rs` | E0025 エントリ追加 |
| `fav/src/driver.rs` | `cmd_check` に E0025 チェックブロック追加、`cmd_fmt --migrate` 実装、`cmd_migrate` 追加 |
| `fav/src/lint.rs` | `check_bang_notation(program)` 実装 |
| `self/compiler.fav` | `Ctx { ... }` 糖衣構文パース処理追加 |
| `fav/Cargo.toml` | `version = "13.10.0"` |
