# Plan: v92.4.0 — `Page<T>` 型 + `fetch_all_pages` 実装

## 実装ステップ

### Step 0: 着手前チェック

```bash
cargo test 2>&1 | grep "test result"
```

期待: `4103 passed; 0 failed`

- `fav/src/driver.rs` に `mod v92300_tests` が存在することを確認
- `runes/sap-odata/query_builder.fav` に `public fn with_top` / `with_skip` / `with_order_by` が含まれることを確認（v92.3.0 完了済みの証拠）
- `fav/tmp/hello.fav` が存在することを確認
- CHANGELOG 更新は v93.0.0 宣言時にまとめて行う（本バージョンでは不要）

### Step 1: `query_builder.fav` に `Page<T>` と `fetch_all_pages<T>` を追加

`with_order_by` の直後に追記する：

```favnir
-- ページ結果型（v92.4.0）
-- items:     取得した 1 ページ分のエンティティリスト
-- next_link: 次ページの OData URL（@odata.nextLink）。最終ページでは none()
-- total:     総件数（@odata.count）。サーバーが返さない場合は none()
public type Page<T> = {
    items:     List<T>,
    next_link: Option<String>,
    total:     Option<Int>
}

-- 全ページを自動取得する関数（v92.4.0）
-- max_pages: 最大ページ数（無限ループ防止）
-- fetcher:   1 ページ分のデータを取得する関数
-- NOTE: v92.4.0 はスタブ実装。完全な再帰実装は v92.5.0 以降
public fn fetch_all_pages<T>(
    ctx:       AppCtx,
    builder:   QueryBuilder<T>,
    max_pages: Int,
    fetcher:   fn(AppCtx, QueryBuilder<T>) -> Result<Page<T>, String>
) -> Result<List<T>, String> {
    Result.err("fetch_all_pages: not yet implemented (v92.4.0 stub)")
}
```

### Step 2: `driver.rs` に `mod v92400_tests` を追加

`mod v92300_tests { ... }` の直後に追加：

```rust
#[cfg(test)]
mod v92400_tests {
    // use super::* は不要（std::fs のみ使用）
    // パス基点: fav/ ディレクトリ（cargo test の実行カレント）
    #[test]
    fn page_type_defined() {
        let content = std::fs::read_to_string("../runes/sap-odata/query_builder.fav")
            .expect("runes/sap-odata/query_builder.fav should exist");
        assert!(
            content.contains("public type Page"),
            "query_builder.fav should define public type Page"
        );
    }
    #[test]
    fn fetch_all_pages_function_defined() {
        let content = std::fs::read_to_string("../runes/sap-odata/query_builder.fav")
            .expect("runes/sap-odata/query_builder.fav should exist");
        assert!(
            content.contains("public fn fetch_all_pages"),
            "query_builder.fav should define public fn fetch_all_pages"
        );
    }
}
```

### Step 3: `cargo test` で全 pass 確認

```bash
cargo test 2>&1 | grep "test result"
```

期待: `4105 passed; 0 failed`

### Step 4: CI 事前確認

```bash
cargo clippy --locked -- -D warnings
./target/debug/fav fmt --check self/compiler.fav
./target/debug/fav fmt --check self/checker.fav
```

---

## 依存順序

```
Step 0（チェック）
  → Step 1（query_builder.fav に Page<T> / fetch_all_pages<T> 追加）
  → Step 2（driver.rs: テスト追加）
  → Step 3（cargo test）
  → Step 4（CI 事前確認）
```
