# Favnir v12.0.0 実装計画

作成日: 2026-06-06

---

## 実装順序

```
Phase A: CHANGELOG.md 更新（v11.1.0〜v12.0.0 全エントリ）
    ↓
Phase B: README.md 更新（Python トランスパイラ機能追記）
    ↓
Phase C: site/content/docs/transpile/python.mdx 新規作成
    ↓
Phase D: Rust テスト追加（v12000_tests）
    ↓
Phase E: バージョン更新・コミット
```

---

## Phase A — CHANGELOG.md 更新

### 追記内容（新しいエントリ順）

```markdown
## [v12.0.0] — 2026-06-06

### Added
- `site/content/docs/transpile/python.mdx` — Python トランスパイラ公式ドキュメント
- Python トランスパイラ完成宣言（v11.1.0〜v11.9.0 の全機能が揃った）

### Changed
- README.md に `fav transpile --target python` を追記
- CHANGELOG に v11.1.0〜v11.9.0 の全履歴を追記

### Notes
- テスト: 1290 件以上


## [v11.9.0] — 2026-06-06

### Added
- `infra/e2e-demo/fav2py/` — Fav ネイティブ vs Python トランスパイル E2E インフラ
  - `src/pipeline.fav` — LoadAndInsert |> Aggregate |> SaveResult
  - `src/sample.csv` — 103 行サンプルデータ（region × category × amount）
  - `terraform/` — VPC / RDS PostgreSQL (t3.micro) / ECS Fargate x2 / ECR
  - `scripts/upload.sh` / `run.sh` / `verify.sh`
  - `Dockerfile` — Ubuntu 22.04 + uv + psycopg2-binary + fav binary
- `driver.rs` `v11900_tests` — `fav2py_e2e_demo_structure` / `fav2py_pipeline_fav_transpiles`

### Notes
- テスト: 1290 件


## [v11.8.0] — 2026-06-06

### Added
- `fav transpile --no-check` オプション（型チェックスキップ）
- `fav transpile --lineage` オプション（生成コードに lineage コメント付与）
- `emit_python.rs` `emit_python_with_lineage` — `HashMap<String,String>` ベースの lineage コメント注入
- `driver.rs` `build_lineage_comments` / `check_source_str_pub`
- `driver.rs` `v11800_tests` — 6 件（checker 統合 / lineage コメント検証）

### Changed
- `fav transpile` 実行前に `checker.fav` で型チェックを走らせる（型エラーで Python 生成をブロック）

### Notes
- テスト: 1288 件


## [v11.7.0] — 2026-06-06

### Added
- `fav transpile --out-dir <dir>` — `main.py` + `pyproject.toml` + `README.md` を出力ディレクトリに生成
- `fav transpile --check` — `python -m py_compile` による構文検証
- `fav transpile --run` — `uv run main.py` まで一括実行
- `driver.rs` `build_pyproject_content` / `build_readme_content`
- `driver.rs` `v11700_tests` — 6 件（pyproject 生成 / README 生成 / uv フラグ）

### Notes
- テスト: 1282 件


## [v11.6.0] — 2026-06-06

### Added
- `emit_python.rs` `!Postgres` → psycopg2 変換
  - `_pg_connect()` / `_pg_execute()` / `_pg_query()` ヘルパー生成
  - `Postgres.execute_raw` → `_pg_execute(sql, params)`
  - `Postgres.query_raw` → `_pg_query(sql, params)`
- `pyproject.toml` に `psycopg2-binary>=2.9` 依存を自動追加（`import psycopg2` 検出時）
- `driver.rs` `v11600_tests` — 6 件

### Notes
- テスト: 1276 件


## [v11.5.0] — 2026-06-06

### Added
- `Effect::Postgres` 追加（ast.rs / parser.rs / fmt.rs / lineage.rs / driver.rs /
  ast_lower_checker.rs / checker.rs / reachability.rs）
- `vm.rs` `Postgres.execute_raw` / `Postgres.query_raw` / `Postgres.query_typed_raw`（tokio-postgres ベース）
- `toml.rs` `PostgresTomlConfig` — `fav.toml` `[postgres]` セクション解析
- `runes/postgres/postgres.fav` — `execute` / `query<T>` Rune 実装
- `checker.fav` `postgres_fn` / `builtin_ret_ty` / `ns_to_effect` に Postgres 追加
- `driver.rs` `v11500_tests` — 6 件

### Notes
- テスト: 1270 件


## [v11.4.0] — 2026-06-06

### Added
- `emit_python.rs` `!AWS` → boto3 変換
  - `AWS.s3_put_object_raw` → `boto3.client("s3").put_object(...)`
  - `AWS.s3_get_object_raw` → `boto3.client("s3").get_object(...)`
- `pyproject.toml` に `boto3>=1.34` 依存を自動追加（`import boto3` 検出時）
- `driver.rs` `v11400_tests` — 4 件

### Notes
- テスト: 1264 件


## [v11.3.0] — 2026-06-06

### Added
- `emit_python.rs` `!IO` → Python 標準 I/O 変換
  - `IO.println` → `print`
  - `IO.read_file_raw` → `open(path).read()`（try/except）
  - `Csv.parse_raw` → `csv.DictReader`
  - `Schema.adapt` → dataclass 変換ヘルパー生成
  - `Schema.to_json_array` → `json.dumps([asdict(r) for r in rows])`
- `driver.rs` `v11300_tests` — 4 件

### Notes
- テスト: 1260 件


## [v11.2.0] — 2026-06-06

### Added
- `emit_python.rs` `stage` / `seq` → Python パイプライン変換
  - `stage Foo: A -> B = |x| { ... }` → `def foo(x: A) -> B: ...`
  - `seq Pipeline = A |> B |> C` → `def pipeline(x): return c(b(a(x)))`
- `fn main()` → `if __name__ == "__main__": main()`
- `IO.argv()` → `sys.argv[1:]`
- `List.*` stdlib → Python リスト内包表記 / `filter` / `map`
- `driver.rs` `v11200_tests` — 4 件

### Notes
- テスト: 1256 件


## [v11.1.0] — 2026-06-06

### Added
- `src/emit_python.rs` 新規作成 — AST → Python コード生成基盤
  - 型定義 → `@dataclass` 変換
  - 基本式 → Python 式（Int / Float / String / Bool / List / if-else）
  - `fn` → Python `def`（引数・戻り型コメント付き）
  - `bind x <- expr` → `x = expr`（モナド脱糖）
  - `match` → `if/elif/else`（Option/Result パターン）
- `fav transpile --target python <file>` CLI エントリ（`cli.fav` + `driver.rs`）
- `driver.rs` `v11100_tests` — 4 件

### Notes
- テスト: 1252 件
```

---

## Phase B — README.md 更新

「主要機能」または「Features」テーブルに以下の行を追加:

```markdown
| `fav transpile --target python` | Fav → Python + `pyproject.toml` 自動生成（boto3 / psycopg2 対応） |
```

---

## Phase C — site/content/docs/transpile/python.mdx

新規ファイル。`site/content/docs/transpile/` ディレクトリを作成して配置。

### ページ構成

1. **概要** — Fav → Python トランスパイラとは何か
2. **インストール** — `fav` バイナリ + `uv` が必要
3. **基本的な使い方**
   ```bash
   fav transpile --target python pipeline.fav
   ```
4. **出力ディレクトリモード**
   ```bash
   fav transpile --target python pipeline.fav --out-dir ./out
   ```
5. **構文チェック / 実行**
   ```bash
   fav transpile --target python pipeline.fav --check
   fav transpile --target python pipeline.fav --out-dir ./out --run
   ```
6. **エフェクト → Python ライブラリ対応表**
7. **`!Postgres` 変換例** — Fav コード → 生成 Python コード対比
8. **`!AWS` 変換例** — S3 put/get の変換例
9. **lineage コメント** — `--lineage` オプション
10. **fav2py E2E デモ** — `infra/e2e-demo/fav2py/` へのリンク

---

## Phase D — Rust テスト（v12000_tests）

```rust
#[cfg(test)]
mod v12000_tests {
    #[test]
    fn version_is_12_0_0() {
        assert_eq!(env!("CARGO_PKG_VERSION"), "12.0.0");
    }

    #[test]
    fn python_mdx_doc_exists() {
        let path = std::path::Path::new("../site/content/docs/transpile/python.mdx");
        assert!(path.exists(), "python.mdx not found");
    }
}
```

---

## Phase E — バージョン更新・コミット

- `fav/Cargo.toml`: `version = "12.0.0"`
- `cargo build` で `Cargo.lock` 更新
- `git commit & push` — CI 確認
