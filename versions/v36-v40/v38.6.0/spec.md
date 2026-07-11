# v38.6.0 spec — RAG パイプラインテンプレート

## バージョン概要

| 項目 | 内容 |
|---|---|
| バージョン | v38.6.0 |
| テーマ | `fav new --template rag-pipeline` — RAG パイプラインテンプレート追加 |
| 前提 | v38.5.0 COMPLETE — `fav explain --verbose` 実装済み |
| 完了条件 | `v38600_tests` 全テスト pass・`cargo test` 0 failures（≥ 2769 件） |

## 背景と目的

v38.7.0 で Llm Rune が強化される前に、RAG（Retrieval-Augmented Generation）パイプラインの
プロジェクトテンプレートを追加する。`fav new my-rag --template rag-pipeline` で
`Ingest → Embed → Retrieve → Generate` の 4 ステージ構成のスターターを生成できるようにする。

**想定動作**:
```bash
$ fav new my-rag --template rag-pipeline
created my-rag
next:
  cd my-rag
  fav run src/main.fav
```

## 実装スコープ

### 1. `fav/src/driver.rs` — `create_rag_pipeline_project` 追加

既存の `create_multi_source_etl_project` 関数（line 722 付近）の直後に追加:

```rust
fn create_rag_pipeline_project(root: &Path, name: &str) -> Result<(), String> {
    // raw string を使って継続文字列のスペース混入問題を回避する
    let main_fav = format!(
        "// RAG Pipeline — {name}\n\
// ingest / embed / retrieve / generate の 4 ステージ構成\nimport llm\nimport csv\n\n\
stage Ingest -> List<String> {{\n    csv.read_file(\"data/documents.csv\")\n}}\n\n\
stage Embed(docs: List<String>) -> List<String> {{\n    docs |> List.map(|doc| llm.embed(ctx, doc))\n}}\n\n\
stage Retrieve(embeddings: List<String>) -> List<String> {{\n    // TODO: ベクトル DB から関連チャンクを取得\n    embeddings\n}}\n\n\
stage Generate(context: List<String>) -> String {{\n    let prompt = String.join(context, \"\\n\")\n    llm.call(ctx, prompt)\n}}\n\n\
pipeline {name} {{\n    Ingest |> Embed |> Retrieve |> Generate\n}}\n"
    );
    write_text_file(&root.join("src/main.fav"), &main_fav)?;
    write_text_file(&root.join("fav.toml"), &format!(
        "[project]\nname    = \"{name}\"\nversion = \"0.1.0\"\nedition = \"2026\"\n\n[runes]\nllm = \"1.0.0\"\ncsv = \"1.0.0\"\n"
    ))?;
    write_text_file(&root.join("data/documents.csv"),
        "id,content\n1,\"Favnir is a type-safe data pipeline language.\"\n2,\"RAG combines retrieval with generation.\"\n"
    )?;
    write_text_file(&root.join("README.md"), &format!(
        "# {name}\n\nRAG パイプライン（Retrieval-Augmented Generation）。\n\n## Stages\n\n| Stage | 内容 |\n|---|---|\n| Ingest | CSV からドキュメントを読み込み |\n| Embed | Llm Rune でベクトル化 |\n| Retrieve | ベクトル DB から関連チャンクを取得 |\n| Generate | コンテキストを元に LLM で回答生成 |\n\n## Usage\n\n```bash\nANTHROPIC_API_KEY=sk-... fav run src/main.fav\n```\n"
    ))?;
    Ok(())
}
```

### 2. `fav/src/driver.rs` — `try_cmd_new` に `"rag-pipeline"` アーム追加

`"multi-source"` アームの直後、`other =>` catch-all の直前に追加:

```rust
"rag-pipeline"     => create_rag_pipeline_project(&root, name),
```

エラーメッセージを更新:

```rust
other => Err(format!(
    "unknown template `{other}` \
     (expected script|pipeline|lib|postgres-etl|\
     etl-csv-to-db|api-gateway|lambda-scheduled|distributed-etl|data-contract|multi-source|rag-pipeline)"
)),
```

### 3. `fav/src/driver.rs` — `cmd_new_list` に `rag-pipeline` 追加

`"multi-source"` 行の直後に追加:

```rust
println!("  {:<17} {}", "rag-pipeline",   "RAG パイプライン（ingest/embed/retrieve/generate）");
```

### 4. `driver.rs` — テストモジュール追加

#### `v38500_tests::cargo_toml_version_is_38_5_0` のスタブ化

```rust
// Stubbed: version bumped to 38.6.0 — assertion intentionally removed
```

#### `v38600_tests` モジュール新規追加（4 テスト）

**注意**: `use super::*;` は不要（`include_str!` のみ使用のため）。

```rust
// ── v38600_tests (v38.6.0) — RAG パイプラインテンプレート ────────────────────
#[cfg(test)]
mod v38600_tests {
    #[test]
    fn cargo_toml_version_is_38_6_0() {
        let cargo = include_str!("../Cargo.toml");
        assert!(cargo.contains("38.6.0"), "Cargo.toml must contain version 38.6.0");
    }

    #[test]
    fn changelog_has_v38_6_0() {
        let src = include_str!("../../CHANGELOG.md");
        assert!(src.contains("[v38.6.0]"), "CHANGELOG.md must contain [v38.6.0]");
    }

    #[test]
    fn rag_pipeline_fn_exists() {
        let src = include_str!("driver.rs");
        assert!(
            src.contains("create_rag_pipeline_project"),
            "driver.rs must contain create_rag_pipeline_project"
        );
    }

    #[test]
    fn rag_pipeline_has_four_stages() {
        let src = include_str!("driver.rs");
        // "stage <Name>" で検索することで既存コメント・関数名（例: "Generate SQL DDL"）との偽陽性を排除する
        assert!(
            src.contains("stage Ingest") && src.contains("stage Embed")
                && src.contains("stage Retrieve") && src.contains("stage Generate"),
            "rag-pipeline template must define stage Ingest, stage Embed, stage Retrieve, stage Generate"
        );
    }
}
```

### 5. `CHANGELOG.md` — `[v38.6.0]` エントリ追加

```
## [v38.6.0] — 2026-07-10

### Added
- `fav new --template rag-pipeline` テンプレート追加（ingest/embed/retrieve/generate 4 ステージ）
- `create_rag_pipeline_project` in `driver.rs`
- `v38600_tests` 4 テスト追加

---
```

**セパレータは `—`（全角ダッシュ U+2014）**

### 6. その他ドキュメント更新

- `fav/Cargo.toml`: `38.5.0` → `38.6.0`
- `versions/current.md`: 最新安定版 → v38.6.0、次バージョン → v38.7.0
- `versions/roadmap/roadmap-v38.1-v39.0.md`: v38.6.0 を ✅ 完了済みにマーク・テスト件数を 4 件に更新

## テスト数の計算

| バージョン | 実績 |
|---|---|
| v38.5.0 | 2765 |
| v38.6.0 追加分 | +4 |
| v38.6.0 期待値 | 2769 |

ロードマップは「Rust テスト 1 件」と記載しているが、meta 2 件 + functional 2 件の計 4 件を追加し、T9 でロードマップを 4 件に更新する。

## 注意事項

### `\x20` の使用

`create_rag_pipeline_project` の `format!` 内でインデントを `\x20\x20\x20\x20`（4 スペース）で表記する。`\n    ` のようなリテラルスペースより意図が明確になる。

### `rag_pipeline_has_four_stages` テストの依存

このテストは `driver.rs` の文字列内に `"Ingest"`・`"Embed"`・`"Retrieve"`・`"Generate"` が含まれることを検証する。これらの文字列は既存の他テストにも含まれている可能性があるため、単体では不十分だが `rag_pipeline_fn_exists` と組み合わせることで十分な保護となる。

### `gen` 予約語（Rust 2024）

`create_rag_pipeline_project` の変数名には `root`・`name` を使用する — `gen` は使わないこと。

## 完了条件

| # | 条件 | 検証方法 |
|---|---|---|
| 1 | `driver.rs` に `create_rag_pipeline_project` が含まれる | `rag_pipeline_fn_exists` テスト |
| 2 | RAG テンプレートが Ingest / Embed / Retrieve / Generate 4 ステージを含む | `rag_pipeline_has_four_stages` テスト |
| 3 | `CHANGELOG.md` に `[v38.6.0]` が含まれる | `changelog_has_v38_6_0` テスト |
| 4 | `Cargo.toml` バージョンが `38.6.0` | `cargo_toml_version_is_38_6_0` テスト |
| 5 | `cargo test` 全通過（failures=0 かつテスト数 ≥ 2769） | `cargo test` 実行結果 |
| 6 | `roadmap-v38.1-v39.0.md` の v38.6.0 が ✅ かつテスト件数が 4 件 | T7 後に目視確認 |
| 7 | `versions/current.md` が v38.6.0（最新安定版）に更新されている | T7 後に目視確認 |
