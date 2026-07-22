# Plan: v53.6.0 — cookbook 更新（parallel-pipeline + schema-validation）

---

## ステップ 1: 事前確認

```bash
cargo test -j 8 -- --test-threads=8 2>&1 | grep "test result"
# → 3173 passed, 0 failed を確認

# v53600_tests が未存在を確認
rg -n "v53600_tests" fav/src/driver.rs  # → 0 件

# v53500_tests の行番号を確認（挿入位置）
rg -n "v53500_tests" fav/src/driver.rs  # → 行番号を特定

# schema-validation.mdx が未存在を確認
ls site/content/cookbook/schema-validation.mdx 2>/dev/null  # → エラー

# parallel-pipeline.mdx が存在し par と Merge を含むことを確認
grep -c "par\|Merge" site/content/cookbook/parallel-pipeline.mdx  # → 1 件以上

# Cargo.toml が 53.5.0 であることを確認
grep "^version" fav/Cargo.toml  # → version = "53.5.0"
```

---

## ステップ 2: `site/content/cookbook/schema-validation.mdx` 新規作成

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

内容確認:
```bash
grep "assert_schema" site/content/cookbook/schema-validation.mdx  # → 1 件以上
grep "\-\-audit-log" site/content/cookbook/schema-validation.mdx  # → 1 件以上
```

---

## ステップ 3: `driver.rs` — `v53600_tests` 追加

`v53500_tests` モジュールの直前に `v53600_tests` を追加:

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

`cargo build` → コンパイルエラーなし確認。

---

## ステップ 4: `fav/Cargo.toml` バージョン更新

`version = "53.5.0"` → `version = "53.6.0"`

v53500_tests にはバージョンピンテストが存在しないため、空化対象なし。

---

## ステップ 5: テスト実行・確認

```bash
cd /c/Users/yoshi/favnir/fav && cargo test -j 8 -- --test-threads=8 2>&1 | grep "test result"
```

期待値: 3175 passed, 0 failed

```bash
cargo clippy -- -D warnings
```

---

## ステップ 6: 後処理

- `CHANGELOG.md` に v53.6.0 エントリ追加（直前の v53.5.0 エントリと同形式であることを確認）
- `versions/current.md` を v53.6.0（3175 tests）に更新
- `roadmap-v53.1-v54.0.md` の v53.6.0 実績欄を COMPLETE に更新・推定値（3169 → 3175）を修正
- `tasks.md` を COMPLETE に更新（T0〜T4 全 `[x]`）
