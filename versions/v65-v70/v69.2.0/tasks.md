# v69.2.0 タスクリスト

Status: COMPLETE
Version: 69.2.0
Note: Playground WASM 版 AI パイプライン — site/content/playground/ai-examples.mdx 作成 + 2 テスト追加
Base tests: 3543
Target tests: 3545

---

## T0: 事前確認

- [x] `cargo test -j 8 -- --test-threads=8` でベース 3543 tests passed, 0 failed を確認
- [x] `fav/Cargo.toml` の version が `"69.0.0"` であることを確認（sub-version では変更しない）
- [x] `driver.rs` に `v69100_tests` が存在することを確認（`v69200_tests` の挿入位置）
  - 注意: driver.rs のテストブロックは降順配置（新しいものが上）。`v69200_tests` を `v69100_tests` の直前に挿入する
- [x] `driver.rs` に `v69200_tests` が存在しないことを確認（新規追加）
- [x] `versions/current.md` の「進行中バージョン」が `v69.1.0` であることを確認（本バージョン T4 で v69.2.0 に更新する）
- [x] `cargo test --bin fav v69100_tests` で 2 件 PASS することを確認（前バージョンが正常）
  - [x] `driver.rs` に `ai_etl_e2e_demo_structure` が存在することを確認
  - [x] `driver.rs` に `ai_etl_demo_has_all_stages` が存在することを確認
- [x] `site/content/playground/` ディレクトリが存在しないことを確認（新規作成）

---

## T1: `site/content/playground/ai-examples.mdx` 作成

- [x] `site/content/playground/` ディレクトリを作成（なければ）
- [x] `ai-examples.mdx` を新規作成
  - [x] `"mock"` キーワードを含む（`playground_ai_wasm_examples` テスト要件）
  - [x] `"Rune.stats"` キーワードを含む（`playground_math_rune_wasm` テスト要件）
  - [x] `"Rune.linalg"` キーワードを含む（`playground_math_rune_wasm` テスト要件）
  - [x] `"WASM"` キーワードを含む（`playground_math_rune_wasm` テスト要件）
  - [x] 4 プリセットサンプル（Linear Algebra / Stats / LLM Extract / Semantic Search）の説明を含む
  - [x] AI Rune モック（Rune.embed.mock / Rune.llm.mock）の説明ドキュメントを含む
  - [x] MDX として valid（先頭に ESM import 行なし）
- [x] `include_str!("../../site/content/playground/ai-examples.mdx")` でアクセスできることを確認

---

## T2: `driver.rs` — `v69200_tests` 追加

- [x] `// -- v69100_tests (v69.1.0) -- E2E デモ（AI ETL） --` の直前に挿入
  - [x] `playground_ai_wasm_examples`:
    - [x] `include_str!("../../site/content/playground/ai-examples.mdx")` でファイルを読み込む
    - [x] `"mock"` を個別 `assert!` で検証
  - [x] `playground_math_rune_wasm`:
    - [x] `include_str!("../../site/content/playground/ai-examples.mdx")` でファイルを読み込む
    - [x] `"Rune.stats"` を個別 `assert!` で検証
    - [x] `"Rune.linalg"` を個別 `assert!` で検証
    - [x] `"WASM"` を個別 `assert!` で検証
- [x] `cargo build` でエラーなし

---

## T3: ビルド・テスト

- [x] `cargo test --bin fav v69200_tests` で 2 件 PASS
  - [x] `playground_ai_wasm_examples` PASS
  - [x] `playground_math_rune_wasm` PASS
- [x] `cargo test -j 8 -- --test-threads=8` で 3545 tests passed, 0 failed を確認

---

## T4: ドキュメント・ステータス更新

- [x] 1. `versions/roadmap/roadmap-v69.1-v70.0.md` の v69.2.0「状態」列を「未着手」→「完了」に変更
- [x] 2. `versions/current.md` の「進行中バージョン」を v69.2.0 に更新
- [x] 3. 本 `tasks.md` を COMPLETE に更新（T0 を含む全チェックボックスを `[x]` に）← 最後に実施

> **sub-version ポリシー**: v69.x では Cargo.toml / CHANGELOG.md は変更しない。v70.0.0 宣言時に一括更新する。

---

## 設計上の意図的省略

- `@favnir/wasm` パッケージの実際のビルド・更新: 将来フェーズ
- 実際のブラウザ動作実装（WASM バイナリ生成）: 将来フェーズ
- Playground UI の実装（Next.js コンポーネント）: 将来フェーズ
- AI Rune モックの実際の実装: 将来フェーズ（今バージョンは説明ドキュメントのみ）
