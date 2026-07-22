# Plan: v53.7.0 — ドキュメントサイト全体最終チェック

---

## ステップ 1: 事前確認

```bash
cargo test -j 8 -- --test-threads=8 2>&1 | grep "test result"
# → 3175 passed, 0 failed を確認

# v53700_tests が未存在を確認
rg -n "v53700_tests" fav/src/driver.rs  # → 0 件

# v53600_tests の行番号を確認（挿入位置）
rg -n "v53600_tests" fav/src/driver.rs  # → 行番号を特定

# glossary.mdx が未存在を確認
ls site/content/docs/glossary.mdx 2>/dev/null  # → エラー

# 主要 docs ファイルが存在することを確認（docs_no_broken_links で検証する対象）
ls site/content/docs/introduction.mdx site/content/docs/quickstart.mdx site/content/docs/installation.mdx

# Cargo.toml が 53.6.0 であることを確認
grep "^version" fav/Cargo.toml  # → version = "53.6.0"
```

---

## ステップ 2: `site/content/docs/glossary.mdx` 新規作成

```mdx
---
title: "用語集"
description: "Favnir の主要な用語・概念の定義一覧"
---

# 用語集

## par

`par [A, B] |> Merge.ordered` — 複数の stage を並列実行するキーワード。
v51.x で導入。各 stage はスレッド分離で並列実行され、結果は `Merge` で結合される。

## assert_schema

`assert_schema<T>(map)` — 実行時にマップのフィールドが型 `T` に一致するか検証する組み込み関数。
v52.x で導入。型不一致は E0419 エラーを発生させる。`--strict-schema` フラグで未知フィールドもエラーにできる。

## lineage

データリネージ — stage 間のデータの流れ（upstream / downstream）とスキーマ変換を追跡する機能。
v53.1.0 で LSP ホバーへの統合が完了し、エディタ上でリネージを確認できるようになった。

## inlay hints

LSP インレイヒント — エディタに型情報・stage 名・推論結果をインラインで表示する機能。
v51.x で導入。`fav` の LSP サーバーが VS Code 等に対して hint を送信する。

## rune

外部サービスとの接続を担う Favnir のプラグイン単位。`import kafka` のように参照する。
`runes/` ディレクトリに配置される。

## stage

パイプラインの処理単位。`stage Name: InputType -> OutputType = |arg| { ... }` の形式で定義する。

## pipeline

`pipeline Name { stage ... }` ブロックで定義される stage の連鎖。
`seq` / `par` / `|>` でデータフローを記述する。
```

内容確認:
```bash
grep "par\|assert_schema\|lineage\|inlay" site/content/docs/glossary.mdx  # → 各用語が含まれる
```

---

## ステップ 3: `driver.rs` — `v53700_tests` 追加

`v53600_tests` モジュールの直前に `v53700_tests` を追加:

```rust
// -- v53700_tests (v53.7.0) -- ドキュメントサイト最終チェック --
#[cfg(test)]
mod v53700_tests {
    #[test]
    fn docs_no_broken_links() {
        // 主要 docs ページの存在確認（Rust テストで確認可能なリンク切れ代理指標）
        let base = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../site/content/docs");
        assert!(base.join("introduction.mdx").exists(), "docs/introduction.mdx must exist");
        assert!(base.join("quickstart.mdx").exists(), "docs/quickstart.mdx must exist");
        assert!(base.join("glossary.mdx").exists(), "docs/glossary.mdx must exist");
        assert!(base.join("installation.mdx").exists(), "docs/installation.mdx must exist");
    }

    #[test]
    fn docs_glossary_updated() {
        let content = include_str!("../../site/content/docs/glossary.mdx");
        assert!(
            content.contains("## par"),
            "glossary.mdx must have ## par section (v51 addition)"
        );
        assert!(
            content.contains("assert_schema"),
            "glossary.mdx must define assert_schema (v52 addition)"
        );
        assert!(
            content.contains("lineage"),
            "glossary.mdx must define lineage (v53.1 addition)"
        );
        assert!(
            content.contains("inlay"),
            "glossary.mdx must define inlay hints (v51 addition)"
        );
    }
}
```

`cargo build` → コンパイルエラーなし確認。

---

## ステップ 4: `fav/Cargo.toml` バージョン更新

`version = "53.6.0"` → `version = "53.7.0"`

v53600_tests にはバージョンピンテストが存在しないため、空化対象なし。

---

## ステップ 5: テスト実行・確認

```bash
cd /c/Users/yoshi/favnir/fav && cargo test -j 8 -- --test-threads=8 2>&1 | grep "test result"
```

期待値: 3177 passed, 0 failed

```bash
cargo clippy -- -D warnings
```

---

## ステップ 6: 後処理

- `CHANGELOG.md` に v53.7.0 エントリ追加（直前の v53.6.0 エントリと同形式であることを確認）
- `versions/current.md` を v53.7.0（3177 tests）に更新
- `roadmap-v53.1-v54.0.md` の v53.7.0 実績欄を COMPLETE に更新・推定値（3171 → 3177）を修正
- `tasks.md` を COMPLETE に更新（T0〜T4 全 `[x]`）
