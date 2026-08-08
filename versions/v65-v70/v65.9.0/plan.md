# v65.9.0 実装計画 — 安定化・コードフリーズ（Math & Science 前調整）

Version: 65.9.0
Status: 未着手
Base tests: 3469
Target tests: 3471

---

## 実装ステップ

### Step 1: MDX ドキュメント作成

`site/content/docs/runes/math-runes-overview.mdx` を新規作成。
最低要件:
- 空でない
- `"Rune.linalg"` を含む
- v65.1〜v65.8 で追加した 7 Rune を一覧で紹介

MDX 構文の注意点（過去の acorn パースエラーを回避）:
- コードブロック内に `import` / `export` を行頭に置かない
- favnir コードブロックは ` ```favnir ` で閉じる

### Step 2: `driver.rs` テスト追加

`// -- v65800_tests (v65.8.0)` コメントの直前に `v65900_tests` を挿入。
2 テスト関数:
- `math_foundation_all_runes_stable`: 7 Rune ファイルすべてを `include_str!` で読み込み、空でないことを assert
- `math_docs_complete`: `math-runes-overview.mdx` を読み込み、空でなく `"Rune.linalg"` を含むことを assert

### Step 3: ビルド・テスト確認

```bash
cargo build
cargo test --bin fav v65900_tests
cargo test --bin fav
```

---

## `driver.rs` 挿入コード

```rust
// -- v65900_tests (v65.9.0) -- Stabilization --
#[cfg(test)]
mod v65900_tests {
    #[test]
    fn math_foundation_all_runes_stable() {
        let linalg   = include_str!("../../runes/linalg/linalg.fav");
        let stats    = include_str!("../../runes/stats/stats.fav");
        let autodiff = include_str!("../../runes/autodiff/autodiff.fav");
        let optim    = include_str!("../../runes/optim/optim.fav");
        let numeric  = include_str!("../../runes/numeric/numeric.fav");
        let ts       = include_str!("../../runes/timeseries/timeseries.fav");
        let ml       = include_str!("../../runes/ml/ml.fav");
        assert!(!linalg.is_empty(),   "linalg.fav should not be empty");
        assert!(!stats.is_empty(),    "stats.fav should not be empty");
        assert!(!autodiff.is_empty(), "autodiff.fav should not be empty");
        assert!(!optim.is_empty(),    "optim.fav should not be empty");
        assert!(!numeric.is_empty(),  "numeric.fav should not be empty");
        assert!(!ts.is_empty(),       "timeseries.fav should not be empty");
        assert!(!ml.is_empty(),       "ml.fav should not be empty");
    }

    #[test]
    fn math_docs_complete() {
        let content = include_str!("../../site/content/docs/runes/math-runes-overview.mdx");
        assert!(!content.is_empty(), "math-runes-overview.mdx should not be empty");
        assert!(
            content.contains("Rune.linalg"),
            "math-runes-overview.mdx should mention Rune.linalg"
        );
    }
}
```

---

## include_str! パス一覧

| ファイル | driver.rs 相対パス |
|---|---|
| `runes/linalg/linalg.fav` | `../../runes/linalg/linalg.fav` |
| `runes/stats/stats.fav` | `../../runes/stats/stats.fav` |
| `runes/autodiff/autodiff.fav` | `../../runes/autodiff/autodiff.fav` |
| `runes/optim/optim.fav` | `../../runes/optim/optim.fav` |
| `runes/numeric/numeric.fav` | `../../runes/numeric/numeric.fav` |
| `runes/timeseries/timeseries.fav` | `../../runes/timeseries/timeseries.fav` |
| `runes/ml/ml.fav` | `../../runes/ml/ml.fav` |
| `site/content/docs/runes/math-runes-overview.mdx` | `../../site/content/docs/runes/math-runes-overview.mdx` |

---

## リスク・注意点

- MDX ファイルの acorn パースエラー: コードブロック内の `import`/`export` を行頭に置かない
- `math_foundation_all_runes_stable` は全 7 ファイルを `include_str!` で読み込むため、1 ファイルでも欠損するとコンパイルエラーになる（意図的な設計）
- `math_docs_complete` の assert は `"Rune.linalg"` の存在のみ確認（MDX の完全なコンテンツ検証ではない）
