# v24.7.0 — 実装計画

## 概要

ドキュメントサイト v2（`learn/` / `cookbook/` / `packages/` / `bench/` / `spec/`）の新規コンテンツを追加し、
driver.rs に v247000_tests を追加する。Rust コードへの変更はテストモジュールのみ。

---

## 実装ステップ

### Step 0: 事前確認

```bash
grep -n "version = " fav/Cargo.toml           # "24.6.0" であること
grep -n "mod v247000_tests" fav/src/driver.rs  # 未存在
ls site/content/learn/ 2>/dev/null             # 未存在
ls site/content/cookbook/ 2>/dev/null          # 未存在
ls site/app/packages/ 2>/dev/null              # 未存在
```

---

### Step 1: `site/content/learn/` 作成（チュートリアル 3 記事）

**1-1. `site/content/learn/getting-started.mdx`**

```mdx
---
title: "はじめての Favnir"
description: "インストールから Hello World まで"
---

# はじめての Favnir

## インストール

```bash
cargo install fav
```

## Hello World

```favnir
fn main() -> String { "Hello, Favnir!" }
```

`fav run hello.fav` → `Hello, Favnir!`
```

> テスト要件: `"Hello"` を含む

**1-2. `site/content/learn/pipeline-basics.mdx`**

- stage / seq / par の基礎
- `|>` パイプライン演算子

**1-3. `site/content/learn/type-system.mdx`**

- 型推論（HM 型推論）
- エフェクトシステム（`!Http` / `!Db`）
- ジェネリクス

---

### Step 2: `site/content/cookbook/` 作成（レシピ 3 記事）

**2-1. `site/content/cookbook/etl-csv-to-db.mdx`**

> テスト要件: `"csv"` と `"db"` を両方含む

- CSV ファイルを読み込んで DB に投入するレシピ
- `runes/csv` と `runes/postgres` の使い方

**2-2. `site/content/cookbook/api-gateway.mdx`**

- HTTP API エンドポイント実装レシピ
- `!Http` エフェクトの使い方

**2-3. `site/content/cookbook/parallel-pipeline.mdx`**

- `par [StageA, StageB] |> Merge` パターン
- 並列 ETL の典型的な実装

---

### Step 3: `site/app/packages/page.tsx` 作成

```tsx
// Rune レジストリページ（静的）
// OFFICIAL_CATALOG の 50 パッケージを表示
export default function PackagesPage() {
  return (
    <main>
      <h1>Favnir Rune Packages</h1>
      <p>Official rune packages for Favnir.</p>
      {/* package list */}
    </main>
  )
}
```

> テスト要件: `"rune"` を含む

---

### Step 4: `site/content/docs/bench/index.mdx` 作成

> テスト要件: `"benchmark"` を含む

- v20.3〜v24.6 のベンチマーク履歴リンク一覧
- `fav bench` コマンドの解説
- 注意: `site/content/docs/tools/` 以下にベンチ関連の既存ファイルが存在する可能性があるが、
  `bench/index.mdx` はサイトナビゲーション用の独立ページとして作成する（重複不可）

---

### Step 5: `site/content/docs/spec/index.mdx` 作成

> テスト要件: `"fav spec"` を含む

- `fav spec --format markdown > SPEC.md` の使い方
- 形式的仕様書の構成説明

---

### Step 6: `fav/src/driver.rs` — v247000_tests 追加

**6-1: `v246000_tests::version_is_24_6_0` を削除**（他 4 件は保持）

**6-2: `v247000_tests` モジュールを追加**

```rust
#[cfg(test)]
mod v247000_tests {
    use super::*;

    #[test]
    fn learn_getting_started_exists() {
        let md = include_str!("../../site/content/learn/getting-started.mdx");
        assert!(md.contains("Hello"),
            "getting-started.mdx must contain 'Hello'");
    }

    #[test]
    fn cookbook_etl_recipe_exists() {
        let md = include_str!("../../site/content/cookbook/etl-csv-to-db.mdx");
        assert!(md.to_lowercase().contains("csv"),
            "etl recipe must mention csv");
        assert!(md.to_lowercase().contains("db"),
            "etl recipe must mention db");
    }

    #[test]
    fn packages_page_has_rune_keyword() {
        let src = include_str!("../../site/app/packages/page.tsx");
        assert!(src.to_lowercase().contains("rune"),
            "packages page must mention rune");
    }

    #[test]
    fn bench_page_exists() {
        let md = include_str!("../../site/content/docs/bench/index.mdx");
        assert!(md.to_lowercase().contains("benchmark"),
            "bench page must contain 'benchmark'");
    }

    #[test]
    fn spec_page_exists() {
        let md = include_str!("../../site/content/docs/spec/index.mdx");
        assert!(md.contains("fav spec"),
            "spec page must contain 'fav spec'");
    }

    #[test]
    fn changelog_has_v24_7_0() {
        let cl = include_str!("../../CHANGELOG.md");
        assert!(cl.contains("[v24.7.0]"),
            "CHANGELOG.md must contain [v24.7.0]");
    }
}
```

---

### Step 7: Cargo.toml + CHANGELOG + benchmarks

- `fav/Cargo.toml`: `"24.6.0"` → `"24.7.0"`
- `CHANGELOG.md` 先頭に v24.7.0 エントリ追加
- `benchmarks/v24.7.0.json` 作成

---

## 注意事項

- `include_str!` パスは `fav/src/driver.rs` 起点で `../../` から始まる
  - `site/content/learn/getting-started.mdx` → `include_str!("../../site/content/learn/getting-started.mdx")`
  - `site/content/cookbook/etl-csv-to-db.mdx` → `include_str!("../../site/content/cookbook/etl-csv-to-db.mdx")`
  - `site/app/packages/page.tsx` → `include_str!("../../site/app/packages/page.tsx")`
  - `site/content/docs/bench/index.mdx` → `include_str!("../../site/content/docs/bench/index.mdx")`
  - `site/content/docs/spec/index.mdx` → `include_str!("../../site/content/docs/spec/index.mdx")`
- `site/app/packages/` ディレクトリが未存在の場合は新規作成
- `site/content/docs/bench/` ディレクトリが未存在の場合は新規作成
- `site/content/docs/spec/` ディレクトリが未存在の場合は新規作成
