# v69.2.0 仕様書 — Playground WASM 版 AI パイプライン

Status: DRAFT
Version: 69.2.0
Date: 2026-08-07

---

## 概要

ブラウザ上で AI パイプラインを試せる Playground の AI 対応拡張。
Math Rune（linalg / stats / autodiff）と AI Rune モック（embed / llm / vector）を
ブラウザ（WASM）で動作させるサンプルと説明ドキュメントを追加する。

---

## スコープ

### IN（本バージョンで実施）

- `site/content/playground/ai-examples.mdx` 新規作成
  - `"mock"` キーワードを含む（`playground_ai_wasm_examples` テスト要件）
  - `"Rune.stats"` / `"Rune.linalg"` / `"WASM"` キーワードを含む（`playground_math_rune_wasm` テスト要件）
  - `Rune.embed.mock` / `Rune.llm.mock` の説明ドキュメント（実際のモック実装は将来フェーズ）
  - 4 種類のプリセットサンプル（Linear Algebra / Stats / LLM Extract / Semantic Search）の説明
  - コスト見積もり表示機能の説明
- `driver.rs` に `v69200_tests` 2 件を追加（`v69100_tests` の直前）

### OUT（本バージョンでは実施しない）

- `@favnir/wasm` パッケージの実際のビルド・更新: 将来フェーズ
- 実際のブラウザ動作実装（WASM バイナリ生成）: 将来フェーズ
- Playground UI の実装（Next.js コンポーネント）: 将来フェーズ
- Cargo.toml / CHANGELOG.md の変更: v70.0.0 宣言時に一括更新（sub-version ポリシー）

---

## 成果物仕様

### `site/content/playground/ai-examples.mdx`

以下のキーワードを含むこと（テスト要件）:
- `"mock"` — AI Rune モック実装（`playground_ai_wasm_examples` テスト要件）
- `"Rune.stats"` — Stats Rune のサンプル（`playground_math_rune_wasm` テスト要件）
- `"Rune.linalg"` — Linear Algebra Rune のサンプル（`playground_math_rune_wasm` テスト要件）
- `"WASM"` — WASM 動作の説明（`playground_math_rune_wasm` テスト要件）

内容:
- Playground 概要（https://favnir.dev/playground）
- 4 種類のプリセットサンプル説明
  1. Linear Algebra Demo（`Rune.linalg`）
  2. Statistics Pipeline（`Rune.stats`）
  3. LLM Extraction（mock）
  4. Semantic Search（mock）
- AI Rune モック説明:
  - `Rune.embed.mock(text)` → ランダム正規化ベクトルを返す
  - `Rune.llm.mock(text, schema)` → schema のデフォルト値を返す
- WASM 動作の説明（Math Rune は pure 関数のため WASM で実動作、AI Rune はモック）
- コスト見積もり表示（本番 API 使用時の推定コスト）

---

## テスト仕様

### `v69200_tests`（2 件、3543 + 2 = **3545**）

```rust
fn playground_ai_wasm_examples()
// include_str!("../../site/content/playground/ai-examples.mdx") で読み込み
// "mock" を assert!

fn playground_math_rune_wasm()
// include_str!("../../site/content/playground/ai-examples.mdx") で読み込み
// "Rune.stats" / "Rune.linalg" / "WASM" を個別 assert!
```

---

## 完了条件

- `cargo test --bin fav v69200_tests` で 2 件 PASS
- `cargo test -j 8 -- --test-threads=8` で **3545 tests passed, 0 failed**
- `versions/roadmap/roadmap-v69.1-v70.0.md` の v69.2.0 行の状態が「完了」になっていること
- `versions/current.md` の「進行中バージョン」が `v69.2.0` に更新されていること（T4 で対応）
