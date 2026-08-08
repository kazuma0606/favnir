# v66.3.0 実装計画 — Embedding Pipeline Rune（`Rune.embed`）

Version: 66.3.0
Status: 未着手
Base tests: 3479
Target tests: 3481

---

## 実装ステップ

### Step 1: ディレクトリ・ファイル作成

1. `runes/embed/` ディレクトリ作成
2. `runes/embed/rune.toml` 作成
3. `runes/embed/embed.fav` 作成（全 5 関数）

### Step 2: `driver.rs` テスト追加

- `// -- v66200_tests (v66.2.0)` コメントの直前に `v66300_tests` を挿入
- 2 テスト関数:
  - `embed_rune_openai`
  - `embed_rune_local_model`

### Step 3: ビルド・テスト確認

```bash
# 以下は順番に実行すること（前コマンドが PASS してから次へ進む）
cargo build
cargo test --bin fav v66300_tests
cargo test -j 8 -- --test-threads=8
```

---

## `embed.fav` 実装方針

- **全 5 関数をスタブとして実装**（シグネチャ確立が目的）
- `bind` / `let` は使用しない
- `Float.from_int` / `Float.sqrt` は使用しない
- 戻り値:
  - `List<Float>` 系 → `[]`
  - `List<List<Float>>` 系 → `[]`
- `EmbedLocalProvider` はコメント中で使用 → `contains("EmbedLocalProvider")` テストにマッチ

## `rune.toml` 形式

```toml
[rune]
name        = "embed"
version     = "0.1.0"
description = "Embedding Pipeline Rune for Favnir — OpenAI / Cohere / local (Ollama) unified embedding interface with batch and cache support"
entry       = "embed.fav"
effects     = []

[dependencies]
```

---

## `driver.rs` 挿入コード

```rust
// -- v66300_tests (v66.3.0) -- Embedding Pipeline Rune --
#[cfg(test)]
mod v66300_tests {
    #[test]
    fn embed_rune_openai() {
        let content = include_str!("../../runes/embed/embed.fav");
        assert!(!content.is_empty(), "embed.fav should not be empty");
        assert!(content.contains("fn openai("), "embed.fav should define openai");
        assert!(content.contains("fn cohere("), "embed.fav should define cohere");
        assert!(
            content.contains("fn embed_batch("),
            "embed.fav should define embed_batch"
        );
    }

    #[test]
    fn embed_rune_local_model() {
        let content = include_str!("../../runes/embed/embed.fav");
        assert!(content.contains("fn local("), "embed.fav should define local");
        assert!(
            content.contains("fn embed_cached("),
            "embed.fav should define embed_cached"
        );
        assert!(
            content.contains("EmbedLocalProvider"),
            "embed.fav should reference EmbedLocalProvider"
        );
    }
}
```

---

## 関数一覧（5 関数）

| カテゴリ | 関数名 | 戻り値 |
|---|---|---|
| プロバイダー別 | `openai(text: String, model: String)` | `[]` |
| プロバイダー別 | `cohere(text: String, model: String)` | `[]` |
| プロバイダー別 | `local(text: String, model: String)` | `[]`（コメントに `EmbedLocalProvider`）|
| バッチ処理 | `embed_batch(texts: List<String>, model: String)` | `[]` |
| キャッシュ付き | `embed_cached(text: String, model: String, cache_key: String)` | `[]` |

---

## リスク・注意点

- `contains("fn embed_batch(")` は `fn embed_cached(` と区別可能（文字列一致で偽陽性なし）
- `contains("EmbedLocalProvider")` はコメント行にのみ存在するため、コメントを変更した場合はテストも連動更新が必要
- `Vec<Float>[N]` 次元型パラメータは未定義のため `List<Float>` で代替（型チェックエラーは無視）
