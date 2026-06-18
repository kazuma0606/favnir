# Favnir v12.2.0 Tasks

Date: 2026-06-07
Theme: lint 強化 — W006（Result を `bind _` で捨てる）+ W007（深い match ネスト）

---

## Phase A — 現状把握

- [x] A-1: `fav/self/compiler.fav` の lint エンジン（W001〜W005 付近）を確認
- [x] A-2: `fav/self/checker.fav` の `infer_hm` — `EBind` 分岐を確認

---

## Phase B — W007 実装（compiler.fav）

- [x] B-1: `lint_fn_w007(expr: Expr, depth: Int) -> List<LintWarning>` を追加
- [x] B-2: `lint_arms_w007(arms: Expr, depth: Int) -> List<LintWarning>` を追加
- [x] B-3: `lint_fn` / `lint_stage` から `lint_fn_w007(body, 1)` を呼ぶ
- [x] B-4: `fav.toml [lint] allow = ["W007"]` 対応（フィルタリング）

---

## Phase C — W006 実装（checker.fav）

- [x] C-1: `EChain` バリアントを `Expr` 型に追加（chain vs bind を区別）
- [x] C-2: `is_result_ty` / `is_call_result_ty` ヘルパーを追加
- [x] C-3: `fmt_w006()` — W006 メッセージ生成を追加
- [x] C-4: `check_w006_bind` / `check_w006_expr` / `check_w006_items` / `check_with_w006` 実装
- [x] C-5: `fav.toml [lint] allow = ["W006"]` 対応
- [x] C-6: `ast_lower_checker.rs` で `Stmt::Chain → EChain` に変更
- [x] C-7: `checker_fav_runner.rs` に `run_checker_fav_full()` 追加

---

## Phase D — driver.rs: v12200_tests モジュール追加

- [x] D-1: W006 テスト 5 件
- [x] D-2: W007 テスト 5 件
- [x] D-3: `version_is_12_2_0`
- [x] D-4: 全テスト通過確認

---

## Phase E — 全テスト通過確認

- [x] E-1: `cargo test` — 1363 件通過（+10 件）

---

## Phase F — バージョン更新 + コミット

- [x] F-1: `fav/Cargo.toml` version → `"12.2.0"`
- [x] F-2: `cargo build` で `Cargo.lock` 更新
- [x] F-3: `git commit & push`

---

## 完了条件サマリー

| 確認項目 | 状態 |
|---|---|
| `compiler.fav` に W007（3段以上 match ネスト）追加 | ✅ |
| `checker.fav` に W006（`bind _` で Result 破棄）追加 | ✅ |
| W006/W007 ともに `fav.toml [lint] allow` 対応 | ✅ |
| `cargo test v12200` 11 件通過 | ✅ |
| `cargo test` 全通過（1363 件） | ✅ |
