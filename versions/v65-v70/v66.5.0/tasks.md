# v66.5.0 タスクリスト

Status: COMPLETE
Version: 66.5.0
Base tests: 3483
Target tests: 3485
Actual tests: 3485

---

## T0: 事前確認

- [x] `cargo test -j 8 -- --test-threads=8` でベース 3483 tests passed, 0 failed を確認
- [x] `fav/Cargo.toml` の version が `"66.0.0"` であることを確認（sub-version では変更しない）
- [x] `runes/inference/` ディレクトリが存在しないことを確認（新規作成対象）
- [x] `driver.rs` に `v66400_tests` が存在することを確認（`v66500_tests` の挿入位置）
- [x] `driver.rs` に `v66500_tests` が存在しないことを確認（新規追加）
- [x] `cargo test --bin fav v66400_tests` で 2 件 PASS することを確認（前バージョンが正常）
  - 前バージョンのテスト関数名: `vector_db_upsert_query`, `vector_db_type_safe_dim`
- [x] `versions/current.md` の「進行中バージョン」が `v66.4.0` であることを確認

---

## T1: Rune ファイル作成

### inference（新規）

- [x] `runes/inference/` ディレクトリ作成
- [x] `runes/inference/rune.toml` 作成（`entry` / `effects = []` / `[dependencies]` 形式）
- [x] `runes/inference/inference.fav` 作成（以下の全 4 関数を定義）
  - [x] `inference_batch(embeddings, model, batch_size)` — `[]` を返すスタブ
  - [x] `stream_with_backpressure(stream, model, buffer_size)` — `[]` を返すスタブ
  - [x] `stream_with_sla(stream, model, max_latency_ms)` — `[]` を返すスタブ
  - [x] `stateful_score(session_id, embedding, model)` — `""` を返すスタブ
  - [x] ヘッダーコメントに `StreamingInferenceInterface` を含む

### 共通確認

- [x] `inference.fav` 内に `let ` が含まれないことを確認
- [x] `inference.fav` 内に `bind.*=`（`<-` でない bind）が含まれないことを確認
- [x] `inference.fav` 内に `Float.from_int` / `Float.sqrt` が含まれないことを確認

---

## T2: `driver.rs` — `v66500_tests` 追加

- [x] `// -- v66400_tests (v66.4.0)` コメントの直前に `v66500_tests` を挿入
  - [x] `streaming_inference_pipeline`:
    - `inference.fav` に `"fn inference_batch("` を含む
    - `inference.fav` に `"fn stream_with_backpressure("` を含む
    - `inference.fav` に `"StreamingInferenceInterface"` を含む
  - [x] `streaming_backpressure_ai`:
    - `inference.fav` に `"fn stream_with_sla("` を含む
    - `inference.fav` に `"fn stateful_score("` を含む
- [x] `use super::*` は不要（`include_str!` のみ使用）
- [x] `cargo build` でエラーなし

---

## T3: ビルド・テスト

- [x] `cargo test --bin fav v66500_tests` で 2 件 PASS
  - [x] `streaming_inference_pipeline` PASS
  - [x] `streaming_backpressure_ai` PASS
- [x] `cargo test -j 8 -- --test-threads=8` で 3485 tests passed, 0 failed を確認

---

## T4: ドキュメント・ステータス更新

> T3 のテスト全通過（3485 tests passed）を確認してから実施すること。

- [x] `versions/roadmap/roadmap-v66.1-v67.0.md` のバージョン一覧表で v66.5.0 の「状態」列を「完了」に変更
- [x] `versions/current.md` の「進行中バージョン」を v66.5.0 に更新
- [x] 本 `tasks.md` を COMPLETE に更新（全チェックボックスを `[x]` に）

> **CHANGELOG 方針**: v66.1〜v66.9 では CHANGELOG.md を更新しない。v67.0.0 宣言時に一括追記する。
> **MDX 方針**: `site/` の MDX ドキュメントは v66.9.0 安定化時に一括作成するため今バージョンは省略。

---

## コードレビュー指摘と対応

<!-- 実装完了後に追記 -->
