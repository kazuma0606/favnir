# Favnir v12.0.0 仕様書

作成日: 2026-06-06
テーマ: Python トランスパイラ完成宣言

---

## 背景と目的

v11.1.0〜v11.9.0 で Python トランスパイラ (`fav transpile --target python`) の全機能が実装された。
v12.0.0 では以下を行い、**Python トランスパイラの完成を正式宣言**する。

1. CHANGELOG.md に v11.1.0〜v12.0.0 の全履歴を記録
2. README.md に Python トランスパイラ機能を追記
3. `site/content/docs/transpile/python.mdx` を新規作成（公式ドキュメント）
4. Rust テスト（バージョン確認）
5. `fav/Cargo.toml` を `12.0.0` にバージョンアップ

---

## 完成定義（v12.0.0 時点の機能一覧）

| 機能 | 実装バージョン | 状態 |
|---|---|---|
| `emit_python.rs` — AST → Python 変換基盤 | v11.1.0 | COMPLETE |
| `stage` / `seq` → Python パイプライン変換 | v11.2.0 | COMPLETE |
| `!IO` → Python 標準 I/O 変換 | v11.3.0 | COMPLETE |
| `!AWS` → boto3 変換 | v11.4.0 | COMPLETE |
| `Effect::Postgres` + Postgres Rune | v11.5.0 | COMPLETE |
| `!Postgres` → psycopg2 変換 | v11.6.0 | COMPLETE |
| `--out-dir` / `--check` / `--run` (uv 統合) | v11.7.0 | COMPLETE |
| checker.fav 統合 + lineage コメント | v11.8.0 | COMPLETE |
| fav2py E2E インフラ (`infra/e2e-demo/fav2py/`) | v11.9.0 | COMPLETE |

---

## ドキュメント対象ファイル

### CHANGELOG.md

以下のエントリを追記:

```
## [v12.0.0] — 2026-06-06
## [v11.9.0] — 2026-06-06
## [v11.8.0] — 2026-06-06
## [v11.7.0] — 2026-06-06
## [v11.6.0] — 2026-06-06
## [v11.5.0] — 2026-06-06
## [v11.4.0] — 2026-06-06
## [v11.3.0] — 2026-06-06
## [v11.2.0] — 2026-06-06
## [v11.1.0] — 2026-06-06
```

### README.md

「主要機能」セクションに Python トランスパイラ行を追加:

```markdown
| `fav transpile --target python` | Fav → Python + pyproject.toml (uv 対応) |
```

### site/content/docs/transpile/python.mdx

新規ページ。以下の構成:

1. 概要（Fav → Python トランスパイラとは）
2. インストール / 事前条件
3. 基本的な使い方 (`fav transpile --target python`)
4. `--out-dir` / `--check` / `--run` オプション
5. エフェクト → Python ライブラリ対応表
6. `!Postgres` → psycopg2 変換例
7. `!AWS` → boto3 変換例
8. lineage コメント (`--lineage` オプション)
9. fav2py E2E デモへのリンク

---

## エフェクト → Python ライブラリ対応表

| Fav エフェクト | Python ライブラリ | pyproject.toml 依存 |
|---|---|---|
| `!IO` | 標準ライブラリ（os, sys, csv, json） | なし |
| `!AWS` | boto3 | `boto3>=1.34` |
| `!Postgres` | psycopg2-binary | `psycopg2-binary>=2.9` |
| `!Snowflake` | snowflake-connector-python | （将来対応） |
| `!Http` | requests / httpx | （将来対応） |
| `!Llm` | anthropic / openai | （将来対応） |

---

## Rust テスト設計（v12000_tests）

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

## バージョン更新

- `fav/Cargo.toml`: `version = "12.0.0"`
