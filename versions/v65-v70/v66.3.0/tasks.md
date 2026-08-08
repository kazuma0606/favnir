# v66.3.0 タスクリスト

Status: COMPLETE
Version: 66.3.0
Base tests: 3479
Target tests: 3481
Actual tests: 3481

---

## T0: 事前確認

- [x] `cargo test -j 8 -- --test-threads=8` でベース 3479 tests passed, 0 failed を確認
- [x] `fav/Cargo.toml` の version が `"66.0.0"` であることを確認（sub-version では変更しない）
- [x] `runes/embed/` ディレクトリが存在しないことを確認（新規作成対象）
- [x] `driver.rs` に `v66200_tests` が存在することを確認（`v66300_tests` の挿入位置）
- [x] `driver.rs` に `v66300_tests` が存在しないことを確認（新規追加）
- [x] `cargo test --bin fav v66200_tests` で 2 件 PASS することを確認（前バージョンが正常）
  - 前バージョンのテスト関数名: `llm_extract_typed_schema`, `llm_extract_schema_mismatch_error`
- [x] `versions/current.md` の「進行中バージョン」が `v66.2.0` であることを確認

---

## T1: Rune ファイル作成

- [x] `runes/embed/` ディレクトリ作成
- [x] `runes/embed/rune.toml` 作成（`entry` / `effects = []` / `[dependencies]` 形式）
- [x] `runes/embed/embed.fav` 作成（以下の全 5 関数を定義）
  - **プロバイダー別埋め込み**
  - [x] `openai` — OpenAI 埋め込み（`[]` を返すスタブ）
  - [x] `cohere` — Cohere 埋め込み（`[]` を返すスタブ）
  - [x] `local` — ローカルモデル埋め込み（`[]` を返すスタブ、コメントに `EmbedLocalProvider` を含む）**※ コメントを変更・削除した場合は T2 の `embed_rune_local_model` テストも連動更新すること**
  - **バッチ処理**
  - [x] `embed_batch` — バッチ埋め込み（`[]` を返すスタブ）
  - **キャッシュ付き**
  - [x] `embed_cached` — キャッシュ付き埋め込み（`[]` を返すスタブ）
- [x] `embed.fav` 内に `let ` が含まれないことを確認
- [x] `embed.fav` 内に `bind.*=`（`<-` でない bind）が含まれないことを確認
- [x] `embed.fav` 内に `Float.from_int` / `Float.sqrt` が含まれないことを確認
- [x] `grep -c 'public fn ' embed.fav` で 5 が出ることを確認

---

## T2: `driver.rs` — `v66300_tests` 追加

- [x] `// -- v66200_tests (v66.2.0)` コメントの直前に `v66300_tests` を挿入
  - [x] `embed_rune_openai` — `fn openai(` / `fn cohere(` / `fn embed_batch(` を含む
  - [x] `embed_rune_local_model` — `fn local(` / `fn embed_cached(` / `EmbedLocalProvider` を含む
- [x] `use super::*` は不要（`include_str!` のみ使用）
- [x] `cargo build` でエラーなし

---

## T3: ビルド・テスト

- [x] `cargo test --bin fav v66300_tests` で 2 件 PASS
  - [x] `embed_rune_openai` PASS
  - [x] `embed_rune_local_model` PASS
- [x] `cargo test -j 8 -- --test-threads=8` で 3481 tests passed, 0 failed を確認

---

## T4: ドキュメント・ステータス更新

- [x] `versions/roadmap/roadmap-v66.1-v67.0.md` のバージョン一覧表で v66.3.0 の「状態」列を「完了」に変更
- [x] `versions/current.md` の「進行中バージョン」を v66.3.0 に更新
- [x] 本 `tasks.md` を COMPLETE に更新（全チェックボックスを `[x]` に）

> **CHANGELOG 方針**: v66.1〜v66.9 では CHANGELOG.md を更新しない。v67.0.0 宣言時に一括追記する。
> **MDX 方針**: `site/` の MDX ドキュメントは v66.9.0 安定化時に一括作成するため今バージョンは省略。

---

## コードレビュー指摘と対応

- [MED] `rune.toml` の `effects = []` はスタブ段階では許容。`openai`/`cohere` は本番実装時に `!Http`/`!Llm` が必要 → `embed.fav` ヘッダーに TODO コメントを追加して対応済み
- [LOW] `embed_rune_openai` テスト関数名が `cohere`/`embed_batch` も検証しておりコンテンツと不一致 → ロードマップ `roadmap-v66.1-v67.0.md` L181 に `fn embed_rune_openai() // embed.fav に openai / cohere / embed_batch 定義を含む` と仕様化されているため変更せず。意図的な設計として受け入れ
