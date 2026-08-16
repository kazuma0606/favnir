# v71.2.0 タスクリスト — Refined Types（型レベル制約 `where self`）

Date: 2026-08-09
Status: 完了

---

## T0: 事前確認

- [x] `fav/Cargo.toml` のバージョンが `71.1.0` であることを確認
- [x] `cargo test` が全 pass（3586 tests）であることを確認
- [x] `type PositiveFloat = Float where self > 0.0` が現行パーサーで通るか確認（`Parser::parse_str` を試す）
  - v41.1.0 で実装済み — パーサー対応済み
- [x] 現行チェッカーに E0425 が未定義であることを確認（E0424 は RBAC 用で使用済み）
  - `grep -rn '"E0425"' fav/src/` で確認 → コメントのみ（エントリなし）

---

## T1: checker.rs に新規フィールドを追加

- [x] `Checker` 構造体に以下を追加する:
  - `fn_alias_refinements: HashMap<String, Vec<(usize, Vec<Expr>)>>`（関数パラメータの alias 制約）
  - 注: `alias_type_invariants` は不要。既存の `type_invariants: HashMap<String, Vec<Expr>>` で代替する
- [x] `Checker::new()` の初期化に `fn_alias_refinements: HashMap::new()` を追加する
- [x] `Checker::new_with_resolver()` の初期化に `fn_alias_refinements: HashMap::new()` を追加する

---

## T2: `register_item_signatures` を更新

- [x] `TypeBody::Alias` ブランチで、`invariants` が非空の場合に `type_invariants` に登録する（`continue` より前）
- [x] `FnDef` の処理で、パラメータの TypeExpr が `type_invariants` に登録済みの型名の場合に `fn_alias_refinements` に登録する
- [x] `cargo test` で既存テスト（3586 件）が全 pass することを確認

---

## T3: `check_type_def` を更新

- [x] `TypeBody::Alias` ブランチを追加する:
  - `self.env.push()` で新スコープを作成
  - ターゲット型を解決して `self` として env に定義
  - 各 invariant 式を `check_expr` で型チェック
  - Bool でない場合は E0245 を発行
  - `self.env.pop()` でスコープを破棄
- [x] `cargo test` で既存テスト（3586 件）が全 pass することを確認

---

## T4: `Expr::Apply` に E0425 呼び出し時チェックを追加

- [x] 既存の `fn_refinement_registry` チェックの直後に alias 制約チェックを追加する:
  - `fn_alias_refinements` でマッチする関数を探す
  - 各 (param_idx, invariants) について、引数がリテラルの場合に `eval_static_expr` で評価
  - 制約違反（`Some(Bool(false))`）の場合に E0425 を発行する
- [x] `cargo test` で既存テスト（3586 件）が全 pass することを確認

---

## T4.5: error_catalog.rs に E0425 エントリを追加

- [x] `fav/src/error_catalog.rs` の E0424（RBAC）エントリの直後に E0425 エントリを追加する:
  - `code: "E0425"`, `title: "Refined type constraint violation"`
  - `category: "type"`, `since` は description 内に `v71.2.0` 記載
- [x] `cargo test` で既存テスト（3586 件）が全 pass することを確認

---

## T5: driver.rs に `v712000_tests` を追加

- [x] driver.rs 末尾（`v711000_tests` の直後）に `v712000_tests` モジュールを追加する
- [x] `refined_type_positive_float` テストを実装する:
  - `type PositiveFloat = Float where self > 0.0` + `fn safe_log(x: PositiveFloat) -> Float { 1.0 }` が errors.is_empty() であることを確認
- [x] `refined_type_violation_compile_error` テストを実装する:
  - `safe_log(0.0)` が E0425 を発生させることを確認
- [x] `cargo test v712000` で 2 件 pass することを確認

---

## T6: Cargo.toml バージョン更新

- [x] `fav/Cargo.toml` の `version` を `"71.1.0"` → `"71.2.0"` に変更する
- [x] driver.rs 内の `"71.1.0"` 文字列リテラルを `"71.2.0"` に一括更新する（replace_all）

---

## T7: CHANGELOG.md 更新

- [x] `## [v71.2.0] — 2026-08-09 — Refined Types（型レベル制約 where self）` エントリを先頭に追加する
- [x] エントリに以下を含める:
  - Added: `v712000_tests` 2 件（3586 → 3588 tests）
  - Added: チェッカー E0425（Refined type 制約違反）
  - Added: `fn_alias_refinements` フィールド（checker.rs）

---

## T8: versions/current.md 更新

- [x] 「進行中バージョン」を `v71.2.0`（Refined Types）に更新する
- [x] 「次に切る版」を `v71.3.0` に更新する

---

## T9: 最終確認

- [x] `cargo test v712000` で 2 件 pass することを確認
- [x] `cargo test` 全体で 3588 tests pass（0 failures）であることを確認
- [x] `fav/Cargo.toml` のバージョンが `71.2.0` であることを確認
- [x] `versions/current.md` が正しく更新されていることを確認

---

## コードレビュー指摘対応

### 第 1 回レビュー指摘対応

#### [HIGH] FnDef が TypeDef より前の場合 `fn_alias_refinements` が空になる（順序依存バグ）
- **対応**: `register_item_signatures` の先頭に alias invariant pre-pass を追加。メインループ前に全 TypeDef の invariants を `type_invariants` に登録（非 opaque のみ）
- `v712000_tests::refined_type_violation_fndef_before_typedef` テストを追加（3 件目、3586 → 3589 tests）

#### [MED] opaque alias + invariants で制約が漏洩する可能性
- **対応**: pre-pass と `check_type_def` の Alias ブランチに `!td.is_opaque` ガードを追加

#### [LOW] `cargo_toml_version_is_71_0_0` 関数名と内容の乖離
- **対応**: `cargo_toml_version_is_71_2_0` に改名

#### [LOW] Clippy: `splitn(2, '#').next()` → `split_once('#').map(|(l,_)| l)`
- **対応**: `is_dim_annotated_name_mismatch` の 2 箇所を修正（v71.1.0 コードの Clippy 修正）

---

## 完了チェックリスト

- [x] 全タスク（T0〜T9）が完了している
- [x] `refined_type_positive_float` が pass
- [x] `refined_type_violation_compile_error` が pass
- [x] テスト総数: 3588（+2）
- [x] E0425 がチェッカーに追加されていることを確認
- [x] `type X = T where self > expr` 構文が typecheck される（check_type_def の Alias ブランチ）
- [x] 関数呼び出し時にリテラル引数の制約違反が E0425 で検出される
