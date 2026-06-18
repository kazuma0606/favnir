# v19.0.0 — Type System Maturity マイルストーン宣言 タスク

## ステータス: COMPLETE

---

## タスク一覧

### T1: `fav/Cargo.toml` バージョン更新

- [x] `version = "18.8.0"` → `"19.0.0"` に変更
- [x] `cargo build` でコンパイルエラーが 0 であることを確認

---

### T2: `CHANGELOG.md` 更新

`## [v18.0.0]` エントリの上に以下を挿入（新しい順）:

- [x] `## [v19.0.0] — 2026-06-16 — Type System Maturity マイルストーン宣言` エントリ追加

  ```markdown
  ## [v19.0.0] — 2026-06-16 — Type System Maturity マイルストーン宣言

  ### Added
  - v18.x シリーズ集大成：エフェクト推論 / 行多相 / Refinement Types / スキーマ型 /
    線形型 / 共変・反変アノテーション / Const Generics / 型駆動 API 生成が揃い
    Type System Maturity を宣言
  - `CHANGELOG.md` / `README.md` 全面更新（v18.1.0〜v19.0.0）
  - `site/content/docs/api/generate.mdx` / `serve.mdx` 新規作成（v18.8.0 で実施済み）

  ### Internal
  - Cargo.toml version: `19.0.0`
  - `v190000_tests`: 5 件追加

  ---
  ```

- [x] `## [v18.8.0] — 2026-06-16 — 型駆動 API 生成` エントリ追加

  ```markdown
  ## [v18.8.0] — 2026-06-16 — 型駆動 API 生成

  ### Added
  - `#[api(method = "GET", path = "/users/:id")]` アノテーション構文
  - `fav generate api` — OpenAPI 3.0 JSON/YAML と GraphQL SDL の自動生成
  - `fav api-serve` — 開発用 HTTP サーバー（TcpListener ベース）
  - `build_openapi_json` / `build_graphql_sdl` / `build_route_table` / `match_route`
  - `site/content/docs/api/generate.mdx` / `serve.mdx` 新規作成

  ### Internal
  - `ast.rs`: `ApiAnnotation` struct + `FnDef.api_annotation: Option<ApiAnnotation>`
  - `parser.rs`: `parse_api_annotation()`（lookahead で `#[api(` を確認）
  - `driver.rs`: API 生成・ルートテーブル・HTTP サーバー実装
  - Cargo.toml version: `18.8.0`（`tiny_http 0.12` は既存依存）

  ---
  ```

- [x] `## [v18.7.0] — 2026-06-16 — Const Generics` エントリ追加

  ```markdown
  ## [v18.7.0] — 2026-06-16 — Const Generics

  ### Added
  - `fn f<const N: Int where { N > 0 }>(x: Int) -> Int` 構文
  - `TypeExpr::ConstInt(i64, Span)` — 型引数位置での整数リテラル
  - E0335 — const constraint 違反エラー
  - `eval_const_int` / `eval_const_constraint` free fn
  - `site/content/docs/language/const-generics.mdx` 新規作成

  ### Internal
  - `ast.rs`: `GenericParam` に `is_const / const_ty / const_constraint` 追加
  - `parser.rs`: `parse_one_type_param()` — `const N: Int where { ... }` パース
  - `checker.rs`: `const_generics_registry` + E0335 チェック
  - Cargo.toml version: `18.7.0`

  ---
  ```

- [x] `## [v18.6.0] — 2026-06-16 — 共変・反変アノテーション` エントリ追加

  ```markdown
  ## [v18.6.0] — 2026-06-16 — 共変・反変アノテーション

  ### Added
  - `interface Source<+T> { ... }` / `interface Sink<-T> { ... }` 構文
  - `Variance { Covariant, Contravariant, Invariant }` — interface 型パラメータの分散
  - E0334 — 分散違反エラー
  - `site/content/docs/language/variance.mdx` 新規作成

  ### Internal
  - `ast.rs`: `GenericParam.variance` フィールド追加
  - `parser.rs`: `parse_variance_type_params()`
  - `checker.rs`: `check_interface_variance()`
  - Cargo.toml version: `18.6.0`

  ---
  ```

- [x] `## [v18.5.0] — 2026-06-16 — 線形型` エントリ追加

  ```markdown
  ## [v18.5.0] — 2026-06-16 — 線形型

  ### Added
  - `fn(T) -o U` — 線形関数型（linear arrow）
  - `Connection` / `Tx` 型は線形型（使用後は再利用不可）
  - E0332 — 線形型の二重使用エラー
  - E0333 — 線形型の未使用エラー
  - `site/content/docs/language/linear-types.mdx` 新規作成

  ### Internal
  - `ast.rs`: `TypeExpr::LinearArrow` / `Type::LinearFn`
  - `lexer.rs`: `TokenKind::LinearArrow`（`-o` トークン）
  - `checker.rs`: `LinearState` / `linear_env` / `check_fn_def` での線形型追跡
  - Cargo.toml version: `18.5.0`

  ---
  ```

- [x] `## [v18.4.0] — 2026-06-16 — スキーマ型` エントリ追加

  ```markdown
  ## [v18.4.0] — 2026-06-16 — スキーマ型

  ### Added
  - `type User = schema "file:./schema/user.json"` 構文
  - `fav check --refresh-schemas` フラグ
  - E0338 — スキーマファイル不存在エラー
  - `site/content/docs/language/schema-types.mdx` 新規作成

  ### Internal
  - `ast.rs`: `TypeExpr::Schema(uri, span)` / `TypeBody::Alias(Schema(...))`
  - `driver.rs`: `schema_loader` モジュール（`parse_schema_uri` / `SchemaSource::File`）
  - Cargo.toml version: `18.4.0`

  ---
  ```

- [x] `## [v18.3.0] — 2026-06-16 — Refinement Types` エントリ追加

  ```markdown
  ## [v18.3.0] — 2026-06-16 — Refinement Types

  ### Added
  - `fn divide(a: Int, b: Int where { b != 0 }) -> Int` 構文
  - コンパイル時リテラル検証 + 実行時アサーションのハイブリッド方式
  - E0331 — Refinement 制約違反エラー（コンパイル時）
  - `site/content/docs/language/refinement-types.mdx` 新規作成

  ### Internal
  - `ast.rs`: `Param.constraint: Option<Box<Expr>>`
  - `parser.rs`: `parse_param_with_constraint()`
  - `checker.rs`: `check_refinement_call_site()`
  - Cargo.toml version: `18.3.0`

  ---
  ```

- [x] `## [v18.2.0] — 2026-06-16 — 行多相` エントリ追加

  ```markdown
  ## [v18.2.0] — 2026-06-16 — 行多相（Row Polymorphism）

  ### Added
  - `fn f<R with { id: Int }>(row: R) -> { ...R, ts: String }` 構文
  - `with { field: Type }` レコード型制約
  - E0329 — レコード制約フィールド欠如エラー
  - E0330 — spread 対象が非レコード型エラー
  - `site/content/docs/language/row-polymorphism.mdx` 新規作成

  ### Internal
  - `ast.rs`: `TypeBound::HasFields` / `TypeExpr::RecordSpread`
  - `checker.rs`: `check_row_constraint()`
  - Cargo.toml version: `18.2.0`

  ---
  ```

- [x] `## [v18.1.0] — 2026-06-16 — エフェクト推論` エントリ追加

  ```markdown
  ## [v18.1.0] — 2026-06-16 — エフェクト推論（Effect Inference）

  ### Added
  - エフェクト宣言（`!Db`, `!IO` 等）を省略可能に
  - 推移的エフェクト推論（fixpoint 最大 10 ラウンド）
  - `fav check --show-effects` フラグ
  - `site/content/docs/language/effect-inference.mdx` 新規作成

  ### Internal
  - `checker.rs`: `EffectSet` / `infer_effects_fn()` / `fn_effects_registry`
  - `ast.rs`: `Effect` に `Eq, Hash` derive 追加
  - Cargo.toml version: `18.1.0`

  ---
  ```

---

### T3: `README.md` 更新

- [x] 「現在のバージョン」の記述を v19.0.0 に変更（v18.0.0 → v19.0.0）
- [x] 「現在の状態」セクションに Type System Maturity の説明を追加:
  - エフェクト推論・行多相・Refinement Types・スキーマ型・線形型・共変反変・Const Generics・API 生成
- [x] バージョン履歴表に以下を追加（v18.0.0 エントリの直前/後）:
  ```
  | v18.1.0 | エフェクト推論 | 完了 |
  | v18.2.0 | 行多相 | 完了 |
  | v18.3.0 | Refinement Types | 完了 |
  | v18.4.0 | スキーマ型 | 完了 |
  | v18.5.0 | 線形型 | 完了 |
  | v18.6.0 | 共変・反変アノテーション | 完了 |
  | v18.7.0 | Const Generics | 完了 |
  | v18.8.0 | 型駆動 API 生成 | 完了 |
  | v19.0.0 | Type System Maturity マイルストーン宣言 | 完了 |
  ```

---

### T4: `fav/src/driver.rs` — `v190000_tests` 追加

- [x] `v188000_tests::version_is_18_8_0` に `#[ignore]` を追加
- [x] `v190000_tests` モジュールを追加（5件）:

  ```rust
  #[test]
  fn version_is_19_0_0() {
      let cargo = include_str!("../Cargo.toml");
      assert!(cargo.contains("19.0.0"), "Cargo.toml should have version 19.0.0");
  }

  #[test]
  fn changelog_has_v18_entries() {
      let changelog = include_str!("../../CHANGELOG.md");
      assert!(changelog.contains("v18.1.0"), "CHANGELOG should have v18.1.0 entry");
      assert!(changelog.contains("v18.8.0"), "CHANGELOG should have v18.8.0 entry");
  }

  #[test]
  fn readme_mentions_effect_inference() {
      let readme = include_str!("../../README.md");
      assert!(
          readme.contains("エフェクト推論") || readme.contains("effect inference"),
          "README should mention effect inference"
      );
  }

  #[test]
  fn readme_mentions_schema_types() {
      let readme = include_str!("../../README.md");
      assert!(
          readme.contains("スキーマ型") || readme.contains("schema"),
          "README should mention schema types"
      );
  }

  #[test]
  fn api_docs_exist() {
      let content = include_str!("../../site/content/docs/api/generate.mdx");
      assert!(content.contains("fav generate api"), "generate.mdx should document fav generate api");
  }
  ```

---

## テスト（v190000_tests、5件）

| テスト名 | 内容 |
|---|---|
| `version_is_19_0_0` | Cargo.toml に `"19.0.0"` が含まれる |
| `changelog_has_v18_entries` | CHANGELOG に v18.1.0 と v18.8.0 エントリが含まれる |
| `readme_mentions_effect_inference` | README にエフェクト推論の記載がある |
| `readme_mentions_schema_types` | README にスキーマ型の記載がある |
| `api_docs_exist` | `site/content/docs/api/generate.mdx` が `fav generate api` を含む |

---

## 完了条件チェックリスト

- [x] `Cargo.toml` に `"19.0.0"` が含まれる
- [x] `CHANGELOG.md` に v18.1.0〜v18.8.0 の全エントリが含まれる
- [x] `CHANGELOG.md` に v19.0.0 エントリが含まれる
- [x] `README.md` に Type System Maturity の記載がある
- [x] `README.md` にエフェクト推論の記載がある
- [x] `README.md` にスキーマ型の記載がある
- [x] `cargo test v190000` — 5/5 PASS
- [x] `cargo test` — リグレッションなし

---

## 優先度

```
T1（Cargo.toml）      ← 最初（T4 の version_is_19_0_0 テストが依存）
T2（CHANGELOG）       ← T1 と並列可
T3（README）          ← T1 と並列可
T4（v190000_tests）   ← T1/T2/T3 完了後
```
