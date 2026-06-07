# Capability-Context 設計仕様

Date: 2026-06-07
Status: 設計提案（未実装）

---

## 背景と問題意識

### 現行のエフェクト型システムの課題

現在の Favnir は関数の副作用を型注釈で表現している。

```
fn migrate() -> Result<MigrationResult, String> !Io !Env !Postgres !Azure !AWS
```

これには複数の問題がある。

**1. 抽象レベルが混在している**

`!Io`（プロセス I/O）と `!Postgres`（特定サービス）と `!AWS`（クラウドプロバイダー）は、
まったく異なる抽象レベルの概念が同列に並んでいる。

**2. ローカル DB とクラウド DB を区別する根拠がない**

副作用の本質は「外部の可変状態を読む」「外部状態に書く」であり、
それがローカルの Postgres か AWS RDS か Azure PG かは実装の詳細に過ぎない。
型シグネチャがサービス名を知るべきではない。

**3. サービスを追加するたびに言語が変わる**

新しいクラウドサービスを追加するたびに `!Snowflake`、`!TiDB` のような
新しいエフェクト型を言語仕様に追加する必要がある。

**4. stdout の消失バグを防げない**

`IO.println` が implicit な `!Io` エフェクトに依存する設計では、
「ログを書くつもりだったが実際は何も出力されていなかった」という
バグをコンパイル時に検出できない。

**5. Rune がエフェクト型に依存している**

現行 Rune（`AWS.*`、`Postgres.*` 等）は暗黙の接続情報を
エフェクト型経由で受け取るため、テスト時のモックが困難。

---

## 設計方針

> **副作用は引数として渡す。型注釈は不要。**

これは型理論における **capability-based effects** として確立したアプローチである。
副作用を特殊構文で表すのではなく、必要なリソースを通常の引数として受け取ることで、
関数シグネチャが自己説明的になり、テストが自然になる。

---

## Capability 型

副作用の性質を表す基本型を定義する。サービス名は含まない。

```
type Db(
  read:  DbRead,   // query / scan 等の読み取り操作
  write: DbWrite   // put / upsert / delete 等の書き込み操作
)

type Storage(
  read:  StorageRead,
  write: StorageWrite
)

type Http(...)      // 外部 HTTP / gRPC
type Io(...)        // プロセス I/O（stdout / stderr）
type Env(...)       // 環境変数・設定
```

### 純粋関数の識別

capability 引数を持たない関数は純粋関数である。引数を見れば即座に分かる。

```
fn validate(d: Loaded) -> Result<Validated, String>
// 引数に capability なし → 純粋、副作用なし
```

---

## AppCtx — capability の束

複数の capability を一つの型にまとめる。

```
type AppCtx(
  db:      Option<Db>,
  storage: Option<Storage>,
  http:    Option<Http>,
  io:      Io,
  env:     Env
)
```

連想配列ではなく型付き struct にする理由：

- `db` キーが存在するかどうかが実行時でなくコンパイル時に分かる
- `Ctx.require_db` が `Option<Db>` を検査して `Result` を返せる
- capability の欠如をパイプライン起動前に検出できる

---

## Ctx Rune — コンテキスト専用ライブラリ

既存の `AWS`、`Postgres` などのドメイン Rune とは**別軸**で、
コンテキストの組み立て・検証・配布だけを担当する専用 Rune。

### 役割

```
ドメイン Rune（AWS / Postgres / S3 等）
  → 特定サービスへの操作を提供する
  → capability を受け取るだけ、自分では解釈しない

Ctx Rune
  → capability を組み立てる
  → 揃っているか検証する
  → 他の Rune に配布する
  → テスト時の差し替え口を提供する
```

### インターフェース

```
rune Ctx {
  // 組み立て
  fn build(env: Env) -> Result<AppCtx, String>
  // 設定ファイル・環境変数から capability を解決し、欠けていれば即エラー

  fn with_db(ctx: AppCtx, db: Db)           -> AppCtx
  fn with_storage(ctx: AppCtx, s: Storage)  -> AppCtx
  fn with_http(ctx: AppCtx, h: Http)        -> AppCtx

  // 取り出し（なければ Result.err）
  fn require_db(ctx: AppCtx)      -> Result<Db, String>
  fn require_storage(ctx: AppCtx) -> Result<Storage, String>
  fn require_http(ctx: AppCtx)    -> Result<Http, String>

  // テスト用
  fn mock(db: Db, storage: Storage, io: Io) -> AppCtx
}
```

### capability の欠如はパイプライン起動前に検出される

```
// パイプライン開始時
bind ctx <- Ctx.build(env);
// ← ここで DB 接続設定がなければ即エラー
// ← 深い呼び出しで発覚しない

seq Load(ctx) |> Validate |> Transform |> Write(ctx)
```

---

## 関数シグネチャの変化

### 基本形

```
// 旧
fn list_runes() -> Result<List<Row>, String> !AWS
fn migrate()    -> Result<MigrationResult, String> !Io !Postgres !Azure

// 新
fn list_runes(ctx: AppCtx) -> Result<List<Row>, String>
fn migrate(ctx: AppCtx)    -> Result<MigrationResult, String>
```

型の後ろに何も付かない。`!` 記法は完全に不要になる。

### Ctx 部分渡し構文（シュガー）

関数が必要とする capability をシグネチャ内で destructuring できる。

```
fn Load(Ctx { db.read }, page: Int)               -> Result<Loaded, String>
fn Write(Ctx { db.write }, d: Transformed)         -> Result<Unit, String>
fn Migrate(Ctx { db.read, db.write, io }, args)    -> Result<MigrationResult, String>
fn Validate(d: Loaded)                             -> Result<Validated, String>
// Ctx なし → 純粋関数
```

`Ctx { db.read }` を受け取った関数は `db.write` を物理的に呼べない。
型システムが許可しないため、アクセス制御が型で完結する。

### これが `!read` / `!write` 問題を解決する

以前「`!r` / `!w` をどう書くか」で悩んでいた read/write の区別が、
capability の部分渡しとして自然に表現される。

| 旧（エフェクト注釈） | 新（Ctx 部分渡し） |
|---|---|
| `fn f() -> T !read` | `fn f(Ctx { db.read }) -> T` |
| `fn f() -> T !write` | `fn f(Ctx { db.write }) -> T` |
| `fn f() -> T !read !write` | `fn f(Ctx { db.read, db.write }) -> T` |
| `fn f() -> T` （純粋） | `fn f() -> T` （Ctx なし） |

特殊記法なし。構造的な型だけで表現される。

---

## Rune 設計の変化

ドメイン Rune は `ctx` を受け取り、`Ctx.require_*` で必要な capability を取り出す。
自分でコンテキストを解釈しない。

```
rune Dynamo {
  fn scan(ctx: AppCtx, table: String) -> Result<List<Row>, String>
    bind db <- Ctx.require_db(ctx);
    db.dynamo_scan(table)

  fn put(ctx: AppCtx, table: String, item: Row) -> Result<Unit, String>
    bind db <- Ctx.require_db(ctx);
    db.dynamo_put(table, item)
}

rune Postgres {
  fn query(ctx: AppCtx, sql: String) -> Result<List<Row>, String>
    bind db <- Ctx.require_db(ctx);
    db.postgres_query(sql)
}
```

`Db` 型の中身（DynamoDB か Postgres か）はランタイムの実装詳細であり、
型シグネチャには現れない。

---

## パイプライン設計

capability は entry point で一度だけ組み立て、pipeline 全体に渡す。

```
public fn main(ctx: AppCtx) -> Unit
  seq Load(ctx) |> Validate |> Transform |> Write(ctx)

fn load(ctx: AppCtx)                -> Result<Loaded, String>
  Dynamo(ctx).scan("customers")

fn validate(d: Loaded)              -> Result<Validated, String>
  // pure — ctx 不要

fn transform(d: Validated)          -> Result<Transformed, String>
  // pure — ctx 不要

fn write(ctx: AppCtx, d: Transformed) -> Result<Unit, String>
  Postgres(ctx).upsert("customers_migrated", d)
```

純粋なステージ（`validate`、`transform`）に capability が混入しない。
パイプラインのどこに副作用があるかが構造から分かる。

---

## テスト設計

```
// 本番
bind ctx <- Ctx.build(env);
migrate(ctx)

// テスト — ctx を差し替えるだけ
bind ctx <- Ctx.mock(
  db:      InMemoryDb.seed([row1, row2]),
  storage: InMemoryStorage.empty(),
  io:      Io.capture()
);
migrate(ctx)

// IO のキャプチャ確認
bind output <- Io.captured(ctx.io);
assert(output == "Processing 2 rows\n")
```

- モックフレームワーク不要
- `Io.capture()` で stdout を取得できるためログ消失バグも検出可能
- 各ステージは `ctx` を渡して独立してテスト可能

---

## 設計の全体像

すべての関心が通常の型システムで表現される。

| 関心 | 表現方法 |
|---|---|
| 副作用の有無 | `ctx` 引数があるか |
| read か write か | `Ctx { db.read }` か `Ctx { db.write }` か |
| どのサービスか | lineage（`fav explain --lineage`） |
| フェーズの順序保証 | 型状態パターン（`Loaded` → `Validated` → `Transformed`） |
| テスト差し替え | `Ctx.mock(...)` |

特殊構文ゼロ。エフェクト型ゼロ。

---

## lineage との関係

`fav explain --lineage` は「どの具体的なサービスか」を記録する責務を持つ。
エフェクト型がその情報を持つ必要はない。

| 責務 | 担当 |
|---|---|
| 副作用の有無 | 関数引数（capability の有無） |
| read か write か | Rune のメソッド名（scan / put 等）から推論 |
| どの具体的なサービスか | lineage（`fav explain --lineage`） |

関心の分離として綺麗に成立する。

---

## 型状態パターンとの組み合わせ

capability-context と型状態パターンを組み合わせると、
フェーズの順序保証と副作用の明示が両立する。

```
type Loaded(List<CustomerRow>)
type Validated(List<CustomerRow>)
type Transformed(List<MigratedRow>)

fn load(ctx: AppCtx)               -> Result<Loaded, String>
fn validate(d: Loaded)             -> Result<Validated, String>   // pure
fn transform(d: Validated)         -> Result<Transformed, String> // pure
fn write(ctx: AppCtx, d: Transformed) -> Result<Unit, String>
```

- フェーズを飛ばした呼び出しはコンパイルエラー
- 純粋フェーズ（validate / transform）には ctx が不要であることが型から明らか
- テストは各フェーズの型を手で作って渡すだけ

---

## 現行設計との比較

| 観点 | 現行（エフェクト型） | 新設計（capability-context） |
|---|---|---|
| 副作用の表現 | `!Postgres !AWS !Io` | 関数引数 `ctx: AppCtx` |
| 純粋関数の識別 | エフェクト注釈なし | capability 引数なし |
| サービス追加時 | 新エフェクト型が必要 | `ctx` にフィールド追加のみ |
| ローカル/クラウド区別 | 別エフェクト型 | 同じ `Db` capability |
| stdout 消失バグ | 検出不可 | `Io` 引数なしでコンパイルエラー |
| テスト | モック困難 | `Ctx.mock(...)` で差し替え |
| Rune 設計 | エフェクト型依存 | `ctx` を受け取るだけ |
| 言語学習コスト | `!` 記法 + エフェクト一覧 | 普通の型システムの知識のみ |

---

## 設計決定事項（2026-06-07 確定）

### 1. `Db` 型の内部表現 → `interface` ベース

Favnir の `interface`/`impl`（v9.12.0〜）を使う。タグ付きユニオンは使わない。

```
interface Db {
  read:  DbRead
  write: DbWrite
}

impl PostgresDb for Db { ... }
impl DynamoDb   for Db { ... }
impl MockDb     for Db { ... }
```

サービスを追加しても `Db` interface は変わらない。テスト時は `MockDb` を差し込む。

### 2. `AppCtx` のユーザー拡張 → 型による合成

```
type MyCtx(
  base:    AppCtx,
  tenant:  String,
  feature: Map<String, Bool>
)
```

`Ctx` Rune は `AppCtx` を提供し、ユーザーは包んで拡張する。特殊な拡張機構は不要。

### 3. `seq` での ctx 受け渡し → interface ベース（`seq(ctx)` 構文は採用しない）

`seq(ctx)` は関数呼び出しに見える上、全ステージが同一コンテキストを持つ前提になるため採用しない。
代わりに、各ステージが必要な interface を個別に宣言する。

**CommonCtx を基底として interface を継承する：**

```
interface CommonCtx {
  io:  Io
  env: Env
}

interface LoadCtx: CommonCtx {
  db: DbRead
}

interface WriteCtx: CommonCtx {
  db:      DbWrite
  storage: StorageWrite
}
```

**AppCtx はすべての interface を実装する：**

```
impl AppCtx for LoadCtx  { ... }
impl AppCtx for WriteCtx { ... }
```

**各ステージは必要な interface だけを宣言する：**

```
fn Load(ctx: LoadCtx, page: Int)        -> Result<Loaded, String>
fn Validate(d: Loaded)                  -> Result<Validated, String>   // pure
fn Write(ctx: WriteCtx, d: Transformed) -> Result<Unit, String>

seq Load |> Validate |> Write   // seq 構文はそのまま — 特別な記法不要
```

- `Load` は `db.write` や `storage` を物理的に見えない
- `Write` は `db.read` を物理的に見えない
- ステージごとに異なるコンテキストを要求できる
- `seq` はシンプルなパイプライン演算子のまま

### 4. 既存 Rune の移行パス → 段階的移行

```
v13.1〜v13.x  新 Rune（ctx ベース）を旧 Rune と並行提供
              旧 Rune（!AWS / !Postgres 等）は deprecated 警告
v14.0         旧 Rune 廃止・エフェクト型完全削除・`!` 記法を言語仕様から削除
```

---

## 実装スケジュール

```
v12.5 〜 v13.0   現ロードマップ完走（言語信頼性宣言）
v13.1            Ctx Rune インターフェース確定・interface Db 定義
v13.2〜v13.x     段階的実装
                  — Ctx Rune 実装
                  — CommonCtx / interface 継承の実装
                  — 既存 Rune の ctx ベース版を並行提供
                  — `Ctx { db.read }` 部分渡し構文
v14.0            capability-context 完成宣言
                  — エフェクト型（!AWS / !Postgres 等）完全廃止
```

---

*仕様 Close: 2026-06-07*
