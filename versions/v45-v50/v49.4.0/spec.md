# Spec: v49.4.0 — ドキュメントサイト全面更新 Phase 1

## 概要

v46〜v48 で追加された主要機能のドキュメントを整備する。
`return` ガード節構文・import 2.0 の MDX ページを新規作成し、
Rust テスト 2 件でファイル���存在と内容を検証する。

---

## 変更ファイル

| ファイル | 変更内容 |
|---|---|
| `site/content/docs/syntax/return.mdx` | 新規作成（`return` ガード節構文ドキュメント）|
| `site/content/docs/modules/import.mdx` | 新規作成（import 2.0 構文ドキュメント）|
| `fav/src/driver.rs` | `v494000_tests` 追加（2テスト）|
| `fav/Cargo.toml` | version → `"49.4.0"` |
| `CHANGELOG.md` | v49.4.0 エントリ追加 |

---

## MDX フ���イ��内容

### `site/content/docs/syntax/return.mdx`

```mdx
---
title: "Return Statement"
order: 1
category: "Syntax"
description: "Favnir の return 文とガード節パターン"
---

# Return Statement

v46.1.0 で導入された `return` 文により、関数の���期リター���とガード節パターンが利用できます。

## 基本構文

```favnir
fn divide(a: Float, b: Float) -> Result<Float, String> {
  return Result.err("division by zero") if b == 0.0
  Result.ok(a / b)
}
```

## ガード節パターン

`return expr if condition` 形式��条件付き早期リターンを簡潔に書けます。

```favnir
fn validate_age(age: Int) -> Result<Int, String> {
  return Result.err("too young") if age < 0
  return Result.err("too old") if age > 150
  Result.ok(age)
}
```

## ネストを減らす

`return` ガード節は深いネストを避け、ハッピーパスを��確にします。
```

### `site/content/docs/modules/import.mdx`

```mdx
---
title: "Import Syntax 2.0"
order: 1
category: "Modules"
description: "Favnir import 2.0 — パッケージ import とローカル import"
---

# Import Syntax 2.0

v48.1.0 で導入された新しい import 構文（import 2.0）を説明します。
旧構文（`import rune "X"`）は v48.5.0 から W035 警告対象となっています。

## パッケージ import

```favnir
import kafka
import postgres
```

## ローカル import

```favnir
import "./stages/validate" as validate
import "./lib/utils" as utils
```

## 移行ガイド

旧構文からの移行は `fav migrate` コマンドまたは
[Import Migration Guide](/docs/migration-guide-import) を参照してください。

旧構文を使い続けると W035 警告が出ます。
```

---

## テスト（+2）

`v494000_tests` を `v493000_tests` の直前に追加:

```rust
#[cfg(test)]
mod v494000_tests {
    #[test]
    fn docs_return_syntax_exists() {
        let content = include_str!("../../site/content/docs/syntax/return.mdx");
        assert!(
            content.contains("return") && content.contains("guard"),
            "syntax/return.mdx should document return guard pattern"
        );
    }

    #[test]
    fn docs_import_v2_exists() {
        let content = include_str!("../../site/content/docs/modules/import.mdx");
        assert!(
            content.contains("import") && content.contains("W035"),
            "modules/import.mdx should document import v2 and W035 deprecation"
        );
    }
}
```

テスト数: 3075 → **3077**（+2）

---

## 注意事項

- `site/content/docs/syntax/` ディレクトリは新規作成（既存なし）
- `site/content/docs/modules/` ディレクトリは新規作成（既存なし）
- `include_str!("../../site/content/docs/syntax/return.mdx")` — `fav/src/driver.rs` からリポジトリルートの `site/` を指す（`fav/src/` → `fav/` → リポジトリルート → `site/content/...`）
- `docs_return_syntax_exists` は `"return"` かつ `"guard"` の両方を確認（`"return"` 単独は偽陽性になりうるため）
- `docs_import_v2_exists` は `"import"` かつ `"W035"` の両方を確認（W035 は import 2.0 の差別化キーワード）
- ロードマップ「stdlib 2.0: `site/content/docs/stdlib/` 各ページ」は、v47 シリーズで作成済みの `stdlib/v2.mdx`（全追加関数の索引）および既存の個別ページ（`list.mdx` / `string.mdx` / `map.mdx` 等）で充足していると判断した。`v2.mdx` が v47.1〜v47.9 の全追加関数を網羅しており、個別ページを再更新する追加価値がないため、本バージョンでは更新なしとする。なお個別ページの詳細補完が必要な場合は v49.5.0 以降のスコープ
- ロードマップの推定テスト数 3070 は旧推定値。実際��� 3075 + 2 = **3077**

---

## 完了条件

- `cargo test` 3077 passed, 0 failed（3075 + 2 件）
- `cargo clippy -- -D warnings` クリーン
- `fav/Cargo.toml` version → `"49.4.0"`
- `CHANGELOG.md` に v49.4.0 エントリ追加（`syntax/return.mdx` / `modules/import.mdx` 新規作成を明記）
- `versions/current.md` を v49.4.0（3077 tests）に更新、進行中バージョンを `v49.5.0` に更新
- `versions/roadmap/roadmap-v49.1-v50.0.md` の v49.4.0 実績を記入
- `tasks.md` を COMPLETE に更新（T0〜T3 全 `[x]`）
