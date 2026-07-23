# Tasks: v47.8.0 — `Map` 拡充

Status: COMPLETE
Date: 2026-07-18

---

## T0 — 事前確認

- [x] `cargo test` 3036 passed, 0 failed を確認
- [x] `vm.rs` に `Map.merge`（12023）・`Map.filter_values`（3834）・`Map.map_values`（3809）が存在することを確認
- [x] `vm.rs` に `Map.keys`（11927）・`Map.values`（11945）が存在することを確認
- [x] `checker.rs` に `("Map", "merge")` / `("Map", "filter_values")` / `("Map", "map_values")` が存在することを確認（line 6805/6809/6806）
- [x] `checker.rs` に `("Map", "keys")` / `("Map", "values")` が存在することを確認（line 6799/6800）

## T1 — `driver.rs` に `v478000_tests` 追加

- [x] `v477000_tests` の直前に `v478000_tests` モジュールを追加（3 テスト）
  - [x] `map_merge`: `Map.merge({"key":1}, {"key":2})` → 右辺優先で `value == 2`
  - [x] `map_filter_values`: `Map.filter_values({"a":1,"b":2}, |v| v > 1)` → `Map.size == 1`
  - [x] `map_map_values`: `Map.map_values({"x":5}, |v| v * 2)` → `"x"` の `value == 10`

## T2 — バージョン更新・テスト・完了

- [x] `fav/Cargo.toml` version → `"47.8.0"`
- [x] `CHANGELOG.md` に v47.8.0 エントリ追加
- [x] `cargo test` 3039 passed, 0 failed（3036 + 3 件）
- [x] `cargo clippy -- -D warnings` クリーン
- [x] `versions/current.md` を v47.8.0（3039 tests）に更新、進行中バージョンを `v47.9.0` に更新
- [x] `versions/roadmap/roadmap-v47.1-v48.0.md` の v47.8.0 完了条件テスト数（3039）を実績で確認・必要に応じて更新
- [x] tasks.md を COMPLETE に更新（T0〜T2 全チェック）

> **注記**: `site/content/docs/stdlib/` のドキュメント更新は v47.9.0 stdlib ドキュメントスプリントで一括実施（本バージョンでは対象外）
> **注記**: マスターロードマップ（`roadmap-v45.1-v50.0.md`）への反映は v48.0.0 マイルストーン宣言時に実施（本バージョンはサブスプリント内マイナー版）

---

## コードレビュー指摘と対応（spec-reviewer）

| 重大度 | 内容 | 対応 |
|---|---|---|
| [MED] | spec.md 完了条件の `tasks.md COMPLETE` に T0 全チェック明示がない | 「T0〜T2 全 [x]」を spec.md 完了条件に追記 |
| [LOW] | plan.md に `roadmap-v47.1-v48.0.md` 更新ステップが欠落 | Step 5 として追加 |
| [LOW] | tasks.md T2 にマスターロードマップ不更新の理由が未明記 | 「v48.0.0 で実施」の注記を追加 |
