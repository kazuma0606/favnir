# Plan: v52.8.0 — ドキュメントサイト Data Quality 記事

---

## ステップ 1: MDX ファイル作成

### 1-1. `site/content/docs/data-quality/assert-schema.mdx`

`site/content/docs/data-quality/` ディレクトリは未存在のため、ファイル作成と同時に作られる。

内容構成:
- title: "assert_schema — スキーマ検証"
- 概要セクション
- 基本例（`type OrderRow = { ... }` + `bind validated <- assert_schema<OrderRow>(row)`）
- nullable フィールド（`field?: Type`）説明
- `--strict-schema` フラグと W036 警告
- E0419 エラーコード説明

### 1-2. `site/content/docs/tools/lineage-enhanced.mdx`

`site/content/docs/tools/` は既存ディレクトリ。

内容構成:
- title: "fav explain --lineage 拡張オプション"
- `--with-schema` オプション（mermaid/dot にスキーマ情報付加）
- `--format html` オプション（インタラクティブ HTML レポート生成）
- `-o <file>` 出力ファイル指定
- 各オプションのコード例・出力例

### 1-3. `site/content/docs/tools/audit-log.mdx`

内容構成:
- title: "fav run --audit-log — データアクセスログ"
- 概要（`!Kafka` / `!Snowflake` アクセスイベント記録）
- 使用例と JSONL 出力例
- フィールド説明（ts / op / effect / topic / table）
- `fav audit`（Enterprise Governance）との違い

---

## ステップ 2: `fav/src/driver.rs` — `v52800_tests` 追加

`v52700_tests` モジュールの直前に `v52800_tests` を追加:

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

`include_str!` パス（`driver.rs` 起点）:
- `"../../site/content/docs/data-quality/assert-schema.mdx"` → `fav/src/` から2階層上 = `favnir/` → `site/...` ✓
- `"../../site/content/docs/tools/audit-log.mdx"` → 同様 ✓

---

## ステップ 3: `fav/Cargo.toml` バージョン更新

`version = "52.7.0"` → `version = "52.8.0"`

---

## ステップ 4: テスト実行・確認

```bash
cd /c/Users/yoshi/favnir/fav && cargo test -j 8 -- --test-threads=8 2>&1 | tail -20
```

期待値: 3153 passed, 0 failed

---

## ステップ 5: 後処理

- `CHANGELOG.md` に v52.8.0 エントリ追加
- `versions/current.md` を v52.8.0（3153 tests）に更新
- `roadmap-v52.1-v53.0.md` の v52.8.0 実績欄を更新（未実施 → COMPLETE）
- `tasks.md` を COMPLETE に更新（T0〜T4 全 `[x]`）
