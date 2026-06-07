# Favnir v12.2.0 Tasks

Date: 2026-06-07
Theme: lint 強化 — W006（Result を `bind _` で捨てる）+ W007（深い match ネスト）

---

## Phase A — 現状把握

- [ ] A-1: `fav/self/compiler.fav` の lint エンジン（W001〜W005 付近）を確認
  - `lint_fn` / `lint_item` の構造を読む
  - warn_list の蓄積方式を確認
  - `fav.toml [lint] allow` の処理場所を確認
- [ ] A-2: `fav/self/checker.fav` の `infer_hm` — `EBind` 分岐を確認
  - `val_expr` の型推論フローを確認
  - 既存 W001〜W005 の warn 追加パターンを確認
  - `chain` vs `bind` の AST 表現を確認（`is_chain` フラグの有無）

---

## Phase B — W007 実装（compiler.fav）

- [ ] B-1: `lint_fn_w007(expr: Expr, depth: Int) -> Option<String>` を追加
  - `EMatch` で depth >= 3 → W007 メッセージを返す
  - `EMatch` で depth < 3 → `lint_arms_w007(arms, depth + 1)` を呼ぶ
  - 他のノード（`EBind`, `EIf`, `EBlock` 等）は再帰的に depth を引き継ぐ
  - `ELambda` / fn 境界では depth を 1 にリセット
- [ ] B-2: `lint_arms_w007(arms: Expr, depth: Int) -> Option<String>` を追加
  - `EArm` → body を `lint_fn_w007(body, depth)` で走査
  - `EArmNil` → `Option.none()`
- [ ] B-3: `lint_fn` から `lint_fn_w007(fn_def.body, 1)` を呼ぶ
  - 既存 W001〜W005 と同じパターンで warn_list に追加
- [ ] B-4: `fav.toml [lint] allow = ["W007"]` 対応
  - 既存 allow リスト処理に `"W007"` を追加

---

## Phase C — W006 実装（checker.fav）

- [ ] C-1: `infer_hm` の `EBind` 分岐で `_0 == "_"` のケースを特定
  - `bind _` が AST 上どう表現されるか確認
  - `chain _` との区別が必要かどうか確認（`is_chain` フラグ等）
- [ ] C-2: `is_result_ty(ty_str: String) -> Bool` ヘルパーを追加
  - `String.starts_with(ty_str, "Result<")` 相当の処理
- [ ] C-3: `fmt_w006(ty_str: String) -> String` — W006 メッセージ生成を追加
  - `help:` メッセージ 2 件（`chain _` 推奨 + explicit match 例示）
  - `note:` メッセージ 1 件
- [ ] C-4: `infer_hm` の `EBind("_", val_e, cont_e)` 分岐で W006 チェックを追加
  - `val_e` を型推論 → `inferred_ty` 取得
  - `is_result_ty(inferred_ty)` が true → warn_list に W006 追加
  - `chain _` は対象外になることを確認
- [ ] C-5: `fav.toml [lint] allow = ["W006"]` 対応
  - 既存 allow リスト処理に `"W006"` を追加

---

## Phase D — driver.rs: v12200_tests モジュール追加

- [ ] D-1: `v12200_tests` モジュールを `driver.rs` に追加（W006 テスト）
  - [ ] `w006_bind_underscore_result` — `bind _ <- Postgres.execute_raw(...)` → W006
  - [ ] `w006_bind_underscore_unit` — `bind _ <- IO.println("hello")` → 警告なし
  - [ ] `w006_chain_underscore_ok` — `chain _ <- Postgres.execute_raw(...)` → 警告なし
  - [ ] `w006_explicit_match_ok` — `match expr { Ok(_) => ... Err(e) => ... }` → 警告なし
  - [ ] `w006_fav_toml_allow` — `[lint] allow = ["W006"]` で抑制 → 警告なし
- [ ] D-2: `v12200_tests` モジュールに W007 テストを追加
  - [ ] `w007_depth_2_ok` — match { match {} } → 警告なし
  - [ ] `w007_depth_3_warn` — match { match { match {} } } → W007
  - [ ] `w007_depth_4_warn` — 4段ネスト → W007
  - [ ] `w007_helper_fn_ok` — 内部をヘルパー関数に切り出し → 警告なし
  - [ ] `w007_fav_toml_allow` — `[lint] allow = ["W007"]` で抑制 → 警告なし
- [ ] D-3: バージョン確認テスト
  - [ ] `version_is_12_2_0` — `CARGO_PKG_VERSION == "12.2.0"`
- [ ] D-4: `cargo test v12200 -- --nocapture` — 11 件通過確認

---

## Phase E — 全テスト通過確認

- [ ] E-1: `cargo test` — 全件通過（前バージョン比 +11 件程度）

---

## Phase F — バージョン更新 + コミット

- [ ] F-1: `fav/Cargo.toml` version → `"12.2.0"`
- [ ] F-2: `cargo build` で `Cargo.lock` 更新
- [ ] F-3: `git commit & push`

---

## 完了条件サマリー

| 確認項目 | 状態 |
|---|---|
| `compiler.fav` に W007（3段以上 match ネスト）追加 | |
| `checker.fav` に W006（`bind _` で Result 破棄）追加 | |
| W006/W007 ともに `fav.toml [lint] allow` 対応 | |
| `cargo test v12200` 11 件通過 | |
| `cargo test` 全通過 | |
