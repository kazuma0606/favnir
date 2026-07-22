# Spec: v52.9.0 — 安定化・コードフリーズ（Data Quality 2.0 前調整）

Status: 計画中
Date: 2026-07-22

---

## 概要

v52.1〜v52.8 で実装した Data Quality & Observability 機能群の安定化バージョン。
全 lint / clippy クリーンを確認し、v53.0 宣言前の最終調整として
`site/content/docs/data-quality-overview.mdx` の骨子を作成する。

---

## 実装スコープ

### 1. `site/content/docs/data-quality-overview.mdx`（新規作成）

v52.x で追加した Data Quality & Observability 2.0 機能群の概要ドキュメント。

内容:
- Data Quality & Observability 2.0 とは何か（概要）
- v52.1〜v52.8 で追加された機能一覧:
  - `assert_schema<T>` — 実行時スキーマ検証（v52.1〜v52.2）
  - `fav explain --lineage --with-schema` — スキーマ付きリネージ表示（v52.3）
  - `fav explain --lineage --format html` — HTML リネージレポート（v52.4）
  - SLA 監視 Rune（v52.5）
  - `fav run --audit-log` — データアクセスログ（v52.6）
  - OTel span 属性強化（schema.name / schema.fields / lineage.upstream / lineage.downstream）（v52.7）
- 各機能へのリンク（docs/data-quality/assert-schema・docs/tools/lineage-enhanced・docs/tools/audit-log）
- 「Data Quality & Observability 2.0」というキーワードを含む

注意: 既存の `site/content/docs/data-quality.mdx`（v36.x 向け Data Quality First）とは別ファイル。

### 2. lint / clippy クリーン確認

`cargo clippy -- -D warnings` が 0 エラー・0 警告であることを確認する。
Rust ソースコードへの変更は原則行わない（clippy 指摘があれば最小限の修正のみ）。

---

## テスト仕様

`v52900_tests` モジュールを `driver.rs` に追加（`v52800_tests` の直前）:

```rust
#[cfg(test)]
mod v52900_tests {
    #[test]
    fn cargo_toml_version_is_52_9_0() {
        let src = include_str!("../Cargo.toml");
        assert!(src.contains("version = \"52.9.0\""), "Cargo.toml must be version 52.9.0");
    }

    #[test]
    fn dq_overview_doc_exists() {
        let src = include_str!("../../site/content/docs/data-quality-overview.mdx");
        assert!(
            src.contains("Data Quality") && src.contains("Observability"),
            "data-quality-overview.mdx must mention Data Quality and Observability"
        );
        assert!(
            src.contains("assert_schema"),
            "data-quality-overview.mdx must mention assert_schema"
        );
        assert!(
            src.contains("audit-log") || src.contains("audit_log"),
            "data-quality-overview.mdx must mention audit-log"
        );
    }
}
```

`include_str!` パス（`fav/src/driver.rs` 起点）:
- `"../Cargo.toml"` → `fav/Cargo.toml` ✓
- `"../../site/content/docs/data-quality-overview.mdx"` → `favnir/site/content/docs/data-quality-overview.mdx` ✓

---

## バージョン更新

- `fav/Cargo.toml`: `"52.8.0"` → `"52.9.0"`

---

## 完了条件

- `cargo clippy -- -D warnings` クリーン（0 エラー・0 警告）
- `cargo test` 3156 passed, 0 failed（3154 + 2 件追加）
- `v52900_tests` 2 件 pass:
  - `cargo_toml_version_is_52_9_0`
  - `dq_overview_doc_exists`

---

## 影響範囲

| ファイル | 変更種別 |
|---|---|
| `site/content/docs/data-quality-overview.mdx` | 新規作成 |
| `fav/src/driver.rs` | `v52900_tests` 追加 |
| `fav/Cargo.toml` | version 更新 |
| `CHANGELOG.md` | v52.9.0 エントリ追加 |
| `versions/current.md` | v52.9.0 / 3156 tests に更新 |

Rust ソースコードの実装変更なし（clippy クリーンに問題なければ）。
