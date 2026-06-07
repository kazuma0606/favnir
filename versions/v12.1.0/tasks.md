# Favnir v12.1.0 Tasks

Date: 2026-06-07
Theme: `bind` イミュータビリティ強制（E0018）

---

## Phase A — checker.fav 現状把握

- [x] A-1: `fav/self/checker.fav` の `EBind` 処理箇所を特定
  - `infer_hm_let` / `infer_hm` / `infer_expr` の `EBind` 分岐を読む
  - `env` の型・構造を確認
  - `chain` が `EBind` と同じノードか別ノードか確認（→ 同じ `EBind` として lowered）

---

## Phase B — checker.fav: bound_set ヘルパー追加

- [x] B-1: `bound_set_contains(set: List<String>, name: String) -> Bool` を追加
- [x] B-2: `bound_set_add(set: List<String>, name: String) -> List<String>` を追加
- [x] B-3: `fmt_e0018(name: String) -> String` — E0018 メッセージ生成関数を追加
  - `help:` メッセージ 2 件を含む
- [x] B-4: `collect_param_names(params: List<Param>) -> List<String>` を追加

---

## Phase C — checker.fav: E0018 チェック実装

- [x] C-1: `check_rebind(expr: Expr, bound: List<String>) -> Option<String>` を追加
  - `EBind` の再束縛を検出
  - `Option.none()` / `Option.some(...)` を使用（`None`/`Some` 直接使用は VM クラッシュするため不可）
- [x] C-2: `check_rebind_ok(expr: Expr, bound: List<String>) -> Result<String, String>` を追加
- [x] C-3: `check_fn_def` に `chain _ok <- check_rebind_ok(...)` を追加（エラー伝播）
  - `chain` キーワードを使用（`bind` は ChainCheck を生成しないため不可）
- [x] C-4: match arm スコープでは `bound` をリセット（arm ごとに独立）
- [x] C-5: lambda スコープは独立（`ELambda` で `List.empty()` リセット）

---

## Phase D — chain 対応確認

- [x] D-1: AST 上の `chain` ノードが `EBind` として lowered されることを確認
  - `ast_lower_checker.rs` の `lower_stmts_and_tail` で `Stmt::Chain` → `EBind` を確認
- [x] D-2: `EBind` 検出で `bind`/`chain` クロスキーワード再束縛も検出される

---

## Phase E — Rust checker.rs: E0018 追加（--legacy モード）

- [ ] E-1: `src/middle/checker.rs` の `IRStmt::Bind` 処理箇所を特定
- [ ] E-2: `seen_names: HashSet<String>` を fn/stage 本体ごとに管理
- [ ] E-3: `name != "_"` かつ `seen_names.contains` で E0018 エラーを発行
- [ ] E-4: `help:` メッセージを Rust 側にも追加

（本バージョンでは未実装 — checker.fav 経由で検出するため十分）

---

## Phase F — driver.rs: v12100_tests モジュール追加

- [x] F-1: `v12100_tests` モジュールを `driver.rs` に追加（正常系）
  - [x] `e0018_underscore_allowed` — `bind _` 連続 → エラーなし
  - [x] `e0018_match_arm_independent` — match arm 内同名変数（別スコープ）→ エラーなし
  - [x] `e0018_lambda_scope_independent` — 別 fn の `bind x` は独立 → エラーなし
- [x] F-2: `v12100_tests` モジュールに異常系テストを追加
  - [x] `e0018_bind_rebind_detected` — `bind x` → `bind x` → E0018
  - [x] `e0018_chain_rebind_detected` — `chain x` → `chain x` → E0018
  - [x] `e0018_bind_then_chain_cross` — `bind x` → `chain x` → E0018（クロスキーワード）
  - [x] `e0018_chain_then_bind_cross` — `chain x` → `bind x` → E0018（逆順）
  - [x] `e0018_param_shadowing_fn` — fn パラメータ名と同名の `bind` → E0018
  - [x] `e0018_param_shadowing_stage` — stage `|param|` と同名の `bind` → E0018
  - [x] `e0018_triple_rebind` — 3 回再束縛でも E0018
- [x] F-3: バージョン確認
  - [x] `version_is_12_1_0` — `CARGO_PKG_VERSION == "12.1.0"`
- [x] F-4: `cargo test v12100 -- --nocapture` — 11 件通過確認

---

## Phase G — 全テスト通過確認

- [x] G-1: `cargo test` — 1353 件通過（bin）+ 705 件（lib）

---

## Phase H — バージョン更新 + コミット

- [x] H-1: `fav/Cargo.toml` version → `"12.1.0"`
- [ ] H-2: `cargo build` で `Cargo.lock` 更新
- [ ] H-3: `git commit & push` — CI 確認

---

## 完了条件サマリー

| 確認項目 | 状態 |
|---|---|
| `checker.fav` に E0018 チェック追加（bound_set 管理） | ✅ |
| E0018 エラーメッセージに `help:` 2 件付き | ✅ |
| `_` の再束縛は許可（例外処理） | ✅ |
| `chain x` の再束縛も E0018 検出 | ✅ |
| match arm スコープは独立（同名変数 OK） | ✅ |
| `cargo test v12100` 11 件通過（正常系 3 + 異常系 7 + バージョン 1） | ✅ |
| `cargo test` 全通過 | ✅ |

## 技術的知見（デバッグ記録）

1. **`None`/`Some` 直接使用は禁止**: Favnir の Rust コンパイラは `None`/`Some` を `ctx.globals` に登録しない。直接使うと `IRExpr::Global(u16::MAX)` → 実行時 "global index out of bounds"。代わりに `Option.none()` / `Option.some(x)` を使う。

2. **`bind` vs `chain` のエラー伝播**: `bind x <- Result<...>` は `IRStmt::Bind`（ChainCheck なし）でエラーを伝播しない。`chain x <- Result<...>` は `IRStmt::Chain`（ChainCheck あり）でエラーを伝播する。

3. **深いネスト `match + bind` 問題**: `match { None => { match { None => { bind ... } } } }` のように `match` の中に `match` があり内側の arm に `bind` があると "global index out of bounds" になる。`bind` を外側の `match` の前に出すか、ヘルパー関数に切り出すことで回避。
