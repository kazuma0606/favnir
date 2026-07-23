# Spec: v49.5.0 — cookbook 更新

## 概要

v46〜v49 の新機能を活用したクックブックレシピを 3 件追加する。
`return` ガード節・`#[test]` インラインテスト・新 import 構文それぞれのレシピを
`site/content/cookbook/` に新規作成し、Rust テスト 2 件でファイルの存在と内容を検証する。

---

## 変更ファイル

| ファイル | 変更内容 |
|---|---|
| `site/content/cookbook/return-guard-pattern.mdx` | 新規作成（`return` ガード節パターンレシピ）|
| `site/content/cookbook/inline-testing.mdx` | 新規作成（`#[test]` インラインテストレシピ）|
| `site/content/cookbook/modular-pipelines.mdx` | 新規作成（新 import 構文モジュール化レシピ）|
| `fav/src/driver.rs` | `v495000_tests` 追加（2テスト）|
| `fav/Cargo.toml` | version → `"49.5.0"` |
| `CHANGELOG.md` | v49.5.0 エントリ追加 |

---

## cookbook ファイル内容

既存 cookbook の frontmatter 形式（`stdlib-v2.mdx` 参照）に準拠:
- `title` / `category: "クックブック"` / `description`（`order` フィールドなし）

### `site/content/cookbook/return-guard-pattern.mdx`

frontmatter:
- `title: "Return Guard Pattern"`
- `category: "クックブック"`
- `description: "return ガード節で早期リターンを使ったバリデーションパターン"`

本文要件:
- H1: "Return Guard Pattern"
- `return expr if condition` の説明と Favnir コード例（`validate_order` 関数）
- pipeline でのコード例（`bind valid <- validate_order(order)`）
- "guard" キーワードを英語で含む（セクション見出しまたは本文）
- "return Result" パターンのコード例を含む（テスト assert 条件）
- `return Result.err(...)` と `return Result.ok(...)` のイディオムを示す

### `site/content/cookbook/inline-testing.mdx`

frontmatter:
- `title: "Inline Testing"`
- `category: "クックブック"`
- `description: "#[test] インラインテストで fav test を活用するレシピ"`

本文要件:
- H1: "Inline Testing"
- `#[test]` アノテーションのコード例（テスト assert 条件）
- `assert_eq(...)` の使用例（テスト assert 条件）
- `fav test` コマンドの実行例（`fav test ./pipeline.fav`）
- Result 型のテスト例（`assert_eq(validate_order(good), Result.ok(good))`）

### `site/content/cookbook/modular-pipelines.mdx`（テスト対象外）

frontmatter:
- `title: "Modular Pipelines"`
- `category: "クックブック"`
- `description: "新 import 構文でパイプラインをモジュール化するレシピ"`

本文要件:
- ローカル import 例（`import "./stages/validate" as validate`）
- パッケージ import 例（`import kafka`）
- ディレクトリ構成例

---

## テスト（+2）

`v495000_tests` を `v494000_tests` の直前に追加:

```rust
#[cfg(test)]
mod v495000_tests {
    #[test]
    fn cookbook_return_guard_exists() {
        let content = include_str!("../../site/content/cookbook/return-guard-pattern.mdx");
        assert!(
            content.contains("return Result") && content.contains("guard"),
            "cookbook/return-guard-pattern.mdx should contain return Result guard recipe"
        );
    }

    #[test]
    fn cookbook_fav_test_exists() {
        let content = include_str!("../../site/content/cookbook/inline-testing.mdx");
        assert!(
            content.contains("#[test]") && content.contains("assert"),
            "cookbook/inline-testing.mdx should contain fav test recipe"
        );
    }
}
```

テスト数: 3077 → **3079**（+2）

---

## 注意事項

- cookbook の frontmatter は `order` フィールドなし（既存 `stdlib-v2.mdx` に準拠）
- `cookbook_return_guard_exists`: `"return"` かつ `"guard"` の両方を確認（v49.4.0 の教訓を踏まえ英語キーワード `"guard"` を本文に明記すること）
- `cookbook_fav_test_exists`: `"#[test]"` かつ `"assert"` の両方を確認（`fav test` コマンド名より `#[test]` アノテーションが構文的に一意）
- `modular-pipelines.mdx` はテスト対象外だが、ロードマップ記載のため作成必須
- `include_str!` パス: `fav/src/driver.rs` → `../../site/content/cookbook/`（リポジトリルートの `site/`）
- ロードマップの推定テスト数 3072 は旧推定値（v49.3.0 実績 3075 がベースの計算）。v49.4.0 の実績が 3077 となったため、本バージョン完了後は 3077 + 2 = **3079** が正しい。ロードマップの推定欄は実績確定後に上書きする

---

## 完了条件

- `cargo test` 3079 passed, 0 failed（3077 + 2 件）
- `cargo clippy -- -D warnings` クリーン
- `fav/Cargo.toml` version → `"49.5.0"`
- `CHANGELOG.md` に v49.5.0 エントリ追加（3 cookbook ファイル新規作成を明記）
- `versions/current.md` を v49.5.0（3079 tests）に更新、進行中バージョンを `v49.6.0` に更新
- `versions/roadmap/roadmap-v49.1-v50.0.md` の v49.5.0 実績を記入
- `tasks.md` を COMPLETE に更新（T0〜T3 全 `[x]`）
