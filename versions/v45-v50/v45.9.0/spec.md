# Spec: v45.9.0 — examples 更新 Phase 2 + v46.0 前調整

Date: 2026-07-16
Status: TODO

---

## 概要

examples 更新の残件を完了し、v46.0 に向けてコードフリーズする。
`site/content/docs/language-refinement-overview.mdx` を新規作成する。

---

## 調査結果（実装前に確認済み）

### examples の残件

Phase 1（v45.8.0）で `!Effect` アノテーション除去確認・`return` ガード節追加を完了。
Phase 2 の残件:

| ファイル | 問題 | 対応 |
|---|---|---|
| `examples/pipeline/stage_seq_demo.fav` | 1行目: 「`stage` は `trf` のエイリアス」誤記（v2.0.0 で `trf` 廃止） | コメントを現在の構文説明に修正 |
| `examples/pipeline/stage_seq_demo.fav` | 8行目: 「`seq` は `flw` のエイリアス」誤記（v2.0.0 で `flw` 廃止） | コメントを現在の構文説明に修正 |

その他 73 件の .fav ファイルは構造的に問題なし。

### site/ の状況

`site/content/docs/language-refinement-overview.mdx` は存在しない（要新規作成）。
参照: 同構造の `precision-and-flow-overview.mdx` を雛形として使用。

---

## 変更対象

### §1 — `examples/pipeline/stage_seq_demo.fav` コメント修正

コメント「`stage` is an alias for `trf`」は現在の Favnir では誤り。
`trf` は v2.0.0 で廃止され `stage` が唯一の正式キーワードとなっている。

**変更前**:
```
// stage_seq_demo.fav — v1.9.0: stage/seq keyword aliases for trf/flw

// `stage` is an alias for `trf` (transform stage in a pipeline)
stage double: Int -> Int = |x| { x * 2 }
...
// `seq` is an alias for `flw` (sequence of stages)
seq pipeline = double |> add_ten
```

**変更後**:
```
// stage_seq_demo.fav — pipeline stage/seq keywords demonstration

// `stage` defines a transform stage in a pipeline
stage double: Int -> Int = |x| { x * 2 }
...
// `seq` defines a sequence of pipeline stages
seq pipeline = double |> add_ten
```

### §2 — `site/content/docs/language-refinement-overview.mdx` 新規作成

Language Refinement スプリント（v45.1〜v45.9）の成果をまとめる概要ページ。

内容:
- タイトル: `Language Refinement Overview`
- v45.1〜v45.9 の達成事項テーブル
- 主要機能（`return` 構文・`match` 完全網羅・型エイリアス・エラーメッセージ改善・数値リテラル・examples 更新）の要約
- v46.0 への展望

### §コードフリーズ確認

v46.0 前コードフリーズとして以下を確認する（新たなコード変更は行わない）:
- `cargo test` 全通過
- `cargo clippy -- -D warnings` クリーン
- 上記 2 点が揃った時点で v45.9.0 を凍結し v46.0 へ移行する

（WASM ビルドや依存バージョン固定などの追加作業は v46.0 のスコープとする）

### §3 — `driver.rs`: v459000_tests 追加

`v458000_tests` の直後に `v459000_tests` モジュールを追加（2件）。

```rust
#[cfg(test)]
#[cfg(not(target_arch = "wasm32"))]
mod v459000_tests {
    use std::path::Path;
    use walkdir::WalkDir;

    #[test]
    fn examples_structure_valid() {
        // examples/ が存在し、少なくとも 50 件の .fav ファイルを含む
        let examples_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("examples");
        assert!(examples_dir.exists(), "examples/ directory not found");
        let fav_count = WalkDir::new(&examples_dir)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.path().extension().map(|x| x == "fav").unwrap_or(false))
            .count();
        // しきい値 70: 現在 73 件。余裕を持たせて将来の追加を許容しつつ大幅な削減を検知する
        assert!(
            fav_count >= 70,
            "expected at least 70 .fav files in examples/, found {}",
            fav_count
        );
    }

    #[test]
    fn language_refinement_overview_doc_exists() {
        let mdx = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../site/content/docs/language-refinement-overview.mdx");
        assert!(mdx.exists(), "language-refinement-overview.mdx not found");
        let content = std::fs::read_to_string(&mdx)
            .unwrap_or_else(|e| panic!("failed to read language-refinement-overview.mdx: {}", e));
        assert!(
            content.contains("Language Refinement"),
            "doc should mention 'Language Refinement'"
        );
    }
}
```

---

## 変更しないファイル

- `ast.rs` / `checker.rs` / `compiler.rs` / `vm.rs` / `lexer.rs` / `parser.rs`
- `error_catalog.rs`
- その他 examples ファイル（Phase 1 で対応済み）

---

## 完了条件

- `cargo test` 全通過（2988 tests passed, 0 failed）
- `cargo clippy -- -D warnings` クリーン
- `CHANGELOG.md` に v45.9.0 エントリ追加
- `versions/current.md` を v45.9.0（2988 tests）に更新
- `fav/Cargo.toml` version → `45.9.0`
