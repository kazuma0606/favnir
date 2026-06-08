# Capability-Context 設計仕様

Date: 2026-06-07
Updated: 2026-06-08（Codex レビュー反映）
Status: 設計提案（未実装）

---

## 一言で言うと

> effect system を捨てるのではなく、effect surface を通常の型と interface に戻す。

`!Postgres !AWS !Io` という特殊記法をやめて、同じ情報を普通の型システムで表現し直す。
effect tracking は残る。記法が変わる。

---

## 背景と問題意識

現在の Favnir は関数の副作用を型注釈で表現している。

```
fn migrate() -> Result<MigrationResult, String> !Io !Env !Postgres !Azure !AWS
```

**1. 抽象レベルが混在している**

`!Io`（プロセス I/O）と `!Postgres`（特定サービス）と `!AWS`（クラウドプロバイダー）は
まったく異なる抽象レベルの概念が同列に並んでいる。

**2. ローカル DB とクラウド DB を区別する根拠がない**

副作用の本質は「外部の可変状態を読む/書く」であり、
それがローカルの Postgres か AWS RDS か Azure PG かは実装の詳細に過ぎない。

**3. サービスを追加するたびに言語が変わる**

新しいクラウドサービスを追加するたびに `!Snowflake`、`!TiDB` のような
新しいエフェクト型を言語仕様に追加する必要がある。

**4. Rune がエフェクト型に依存している**

現行 Rune（`AWS.*`、`Postgres.*` 等）は暗黙の接続情報をエフェクト型経由で受け取るため、
テスト時のモックが困難。

---

## 設計方針

**副作用は引数として渡す。`!` 記法は不要。**

これは型理論における **capability-based effects** として確立したアプローチ。
副作用を特殊構文で表すのではなく、必要なリソースを通常の引数として受け取る。

### 純粋性の前提条件

「capability 引数がなければ純粋」が成立するには、
**言語全体で ambient effect を禁止する**必要がある。

- `IO.println` 等のビルトインを ctx なしで呼べる状態を残すと、
  「capability 引数なし = 純粋」は嘘になる
- Favnir の全 primitive を capability 必須にすることが前提
- これが本設計の最大の仕様変更点

---

## Capability 型

サービス名を含まない。操作の性質だけで分類する。

```
interface DbRead {
  fn query(sql: String, params: List<String>) -> Result<List<Row>, String>
  fn scan(table: String)                      -> Result<List<Row>, String>
}

interface DbWrite {
  fn execute(sql: String, params: List<String>) -> Result<Unit, String>
  fn upsert(table: String, item: Row)           -> Result<Unit, String>
}

interface StorageRead {
  fn get(bucket: String, key: String) -> Result<Bytes, String>
}

interface StorageWrite {
  fn put(bucket: String, key: String, data: Bytes) -> Result<Unit, String>
}

interface HttpClient {
  fn get(url: String)                   -> Result<String, String>
  fn post(url: String, body: String)    -> Result<String, String>
}

interface Io {
  fn println(msg: String) -> Unit
  fn capture()            -> IoCapture   // テスト用
}

interface Env {
  fn require(key: String) -> Result<String, String>
}
```

**read/write の区別は capability 型から出す。メソッド名からの推論は行わない。**
`upsert` / `sync` / `apply` のような複合操作でも意味がぶれない。

---

## interface ベースのコンテキスト設計（本命）

### CommonCtx — 全ステージが共有する基底

```
interface CommonCtx {
  io:  Io
  env: Env
}
```

### 用途別 interface — ステージが必要な capability だけを宣言

```
interface LoadCtx: CommonCtx {
  db: DbRead
}

interface WriteCtx: CommonCtx {
  db:      DbWrite
  storage: StorageWrite
}

interface MigrateCtx: CommonCtx {
  db:      DbRead
  db_out:  DbWrite
  storage: StorageWrite
}
```

### AppCtx — 実行時の実体（すべての interface を実装）

```
type AppCtx(
  db:      Db,        // DbRead + DbWrite を実装した concrete 型
  storage: Storage,   // StorageRead + StorageWrite を実装した concrete 型
  http:    HttpClient,
  io:      Io,
  env:     Env
)

impl AppCtx for LoadCtx   { ... }
impl AppCtx for WriteCtx  { ... }
impl AppCtx for MigrateCtx { ... }
```

**`AppCtx` は実行時の実体、`LoadCtx` / `WriteCtx` は静的な要求型。**
ステージの関数シグネチャに `AppCtx` を直接書かず interface を書くことで、
コンパイル時に capability の充足を検査できる。

---

## 関数シグネチャ

```
// 旧
fn load()      -> Result<Loaded, String>      !Postgres
fn validate()  -> Result<Validated, String>
fn write()     -> Result<Unit, String>         !Postgres !AWS

// 新
fn load(ctx: LoadCtx)                        -> Result<Loaded, String>
fn validate(d: Loaded)                       -> Result<Validated, String>
fn write(ctx: WriteCtx, d: Transformed)      -> Result<Unit, String>
```

- `load` は `DbRead` しか見えない。`DbWrite` は物理的にアクセスできない
- `validate` は capability 引数なし → 純粋（ambient effect 禁止が前提）
- `write` は `DbWrite` と `StorageWrite` しか見えない

型の後ろに何も付かない。`!` 記法は不要になる。

---

## パイプライン設計

`seq` 構文はそのまま。特別な記法（`seq(ctx)` 等）は不採用。
各ステージが自分の interface を宣言するため、pipeline レベルで ctx を宣言する必要がない。

```
seq Load |> Validate |> Transform |> Write
```

```
fn Load(ctx: LoadCtx, input: String)      -> Result<Loaded, String>
fn Validate(d: Loaded)                    -> Result<Validated, String>   // pure
fn Transform(d: Validated)                -> Result<Transformed, String> // pure
fn Write(ctx: WriteCtx, d: Transformed)   -> Result<Unit, String>
```

---

## Db の内部表現 — interface ベース

タグ付きユニオンではなく、`interface`/`impl`（v9.12.0〜）を使う。

```
interface Db {
  read:  DbRead
  write: DbWrite
}

impl PostgresDb for Db { ... }
impl DynamoDb   for Db { ... }
impl MockDb     for Db { ... }
```

サービスを追加しても `Db` interface は変わらない。
テスト時は `MockDb` を差し込む。

---

## AppCtx のユーザー拡張 — 型による合成

```
type MyCtx(
  base:    AppCtx,
  tenant:  String,
  feature: Map<String, Bool>
)

impl MyCtx for LoadCtx  { db: self.base.db.read  ... }
impl MyCtx for WriteCtx { db: self.base.db.write ... }
```

特殊な拡張機構は不要。型の合成で表現する。

---

## Ctx Rune — 組み立てと差し替え

コンテキストの組み立て・検証・テスト差し替えを担当する専用 Rune。

```
rune Ctx {
  fn build(env: Env)  -> Result<AppCtx, String>
  // 接続設定を検証し、欠けていれば起動時にエラー（実行時だが起動直後）

  fn mock(db: MockDb, storage: MockStorage, io: IoCapture) -> AppCtx
}
```

**注意**: `Ctx.build` の欠如検出は起動時の実行時チェック。
コンパイル時に「capability が揃っているか」を保証するのは
interface 型（`LoadCtx` 等）による静的チェックである。両者は役割が異なる。

---

## テスト設計

```
// 本番
bind ctx <- Ctx.build(env);
migrate(ctx)

// テスト
bind ctx <- Ctx.mock(
  db:      MockDb.seed([row1, row2]),
  storage: MockStorage.empty(),
  io:      Io.capture()
);
migrate(ctx)

// stdout キャプチャ確認
bind output <- ctx.io.captured();
assert(output == "Processing 2 rows\n")
```

---

## lineage との責務分離

| 責務 | 担当 |
|---|---|
| 副作用の有無 | capability 引数の有無（コンパイル時） |
| read か write か | `DbRead` / `DbWrite` capability 型（コンパイル時） |
| どの具体的なサービスか | lineage（`fav explain --lineage`、静的解析） |

メソッド名（`scan`, `upsert` 等）からの read/write 推論は行わない。
capability 型が明示しているため推論は不要。

---

## 型状態パターンとの組み合わせ

```
type Loaded(List<CustomerRow>)
type Validated(List<CustomerRow>)
type Transformed(List<MigratedRow>)

fn load(ctx: LoadCtx)                   -> Result<Loaded, String>
fn validate(d: Loaded)                  -> Result<Validated, String>   // pure
fn transform(d: Validated)              -> Result<Transformed, String> // pure
fn write(ctx: WriteCtx, d: Transformed) -> Result<Unit, String>
```

フェーズを飛ばした呼び出しはコンパイルエラー。
純粋フェーズに capability が混入しないことが型から明らか。

---

## 設計の全体像

| 関心 | 表現方法 | チェックタイミング |
|---|---|---|
| 副作用の有無 | capability 引数があるか | コンパイル時 |
| read か write か | `DbRead` / `DbWrite` capability 型 | コンパイル時 |
| capability が揃っているか | interface の静的充足（`impl AppCtx for LoadCtx`） | コンパイル時 |
| 接続設定の欠如 | `Ctx.build` の Result | 起動時（実行時） |
| どのサービスか | lineage | 静的解析 |
| フェーズの順序 | 型状態パターン | コンパイル時 |
| テスト差し替え | `Ctx.mock(...)` | — |

---

## 糖衣構文（後回し）

`Ctx { db.read }` のような部分渡し構文は、interface ベース設計が安定してから追加する。

```
// 糖衣構文（v14.x 以降）
fn Load(Ctx { db: DbRead, io }, page: Int) -> Result<Loaded, String>

// 脱糖後
fn Load(ctx: LoadCtx, page: Int) -> Result<Loaded, String>
```

本命は `LoadCtx` / `WriteCtx` による interface 設計。糖衣構文はその上に乗る。

---

## 現行設計との比較

| 観点 | 現行（エフェクト型） | 新設計（capability-context） |
|---|---|---|
| 副作用の表現 | `!Postgres !AWS !Io` | capability 引数 + interface |
| read/write の区別 | なし（サービス名のみ） | `DbRead` / `DbWrite` 型で明示 |
| コンパイル時 capability 検査 | なし | interface 充足チェック |
| 純粋関数の識別 | エフェクト注釈なし | capability 引数なし（ambient 禁止前提） |
| サービス追加時 | 新エフェクト型が必要 | interface に impl を追加するだけ |
| テスト | モック困難 | `Ctx.mock(...)` で差し替え |
| 言語学習コスト | `!` 記法 + エフェクト一覧 | 普通の型システムの知識のみ |

---

## 実装スケジュール

```
v12.5 〜 v13.0   現ロードマップ完走（言語信頼性宣言）
v13.1            interface 継承仕様の詳細化
                  — CommonCtx / LoadCtx / WriteCtx の定義確定
                  — ambient effect 禁止の影響範囲調査
                  — Ctx Rune インターフェース確定
v13.2〜v13.x     段階的実装
                  — interface Db / Storage / Http 実装
                  — 既存 Rune の ctx ベース版を並行提供（旧 Rune に deprecated 警告）
                  — `seq` との統合
v14.0            capability-context 完成宣言
                  — エフェクト型（!AWS / !Postgres 等）完全廃止
                  — `!` 記法を言語仕様から削除
                  — 糖衣構文（`Ctx { db: DbRead }`）追加
```

---

*仕様 Close: 2026-06-08*
