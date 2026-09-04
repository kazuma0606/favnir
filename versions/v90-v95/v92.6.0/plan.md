# Plan: v92.6.0 — QueryBuilder<T> E2E テストパイプライン

## 実装ステップ

### Step 0: 着手前チェック

```bash
cargo test 2>&1 | grep "test result"
```

期待: `4107 passed; 0 failed`

- `fav/src/driver.rs` に `mod v92500_tests` が存在することを確認
- `runes/sap-odata/query_builder.fav` に `public fn fetch_all_pages` が含まれることを確認（v92.4.0 完了済みの証拠）
- `fav/tmp/hello.fav` が存在することを確認
- CHANGELOG 更新は v93.0.0 宣言時にまとめて行う（本バージョンでは不要）

### Step 1: `pipeline_query.fav` を新規作成

`infra/e2e-demo/sap-odata/pipeline_query.fav` を作成する。

```favnir
-- infra/e2e-demo/sap-odata/pipeline_query.fav — QueryBuilder<T> パターン E2E デモ（v92.6.0）
-- query_builder.fav の公開 API（query / with_filter / with_select / fetch_all_pages）を使用する。
-- fetch_all_pages は v92.4.0 スタブ（Result.err）のため、本デモはパターン検証を目的とする。

import rune "sap-odata"

-- シナリオ 5: QueryBuilder<T> を使ったページネーション取得（v92.6.0）
-- with_filter / with_select チェーンで型安全に OData クエリを組み立て、
-- fetch_all_pages で全ページを自動取得するパターンを示す。
-- NOTE: fetch_all_pages は現在スタブ（v92.4.0）。実装完了後に本デモが実際に動作する。
fn sync_business_partners_paged(ctx: AppCtx) -> Result<String, String> {
    bind q1  <- Result.ok(query<BusinessPartner>())
    bind q2  <- Result.ok(with_filter(q1, Eq("Country", "JP")))
    bind q3  <- Result.ok(with_select(q2, ["BusinessPartner", "BusinessPartnerName"]))
    bind bps <- fetch_all_pages(ctx, q3, 20, fn(c, b) { Result.err("fetcher: not yet wired") })
    bind enc <- Json.encode(bps)
    bind _   <- ctx.s3.put_object("sap-sync", "business_partners_jp.json", enc)
    Result.ok("synced " ++ Int.to_string(List.length(bps)) ++ " business partners")
}
```

注意:
- `bind q <- ...` 再束縛は E0018 違反 → `q1` / `q2` / `q3` を使う
- `ctx.s3.put_object` の第3引数は `String` → `Json.encode` を使う

### Step 2: `driver.rs` に `mod v92600_tests` を追加

`mod v92500_tests { ... }` の直後に追加：

```rust
#[cfg(test)]
mod v92600_tests {
    // use super::* は不要（std::fs のみ使用）
    // パス基点: fav/ ディレクトリ（cargo test の実行カレント）
    #[test]
    fn pipeline_query_fav_exists() {
        assert!(
            std::path::Path::new("../infra/e2e-demo/sap-odata/pipeline_query.fav").exists(),
            "infra/e2e-demo/sap-odata/pipeline_query.fav should exist"
        );
    }
    #[test]
    fn pipeline_query_uses_fetch_all_pages() {
        let content = std::fs::read_to_string("../infra/e2e-demo/sap-odata/pipeline_query.fav")
            .expect("pipeline_query.fav should exist");
        assert!(
            content.contains("fetch_all_pages"),
            "pipeline_query.fav should use fetch_all_pages"
        );
    }
}
```

注意: `infra/` は `fav/` の親ディレクトリ内にあるため、パスは `"../infra/e2e-demo/sap-odata/pipeline_query.fav"`。

### Step 3: `cargo test` で全 pass 確認

```bash
cargo test 2>&1 | grep "test result"
```

期待: `4109 passed; 0 failed`

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
  → Step 1（pipeline_query.fav 作成）
  → Step 2（driver.rs: テスト追加）
  → Step 3（cargo test）
  → Step 4（CI 事前確認）
```
