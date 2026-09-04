# Plan: v95.1.0 — OData `$delta` / `DeltaLink` 型定義

## 実装ステップ

### Step 1: `runes/sap-odata/delta.fav` を新規作成する

`runes/sap-odata/` ディレクトリに `delta.fav` を作成する。

内容:
1. `DeltaResult<T>` 型定義（ジェネリック）
   - `entities: List<T>` — 変更・新規エンティティの一覧
   - `delta_link: String` — 次回呼び出し用の deltaLink（`@odata.deltaLink` の値）
   - `has_more: Bool` — `@odata.nextLink` が存在するかどうか
2. `DeletedEntity` 型定義
   - `id: String` — 削除されたエンティティの key
   - `reason: String` — `"deleted"` または `"changed"`
3. `delta_link_is_valid` ヘルパー関数（`public fn`）
   - 引数: `link: String`
   - 戻り値: `Bool`
   - 実装: `String.length(link) > 0`

**依存**: なし（純粋な型定義のみ）

---

### Step 2: `fav/src/driver.rs` に `mod v95100_tests` を追加する

`mod v94900_tests { ... }` または `mod v95000_tests { ... }` の直後に追加する。

テスト 1: `delta_fav_exists`
```rust
#[test]
fn delta_fav_exists() {
    assert!(
        std::path::Path::new("../runes/sap-odata/delta.fav").exists(),
        "runes/sap-odata/delta.fav が存在しない"
    );
}
```

テスト 2: `delta_result_type_defined`
```rust
#[test]
fn delta_result_type_defined() {
    let content = std::fs::read_to_string("../runes/sap-odata/delta.fav")
        .expect("delta.fav を読み込めない");
    assert!(
        content.contains("DeltaResult"),
        "delta.fav に DeltaResult 型が定義されていない"
    );
}
```

**依存**: Step 1 完了後

---

## 実装順序

```
Step 1: delta.fav 作成（DeltaResult<T> / DeletedEntity / delta_link_is_valid）
    ↓
Step 2: driver.rs に v95100_tests 追加
    ↓
cargo test で 4,166 tests, 0 failures を確認
```

## 変更ファイル一覧

| ファイル | 操作 |
|---|---|
| `runes/sap-odata/delta.fav` | 新規作成 |
| `fav/src/driver.rs` | 追記（`mod v95100_tests`） |
