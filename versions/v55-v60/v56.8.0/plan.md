# Plan — v56.8.0 — ドキュメントサイト Language Power 2.0 記事

## 実装順序

```
Cargo.toml → bounded-generics.mdx（新規） → row-polymorphism.mdx（更新）
→ effect-inference.mdx（更新） → driver.rs（テスト追加 + バージョンチェック更新）
```

依存関係:
- MDX ファイルは互いに独立（並行作成可能）
- `driver.rs` のテスト（`include_str!`）はMDXファイル作成後でないとビルドエラーになる
- `Cargo.toml` 更新は最初に行う

---

## Step 1: `fav/Cargo.toml` — バージョン更新

```toml
version = "56.8.0"
```

---

## Step 2: `site/content/docs/language/bounded-generics.mdx` — 新規作成

`generics.mdx` の「Bounded Generics」セクションを出発点に、
v56.1.0 / v56.2.0 の実装内容を本番品質のドキュメントとして整理する。

**構成**:
1. 概要（v56.1.0 正式化の背景）
2. `where T: Interface` 構文（`T with Interface` との等価性を明記）
3. 複数 constraint（`T with Ord with Serialize`）
4. カスタム interface との組み合わせ（`Serializable` 例）
5. stdlib での使用例（`List.sort`、`List.max`）
6. E0422 — constraint 違反エラー
7. E0423 — coherence 違反（重複 `impl`）エラー
8. `generics.mdx` への参照リンク

**ファイルパス**: `site/content/docs/language/bounded-generics.mdx`

---

## Step 3: `site/content/docs/language/row-polymorphism.mdx` — 更新

既存ファイルの末尾に「行変数の明示（v56.3.0）」セクションを追記する。

**追記位置**: ファイル末尾（最後の表の後）

**追記内容**:

```markdown
## 行変数の明示（v56.3.0）

v56.3.0 以降、行変数 `<r>` を型パラメータとして明示し、`{ field: Type | r }` 構文で
レコードの「残りのフィールド」を表現できる。

\```favnir
fn get_name<r>(record: { name: String | r }) -> String {
  record.name
}

let user_name    = get_name({ name: "Alice", age: 30 })
let product_name = get_name({ name: "Widget", price: 9.99 })
\```

`{ name: String | r }` は「`name: String` フィールドを持ち、残りのフィールドは `r` で表される任意のレコード」を意味する。

## LSP ホバー表示

LSP 対応エディタでは、行変数を含む関数の型を `{ name: String | ... }` 形式で表示する。

\```
get_name : { name: String | ... } -> String
\```
```

---

## Step 4: `site/content/docs/language/effect-inference.mdx` — 更新

既存ファイルの末尾（「注意事項」セクションの後）に「エディタ統合（v56.4.0）」セクションを追記する。

**追記内容**:

```markdown
## エディタ統合（v56.4.0）

v56.4.0 以降、LSP 対応エディタでエフェクト注釈を省略した関数定義に推論エフェクトを
**inlay hints** としてインライン表示する。

\```favnir
// エフェクト注釈を省略
fn load_data() -> List<Row> {
  bind rows <- kafka.consume("orders")
  bind _ <- snowflake.insert(rows)
  rows
}
// エディタ inlay hint: fn load_data() -> List<Row> /*!Kafka !Snowflake*/
\```

### `fav check --show-types`

`fav check --show-types` を使うと、推論されたエフェクトセットを型情報として確認できる。

\```sh
fav check src/pipeline.fav --show-types
\```

出力例:

\```
fn load_data   inferred: !Kafka !Snowflake
fn pure_fn     inferred: (none)
fn wrap        inferred: !Kafka !Snowflake
\```
```

---

## Step 5: `fav/src/driver.rs` — `v56800_tests` 追加

`v56700_tests` モジュールの直前に挿入する。

```rust
// -- v56800_tests (v56.8.0) -- Language Power 2.0 docs --
#[cfg(test)]
mod v56800_tests {
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
}
```

---

## Step 6: `fav/src/driver.rs` — バージョンチェックテスト更新

`v56300_tests::cargo_toml_version_is_56_3_0` の期待値を更新:

```rust
// 変更前
cargo_toml.contains("version = \"56.7.0\"")
"Cargo.toml version should be 56.7.0, got: {}"

// 変更後
cargo_toml.contains("version = \"56.8.0\"")
"Cargo.toml version should be 56.8.0, got: {}"
```

---

## ポスト処理

1. `CHANGELOG.md` に v56.8.0 エントリ追加
2. `versions/current.md` を v56.8.0 / 3245 tests に更新
3. `versions/roadmap/roadmap-v56.1-v57.0.md` の v56.8.0 実績を COMPLETE に更新
4. `versions/roadmap/roadmap-v55.1-v60.0.md` の v56.8.0 実績欄も COMPLETE に更新

---

## リスク・注意点

| リスク | 対策 |
|---|---|
| `include_str!` のパスが `driver.rs` から見た相対パスと合わない | `../../site/content/docs/language/bounded-generics.mdx` — `fav/src/driver.rs` → `fav/` → プロジェクトルート → `site/` の 2 段上り |
| `row-polymorphism.mdx` の既存テーブル末尾への追記で Markdown 壊れる | 追記前に既存ファイルを読み込み末尾を確認してから `\n\n` で区切る |
| `bounded-generics.mdx` が `generics.mdx` と内容重複 | 概要のみ `generics.mdx` に残し、詳細は `bounded-generics.mdx` に委譲する（相互リンク） |
| v56.4.0 の inlay hints が実際の LSP 実装と差異がある | ドキュメントは「エディタに表示されるイメージ」として記述する（テスト検証なし） |
