# Favnir Roadmap — v13.1.0 〜 v14.0.0

Date: 2026-06-09

---

## 背景：エフェクト型設計の限界

v13.0.0（言語信頼性宣言）の完了により、`bind` / `chain` / `seq` の意味論と
エラー伝播の正確性は保証された。
次の課題は、副作用の表現方法そのものにある。

現行の `!Postgres !AWS !Io` 記法は v5.0.0 前後から積み重なった設計であり、
v10.x（Snowflake）・v11.x（Postgres TLS）・v12.x（fav2py/airgap E2E）の実装過程で
複数の構造的な問題が顕在化した。

本ロードマップは `lab/design/capability-context.md`（2026-06-08 確定）に基づき、
エフェクト型を通常の型システム（interface / capability 引数）で置き換える
**capability-context 設計**を v14.0.0「能力型完成宣言」として完走する計画である。

---

## 判明した問題の全体マップ

### 🔴 Critical（エフェクト型設計の根幹）

| # | 問題 | 発見経緯 |
|---|---|---|
| C-1 | `!Io` と `!Postgres` と `!AWS` が同列に並び抽象レベルが混在 | 設計レビュー（capability-context.md） |
| C-2 | ローカル DB と RDS と Snowflake を区別する根拠が `!` 記法にない | v10.x Snowflake 実装時 |
| C-3 | 新サービス追加ごとに言語仕様（`!Effect`）が肥大化する | v7.x〜v10.x の歴史 |
| C-4 | Rune が暗黙の接続情報をエフェクト経由で受け取るためテストでモックできない | E2E デモ全件 |

### 🟠 High（コンパイル時保証の欠落）

| # | 問題 | 発見経緯 |
|---|---|---|
| H-1 | read/write の区別がなく、`!Postgres` で読み書き両方できてしまう | lineage 解析強化時（v11.0.0） |
| H-2 | capability が揃っているかをコンパイル時に検査する手段がない | 実行時に接続エラーになって初めてわかる |
| H-3 | `validate` / `transform` ステージの純粋性を型から保証できない | コードレビュー時に判断できない |
| H-4 | ambient effect（ctx なしで `IO.println` を呼べる）が純粋性の主張を崩す | 設計レビュー |

### 🟡 Medium（開発体験・テスト容易性）

| # | 問題 | 発見経緯 |
|---|---|---|
| M-1 | Rune のモック差し替えに標準的な方法がない | E2E テスト実装時 |
| M-2 | 型シグネチャに `!` が並ぶと関数の要求が直感的に読めない | AI コード生成時に誤解が多い |
| M-3 | ステージ間でどの capability が流れているかを静的に確認できない | pipeline デバッグ時 |
| M-4 | ユーザー独自の ctx 型を追加する標準パターンがない | マルチテナント実装時 |

### 🤖 AI フレンドリー（Claude Code / Codex が混乱する原因）

| # | 問題 | 影響 |
|---|---|---|
| A-1 | `!Postgres` と `!Snowflake` と `!AWS` の関係を AI が推論できない | 同等の操作に異なる `!` を付けるコードを生成する |
| A-2 | 純粋ステージに `!Io` が混入してもコンパイラが警告しない | AI が副作用を誤って混入させる |
| A-3 | interface ベースの型が明示されていないため AI が型から要求を読めない | 引数パターンを毎回推測する |
| A-4 | テスト差し替えパターンが標準化されていないため AI がテストを書けない | E2E デモレベルでしかテストが書かれない |

---

## バージョン別ロードマップ

---

### v13.1.0 — interface 継承仕様確定 + ambient effect 禁止調査
**テーマ**: capability-context 設計の仕様を確定し、移行コストを正確に測る

**背景**:
`lab/design/capability-context.md` は設計提案であり、未実装。
実装前に「ambient effect 禁止が既存コードにどう影響するか」を
静的解析で全件洗い出す必要がある。
また `interface` 継承（`LoadCtx: CommonCtx`）の型チェックルールを
checker.fav / compiler.fav に追加するための仕様詳細化が必要。

**実装**:
- `interface A: B { ... }` の継承構文を parser.rs・compiler.fav・checker.fav に追加
  - `LoadCtx: CommonCtx` → `LoadCtx` は `CommonCtx` のフィールドを継承
  - 循環継承の検出（E0019）
- `fav check --ambient` スキャン: ctx なしで呼ばれるエフェクト付き呼び出しを列挙:
  ```
  W008: ambient effect call — IO.println called without ctx argument
    --> pipeline.fav:5:3
     |
   5 | bind _ <- IO.println("done")
     |           ^^^^^^^^^^^^^^^^^^
     |
     = help: pass io capability: `ctx.io.println("done")`
     = note: ambient effects will be an error in v14.0
  ```
- `lab/design/capability-context.md` の全 interface 型定義を仕様書として確定:
  - `DbRead` / `DbWrite` / `StorageRead` / `StorageWrite` / `HttpClient` / `Io` / `Env`
  - `CommonCtx` / `LoadCtx` / `WriteCtx` / `MigrateCtx`
- `Ctx` Rune インターフェース（`build` / `mock`）の型シグネチャ設計確定
- `self/compiler.fav` / `self/checker.fav` への W008 件数調査レポートを `lab/audit/w008-ambient.md` に出力
- テスト: `interface_inheritance_parsed` / `interface_inheritance_field_access` / `w008_ambient_effect_detected` / `e0019_circular_interface`

---

### v13.2.0 — DbRead / DbWrite / StorageRead / StorageWrite interface 実装
**テーマ**: データ操作 capability の interface 型を言語に導入する

**背景**:
`!Postgres` と `!AWS` の実体は「DB を読む」「DB に書く」「Storage に書く」の 3 操作。
まずデータ操作に関わる 4 interface を完全実装し、
既存の `Postgres.*` / `AWS.*` / `Snowflake.*` Rune を新 interface に対応させる。

**実装 — interface 型追加**:
- `checker.fav` / `compiler.fav` に組み込み interface として `DbRead` / `DbWrite` / `StorageRead` / `StorageWrite` を登録
- interface フィールドへのメソッド呼び出し構文 `ctx.db.query(...)` を parser・compiler に追加
- 型チェック: `ctx: DbRead` が要求される箇所で `DbRead` を実装していない型を渡すとエラー（E0020）

**実装 — 既存 Rune 対応**:
- `runes/postgres/` に `PostgresDb` 型（`impl PostgresDb for Db`）追加
  - `DbRead.query` → `Postgres.query_raw` に委譲
  - `DbWrite.execute` → `Postgres.execute_raw` に委譲
- `runes/aws/` に `DynamoDb` / `S3Storage` 型追加（同様）
- `runes/snowflake/` に `SnowflakeDb` 型追加（同様）
- 旧 `Postgres.*` / `AWS.*` / `Snowflake.*` の直接呼び出しに deprecated 警告（W009）:
  ```
  W009: direct Rune call is deprecated — use capability interface instead
    --> pipeline.fav:10:10
     |
  10 | bind _ <- Postgres.execute_raw(...)
     |           ^^^^^^^^^^^^^^^^^^^^ deprecated
     |
     = help: migrate to `chain _ <- ctx.db.execute(...)`
     = note: direct Rune calls will be an error in v14.0
  ```
- テスト: `db_read_interface_type_check` / `db_write_rejects_read_ctx` / `w009_postgres_direct_deprecated`

---

### v13.3.0 — HttpClient / Io / Env interface 実装
**テーマ**: I/O 操作 capability の interface 型を言語に導入する

**背景**:
`!Http`（HTTP クライアント）、`!IO`（標準 I/O）、環境変数読み取りは
データ操作とは独立した capability である。
これら 3 interface を実装し、全 capability 型の初期セットを完成させる。

**実装**:
- `HttpClient` / `Io` / `Env` を組み込み interface として登録
- `runes/http/` に `HttpClientImpl` 型（`impl HttpClientImpl for HttpClient`）追加
  - `HttpClient.get` / `HttpClient.post` → 既存 `Http.get_raw` / `Http.post_raw` に委譲
- `Io` interface:
  - `println(msg: String) -> Unit`
  - `capture() -> IoCapture`（テスト用 — stdout を文字列として収集）
- `Env` interface:
  - `require(key: String) -> Result<String, String>`（未設定なら Err）
- 旧 `IO.*` / `Http.*` への W009 deprecated 警告を同様に追加
- `IoCapture` 型: `captured() -> String` でキャプチャした出力を返す
- テスト: `io_interface_println` / `io_capture_test_pattern` / `env_require_missing_key` / `http_client_get_type_check`

---

### v13.4.0 — CommonCtx / LoadCtx / WriteCtx / MigrateCtx 実装
**テーマ**: ステージ別コンテキスト interface を型システムに組み込む

**背景**:
個別 capability interface が揃ったので、それらを組み合わせた
「用途別コンテキスト interface」を実装する。
これにより、ステージが必要な capability だけを宣言し、
不要な capability に物理的にアクセスできない設計が実現する。

**実装**:
- `CommonCtx` / `LoadCtx` / `WriteCtx` / `MigrateCtx` を組み込み interface として登録
- `interface LoadCtx: CommonCtx { db: DbRead }` の継承フィールド解決をコンパイラに実装
- 関数シグネチャ `fn load(ctx: LoadCtx) -> Result<Loaded, String>` の型チェック:
  - `ctx.io.println(...)` は OK（`CommonCtx` から継承）
  - `ctx.db.query(...)` は OK（`LoadCtx.db: DbRead`）
  - `ctx.db.execute(...)` はエラー（`DbRead` に `execute` はない）（E0020）
  - `ctx.storage.put(...)` はエラー（`LoadCtx` に `storage` はない）（E0021: capability not in context）
- コンテキスト型によるステージ純粋性検査:
  - capability 引数なし → 「純粋関数」として型レベルで保証
  - `fn transform(d: Validated) -> Result<Transformed, String>` に `IO.println` が混入していたら W008
- テスト: `load_ctx_allows_db_read` / `load_ctx_rejects_db_write` / `e0021_capability_not_in_context` / `pure_fn_no_ambient_effect`

---

### v13.5.0 — AppCtx 具象型 + `Ctx.build` / `Ctx.mock` Rune 実装
**テーマ**: 本番・テスト双方で使えるコンテキスト組み立て標準を提供する

**背景**:
interface 型が揃ったので、すべての interface を実装した具象型 `AppCtx` と、
それを組み立て・差し替えるための `Ctx` Rune を実装する。
これにより「本番は `Ctx.build(env)` でリソースを初期化、
テストは `Ctx.mock(...)` でモックを差し込む」パターンが標準化される。

**実装**:
- `AppCtx` 型定義（名目型ラッパー）:
  ```
  type AppCtx(
    db:      Db,
    storage: Storage,
    http:    HttpClient,
    io:      Io,
    env:     Env
  )
  ```
  - `impl AppCtx for LoadCtx` / `impl AppCtx for WriteCtx` / `impl AppCtx for MigrateCtx`
- `Ctx` Rune 実装（`runes/ctx/`）:
  - `Ctx.build(env: Env) -> Result<AppCtx, String>`:
    - `DATABASE_URL` / `AWS_REGION` 等の環境変数を検証
    - 欠如していれば起動時に即エラー（実行時だが起動直後）
  - `Ctx.mock(db, storage, io) -> AppCtx`:
    - `MockDb` / `MockStorage` / `IoCapture` を受け取る
    - テスト専用コンストラクタ
- `MockDb` / `MockStorage` 型:
  - `MockDb.seed(rows: List<Row>) -> MockDb`
  - `MockStorage.empty() -> MockStorage`
- fav.toml に `[context]` セクション追加:
  ```toml
  [context]
  db_url     = "${DATABASE_URL}"
  storage    = "s3"
  http       = "ureq"
  ```
- テスト: `ctx_build_missing_db_url_returns_err` / `ctx_mock_db_query_returns_seeded` / `ctx_mock_io_capture_output`

---

### v13.6.0 — `ctx.field.method()` フィールドアクセス構文の実装 + E2E デモ書き換え（型チェックのみ）
**テーマ**: capability-context 設計の核となるフィールドアクセスメソッド呼び出し構文を言語に実装する

**背景**:
v13.5.0 で `AppCtx` 具象型と `Ctx.build` / `Ctx.mock` Rune が実装されたが、
実際のフィールドアクセス構文 `ctx.io.println(msg)` / `ctx.db.query(sql)` は
まだ言語に存在しない。
現状は `AppCtx.io_println(ctx, msg)` という名前空間ワークアラウンドで対処しているが、
これは `lab/design/capability-context.md` の設計意図（`ctx.io.println(msg)` スタイル）と
異なり、AI 可読性・型安全性の両面で設計目標を達成できていない。

この版で `ctx.field.method(args)` 構文をパーサー・コンパイラ・VM に実装し、
E2E デモファイル（fav2py / airgap）を新構文で書き換えて型チェックが通ることを確認する。
**E2E デモの実際の実行（PASS=5）は v14.0.0 完了後に実施する**（インフラ変更不要のため後回し）。

**実装 — 言語機能**:
- `ctx.field.method(args)` 構文を parser.rs に追加:
  - `EFieldCall { receiver: Box<Expr>, field: String, method: String, args: Vec<Expr> }` AST ノード
  - `ctx.io.println("msg")` → `EFieldCall { receiver: ctx, field: "io", method: "println", args: [...] }`
- compiler.fav の `compile_expr` に `EFieldCall` コンパイルケースを追加:
  - フィールド型（`io: Io`, `db: DbRead` 等）を参照して対応する VM primitive を dispatch
- checker.fav の `infer_hm` に `EFieldCall` 型推論ケースを追加:
  - `ctx: AppCtx` → `ctx.io` の型は `Io` → `ctx.io.println` のシグネチャから戻り型推論
- VM の `Opcode::FieldCall` 追加（または既存の呼び出し機構を再利用）:
  - `ctx.io.println` → IoCapture または実 stdout 出力に dispatch

**実装 — E2E デモ書き換え（型チェックのみ）**:
- `infra/e2e-demo/fav2py/src/pipeline.fav` を `ctx.field.method()` 構文に書き換え:
  ```
  // 旧（ワークアラウンド）
  bind _ <- AppCtx.io_println(ctx, "message")
  // 新（設計通り）
  bind _ <- ctx.io.println("message")
  ```
- `infra/e2e-demo/airgap/src/analyze.fav` 同様
- `fav check` で両デモファイルがエラーなく通ることをテストで確認（実行は別途）
- W009 件数がデモファイルから 0 になることを確認:
  ```yaml
  ./target/debug/fav check infra/e2e-demo/fav2py/src/pipeline.fav
  ./target/debug/fav check infra/e2e-demo/airgap/src/analyze.fav
  ```

**備考**: E2E デモの実際の実行（PASS=5 確認）は v14.0.0 完了後に実施する。
インフラ（Docker / Terraform / run.sh）への変更は最小限にとどめ、
`--legacy` フラグ除去は型チェック通過後に行う。

- テスト: `field_call_syntax_parsed` / `field_call_type_checked` / `e2e_fav2py_ctx_based_compiles` / `e2e_airgap_ctx_based_compiles` / `w009_count_fav2py_zero` / `w009_count_airgap_zero`

---

### v13.7.0 — `seq` pipeline と ctx の統合
**テーマ**: パイプライン定義で ctx を自然に扱えるようにする

**背景**:
`seq Pipeline = LoadAndInsert |> Aggregate |> SaveResult` の各ステージが
異なる ctx 型（`LoadCtx` / `WriteCtx`）を要求するとき、
pipeline レベルでの型チェックと ctx の受け渡しルールを整備する必要がある。

**実装**:
- `seq` パイプラインの ctx 型推論:
  - 各ステージが要求する ctx の和型を pipeline の要求 ctx として推論
  - `LoadAndInsert: LoadCtx` + `SaveResult: WriteCtx` → pipeline は `AppCtx`（両方を実装）を要求
- `fn main()` での ctx 渡しパターン確立:
  ```
  fn main() -> Unit {
    chain ctx <- Ctx.build(Env.process())
    Pipeline(ctx, get_csv_path(IO.argv()))
  }
  ```
- checker.fav に seq-ctx 型チェックルールを追加:
  - 前ステージが要求する ctx と後ステージが要求する ctx が `AppCtx` で充足されるか確認
  - 不足していれば E0022（capability missing in pipeline）
- `par` 並列パイプライン（v9.13.0）でも同様に対応
- テスト: `seq_ctx_type_inferred` / `seq_ctx_appctx_satisfies_both` / `e0022_pipeline_capability_missing` / `par_ctx_type_check`

---

### v13.8.0 — ambient effect 禁止（W008 → E0023）
**テーマ**: capability 引数なしのエフェクト呼び出しをコンパイルエラーに昇格する

**背景**:
v13.1.0 で W008 として警告していた ambient effect 呼び出しを、
この版からコンパイルエラー（E0023）に昇格する。
これにより「capability 引数がなければ純粋」が言語レベルで保証される。

`self/compiler.fav` / `self/checker.fav` は v13.6.0 時点ではまだ旧 `IO.*` 呼び出しを
一部含んでいるため、この版で完全移行する。

**実装**:
- W008 を E0023 に昇格（コンパイルエラー）:
  ```
  E0023: ambient effect call is not allowed
    --> compiler.fav:142:5
     |
  142 | bind _ <- IO.println("compiling...")
      |           ^^^^^^^^^^^^^^^^^^^^^^^^^^^
      |
      = help: pass io capability: `ctx.io.println("compiling...")`
      = note: all side effects must be explicit capability arguments
  ```
- `self/compiler.fav` の `IO.println` / `IO.read_file_raw` 等を全件 ctx ベースに移行
- `self/checker.fav` 同様
- `--legacy` モードでは E0023 を W008 に降格（後方互換）
- `fav check --ambient` オプション廃止（通常チェックに統合）
- CI self-check ステップに E0023 件数ゼロチェックを追加
- テスト: `e0023_ambient_io_println` / `e0023_ambient_postgres_raw` / `legacy_mode_allows_ambient` / `ctx_based_compiler_fav_compiles`

---

### v13.9.0 — 型状態パターン統合 + lineage 更新
**テーマ**: capability-context と型状態パターンの相互作用を整備し、lineage 解析を新設計に対応させる

**背景**:
`capability-context.md` は型状態パターン（`Loaded` / `Validated` / `Transformed`）と
capability-context の組み合わせを設計の柱の一つとしている。
また、lineage 解析（`fav explain --lineage`）は `!Snowflake(read/write)` の区別を
エフェクト型から読んでいるため、capability 型への移行後に再実装が必要。

**実装**:
- 型状態パターンのコンパイル時チェック強化:
  - `fn validate(d: Loaded)` が `Validated` を受け取ろうとするとエラー（E0024: type state mismatch）
  - フェーズを飛ばした呼び出しをコンパイルエラーとして検出
- lineage 解析の更新（`fav/src/lineage.rs`）:
  - `DbRead` capability を持つステージ → `read` ノード
  - `DbWrite` capability を持つステージ → `write` ノード
  - capability なし（純粋）ステージ → `transform` ノード
  - `StorageWrite` → `sink` ノード
  - `fav explain --lineage` の出力形式を更新（サービス名ではなく capability 種別を表示）
- `fav doc --builtins --format json` に capability 情報を追加:
  ```json
  "DbRead.query": {
    "signature": "(sql: String, params: List<String>) -> Result<List<Row>, String>",
    "capability": "DbRead",
    "impls": ["PostgresDb", "SnowflakeDb", "DynamoDb", "MockDb"]
  }
  ```
- テスト: `e0024_type_state_skip_phase` / `lineage_db_read_node` / `lineage_pure_transform_node` / `doc_builtins_capability_field`

---

### v13.10.0 — `!` 記法廃止 + 糖衣構文追加
**テーマ**: `!` 記法を言語仕様から削除し、capability-context 設計を完成させる

**背景**:
v13.8.0 で ambient effect が禁止され、v13.6.0 以降では
新しい E2E デモも ctx ベースで動作している。
この版で旧 `!` 記法を言語仕様から正式に削除し、
`capability-context.md` が「後回し」とした糖衣構文を追加する。

**実装 — `!` 記法削除**:
- parser.rs から `!Effect` トークン解析を削除
- compiler.fav / checker.fav から `!` エフェクト処理コードを削除
- `--legacy` モードでのみ `!` を許容（後方互換フラグとして残存）
- `fav fmt` が `!Effect` を含む旧コードを自動変換:
  ```
  before: fn load() -> Result<Loaded, String> !Postgres
  after:  fn load(ctx: LoadCtx) -> Result<Loaded, String>
  ```
  （シグネチャ変換は `--migrate` フラグで実行、手動確認が必要な箇所は W010 で警告）

**実装 — 糖衣構文**:
- `Ctx { db: DbRead }` による部分渡し構文（`capability-context.md` 「糖衣構文（後回し）」）:
  ```
  // 糖衣構文
  fn Load(Ctx { db: DbRead, io }, page: Int) -> Result<Loaded, String>
  // 脱糖後
  fn Load(ctx: LoadCtx, page: Int) -> Result<Loaded, String>
  ```
- compiler.fav の `parse_fn_def` に `Ctx { ... }` パターンを追加
- `fav fmt` が糖衣構文を正規化（整形フォーマット統一）

**実装 — 移行ガイド自動生成**:
- `fav migrate --from-effects <file>`: 旧 `!` 記法を新 ctx ベースに自動変換（W010 で要確認箇所を列挙）
- `site/content/docs/migration/` に capability-context 移行ガイドを追加

- テスト: `bang_notation_removed_in_non_legacy` / `fmt_auto_migrates_effect_notation` / `ctx_destructure_sugar_desugars` / `migrate_tool_converts_postgres_effect`

---

### v14.0.0 — 能力型完成宣言
**テーマ**: `!` 記法の完全廃止と capability-context 設計の安定宣言

**完成条件**:

| 確認項目 | 対応バージョン |
|---|---|
| `interface` 継承構文（`LoadCtx: CommonCtx`）のコンパイル時チェック | v13.1.0 |
| `DbRead` / `DbWrite` / `StorageRead` / `StorageWrite` interface 実装 | v13.2.0 |
| `HttpClient` / `Io` / `Env` interface 実装 | v13.3.0 |
| `LoadCtx` / `WriteCtx` / `MigrateCtx` による capability 充足チェック | v13.4.0 |
| `AppCtx` + `Ctx.build` / `Ctx.mock` Rune 実装 | v13.5.0 |
| `ctx.field.method()` 構文実装 + E2E デモ書き換え（型チェック通過） | v13.6.0 |
| E2E デモ実際の実行 PASS=5 確認 | **v14.0.0 完了後** |
| `seq` pipeline での ctx 型推論 + E0022 | v13.7.0 |
| ambient effect 禁止（E0023）+ `self/` 全件移行 | v13.8.0 |
| 型状態パターン統合 + lineage 解析更新 | v13.9.0 |
| `!` 記法廃止 + 糖衣構文 + `fav migrate` ツール | v13.10.0 |
| `--legacy` 以外で `!` 記法が完全に使えないことを CI で確認 | v14.0.0 |

**宣言内容**:
「Favnir の副作用は通常の型システムで表現される。
`capability 引数がなければ純粋` が言語レベルで保証される。
`!Postgres` / `!AWS` 等のエフェクト型は廃止され、
`DbRead` / `DbWrite` / `StorageWrite` 等の capability interface で置き換えられた。
新しいクラウドサービスの追加は言語仕様の変更を必要とせず、
interface に `impl` を追加するだけで完了する。
Claude Code / Codex 等の AI ツールは `Ctx.mock(...)` によって
本番接続なしにパイプライン全体をテスト可能である。」

---

## 優先度サマリー

```
🔴 v13.1〜13.4   interface 型基盤の確立
                  ← capability-context の根幹。これがないと全て始まらない

🟠 v13.5〜13.6   AppCtx + Ctx Rune + ctx.field.method() 構文実装
                  ← 旧 API から新 API への言語実装。W009 ゼロを確認（実行は v14.0 後）

🟡 v13.7〜13.8   seq 統合 + ambient effect 禁止
                  ← 純粋性保証の完成。E0023 で設計を言語レベルで強制

🔵 v13.9〜13.10  型状態・lineage 統合 + `!` 廃止 + 糖衣構文
                  ← 設計の仕上げと後方互換移行パスの整備

🟢 v14.0.0       能力型完成宣言 + AI テストパターン確立
```
