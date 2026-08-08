# Spec — v56.8.0 — ドキュメントサイト Language Power 2.0 記事

## 概要

Language Power 2.0 スプリント（v56.1〜v56.7）で実装した型システム機能を、
ドキュメントサイト（`site/content/docs/language/`）に反映する。

- **新規作成**: `bounded-generics.mdx` — `where T: Interface` 本番品質化・coherence ルール
- **更新**: `row-polymorphism.mdx` — v56.3.0 行変数 `<r>` 明示・`{ field: Type | r }` 記法・LSP ホバー表示
- **更新**: `effect-inference.mdx` — v56.4.0 inlay hints（`/*!Kafka !Snowflake*/`）・`fav check --show-types` 統合

Rust テスト 2 件（`docs_bounded_generics_page_exists` / `docs_row_poly_page_exists`）を
`v56800_tests` モジュールとして `driver.rs` に追加する。

---

## ロードマップ参照

- `versions/roadmap/roadmap-v56.1-v57.0.md` — v56.8.0 セクション
- `versions/roadmap/roadmap-v55.1-v60.0.md` — v56.8.0 行
- ベーステスト数: **3243**（v56.7.0 完了時点の実績値）
- 目標テスト数: **3246**（+3）

---

## 実装内容

### 1. `fav/Cargo.toml` バージョン更新

```toml
version = "56.8.0"
```

---

### 2. `site/content/docs/language/bounded-generics.mdx` — 新規作成

v56.1.0（`where T: Interface` 正式化・E0422）と v56.2.0（複数 constraint・coherence E0423）を
まとめた専用記事を新規作成する。

**主要セクション**:

- **基本構文** — `fn name<T with Interface>(...)` フォーム
- **`where` 節フォーム** — `fn serialize_all<T>(items: List<T>) -> List<String> where T: Serializable`
- **複数 constraint** — `fn pick<T with Ord with Serialize>(a: T, b: T) -> T`
- **カスタム interface との組み合わせ**
- **E0422 エラー** — constraint 違反（未実装インターフェース）
- **E0423 エラー** — coherence 違反（重複 `impl`）
- **stdlib 使用例** — `List.sort`・`List.max` の `where Ord` 制約
- **`generics.mdx` との役割分担** — `generics.mdx` は基礎、`bounded-generics.mdx` は本番品質化

```favnir
interface Serializable {
  fn to_json(self: Self) -> String
}

fn serialize_all<T>(items: List<T>) -> List<String>
  where T: Serializable
{
  List.map(items, |x| x.to_json())
}

fn pick<T with Ord with Serialize>(a: T, b: T) -> T {
  if a > b { a } else { b }
}
```

---

### 3. `site/content/docs/language/row-polymorphism.mdx` — 更新

v56.3.0 で追加した**行変数 `<r>` の明示記法**と `{ field: Type | r }` 構文を追記する。

**追記セクション**:

- **行変数の明示（v56.3.0）** — `fn get_name<r>(record: { name: String | r }) -> String`
- **LSP ホバー表示** — 行変数の型が `{ name: String | ... }` 形式でエディタに表示される
- **異なるレコード型への適用例** — `user_name` と `product_name` を同じ関数で処理

```favnir
fn get_name<r>(record: { name: String | r }) -> String {
  record.name
}

let user_name    = get_name({ name: "Alice", age: 30 })
let product_name = get_name({ name: "Widget", price: 9.99 })
```

LSP ホバー（型が推論された関数呼び出し上でホバー）:
```
get_name : { name: String | ... } -> String
```

---

### 4. `site/content/docs/language/effect-inference.mdx` — 更新

v56.4.0 で追加した**LSP inlay hints** と `fav check --show-types` への統合を追記する。

**追記セクション**:

- **エディタ inlay hints（v56.4.0）** — エフェクト注釈を省略した関数定義でのインライン表示
- **`fav check --show-types`** — 推論エフェクトを型情報として出力

```favnir
// エフェクト注釈を省略
fn load_data() -> List<Row> {
  bind rows <- kafka.consume("orders")
  bind _ <- snowflake.insert(rows)
  rows
}
// エディタ inlay hint: fn load_data() -> List<Row> /*!Kafka !Snowflake*/
```

`fav check --show-types` の出力例:
```
fn load_data   inferred: !Kafka !Snowflake
fn pure_fn     inferred: (none)
```

---

### 5. `fav/src/driver.rs` — `v56800_tests` 追加

`v56700_tests` の直前に挿入する。

**テスト 1: `docs_bounded_generics_page_exists`**

```rust
#[test]
fn docs_bounded_generics_page_exists() {
    let content = include_str!(
        "../../site/content/docs/language/bounded-generics.mdx"
    );
    assert!(
        content.contains("Serializable"),
        "bounded-generics.mdx should mention Serializable interface"
    );
    assert!(
        content.contains("E0422"),
        "bounded-generics.mdx should document E0422 error"
    );
}
```

**テスト 2: `docs_row_poly_page_exists`**

```rust
#[test]
fn docs_row_poly_page_exists() {
    let content = include_str!(
        "../../site/content/docs/language/row-polymorphism.mdx"
    );
    assert!(
        content.contains("fn get_name<r>"),
        "row-polymorphism.mdx should document row variable with explicit <r> type param"
    );
}
```

**テスト 3: `docs_effect_inference_updated`**

```rust
#[test]
fn docs_effect_inference_updated() {
    let content = include_str!(
        "../../site/content/docs/language/effect-inference.mdx"
    );
    assert!(
        content.contains("inlay"),
        "effect-inference.mdx should document v56.4.0 inlay hints"
    );
}
```

---

### 6. `fav/src/driver.rs` — バージョンチェックテスト更新

`v56300_tests::cargo_toml_version_is_56_3_0` の期待値を `"56.7.0"` → `"56.8.0"` に更新。

---

---

## テスト仕様

| テスト名 | 検証内容 |
|---------|---------|
| `docs_bounded_generics_page_exists` | `bounded-generics.mdx` が `Serializable` と `E0422` を含む |
| `docs_row_poly_page_exists` | `row-polymorphism.mdx` が `fn get_name<r>` を含む（v56.3.0 行変数記法） |
| `docs_effect_inference_updated` | `effect-inference.mdx` が `inlay` を含む（v56.4.0 inlay hints セクション） |

---

## 完了条件

- `cargo build` コンパイルエラーなし
- `cargo test` 全通過（**3246 tests passed, 0 failed**）
- `cargo clippy -- -D warnings` クリーン
- `docs_bounded_generics_page_exists` pass
- `docs_row_poly_page_exists` pass
- `site/content/docs/language/bounded-generics.mdx` が新規作成されている
- `row-polymorphism.mdx` に `{ field: Type | r }` 行変数記法が追記されている
- `effect-inference.mdx` に v56.4.0 inlay hints セクションが追記されている
- `CHANGELOG.md` に v56.8.0 エントリが追加されている
- `versions/current.md` が v56.8.0 / 3245 tests を反映
- 両ロードマップの v56.8.0 実績を COMPLETE に更新

---

## 備考

- **`bounded-generics.mdx` と `generics.mdx` の役割分担**:
  - `generics.mdx` — 基礎的なジェネリクス（型パラメータ、bounded generics の概要）
  - `bounded-generics.mdx` — 本番品質化（`where` 節・複数 constraint・coherence・stdlib 使用例）
  - 両ファイルは相互リンクするが重複は最小化する
- **`effect-inference.mdx` の既存内容**:
  - v18.1.0 の自動推論（`!Db`/`!IO` 等の手書き不要化）が既に記述済み
  - v56.4.0 の inlay hints・`--show-types` を「エディタ統合」セクションとして末尾に追記する
- **`row-polymorphism.mdx` の既存内容**:
  - `with { id: Int }` 形式の基本的な行多相は既に記述済み
  - v56.3.0 の `{ field: Type | r }` 行変数明示を「行変数の明示（v56.3.0）」セクションとして追記する
- **テスト数**: `v56800_tests` に 3 件追加。ベース 3243 + 3 = 3246。
- **`docs_row_poly_page_exists` のアサーション**: `"fn get_name<r>"` を使用 — 既存 `row-polymorphism.mdx`
  には存在しない文字列であり、T3 実施前にテストが通過しないことを保証する（T0 で確認）。
- **`docs_effect_inference_updated` のアサーション**: `"inlay"` を使用 — 既存 `effect-inference.mdx`
  には存在しない文字列であり、T4 実施前にテストが通過しないことを保証する（T0 で確認）。
