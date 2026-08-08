# v66.2.0 タスクリスト

Status: COMPLETE
Version: 66.2.0
Base tests: 3477
Target tests: 3479
Actual tests: 3479

---

## T0: 事前確認

- [x] `cargo test -j 8 -- --test-threads=8` でベース 3477 tests passed, 0 failed を確認
- [x] `fav/Cargo.toml` の version が `"66.0.0"` であることを確認（sub-version では変更しない）
- [x] `runes/llm/llm_extract.fav` が存在しないことを確認（新規作成対象）
- [x] `driver.rs` に `v66100_tests` が存在することを確認（`v66200_tests` の挿入位置）
- [x] `driver.rs` に `v66200_tests` が存在しないことを確認（新規追加）
- [x] `cargo test --bin fav v66100_tests` で 2 件 PASS することを確認（前バージョンが正常）
- [x] `versions/current.md` の「進行中バージョン」が `v66.1.0` であることを確認

---

## T1: Rune ファイル作成

- [x] `runes/llm/llm_extract.fav` 作成（以下の全 4 関数を定義）
  - **基本抽出**
  - [x] `extract` — スキーマ付き単一レコード抽出（`""` を返すスタブ）
  - [x] `extract_list` — スキーマ付き複数レコード抽出（`[]` を返すスタブ）
  - **フォールバック**
  - [x] `extract_or_default` — デフォルト値付き抽出（`default_val` を返すスタブ、コメントに `LLMExtractionFallback` を含む）
  - [x] `extract_maybe` — Option 型抽出（`""` を返すスタブ）
- [x] `llm_extract.fav` 内に `let ` が含まれないことを確認
- [x] `llm_extract.fav` 内に `bind.*=`（`<-` でない bind）が含まれないことを確認
- [x] `llm_extract.fav` 内に `Float.from_int` / `Float.sqrt` が含まれないことを確認
- [x] `grep -c 'public fn ' llm_extract.fav` で 4 が出ることを確認
- [x] 既存 `runes/llm/llm.fav` / `client.fav` / `rune.toml` を変更していないことを確認

---

## T2: `driver.rs` — `v66200_tests` 追加

- [x] `// -- v66100_tests (v66.1.0)` コメントの直前に `v66200_tests` を挿入
  - [x] `llm_extract_typed_schema` — `fn extract(` / `fn extract_list(` / `schema` を含む
  - [x] `llm_extract_schema_mismatch_error` — `fn extract_or_default(` / `fn extract_maybe(` / `LLMExtractionFallback` を含む
- [x] `use super::*` は不要（`include_str!` のみ使用）
- [x] `cargo build` でエラーなし

---

## T3: ビルド・テスト

- [x] `cargo test --bin fav v66200_tests` で 2 件 PASS
  - [x] `llm_extract_typed_schema` PASS
  - [x] `llm_extract_schema_mismatch_error` PASS
- [x] `cargo test -j 8 -- --test-threads=8` で 3479 tests passed, 0 failed を確認

---

## T4: ドキュメント・ステータス更新

- [x] `versions/roadmap/roadmap-v66.1-v67.0.md` のバージョン一覧表で v66.2.0 の「状態」列を「完了」に変更
- [x] `versions/current.md` の「進行中バージョン」を v66.2.0 に更新
- [x] 本 `tasks.md` を COMPLETE に更新（全チェックボックスを `[x]` に）

> **CHANGELOG 方針**: v66.1〜v66.9 では CHANGELOG.md を更新しない。v67.0.0 宣言時に一括追記する。
> **MDX 方針**: `site/` の MDX ドキュメントは v66.9.0 安定化時に一括作成するため今バージョンは省略。

---

## コードレビュー指摘と対応

<!-- 実装完了後に追記 -->
