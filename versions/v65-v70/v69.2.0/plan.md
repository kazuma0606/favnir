# v69.2.0 実装計画 — Playground WASM 版 AI パイプライン

Status: DRAFT
Version: 69.2.0

---

## 実装ステップ

### Step 1: `site/content/playground/ai-examples.mdx` 作成

`site/content/playground/` ディレクトリを作成し `ai-examples.mdx` を新規作成。

内容の要点:
- 4 種類のプリセットサンプル（Linear Algebra / Stats / LLM Extract / Semantic Search）
- AI Rune モック（`Rune.embed.mock` / `Rune.llm.mock`）の説明
- `Rune.stats` / `Rune.linalg` の WASM 動作説明
- コスト見積もり表示の説明
- MDX として valid（先頭に ESM import 行なし）

必須キーワード:
- `"mock"` — テスト `playground_ai_wasm_examples` で検証
- `"Rune.stats"` — テスト `playground_math_rune_wasm` で検証
- `"Rune.linalg"` — テスト `playground_math_rune_wasm` で検証
- `"WASM"` — テスト `playground_math_rune_wasm` で検証

### Step 2: `driver.rs` — `v69200_tests` 追加

`v69100_tests` ブロックの直前に挿入（driver.rs は降順配置）。

テスト 2 件:
- `playground_ai_wasm_examples`: `include_str!("../../site/content/playground/ai-examples.mdx")` → `"mock"` assert
- `playground_math_rune_wasm`: 同ファイルを読み込み → `"Rune.stats"` / `"Rune.linalg"` / `"WASM"` を個別 assert

- `cargo build` でコンパイルエラーがないことを確認

### Step 3: テスト実行

```bash
cargo test --bin fav v69200_tests  # 2 件 PASS
cargo test -j 8 -- --test-threads=8  # 3545 tests PASS
```

---

## ファイルパス参照（include_str! 基準）

`driver.rs` は `fav/src/driver.rs` のため:
- `include_str!("../../site/content/playground/ai-examples.mdx")` → repo root の `site/content/playground/ai-examples.mdx`

> 確認済みパターン: stdlib ドキュメントで `include_str!("../../site/content/docs/stdlib/datetime.mdx")` を使用

---

## sub-version ポリシー

v69.x では `Cargo.toml` / `CHANGELOG.md` は変更しない。v70.0.0 宣言時に一括更新する。
