# v72.3.0 仕様 — `fav ai generate`（自然言語 → Favnir パイプライン）

Date: 2026-08-12
Status: 計画中

---

## Background

v72.2.0 で `fav ai explain` / `fav ai fix` を実装した。
v72.3.0 では自然言語の要求仕様から Favnir パイプラインの雛形コードを生成する `fav ai generate` コマンドを追加する。
実際の LLM API 呼び出しは v73.x 以降。本バージョンではキーワードベースのテンプレート生成（ルールベース）で骨格を生成する。

---

## Goals

1. `cmd_ai_generate(description: &str) -> String` を `driver.rs` に追加する
2. 説明文からスキーマフィールドを推論し `schema` ブロックを生成する
3. 説明文からインポートすべき Rune（csv / postgres / s3 等）を推論する
4. 生成された Favnir コードを文字列として返す
5. `fav ai generate <description>` CLI コマンドを `main.rs` に追加する
6. テスト 2 件を `v723000_tests` モジュールとして `driver.rs` に追加する

---

## CLI 例

```bash
$ fav ai generate "S3のCSVを読んでスキーマ検証しPostgresに挿入するパイプライン"
Generating pipeline...

# Generated: pipeline.fav
import rune "csv"
import rune "postgres"

schema OrderRow {
    order_id: String
    amount:   Float
    status:   String
}

fn main(ctx: AppCtx) -> Result<Unit, String> {
    bind raw   <- ctx.io.read_file_raw("s3://bucket/data.csv")
    bind rows  <- Csv.parse_typed(raw, OrderRow)
    bind valid <- Schema.validate_all(rows)
    bind _     <- Postgres.execute_raw("INSERT INTO orders ...", valid)
    ctx.io.println("Done.")
}
```

---

## 実装詳細

### `cmd_ai_generate(description: &str) -> String`

- 説明文に含まれるキーワードで Rune を推論:
  - "csv" / "CSV" → `import rune "csv"`
  - "postgres" / "Postgres" / "PostgreSQL" → `import rune "postgres"`
  - "s3" / "S3" → `import rune "s3"`
  - "json" / "JSON" → `import rune "json"`
- スキーマ推論（`infer_schema_from_description`）:
  - "注文" / "order" → `order_id: String`, `amount: Float`, `status: String`
  - 上記に該当しない場合 → `id: String`, `value: String`
- `schema` ブロック + `fn main(ctx: AppCtx)` の雛形を文字列結合で生成
- 生成コードを返す（ファイル書き込みは行わない）

### `infer_schema_from_description(description: &str) -> Vec<(&'static str, &'static str)>`

- 戻り値: `(field_name, field_type)` のベクタ
- 比較前に `description.to_lowercase()` で正規化する（大文字小文字非依存）
- "order" / "注文" を含む → `[("order_id", "String"), ("amount", "Float"), ("status", "String")]`
- それ以外 → `[("id", "String"), ("value", "String")]`

### `main.rs` 変更

- `Some("ai")` アームの `"generate"` ブランチを追加:
  - `args[3..]` を結合してスペース区切りの description 文字列を構築
  - `driver::cmd_ai_generate(&description)` を呼び出し結果を `println!`

---

## 成功条件

- `ai_generate_returns_valid_fav_code`: `cmd_ai_generate("csv pipeline")` の結果が `"fn main"` と `"ctx: AppCtx"` を含む
- `ai_generate_schema_inferred_from_description`: `cmd_ai_generate("注文データのETL")` の結果が `"order_id"` を含む
- `cargo test v723000` で 2 件 pass
- `cargo test` 全体で 3618 tests pass（v72.2.0 完了時点 3616 + 2）

**WASM への影響**: なし（native-only crate を追加しない）。

---

## エラーコード

新規エラーコードなし（テンプレート生成の失敗は panic しない — デフォルトテンプレートにフォールバック）。

---

## 変更対象ファイル

| ファイル | 変更内容 |
|---|---|
| `fav/src/driver.rs` | `infer_schema_from_description` / `cmd_ai_generate` / `v723000_tests` 追加 |
| `fav/src/main.rs` | `"ai"` アームに `"generate"` ブランチ追加 |
| `fav/Cargo.toml` | `version = "72.2.0"` → `"72.3.0"` |
| `CHANGELOG.md` | `## [v72.3.0]` エントリ追加 |
| `versions/current.md` | 進行中バージョンを v72.3.0 に更新 |

---

## スコープ外（明示的除外）

- 実際の Claude / OpenAI API HTTP 呼び出し（v73.x 以降）
- 生成コードのファイルへの書き込み・エディタ起動（v72.4.0 以降）
- `fav check` による生成コードの自動検証（v72.4.0 以降 — サブプロセス実行が必要なため）
- CSV / Postgres 以外のすべてのスキーマパターン（段階的追加）
- `site/content/docs/cli/ai.mdx` への `fav ai generate` 追記（v72.4.0 以降）
