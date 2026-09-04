# Plan: v92.8.0 — サイトドキュメント更新（QueryBuilder<T> パターン）

## 実装ステップ

### Step 0: 着手前チェック

```bash
cargo test 2>&1 | grep "test result"
```

期待: `4112 passed; 0 failed`

- `fav/src/driver.rs` に `mod v92700_tests` が存在することを確認
- `site/content/docs/runes/sap-odata.mdx` を Read し、現状のセクション構成を確認する
- `fav/tmp/hello.fav` が存在することを確認
- CHANGELOG 更新は v93.0.0 宣言時にまとめて行う（本バージョンでは不要）

### Step 1: `sap-odata.mdx` にセクションを追加

`site/content/docs/runes/sap-odata.mdx` の末尾に以下の4セクションを追記する。

追記内容（順序）:
1. `## QueryBuilder<T> Fluent API（v92.1.0〜）` — query<T>() とチェーン関数の一覧・使用例
2. `## Page<T> によるページネーション（v92.4.0〜）` — Page<T> 型定義・fetch_all_pages の説明
3. `## W060 N+1 lint（v92.5.0〜）` — 検出パターンと推奨コード（W060 が正しい、W020 は別ルール）
4. `## fetch_all_pages パターン（v92.6.0 デモ）` — pipeline_query.fav の全文

コード例での注意事項:
- `bind q` 再束縛は E0018 違反 → `q1` / `q2` / `q3` / `q4` を使う
- W060 説明の推奨コードでは `bind q <- ...` の E0018 回避パターンを示す

### Step 2: `driver.rs` に `mod v92800_tests` を追加

`mod v92700_tests { ... }` の直後に追加：

```rust
#[cfg(test)]
mod v92800_tests {
    // use super::* は不要（std::fs のみ使用）
    // パス基点: fav/ ディレクトリ（cargo test の実行カレント）
    #[test]
    fn docs_sap_odata_mentions_query_builder() {
        let content = std::fs::read_to_string("../site/content/docs/runes/sap-odata.mdx")
            .expect("site/content/docs/runes/sap-odata.mdx should exist");
        assert!(
            content.contains("QueryBuilder"),
            "sap-odata.mdx should mention QueryBuilder"
        );
    }
    #[test]
    fn docs_sap_odata_mentions_fetch_all_pages() {
        let content = std::fs::read_to_string("../site/content/docs/runes/sap-odata.mdx")
            .expect("site/content/docs/runes/sap-odata.mdx should exist");
        assert!(
            content.contains("fetch_all_pages"),
            "sap-odata.mdx should mention fetch_all_pages"
        );
    }
}
```

注意: MDX ファイルは `site/` にあるため、パスは `"../site/content/docs/runes/sap-odata.mdx"`。

### Step 3: `cargo test` で全 pass 確認

```bash
cargo test 2>&1 | grep "test result"
```

期待: `4114 passed; 0 failed`

### Step 4: CI 事前確認

```bash
cargo clippy --locked -- -D warnings
./target/debug/fav fmt --check self/compiler.fav
./target/debug/fav fmt --check self/checker.fav
```

---

## 依存順序

```
Step 0（チェック + MDX 現状確認）
  → Step 1（MDX にセクション追記）
  → Step 2（driver.rs: テスト追加）
  → Step 3（cargo test）
  → Step 4（CI 事前確認）
```
