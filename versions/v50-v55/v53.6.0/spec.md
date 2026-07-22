# Spec: v53.6.0 — cookbook 更新（parallel-pipeline + schema-validation）

Status: 計画中
Date: 2026-07-22

---

## 概要

v51〜v53 で実装した `par` 並列 stage と `assert_schema` を
サイトの cookbook ページとして文書化する。

- `site/content/cookbook/parallel-pipeline.mdx` — 既存ファイル（v24.7.0 で作成）。v53 era の `par [A, B] |> Merge.ordered` 構文に対応したコンテンツが既に含まれている。テストで存在確認と内容確認のみ行う。
- `site/content/cookbook/schema-validation.mdx` — 新規作成。`assert_schema` + nullable + OTel span + `--audit-log` のレシピ。

---

## 実装スコープ

### 1. `site/content/cookbook/schema-validation.mdx` 新規作成

```mdx
---
title: "assert_schema でスキーマ検証"
description: "assert_schema<T> を使って実行時にフィールド型を検証し、OTel span と audit-log に記録するレシピ"
---

# assert_schema でスキーマ検証

`assert_schema<T>` は実行時にマップのフィールドが型 `T` に一致するか検証します。
失敗時は E0419 エラーを発生させ、`fav run --audit-log` 実行時はアクセスログに記録されます。

## コード例

```favnir
type OrderRow = { id: Int, amount: Float, status: String }

stage ValidateSchema: Map -> Result<OrderRow> = |row| {
  bind checked <- assert_schema<OrderRow>(row)
  // OTel span に schema.name = "OrderRow" が自動付与
  Ok(checked)
}
```

## nullable フィールド

フィールドが省略可能な場合は `?` を付与します。

```favnir
type NullableRow = { id: Int, amount: Float, note: String? }
```

## audit-log との統合

`fav run --audit-log ./audit.log` で実行すると、`assert_schema` の成否が監査ログに記録されます。

```bash
fav run pipeline.fav --audit-log ./audit.log
```

## OTel span

`fav run` に OTel エクスポーターが設定されている場合、各 `assert_schema` 呼び出しに
`schema.name` / `schema.result` アトリビュートが付与されます。

## エラー確認

```bash
fav explain --error E0419
# フィールド差分と型変換ヒントを表示
```
```

### 2. `site/content/cookbook/parallel-pipeline.mdx` — 確認のみ

既存ファイルの存在確認と `par [` を含むことを assert するテストを追加する。
ファイル内容の変更は不要（既存コンテンツで条件を満たす）。

---

### 3. テスト仕様

`v53600_tests` モジュールを `driver.rs` に追加（`v53500_tests` の直前）:

```rust
// -- v53600_tests (v53.6.0) -- cookbook 更新 --
#[cfg(test)]
mod v53600_tests {
    #[test]
    fn cookbook_parallel_pipeline_exists() {
        let content = include_str!("../../site/content/cookbook/parallel-pipeline.mdx");
        assert!(
            content.contains("par [") || content.contains("par [A"),
            "parallel-pipeline.mdx must contain par syntax example"
        );
        assert!(
            content.contains("Merge") || content.contains("merge"),
            "parallel-pipeline.mdx must mention merge behavior"
        );
    }

    #[test]
    fn cookbook_schema_validation_exists() {
        let content = include_str!("../../site/content/cookbook/schema-validation.mdx");
        assert!(
            content.contains("assert_schema"),
            "schema-validation.mdx must contain assert_schema"
        );
        assert!(
            content.contains("--audit-log"),
            "schema-validation.mdx must mention --audit-log"
        );
    }
}
```

---

## バージョン更新

- `fav/Cargo.toml`: `"53.5.0"` → `"53.6.0"`

---

## 完了条件

- `cargo test` 3175 passed, 0 failed（ベース 3173 + 2 件追加）
  - 注: ロードマップ推定値 3169 との差 +6 = 累積差 +4（v53.1.0 コードレビュー起因）+ 今回追加 +2
- `v53600_tests` 2 件 pass:
  - `cookbook_parallel_pipeline_exists`
  - `cookbook_schema_validation_exists`
- `cargo clippy -- -D warnings` クリーン
- `site/content/cookbook/schema-validation.mdx` に `assert_schema` / `--audit-log` が含まれる

---

## 影響範囲

| ファイル | 変更種別 |
|---|---|
| `site/content/cookbook/schema-validation.mdx` | 新規作成 |
| `site/content/cookbook/parallel-pipeline.mdx` | 変更なし（既存） |
| `fav/src/driver.rs` | `v53600_tests` 追加 |
| `fav/Cargo.toml` | version 更新 |
| `CHANGELOG.md` | v53.6.0 エントリ追加 |
| `versions/current.md` | v53.6.0 / 3175 tests に更新 |
| `versions/roadmap/roadmap-v53.1-v54.0.md` | v53.6.0 実績欄を COMPLETE に更新・推定値修正 |

---

## 設計上の注意

- `parallel-pipeline.mdx` は v24.7.0 で既に作成済みで内容も `par` / `Merge` を含む。今回は新規テストを追加するだけで既存ファイルを変更しない。
- `include_str!("../../site/content/cookbook/...")` は `fav/src/driver.rs` から `../../` で `favnir/` ルートに上がる。既存テスト（`etl-csv-to-db.mdx` 等）と同パターンで正しい。
- `cookbook_parallel_pipeline_exists` の assert は OR 条件（`"par ["` OR `"par [A"`）で既存コンテンツのどちらの表記も許容する。既存ファイルには `"par [FetchUsers"` が含まれるため通過する。
- v53500_tests にバージョンピンテストは存在しないため、バージョン更新時の空化対象なし。
