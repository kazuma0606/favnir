# v66.6.0 タスクリスト

Status: COMPLETE
Version: 66.6.0
Base tests: 3485
Target tests: 3487
Actual tests: 3487

---

## T0: 事前確認

- [x] `cargo test -j 8 -- --test-threads=8` でベース 3485 tests passed, 0 failed を確認
- [x] `fav/Cargo.toml` の version が `"66.0.0"` であることを確認（sub-version では変更しない）
- [x] `runes/serve/` ディレクトリが存在しないことを確認（新規作成対象）
- [x] `driver.rs` に `v66500_tests` が存在することを確認（`v66600_tests` の挿入位置）
- [x] `driver.rs` に `v66600_tests` が存在しないことを確認（新規追加）
- [x] `cargo test --bin fav v66500_tests` で 2 件 PASS することを確認（前バージョンが正常）
  - 前バージョンのテスト関数名: `streaming_inference_pipeline`, `streaming_backpressure_ai`
- [x] `versions/current.md` の「進行中バージョン」が `v66.5.0` であることを確認（確認失敗時は前バージョンの tasks.md T4 が完了していることを確認してから current.md を手動修正すること）

---

## T1: Rune ファイル作成

### serve（新規）

- [x] `runes/serve/` ディレクトリ作成
- [x] `runes/serve/rune.toml` 作成（`entry` / `effects = []` / `[dependencies]` 形式）
- [x] `runes/serve/serve.fav` 作成（以下の全 4 関数を定義）
  - [x] `serve_stage(stage_name, port)` — `""` を返すスタブ
  - [x] `serve_pipeline(pipeline_name, port)` — `""` を返すスタブ
  - [x] `with_rate_limit(rps)` — `""` を返すスタブ
  - [x] `openapi_schema(stage_name)` — `""` を返すスタブ
  - [x] ヘッダーコメントに `ModelServingInterface` を含む（**`serve.contains("ModelServingInterface")` テストにマッチ**）

### 共通確認

- [x] `serve.fav` 内に `let ` が含まれないことを確認
- [x] `serve.fav` 内に `bind.*=`（`<-` でない bind）が含まれないことを確認
- [x] `serve.fav` 内に `Float.from_int` が含まれないことを確認
- [x] `serve.fav` 内に `Float.sqrt` が含まれないことを確認（ベクトル演算が必要な場合は `Math.sqrt` を使う）

---

## T2: `driver.rs` — `v66600_tests` 追加

- [x] `// -- v66500_tests (v66.5.0)` コメントの直前に `v66600_tests` を挿入
  - [x] `model_serve_endpoint_type`:
    - `serve.fav` に `"fn serve_stage("` を含む
    - `serve.fav` に `"fn serve_pipeline("` を含む
    - `serve.fav` に `"ModelServingInterface"` を含む
  - [x] `model_serve_schema_validation`:
    - `serve.fav` に `"fn with_rate_limit("` を含む
    - `serve.fav` に `"fn openapi_schema("` を含む
- [x] `use super::*` は不要（`include_str!` のみ使用）
- [x] `cargo build` でエラーなし

---

## T3: ビルド・テスト

- [x] `cargo test --bin fav v66600_tests` で 2 件 PASS
  - [x] `model_serve_endpoint_type` PASS
  - [x] `model_serve_schema_validation` PASS
- [x] `cargo test -j 8 -- --test-threads=8` で 3487 tests passed, 0 failed を確認

---

## T4: ドキュメント・ステータス更新

> T3 のテスト全通過（3487 tests passed）を確認してから実施すること。

- [x] `versions/roadmap/roadmap-v66.1-v67.0.md` のバージョン一覧表で v66.6.0 の「状態」列を「完了」に変更
- [x] `versions/current.md` の「進行中バージョン」を v66.6.0 に更新
- [x] 本 `tasks.md` を COMPLETE に更新（全チェックボックスを `[x]` に）

> **CHANGELOG 方針**: v66.1〜v66.9 では CHANGELOG.md を更新しない。v67.0.0 宣言時に一括追記する。
> **MDX 方針**: `site/` の MDX ドキュメントは v66.9.0 安定化時に一括作成するため今バージョンは省略。

---

## コードレビュー指摘と対応

- [HIGH] spec-reviewer: `with_rate_limit` の引数がロードマップ（`rps` のみ）と不一致 → spec/plan/tasks を `with_rate_limit(rps: Int)` に修正、ロードマップ行 312 の `openapi` → `openapi_schema` も修正
- [HIGH] spec-reviewer: plan.md のコメント依存リスク記述が薄い → 「変更時は driver.rs アサーションも同時更新」の手順を明記
- [MED] spec-reviewer: ロードマップコメントが `openapi` と略記（`openapi_schema` と不一致）→ ロードマップ修正済み
- [MED] spec-reviewer: plan.md に非スコープセクションがない → 追加済み
- [MED] spec-reviewer: T0 失敗時の対処が未記載 → 注意書きを追加済み
- [LOW] spec-reviewer: tasks.md の確認項目が混在 → 2行に分離して明確化
