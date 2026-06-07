# Favnir Roadmap — v12.1.0 〜 v13.0.0

Date: 2026-06-07

---

## 背景：fav2py E2E デモで判明した問題

v12.0.0（Python トランスパイラ完成宣言）の E2E 実施により、
言語設計・ランタイム・インフラの各層で複数の重大な問題が発覚した。
また、Claude Code / Codex 等の AI が Favnir を書く場面が増えており、
「人間にもAIにも混乱しない言語」を目指す観点での強化も必要。
本ロードマップはそれらを優先度順に修正しながら、
v13.0.0「言語信頼性宣言」を目指す計画である。

---

## 判明した問題の全体マップ

### 🔴 Critical（言語設計の根幹）

| # | 問題 | 発見経緯 |
|---|---|---|
| C-1 | `bind` が monadic bind ではなく単純代入 | E2E で Err がサイレント通過 |
| C-2 | `bind x → bind x` の再束縛がコンパイルエラーにならない | 設計レビュー |
| C-3 | `bind _` で Result を黙って捨てられる（`#[must_use]` 相当なし） | E2E デバッグ中 |
| C-4 | `seq A |> B |> C` で A が Err を返しても B・C が動き続ける | E2E で exit 0 のまま誤動作 |

### 🟠 High（ランタイム・エフェクト）

| # | 問題 | 発見経緯 |
|---|---|---|
| H-1 | Postgres Rune が `NoTls` のみ → RDS `force_ssl=1` で全接続失敗 | E2E で "db error" |
| H-2 | `tokio_postgres::Error::to_string()` が "db error" のみで詳細不明 | E2E デバッグ中 |
| H-3 | `$1::json` パラメータ方式が tokio_postgres と PostgreSQL 間で型不一致 | E2E で INSERT 失敗 |
| H-4 | Primitive の戻り値型がユーザーコードから参照できない | `Csv.parse_raw` の型を vm.rs を読むまで把握できなかった |

### 🟡 Medium（開発体験）

| # | 問題 | 発見経緯 |
|---|---|---|
| M-1 | `fav run` に `--verbose` / `--trace` がなく stage 内の失敗箇所が不明 | ECS ログが空で原因特定に長時間 |
| M-2 | `fav doc` が組み込み Primitive をカバーしない | Csv.parse_raw の型を調べる手段がない |
| M-3 | Terraform テンプレートに RDS SSL 設定が標準化されていない | 毎回 RDS 再起動が必要 |
| M-4 | verify.sh が `jq` に依存（環境依存） | Git Bash 環境に jq がなく失敗 |
| M-5 | Windows 環境で git bash `/tmp/` と Python `tempfile.gettempdir()` が不一致 | verify.sh がファイルを見つけられなかった |

### 🤖 AI フレンドリー（Claude Code / Codex が混乱する原因）

| # | 問題 | 影響 |
|---|---|---|
| A-1 | Primitive の型シグネチャを機械可読な形で参照できない | AI が存在しない関数を生成する・型を誤解する |
| A-2 | エラーメッセージに `help:` / `suggestion` がなく次の行動が不明 | AI が試行錯誤を繰り返しデバッグサイクルが増加 |
| A-3 | `fav check` の出力がテキストのみで AI ツールが解析できない | AI が自動修正ループを回せない |
| A-4 | `bind` と `chain` のセマンティクスが曖昧で AI が誤用する | 学習データがないため AI は構文から意味を推論するしかない |
| A-5 | 正しい雛形を生成する手段がなく AI が「それっぽいが間違った」構文から始める | 最初の一歩でエラーが出てデバッグコストが高い |
| A-6 | `fav explain <code>` がなく AI がエラーコードの意味を知る手段がない | エラーコードを渡されても対処法がわからない |

---

## バージョン別ロードマップ

---

### v12.1.0 — `bind` イミュータビリティ強制（E0018）
**テーマ**: 変数束縛の意味論を正しく定義する

**背景**:
`bind` は「変数束縛」であるにもかかわらず、同一スコープで同じ名前に再 `bind` できてしまう。
これは関数型言語の束縛（一度束縛したら変更不可）の原則に反しており、
AI が同一名を複数回 `bind` しても検出されない。

**実装**:
- `checker.fav`: stage / fn 本体の bind チェック時に「束縛済みセット」を管理
- `bind x <- expr` → x が既に束縛済みなら **E0018** を発行（`help:` 付き）:
  ```
  E0018: variable 'x' is already bound in this scope
    --> pipeline.fav:12:3
     |
   8 | bind x <- compute1()
     |      - first bound here
  ...
  12 | bind x <- compute2()
     |      ^ cannot rebind 'x'
     |
     = help: use a different name: `bind x2 <- compute2()`
     = help: or discard the value: `bind _ <- compute2()`
  ```
- `_`（アンダースコア）は例外 — 何度でも使用可（捨て変数の慣例）
- `chain x` も同様に二重束縛禁止
- **AI 対応**: `help:` で次の行動を明示することで AI の自己修正ループが機能する
- テスト: `e0018_rebind_detected` / `e0018_underscore_allowed` / `e0018_help_message_shown`

---

### v12.2.0 — `bind _` で Result を捨てると警告（W006）
**テーマ**: `#[must_use]` 相当の静的検出を `fav lint` / `fav check` に追加

**背景**:
`bind _ <- Postgres.execute_raw(...)` のように、
エフェクトが返す `Result<T, E>` をアンダースコアで捨てても
現状は何の警告も出ない。AI も人間も「これは安全」と誤解しやすい。

**実装**:
- `fav check` / `fav lint` で W006 を発行（`help:` + 代替案付き）:
  ```
  W006: discarding Result value with bind _
    --> pipeline.fav:10:3
     |
  10 | bind _ <- Postgres.execute_raw(...)
     |           ^^^^^^^^^^^^^^^^^^^^^^^^^ this returns Result<Unit, String>
     |
     = help: use `chain _` to propagate errors automatically
     = help: or handle explicitly: `match Postgres.execute_raw(...) { Ok(_) => ... Err(e) => ... }`
     = note: silent failure here caused the pipeline to continue after a connection error
  ```
- 対象: `Postgres.*` / `AWS.*` / `IO.*` 等エフェクト付き呼び出しの戻り値が `Result` の場合
- `_` だけでなく `bind x <- result_expr` の後 `x` が一度も使われない場合も対象（W006b）
- `fav.toml` の `[lint] allow = ["W006"]` で個別抑制可能
- テスト: `w006_bind_underscore_result` / `w006_unused_result_binding`

---

### v12.3.0 — `bind` を真の monadic bind に修正
**テーマ**: `bind` が Err を黙って捨てる問題を言語レベルで解消する

**背景**:
関数型言語の `bind`（モナドの `>>=`）は本来：
- `Ok(v)` → `v` を x に束縛して続行
- `Err(e)` → そこで短絡し Err を上位に伝播

現状の Favnir `bind` は「単純代入」であり `bind` ≠ monadic bind になっている。
`chain` が本来の monadic bind の動作をしているため、キーワードの意味と実装が乖離しており、
AI が `bind` と `chain` を適切に使い分けることができない。

**実装**:
- `--legacy` モード: `bind x <- expr` の `expr` が `Result<T, E>` を返す場合
  - `Ok(v)` → `x = v`（unwrap して束縛）
  - `Err(e)` → stage 全体が即座に `Err(e)` を返す（短絡）
- Favnir pipeline（デフォルト）モードは `chain` ベースのため変更不要
- `chain` との差を整理: `chain` = エフェクトチェーン付き monadic bind
- 既存の `bind` 利用箇所への影響を `fav check` で事前検出（W006 と連動）
- テスト: `bind_propagates_err_in_legacy` / `bind_ok_unwraps_value`

---

### v12.4.0 — `seq` pipeline fail-fast
**テーマ**: パイプラインが途中で失敗したら後続 stage を止める

**背景**:
`seq Pipeline = LoadAndInsert |> Aggregate |> SaveResult` において、
`LoadAndInsert` が実質失敗しても `Aggregate` が動き続ける。
ECS タスクが exit 0 で終了し S3 にエラー文字列が保存されるだけという状況は
「成功に見えるが実は失敗」で最も診断困難なバグ。
AI がこのパターンで書いたコードのデバッグは特に困難。

**実装**:
- `seq` パイプラインの各 stage 呼び出しに ChainCheck を追加
- 前 stage が `Err` を返したら後続 stage を実行せず pipeline 全体を失敗
- エラーには stage 名と位置を付与して追跡可能にする:
  ```
  [ERROR] pipeline stopped at stage 1/3 'LoadAndInsert': db error: SSL connection required
  ```
- vm.rs の stage 呼び出しシーケンス修正
- テスト: `seq_stops_on_stage_err` / `seq_passes_ok_through` / `seq_error_includes_stage_name`

---

### v12.5.0 — `fav run --verbose` + `fav check --json` / `--show-types`
**テーマ**: 実行時・静的解析のデバッグ可視性を人間と AI の両方に最適化する

**背景**:
ECS Fargate 上でのデバッグは 1 サイクル 6〜8 分かかる。
`--verbose` があれば CloudWatch に詳細が残り原因特定が短縮できる。
また AI は `fav check` のテキスト出力を解析して自己修正ループを回せない。
機械可読な出力形式と型推論の可視化が必要。

**実装 — `fav run --verbose`**:
- `fav run --verbose <file>`: 以下をすべて stderr に出力
  ```
  [TRACE] stage LoadAndInsert: enter(path="/app/sample.csv")
  [TRACE]   bind rows_json <- load_csv_rows_json(...) → Ok("..."[312 chars])
  [TRACE]   bind _ <- Postgres.execute_raw("CREATE TABLE...") → Err("db error: SSL required")
  [TRACE] stage LoadAndInsert: exit Err("db error: SSL required")
  [TRACE] seq Pipeline: stopped at stage 1/3 (LoadAndInsert)
  ```
- 値の出力は最大 200 文字でトランケート
- `--trace` でフル出力（引数・戻り値の完全表示）
- `fav.toml` の `[run] verbose = true` でも有効化

**実装 — `fav check --json`（AI フレンドリー）**:
- エラー・警告を JSON 形式で出力:
  ```json
  {
    "errors": [
      {
        "code": "E0018",
        "message": "variable 'x' is already bound in this scope",
        "file": "pipeline.fav",
        "line": 12, "col": 3,
        "suggestion": "rename to 'x2' or use 'bind _'"
      }
    ],
    "warnings": [
      {
        "code": "W006",
        "message": "discarding Result value with bind _",
        "file": "pipeline.fav",
        "line": 10, "col": 3,
        "suggestion": "use 'chain _' to propagate errors"
      }
    ]
  }
  ```
- AI ツールが `fav check --json` → JSON パース → エラー読み取り → 修正 のループを回せる

**実装 — `fav check --show-types`（AI フレンドリー）**:
- 各 `bind` / `chain` の右辺型を表示:
  ```
  pipeline.fav:8   bind rows_json : String
  pipeline.fav:10  bind _         : Result<Unit, String>  ← W006
  pipeline.fav:12  bind _         : Result<Unit, String>  ← W006
  ```
- AI が「自分が書いたコードの型が何か」を確認でき、Result を捨てているかが一目瞭然

**テスト**: `verbose_logs_stage_enter_exit` / `check_json_output_format` / `check_show_types_bind`

---

### v12.6.0 — Postgres Rune TLS 対応 + エラー詳細化
**テーマ**: RDS 等の SSL 必須環境で Postgres Rune を動作させる

**背景**:
`tokio_postgres::NoTls` のみの実装では `rds.force_ssl=1`（RDS PostgreSQL 16 のデフォルト）に
接続できない。また `e.to_string()` が "db error" のみで詳細が失われる問題もある。
AI にとって "db error" は診断不能な情報であり、詳細エラーが必須。

**実装**:
- `Cargo.toml` に `tokio-postgres-native-tls` または `tokio-postgres-rustls` を追加
- `pg_connect` で `sslmode` を env / fav.toml から読んで TLS を切り替え:
  - `sslmode=disable` → `NoTls`（既存動作）
  - `sslmode=prefer` → TLS 試行、失敗なら NoTls にフォールバック（psycopg2 デフォルト相当）
  - `sslmode=require` → TLS 必須
- `DATABASE_URL` の `?sslmode=require` クエリパラメータも解析
- エラー詳細化: `DbError` の `message()` / `code()` / `detail()` を連結:
  ```
  before: "db error"
  after:  "db error: SSL connection is required (SQLSTATE 08P01)"
  ```
- `fav.toml [postgres]` に `sslmode` キー追加
- Terraform テンプレートに `aws_db_parameter_group`（`rds.force_ssl=0`）を標準同梱
- テスト: `postgres_ssl_mode_disable` / `postgres_ssl_mode_require` / `postgres_error_includes_detail`

---

### v12.7.0 — Primitive 型リファレンス（`fav doc --builtins`）
**テーマ**: 組み込み Primitive の型シグネチャを人間・AI の両方が参照できるようにする

**背景**:
`Csv.parse_raw(text, sep, header)` が `Result<List<Record>, String>` を返すことを
知るには `vm.rs` を読む必要があった。AI は vm.rs を参照できないため、
型を推測して誤ったコードを書く。機械可読形式での提供が必須。

**実装 — `fav doc --builtins`（人間向け Markdown）**:
  ```markdown
  ## Csv

  ### Csv.parse_raw
  `(text: String, sep: String, header: Bool) -> Result<List<Record>, String> !IO`

  CSV テキストを解析してレコードのリストを返す。
  header=true の場合、1行目をフィールド名として使用する。
  ```

**実装 — `fav doc --builtins --format json`（AI 向け機械可読）**:
  ```json
  {
    "Csv.parse_raw": {
      "signature": "(text: String, sep: String, header: Bool) -> Result<List<Record>, String>",
      "effects": ["!IO"],
      "returns_result": true,
      "description": "CSV テキストを解析してレコードのリストを返す"
    },
    "Postgres.execute_raw": {
      "signature": "(sql: String, params: String) -> Result<Unit, String>",
      "effects": ["!Postgres"],
      "returns_result": true,
      "description": "SQL を実行する（SELECT 以外）。params は JSON 配列文字列"
    }
  }
  ```
- AI がコード生成前に「この namespace に何があるか」をツールとして問い合わせ可能
- 対象 namespace: IO / Csv / Schema / Json / Gen / AWS / Postgres / Snowflake / Http / Llm
- `vm.rs` の各ハンドラに `// @doc` コメントとして型情報を付与し自動抽出
- `site/content/docs/primitives/` に生成ドキュメントを組み込み

**実装 — `fav explain <error-code>`（AI フレンドリー）**:
  ```bash
  fav explain E0018
  ```
  ```
  E0018: Variable already bound

  Favnir では変数は一度だけ束縛できます（イミュータブル）。
  同一スコープで bind x を 2 回書くことはできません。

  修正例:
    誤: bind x <- step1()
        bind x <- step2(x)   ← E0018

    正: bind x  <- step1()
        bind x2 <- step2(x)  ← OK

  関連: W006（Result を bind _ で捨てる）
  ```
- AI がエラーコードを受け取った際にコンパイラ自身に意味を聞ける

**テスト**: `doc_builtins_json_format` / `doc_builtins_csv_parse_raw` / `explain_e0018`

---

### v12.8.0 — `fav scaffold <template>` — 正しい雛形生成
**テーマ**: AI・人間が正しい構文から始められるスキャフォールディング

**背景**:
AI が Favnir を書く際に「それっぽいが文法的に誤った」コードから始めると、
最初のコンパイルエラーから修正するコストが高い。
正しい雛形があればそこに肉付けするだけで済む。

**実装**:
- `fav scaffold stage <name>` — stage の雛形を生成:
  ```favnir
  stage MyStage: String -> String !IO = |input| {
    bind result <- IO.println(input)
    input
  }
  ```
- `fav scaffold seq <name>` — seq パイプラインの雛形を生成
- `fav scaffold postgres-etl` — Postgres ETL の完全雛形:
  ```favnir
  // LoadAndInsert / Aggregate / SaveResult の正しいパターン
  ```
- `fav scaffold rune <name>` — Rune の雛形を生成
- `fav new --template postgres-etl <dir>` — fav.toml 込みのプロジェクト生成
- テスト: `scaffold_stage_compiles` / `scaffold_postgres_etl_compiles`

---

### v12.9.0 — CI ローカル統合テスト（docker-compose Postgres）
**テーマ**: ECS に上げる前に Postgres Rune の動作を CI で検証する

**背景**:
今回の fav2py E2E では「ECS に上げてみて初めてわかる」バグが多く、
デバッグサイクルが長かった。CI の docker-compose で Postgres を立てて
プリミティブレベルの動作を確認できれば、E2E 前に多くの問題を検出できる。

**実装**:
- `.github/workflows/ci.yml` に `integration` ジョブを追加:
  ```yaml
  services:
    postgres:
      image: postgres:16
      env:
        POSTGRES_PASSWORD: test
      options: --health-cmd pg_isready
  ```
- `fav/tests/integration/postgres_rune_test.rs`:
  - `CREATE TABLE` / `INSERT` / `SELECT` / `DROP TABLE` が通ること
  - SSL なし接続のスモークテスト
  - `$1::text` パラメータ / JSON 直接埋め込みの両方をテスト
- テストは `#[cfg(feature = "integration")]` で分離
- テスト: `postgres_create_insert_select` / `postgres_json_embed_insert` / `postgres_error_table_not_found`

---

### v12.10.0 — 全エラーメッセージに `help:` + `fav check --strict`
**テーマ**: コンパイラの全出力を「次の行動が明示される」形式に統一する

**背景**:
Rust のコンパイラが AI に強い理由は `help:` / `note:` で次の行動が明示されているから。
Favnir の全エラー・警告に同様の `help:` を追加し、
AI がコンパイラの出力だけで自己修正できる状態を目指す。

**実装**:
- E0001〜E0018 / W001〜W006 の全メッセージに `help:` suggestion を追加
- `fav check --strict`: W001〜W006 すべてをエラーとして扱い、exit 1
- `fav.toml` の `[lint]` セクション拡充:
  ```toml
  [lint]
  warn_as_error = ["W006"]   # 特定警告のみエラー化
  allow = ["W004"]           # 特定警告を抑制
  ```
- 既存の self/ コードベース（compiler.fav / checker.fav）を --strict 対応に修正
- テスト: `all_errors_have_help` / `strict_mode_w006_is_error` / `strict_mode_allow_overrides`

---

### v13.0.0 — 言語信頼性宣言
**テーマ**: E2E で発覚した全問題の解消と、人間・AI 双方に信頼できる言語としての安定性確認

**完成条件**:

| 確認項目 | 対応バージョン |
|---|---|
| `bind` 再束縛がコンパイルエラー（E0018 + `help:`） | v12.1.0 |
| `bind _` で Result を捨てると警告（W006 + `help:`） | v12.2.0 |
| `bind` が Err で短絡（monadic bind 修正） | v12.3.0 |
| `seq` pipeline が fail-fast | v12.4.0 |
| `fav run --verbose` / `fav check --json` / `--show-types` | v12.5.0 |
| Postgres Rune が TLS 対応 + エラー詳細化 | v12.6.0 |
| `fav doc --builtins --format json` / `fav explain <code>` | v12.7.0 |
| `fav scaffold <template>` | v12.8.0 |
| CI ローカル Postgres 統合テスト | v12.9.0 |
| 全エラーに `help:` + `fav check --strict` | v12.10.0 |
| 全 E2E デモ（airgap / fav2py）再実行 PASS | v13.0.0 |

**宣言内容**:
「型安全・エラー伝播・デバッグ可視性の三点において、
Favnir のランタイム挙動は型システムの宣言と一致することを保証する。
また、Claude Code / Codex 等の AI ツールが `fav check --json` と
`fav doc --builtins --format json` を用いて自律的にコードを修正できることを確認する。」

---

## 優先度サマリー

```
🔴 v12.1〜12.4   bind セマンティクス修正 + seq fail-fast
                  ← 言語の根幹・最優先。AI も人間も正しく使えない

🟠 v12.5〜12.6   --verbose / --json / --show-types + Postgres TLS
                  ← デバッグコスト削減。AI の自己修正ループを可能にする

🟡 v12.7〜12.8   doc --builtins --format json / fav explain / fav scaffold
                  ← AI フレンドリー強化。Primitive 参照・雛形生成

🔵 v12.9〜12.10  CI 統合テスト / 全 help: / --strict
                  ← 開発体験の底上げと品質ゲート

🟢 v13.0.0       全問題解消の安定宣言 + AI 自律修正確認
```
