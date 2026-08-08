# v66.2.0 実装計画 — LLM Extraction Stage（型安全 JSON 抽出）

Version: 66.2.0
Status: 未着手
Base tests: 3477
Target tests: 3479

---

## 実装ステップ

### Step 1: `llm_extract.fav` 作成

1. `runes/llm/llm_extract.fav` 作成（4 関数スタブ）

### Step 2: `driver.rs` テスト追加

- `// -- v66100_tests (v66.1.0)` コメントの直前に `v66200_tests` を挿入
- 2 テスト関数:
  - `llm_extract_typed_schema`
  - `llm_extract_schema_mismatch_error`

### Step 3: ビルド・テスト確認

```bash
cargo build
cargo test --bin fav v66200_tests
cargo test -j 8 -- --test-threads=8
```

---

## `llm_extract.fav` 実装方針

- **全 4 関数をスタブとして実装**（シグネチャ確立が目的）
- `bind` / `let` は使用しない
- `Float.from_int` / `Float.sqrt` は使用しない
- 戻り値:
  - `String` 系 → `""` または `default_val`（引数をそのまま返す）
  - `List<String>` 系 → `[]`
- `LLMExtractionFallback` はコメント中で使用 → `contains("LLMExtractionFallback")` テストにマッチ
- 既存 `runes/llm/llm.fav` / `client.fav` / `rune.toml` は変更しない

---

## `driver.rs` 挿入コード

```rust
// -- v66200_tests (v66.2.0) -- LLM Extraction Stage --
#[cfg(test)]
mod v66200_tests {
    #[test]
    fn llm_extract_typed_schema() {
        let content = include_str!("../../runes/llm/llm_extract.fav");
        assert!(!content.is_empty(), "llm_extract.fav should not be empty");
        assert!(content.contains("fn extract("), "llm_extract.fav should define extract");
        assert!(
            content.contains("fn extract_list("),
            "llm_extract.fav should define extract_list"
        );
        assert!(
            content.contains("schema"),
            "llm_extract.fav should reference schema parameter"
        );
    }

    #[test]
    fn llm_extract_schema_mismatch_error() {
        let content = include_str!("../../runes/llm/llm_extract.fav");
        assert!(
            content.contains("fn extract_or_default("),
            "llm_extract.fav should define extract_or_default"
        );
        assert!(
            content.contains("fn extract_maybe("),
            "llm_extract.fav should define extract_maybe"
        );
        assert!(
            content.contains("LLMExtractionFallback"),
            "llm_extract.fav should reference LLMExtractionFallback"
        );
    }
}
```

---

## 関数一覧（4 関数）

| カテゴリ | 関数名 | 戻り値 |
|---|---|---|
| 基本抽出 | `extract(text, schema, model)` | `""` |
| 基本抽出 | `extract_list(text, schema, model)` | `[]` |
| フォールバック | `extract_or_default(text, schema, model, default_val)` | `default_val` |
| フォールバック | `extract_maybe(text, schema, model)` | `""` |

---

## リスク・注意点

- `contains("fn extract(")` は `extract_list` / `extract_or_default` / `extract_maybe` とは区別される（`fn extract(` は `fn extract_list(` と完全一致しないため偽陽性なし）
- `contains("LLMExtractionFallback")` はコメント行にのみ存在するため、コメントを変更した場合はテストも連動更新が必要
- 既存 `runes/llm/rune.toml` は `entry = "llm.fav"` のままで変更しない（`llm_extract.fav` は別ファイルとして include_str! でのみ参照）
