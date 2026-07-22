# Plan: v53.9.0 — 安定化・コードフリーズ（Integration Sprint 前調整）

---

## ステップ 1: 事前確認

```bash
cargo test -j 8 -- --test-threads=8 2>&1 | grep "test result"
# → 3179 passed, 0 failed を確認

cargo clippy -- -D warnings
# → warnings なしであることを確認

# v53900_tests が未存在を確認
rg -n "v53900_tests" fav/src/driver.rs  # → 0 件

# v53800_tests の行番号を確認（挿入位置）
rg -n "v53800_tests" fav/src/driver.rs  # → 行番号を特定

# integration-overview.mdx が未存在を確認
ls site/content/docs/integration-overview.mdx 2>/dev/null  # → エラー

# Cargo.toml が 53.8.0 であることを確認
grep "^version" fav/Cargo.toml  # → version = "53.8.0"
```

---

## ステップ 2: `site/content/docs/integration-overview.mdx` 新規作成

```mdx
---
title: "Integration Sprint 概要"
description: "v51〜v53 の 3 スプリントを統合する Integration Sprint の全体像"
---

# Integration Sprint 概要

v51.0（Developer Experience 3.0）・v52.0（Performance & Scale）・v53.0（Data Quality & Observability 2.0）の
3 マイルストーンを統合し、Favnir の核となる 3 つの柱を連携させるスプリントです。

## 統合される機能

### lineage × LSP（v53.1）
`fav` の LSP サーバーがリネージ情報（upstream / downstream stage・スキーマ名）をホバー時に表示します。

### par bench（v53.2）
`fav bench` が `par` ブロック内の各 stage を個別計測し、ボトルネックを明示します。

### assert_schema 詳細診断（v53.3）
`assert_schema` 失敗時に E0419 のフィールド差分・変換 suggestion を表示します。

## E2E デモ（examples/v55-demo）

`examples/v55-demo/` に Integration Sprint の全機能を組み合わせた E2E パイプラインデモがあります。
（ディレクトリ名 `v55-demo` は v54.0「Integration Sprint 宣言」に向けた先行デモとして v53.4.0 で作成されたものです。）

```favnir
pipeline OrderIngestion {
  stage Consume: Stream<RawOrder> -> Stream<Order> = |_raw| { ... }
  stage SchemaCheck: Order -> Result<ValidOrder> = |order| {
    bind checked <- assert_schema<ValidOrder>(order)
    Result.ok(checked)
  }
  stage Process: ValidOrder -> Result<EnrichedOrder> = |valid_order| {
    par [enrich.run(valid_order), validate.run(valid_order)] |> Merge.ordered
  }
  stage Store: EnrichedOrder -> Unit = |enriched| { ... }
}
```

## 関連ドキュメント
- [用語集](/docs/glossary) — par / assert_schema / lineage / inlay hints の定義
- [cookbook: parallel-pipeline](/cookbook/parallel-pipeline) — par stage の使い方
- [cookbook: schema-validation](/cookbook/schema-validation) — assert_schema + audit-log レシピ
```

内容確認:
```bash
grep "Integration Sprint" site/content/docs/integration-overview.mdx  # → 1 件以上
grep "lineage" site/content/docs/integration-overview.mdx              # → 1 件以上
```

---

## ステップ 3: `driver.rs` — `v53900_tests` 追加

`v53800_tests` モジュールの直前（ファイル先頭側）に `v53900_tests` を追加:

```rust
// -- v53900_tests (v53.9.0) -- 安定化・コードフリーズ --
#[cfg(test)]
mod v53900_tests {
    #[test]
    fn cargo_toml_version_is_53_9_0() {
        let cargo = include_str!("../Cargo.toml");
        assert!(cargo.contains("\"53.9.0\""), "Cargo.toml must have version 53.9.0");
    }

    #[test]
    fn integration_overview_doc_exists() {
        let content = include_str!("../../site/content/docs/integration-overview.mdx");
        assert!(
            content.contains("Integration Sprint"),
            "integration-overview.mdx must mention Integration Sprint"
        );
        assert!(
            content.contains("lineage"),
            "integration-overview.mdx must reference lineage"
        );
    }
}
```

`cargo build` → コンパイルエラーなし確認。

---

## ステップ 4: `fav/Cargo.toml` バージョン更新

`version = "53.8.0"` → `version = "53.9.0"`

---

## ステップ 5: テスト実行・確認

```bash
cd /c/Users/yoshi/favnir/fav && cargo test -j 8 -- --test-threads=8 2>&1 | grep "test result"
```

期待値: 3181 passed, 0 failed

```bash
cargo clippy -- -D warnings
```

---

## ステップ 6: 後処理

- `CHANGELOG.md` に v53.9.0 エントリ追加（直前の v53.8.0 エントリと同形式）
- `versions/current.md` を v53.9.0（3181 tests）に更新
- `roadmap-v53.1-v54.0.md` の v53.9.0 実績欄を COMPLETE に更新（推定値 3175 → 実績 3181 の差異を注記）
- `tasks.md` を COMPLETE に更新（T0〜T4 全 `[x]`）
