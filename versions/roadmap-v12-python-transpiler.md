# Favnir Roadmap — Python トランスパイラ（v11.1.0 〜 v12.0.0）

Date: 2026-06-06

---

## 概要

Fav コードを Python コードに変換する `fav transpile --target python` を実装する。
Fav の型安全・エフェクト宣言・リネージをそのまま保ちながら、
boto3 / psycopg2 / uv などの Python エコシステムに展開できるようにする。

### なぜ Python トランスパイルか

- Fav Rune が未対応の AWS サービス（DynamoDB, Kinesis, Bedrock 等）を boto3 経由で即日サポートできる
- 既存 Python インフラ（Lambda zip, ECS, Glue）にそのまま乗せられる
- uv で仮想環境 + 依存解決 + デプロイまで完結する
- Fav コードが「型付き仕様書」として残り、Python は実行基盤になる分業構造

### 完成の定義（v12.0.0）

- `fav transpile --target python <file.fav>` で `.py` + `pyproject.toml` を生成
- 生成 Python が `uv run` で動作する
- `infra/e2e-demo/fav2py/` — Fav ネイティブ vs Python トランスパイルで RDS Postgres への
  ETL/CRUD 結果が一致することを E2E で実証（PASS=5 以上）

---

## バージョン別ロードマップ

### v11.1.0 — emit_python 基盤
**テーマ**: AST → Python コード生成の土台

- `src/emit_python.rs` 新規作成
- 型定義 → `@dataclass` 変換（`type Foo = { ... }` → `@dataclass class Foo`）
- 基本式 → Python 式（`Int`, `Float`, `String`, `Bool`, `List`, `if/else`）
- `fn` → Python `def`（引数・戻り型コメント付き）
- `bind x <- expr` → `x = expr`（モナド脱糖）
- `match` → `if/elif/else`（Option/Result パターン）
- `fav transpile --target python <file>` CLI エントリ（`cli.fav` + `driver.rs`）
- テスト: 型定義・fn・基本式の Python 出力スナップショット

---

### v11.2.0 — stage / seq → Python パイプライン変換
**テーマ**: パイプライン構造を Python 関数チェーンに変換

- `stage Foo: A -> B = |x| { ... }` → `def foo(x: A) -> B: ...`
- `seq Pipeline = A |> B |> C` → `def pipeline(x): return c(b(a(x)))`
- `fn main()` → `if __name__ == "__main__": main()`
- `IO.argv()` → `sys.argv[1:]`
- `List.*` stdlib → Python リスト内包表記 / `filter` / `map`
- テスト: LoadAll |> Validate |> WriteOutput の Python 出力検証

---

### v11.3.0 — IO エフェクト → Python 変換
**テーマ**: `!IO` を Python の標準 I/O に変換

- `IO.println(s)` → `print(s)`
- `IO.read_file_raw(path)` → `open(path).read()`（`Result` → try/except）
- `Csv.parse_raw(text, ",", true)` → `csv.DictReader`
- `Schema.adapt(raw, "T")` → dataclass 変換ヘルパー生成
- `Schema.to_json_array(rows, "T")` → `json.dumps([asdict(r) for r in rows])`
- テスト: airgap の analyze.fav を Python に変換して同じ CSV を処理

---

### v11.4.0 — AWS エフェクト → boto3 変換
**テーマ**: `!AWS` を boto3 S3 に変換

- `AWS.s3_put_object_raw(bucket, key, body)` → `boto3.client("s3").put_object(...)`
- `AWS.s3_get_object_raw(bucket, key)` → `boto3.client("s3").get_object(...)`
- `import rune "aws"` → `import boto3` を `pyproject.toml` 依存に追加
- `uv` 統合: `fav transpile` が `pyproject.toml`（boto3 依存付き）を生成
- テスト: airgap の analyze.fav を Python 変換 → S3 書き込みが boto3 コードになる

---

### v11.5.0 — Postgres Rune（Fav ネイティブ側）
**テーマ**: `!Postgres` エフェクト + Fav ネイティブ Postgres 操作

- `Effect::Postgres` 追加（ast.rs / parser.rs / fmt.rs 等 8 ファイル）
- `Postgres.execute_raw(sql, params)` → `Result<Unit, String>`
- `Postgres.query_raw(sql, params)` → `Result<String, String>`（JSON 文字列）
- `Postgres.query_typed<T>(sql, params)` → `Result<List<T>, String>`
- vm.rs: `tokio-postgres` / `deadpool-postgres` ベース実装
- `fav.toml` `[postgres]` セクション（host/port/dbname/user/password/sslmode）
- `fav infer --from postgres --table <name>` 対応（Snowflake と同様）
- `runes/postgres/postgres.fav` Rune 実装
- checker.fav 更新（`postgres_fn` / `builtin_ret_ty` / `ns_to_effect`）
- テスト: `postgres_effect_requires_annotation` 等

---

### v11.6.0 — Postgres → psycopg2 変換（トランスパイル側）
**テーマ**: `!Postgres` を psycopg2 Python コードに変換

- `Postgres.*` → `psycopg2.connect(...).cursor().execute(...)` に変換
- `fav.toml [postgres]` → Python の `os.environ` / `dotenv` 読み込みコードを生成
- `pyproject.toml` に `psycopg2-binary` 依存を自動追加
- `Schema.adapt` → psycopg2 `RealDictCursor` + dataclass ヘルパーに変換
- テスト: Postgres Rune を使った Fav パイプラインの Python 出力検証

---

### v11.7.0 — uv 統合
**テーマ**: `fav transpile` が uv プロジェクトとして完結する出力を生成

- `fav transpile --target python <file.fav> --out-dir <dir>` で以下を生成:
  - `<dir>/main.py`（トランスパイル済みコード）
  - `<dir>/pyproject.toml`（uv 形式、依存自動追加）
  - `<dir>/README.md`（生成元 .fav ファイルの情報）
- `fav transpile --run` オプション: 生成後に `uv run main.py` まで実行
- `fav transpile --check` オプション: Python 構文検証（`python3 -m py_compile`）
- テスト: `transpile_generates_valid_pyproject` 等

---

### v11.8.0 — fav transpile CLI 完成 + checker 統合
**テーマ**: 型チェック → Python 生成パイプラインの完成

- `fav transpile` 実行前に `checker.fav` で型チェックを走らせる
- 型エラーがあれば Python 生成をブロック（安全な変換のみ）
- `--legacy` なしで動作（compiler.fav パスで型情報を emit_python に渡す）
- エフェクト不整合の検出（`!AWS` なのに `import rune "aws"` がないケース等）
- `fav explain --lineage` の Python 出力版（生成コードのリネージコメント付与）
- テスト: `transpile_blocks_on_type_error` / `transpile_lineage_comment` 等

---

### v11.9.0 — fav2py E2E インフラ
**テーマ**: `infra/e2e-demo/fav2py/` の Terraform + スクリプト整備

- `infra/e2e-demo/fav2py/` ディレクトリ構造:
  ```
  fav2py/
    src/
      pipeline.fav        # Fav ネイティブパイプライン
    terraform/
      main.tf             # VPC / RDS PostgreSQL / ECS Fargate
      iam.tf
      variables.tf
      outputs.tf
    scripts/
      upload.sh           # S3 へ fav バイナリ + ソースをアップロード
      run.sh              # terraform apply → ECS タスク起動
      verify.sh           # Fav 版 vs Python 版の結果比較
    tasks.md
  ```
- RDS PostgreSQL (t3.micro, ap-northeast-1)
- ECS Fargate タスク x2（Fav ネイティブ実行 / Python トランスパイル実行）
- 同一 RDS に INSERT → SELECT → 集計を実行し結果を S3 に保存
- `verify.sh` で両者の S3 出力を比較（PASS/FAIL 判定）

---

### v12.0.0 — Python トランスパイラ完成宣言
**テーマ**: fav2py E2E PASS + ドキュメント整備

- `infra/e2e-demo/fav2py/scripts/verify.sh` — PASS=5 以上を確認
- CHANGELOG.md 更新（v11.1.0 〜 v12.0.0 全エントリ）
- README.md 更新（Python トランスパイラ機能追加）
- `site/content/docs/transpile/python.mdx` 新規作成
  - `fav transpile --target python` 使用方法
  - uv との組み合わせ
  - AWS / Postgres エフェクトの Python マッピング表
  - fav2py E2E デモへのリンク

---

## 技術アーキテクチャメモ

### emit_python の位置づけ
```
.fav ソース
    ↓ parser.rs (既存)
AST
    ↓ checker.fav (既存)
型チェック済み AST
    ↓ emit_python.rs (新規)
Python ソース (.py)
    + pyproject.toml (uv 形式)
```

### エフェクト → Python ライブラリ対応表
| Fav エフェクト | Python ライブラリ |
|---|---|
| `!IO` | 標準ライブラリ（os, sys, csv, json） |
| `!AWS` | boto3 |
| `!Postgres` | psycopg2-binary |
| `!Snowflake` | snowflake-connector-python |
| `!Http` | requests / httpx |
| `!Llm` | anthropic / openai |

### fav2py E2E シナリオ
1. `pipeline.fav` — 同一コードを Fav ネイティブと Python トランスパイルで実行
2. RDS Postgres に txn テーブル作成 → INSERT（103行）
3. 集計クエリ（region × category の合計金額）
4. 結果を S3 に JSON で保存
5. `verify.sh` — Fav 版出力 vs Python 版出力が一致することを確認
