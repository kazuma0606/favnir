# v66.7.0 タスクリスト

Status: COMPLETE
Version: 66.7.0
Base tests: 3487
Target tests: 3489
Actual tests: 3489

---

## T0: 事前確認

- [x] `cargo test -j 8 -- --test-threads=8` でベース 3487 tests passed, 0 failed を確認
- [x] `fav/Cargo.toml` の version が `"66.0.0"` であることを確認（sub-version では変更しない）
- [x] `runes/featurestore/` ディレクトリが存在しないことを確認（新規作成対象）
- [x] `driver.rs` に `v66600_tests` が存在することを確認（`v66700_tests` の挿入位置）
- [x] `driver.rs` に `v66700_tests` が存在しないことを確認（新規追加）
- [x] `cargo test --bin fav v66600_tests` で 2 件 PASS することを確認（前バージョンが正常）
  - 前バージョンのテスト関数名: `model_serve_endpoint_type`, `model_serve_schema_validation`
- [x] `versions/current.md` の「進行中バージョン」が `v66.6.0` であることを確認（確認失敗時は前バージョンの tasks.md T4 が完了していることを確認してから current.md を手動修正すること）

---

## T1: Rune ファイル作成

### featurestore（新規）

- [x] `runes/featurestore/` ディレクトリ作成
- [x] `runes/featurestore/rune.toml` 作成（`entry` / `effects = []` / `[dependencies]` 形式）
- [x] `runes/featurestore/featurestore.fav` 作成（以下の全 5 関数を定義）
  - [x] `define_feature(name, version, schema, compute_fn)` — `""` を返すスタブ
  - [x] `get(feature_name, entity_key)` — `""` を返すスタブ
  - [x] `get_batch(feature_name, keys)` — `[]` を返すスタブ
  - [x] `get_version(feature_name, version)` — `""` を返すスタブ
  - [x] `get_at(feature_name, key, timestamp)` — `""` を返すスタブ
  - [x] ヘッダーコメントに `FeatureStoreInterface` を含む

### 共通確認

- [x] `featurestore.fav` 内に `let ` が含まれないことを確認
- [x] `featurestore.fav` 内に `bind.*=`（`<-` でない bind）が含まれないことを確認
- [x] `featurestore.fav` 内に `Float.from_int` が含まれないことを確認
- [x] `featurestore.fav` 内に `Float.sqrt` が含まれないことを確認

---

## T2: `driver.rs` — `v66700_tests` 追加

- [x] `// -- v66600_tests (v66.6.0)` コメントの直前に `v66700_tests` を挿入
  - [x] `feature_store_define_feature`: define_feature / get / get_batch / FeatureStoreInterface
  - [x] `feature_store_versioned_retrieval`: get_version / get_at
- [x] `use super::*` は不要（`include_str!` のみ使用）
- [x] `cargo build` でエラーなし

---

## T3: ビルド・テスト

- [x] `cargo test --bin fav v66700_tests` で 2 件 PASS
  - [x] `feature_store_define_feature` PASS
  - [x] `feature_store_versioned_retrieval` PASS
- [x] `cargo test -j 8 -- --test-threads=8` で 3489 tests passed, 0 failed を確認

---

## T4: ドキュメント・ステータス更新

> T3 のテスト全通過（3489 tests passed）を確認してから実施すること。

- [x] `versions/roadmap/roadmap-v66.1-v67.0.md` のバージョン一覧表で v66.7.0 の「状態」列を「完了」に変更し、変更後に当該行が「完了」になっていることを目視確認
- [x] `versions/current.md` の「進行中バージョン」を v66.7.0 に更新
- [x] 本 `tasks.md` を COMPLETE に更新（全チェックボックスを `[x]` に）

> **CHANGELOG 方針**: v66.1〜v66.9 では CHANGELOG.md を更新しない。v67.0.0 宣言時に一括追記する。
> **MDX 方針**: `site/` の MDX ドキュメントは v66.9.0 安定化時に一括作成するため今バージョンは省略。

---

## コードレビュー指摘と対応

<!-- 実装完了後に追記 -->
