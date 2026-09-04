# Plan: v92.0.0 — SAP OData Query 1.0 宣言 ★クリーンアップ

## 実装ステップ

### Step 0: 着手前チェック

```bash
cargo test 2>&1 | grep "test result"
```

期待: `4090 passed; 0 failed`

- `fav/src/driver.rs` に `mod v91900_tests` が存在することを確認
- `runes/sap-odata/query_client.fav` が存在することを確認
- `fav/tmp/hello.fav` が存在することを確認

### Step 1: `CHANGELOG.md` に v91.1.0〜v92.0.0 エントリを追加

**重要**: `changelog_has_v92_0_0` テスト（Step 7）より前に更新すること。

`CHANGELOG.md` の先頭（`## [v91.0.0]` の前）に v92.0.0 エントリを追加し、
直後に v91.1〜v91.9 の各エントリを追加する。

v92.0.0 エントリ例:
```markdown
## [v92.0.0] — 2026-08-27 — SAP OData Query 1.0 宣言 ★クリーンアップ

### 宣言
> 「SapQueryClient を通じて `sales_orders_query(q)` と書けば、
>  $filter・$select・$expand を型で組み立てた OData クエリが発行できる。
>  それが、Favnir SAP OData Query 1.0 である。」

### Changed
- `fav/Cargo.toml` — バージョンを `92.0.0` に更新
- `MILESTONE.md` / `README.md` / `versions/current.md` — v92.0.0 SAP OData Query 1.0 宣言を反映

### Added
- `fav/src/driver.rs` — `mod v92000_tests`（テスト 4 件）を追加
- 合計テスト数: **4,094**（+4）
```

### Step 2: `fav/Cargo.toml` バージョンを `92.0.0` に更新

```toml
version = "92.0.0"
```

### Step 3: `MILESTONE.md` に v92.0.0 宣言を追加

`## v91.0.0` の前に以下を追加：

```markdown
## v92.0.0（2026-08-27）— SAP OData Query 1.0 宣言

> 「SapQueryClient を通じて `sales_orders_query(q)` と書けば、
>  $filter・$select・$expand を型で組み立てた OData クエリが発行できる。
>  誤フィールド指定はコンパイル時に検出される。
>  それが、Favnir SAP OData Query 1.0 である。」

**SAP OData Query 1.0** の宣言バージョン。v91.1.0〜v91.9.0 で実装した
OData クエリ型基盤（SelectClause / ExpandClause / FilterExpr / 各エンティティQuery / ODataQueryBuilder / SapQueryClient）の完成を宣言した。テスト数: 4,094。

**SAP OData Query 1.0（v91.1〜v91.9）達成内容:**
- **型**: `SelectClause<T>` / `ExpandClause<T>` / `FilterExpr<T>`（ファントム型付きクエリ型）
- **クエリ型**: `SalesOrderQuery` / `BusinessPartnerQuery` / `MaterialQuery` / `PurchaseOrderQuery` / `JournalEntryQuery`
- **URL生成**: `ODataQueryBuilder<T, Q>` / `build_url`（エンティティパス結合）
- **interface**: `SapQueryClient`（5 クエリメソッド）— 循環 dep 回避のため `query_client.fav` に独立定義
- **実装**: `SapODataClient` / `MockSapClient` が `SapQueryClient` を impl

---
```

### Step 4: `README.md` に OData Query への言及を追加

SAP 統合セクションまたは機能一覧に `SapQueryClient` / `OData Query` を追記する。

### Step 5: `versions/current.md` を v92.0.0 に更新

### Step 6: `driver.rs` 内の旧バージョン文字列を一括置換

`driver.rs` に `"91.0.0"` が 44 箇所存在する（宣言バージョン時に一括更新するパターン）。
**既存 mod ブロック内の関数定義を変更**（新規 mod 追加ではない）：

```bash
sed -i 's/"91\.0\.0"/"92.0.0"/g' fav/src/driver.rs
```

これにより `cargo_toml_version_is_91_0_0` / `cargo_toml_version_is_90_0_0` 等の assertion が
`"92.0.0"` を確認するように一括更新される（関数名はそのまま）。

### Step 7: `driver.rs` に `mod v92000_tests` を追加

`mod v91900_tests { ... }` の直後に追加：

```rust
#[cfg(test)]
mod v92000_tests {
    #[test]
    fn cargo_toml_version_is_92_0_0() {
        let content = std::fs::read_to_string("Cargo.toml").expect("Cargo.toml should exist");
        assert!(
            content.contains("version = \"92.0.0\""),
            "Cargo.toml version should be 92.0.0"
        );
    }
    #[test]
    fn changelog_has_v92_0_0() {
        let content = std::fs::read_to_string("../CHANGELOG.md")
            .expect("CHANGELOG.md should exist");
        assert!(
            content.contains("v92.0.0"),
            "CHANGELOG.md should have v92.0.0 entry"
        );
    }
    #[test]
    fn milestone_has_sap_odata_query() {
        let content = std::fs::read_to_string("../MILESTONE.md")
            .expect("MILESTONE.md should exist");
        assert!(
            content.contains("SAP OData Query 1.0"),
            "MILESTONE.md should mention SAP OData Query 1.0"
        );
    }
    #[test]
    fn readme_mentions_odata_query() {
        let content = std::fs::read_to_string("../README.md")
            .expect("README.md should exist");
        assert!(
            content.contains("OData Query") || content.contains("SapQueryClient"),
            "README.md should mention OData Query or SapQueryClient"
        );
    }
}
```

### Step 8: `cargo test` で全 pass 確認

```bash
cargo test 2>&1 | grep "test result"
```

期待: `4094 passed; 0 failed`

### Step 9: CI 事前確認（cargo clean 前に実施）

```bash
cargo clippy --locked -- -D warnings
./target/debug/fav fmt --check self/compiler.fav
./target/debug/fav fmt --check self/checker.fav
```

### Step 10: `cargo clean`

```bash
cargo clean
```

---

## 依存順序

```
Step 0（チェック）
  → Step 1（CHANGELOG.md）
  → Step 2（Cargo.toml）
  → Step 3（MILESTONE.md）
  → Step 4（README.md）
  → Step 5（current.md）
  → Step 6（driver.rs: バージョンテスト更新）
  → Step 7（driver.rs: v92000_tests 追加）
  → Step 8（cargo test）
  → Step 9（CI 事前確認）
  → Step 10（cargo clean）
```

**重要**: Step 1（CHANGELOG）は Step 7 より前。Step 9（CI）は Step 10 より前（cargo clean 後は `./target/debug/fav` が消える）。
