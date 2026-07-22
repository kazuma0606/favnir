# Spec: v52.8.0 — ドキュメントサイト Data Quality 記事

Status: 計画中
Date: 2026-07-22

---

## 概要

v52.1〜v52.7 で実装した Data Quality & Observability 機能（`assert_schema`、`fav explain --lineage --with-schema / --format html`、`fav run --audit-log`、OTel span 属性）を
ドキュメントサイトに追加する。

3 つの MDX ファイルを新規作成し、`v52800_tests` で存在確認テストを追加する。

---

## 実装スコープ

### 1. `site/content/docs/data-quality/assert-schema.mdx`

`assert_schema<T>(value)` の使い方を説明する。

内容:
- 概要（スキーマ検証 primitive）
- 基本的な使用例（`type OrderRow = { id: Int, amount: Float, status: String }`）
- nullable フィールド（`field?: Type`）の説明
- `--strict-schema` フラグの説明（W036 をエラー化）
- E0419 エラーコードの説明
- エラーハンドリングパターン（`bind validated <- assert_schema<OrderRow>(row)`）

注意: `site/content/docs/data-quality/` ディレクトリは未存在のため新規作成が必要。

### 2. `site/content/docs/tools/lineage-enhanced.mdx`（テスト対象外・目視確認のみ）

注意: このファイルは `include_str!` でコンパイル時参照しない（内容が多様でキーワード固定が困難なため）。
`docs_assert_schema_page_exists` / `docs_audit_log_page_exists` の 2 テストが本バージョンのテスト追加分。
lineage-enhanced.mdx の存在・内容は tasks.md T1 の目視確認チェックリストで担保する。

### （実際の見出し）`site/content/docs/tools/lineage-enhanced.mdx`

`fav explain --lineage` の拡張オプション（`--with-schema`、`--format html`）の使い方を説明する。

内容:
- `--with-schema` オプション（スキーマ情報付き mermaid/dot）
- `--format html` オプション（インタラクティブ HTML レポート）
- `-o <file>` オプション（出力ファイル指定）
- mermaid 出力例と HTML 出力の説明

### 3. `site/content/docs/tools/audit-log.mdx`

`fav run --audit-log <output.jsonl>` の使い方と JSONL フォーマットを説明する。

内容:
- 概要（`!Kafka` / `!Snowflake` アクセスイベント記録）
- 使用例: `fav run pipeline.fav --audit-log audit.jsonl`
- JSONL フォーマット説明（ts/op/effect/topic/table フィールド）
- 既存の `fav audit`（Enterprise Governance）との違い

注意: `site/content/docs/governance/audit-log.mdx`（Audit Rune を説明するドキュメント）が既存だが、
本ファイル（`tools/audit-log.mdx`）は `fav run --audit-log` フラグの実行時ログ機能を説明するものであり
別概念・別パスであることを doc 内で明示する。

---

## テスト仕様

`v52800_tests` モジュールを `driver.rs` に追加（`v52700_tests` の直前）:

```rust
#[cfg(test)]
mod v52800_tests {
    #[test]
    fn docs_assert_schema_page_exists() {
        let src = include_str!("../../site/content/docs/data-quality/assert-schema.mdx");
        assert!(src.contains("assert_schema"));
        assert!(src.contains("nullable") || src.contains("optional"));
        assert!(src.contains("strict-schema") || src.contains("strict_schema"));
    }

    #[test]
    fn docs_audit_log_page_exists() {
        let src = include_str!("../../site/content/docs/tools/audit-log.mdx");
        assert!(src.contains("audit-log") || src.contains("audit_log"));
        assert!(src.contains("jsonl") || src.contains("JSONL"));
    }
}
```

---

## バージョン更新

- `fav/Cargo.toml`: `"52.7.0"` → `"52.8.0"`

---

## 完了条件

- `cargo test` 3153 passed, 0 failed（3151 + 2 件追加）
- `v52800_tests` 2 件 pass:
  - `docs_assert_schema_page_exists`
  - `docs_audit_log_page_exists`
- `cargo clippy -- -D warnings` クリーン

---

## 影響範囲

| ファイル | 変更種別 |
|---|---|
| `site/content/docs/data-quality/assert-schema.mdx` | 新規作成（ディレクトリも新規） |
| `site/content/docs/tools/lineage-enhanced.mdx` | 新規作成 |
| `site/content/docs/tools/audit-log.mdx` | 新規作成 |
| `fav/src/driver.rs` | `v52800_tests` 追加 |
| `fav/Cargo.toml` | version 更新 |
| `CHANGELOG.md` | v52.8.0 エントリ追加 |
| `versions/current.md` | v52.8.0 / 3153 tests に更新 |

Rust ソースコードの実装変更なし（テスト追加と MDX 作成のみ）。
