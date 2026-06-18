# Favnir v9.7.5 Tasks

Date: 2026-06-02
Theme: 名目型ラッパー完成 — `where` バリデーター + `with` 解析 + E0013

---

## Phase A: compiler.fav — `TkWith` トークン追加

- [x] A-1: `Token` type に `TkWith` variant を追加（`TkWhere` の直後）
- [x] A-2: `keyword_token` 関数に `"with"` → `Option.some(TkWith)` を追加
- [x] A-3: `token_eq` に `TkWith` arm を追加（`TkWith => { match b { TkWith => true _ => false } }`）
- [x] A-4: `token_to_string` に `TkWith => "with"` を追加
- [x] A-5: `cargo test compiler_fav_self_check` など簡単な既存テストで Token 追加の影響がないことを確認

---

## Phase B: compiler.fav — `WrapperDef` 拡張 + パーサー更新

- [x] B-1: `WrapperDef` の `has_where: Bool` を `where_pred: Expr` に置き換え
  - 既存の `IWrapper(WrapperDef { ..., has_where: false, ... })` 生成箇所を `where_pred: ELit(LUnit)` に更新
- [x] B-2: `WithClauseParse` 型を追加（`impls: List<String>`, `rest: List<Token>`）
- [x] B-3: `parse_with_idents` ヘルパー関数を追加
- [x] B-4: `parse_with_clause` ヘルパー関数を追加（`TkWith` がなければ空リスト + 元の toks を返す）
- [x] B-5: `parse_where_clause` ヘルパー関数を追加（`TkWhere` がなければ `ELit(LUnit)` を返す）
- [x] B-6: `parse_type_or_wrapper_item` を更新して `with` → `where` の順でパース
  - `)` 消費後、`parse_with_clause` → `parse_where_clause` を呼び出す
  - 取得した `impls` と `where_pred` を `WrapperDef` に設定
- [x] B-7: `pretty_wrapper_def` を更新（`with_impls` / `where_pred` を出力に反映）

---

## Phase C: compiler.fav — `where` バリデーター コード生成

- [x] C-1: `compile_wrapper_validator` 関数を追加
  - `wd.where_pred` が `ELambda(param, body)` の場合、バリデーター `FnDef` を構築
  - `fn Name(param: Inner) -> Result<Inner, String> { if body { Result.ok(param) } else { Result.err("Name: validation failed") } }`
  - `compile_fn_def(fn_def)` でコンパイルし、`compile_items` で `acc` に追加
- [x] C-2: `compile_items` の `IWrapper(wd)` ハンドラを更新
  - `match wd.where_pred { ELit(_) => skip  _ => compile_wrapper_validator(wd, items, acc) }`
  - `ELit` の場合は従来どおりスキップ（`looks_like_variant_ctor` による VM 自動処理）
- [x] C-3: `compile_wrapper_validator` が `IWrapper` の `with_impls` を無視する（v9.8.0 で合成）
- [x] C-4: 手動で簡単なテストソースをコンパイルして動作確認（cargo test の前）

---

## Phase D: checker.fav — E0013 + `has_where` コンストラクタ型

- [x] D-1: `infer_hm` の `EQuestion(inner)` ケースに E0013 チェックを追加
  - `infer_expr(inner, env)` で `ity` を取得
  - `String.starts_with(ity, "Result")` が偽の場合 `Result.err("E0013: ? requires a Result expression, got " + ity)`
  - 真の場合 `Result.ok("Unknown")`（既存動作）
- [x] D-2: `collect_variant_constructors` の `IWrapper(wd)` ハンドラを更新
  - `if wd.has_where { ret_ty = "Result<Name, String>" } else { ret_ty = wd.name }`
  - `env_insert(env, wd.name, make_fn_scheme_str("", wd.inner, ret_ty))`
- [x] D-3: `cargo test checker_fav_wire_self_check` でチェッカー self-check 通過を確認

---

## Phase E: 統合テスト（`fav/src/driver.rs`）

`v975_tests` モジュールを `driver.rs` 末尾に追加。

- [x] E-1: `where_validator_ok`
  - `type Percent(Float) where |v| v >= 0.0 && v <= 100.0`
  - `match Percent(50.0) { Ok(p) => Float.to_string(p)  Err(_) => "err" }` → `"50"`
- [x] E-2: `where_validator_err`
  - `match Percent(150.0) { Ok(_) => "ok"  Err(e) => e }` → `"Percent: validation failed"`
- [x] E-3: `where_validator_in_fn`
  - `where` バリデーターを含む関数: `apply_discount(1000.0, 20.0)` → `Ok(200.0)`
  - `apply_discount(1000.0, 150.0)` → `Err("Percent: validation failed")`
- [x] E-4: `with_clause_parses_ok`
  - `type UserId(Int) with Serialize` → parse エラーなし、コンストラクタ `UserId(42)` → `42`
- [x] E-5: `e0013_expr_question_on_option`
  - `Option.some(42)?` を含むコードが compiler.fav または checker.fav エラーになること
  - `compile_and_run` がパニックすること（または専用の check 関数でエラーメッセージを確認）
- [x] E-6: `where_validator_combined_with_and`
  - `type Username(String) where |s| String.length(s) > 0 && String.length(s) <= 5`
  - `Username("hi")` → `Ok("hi")`
  - `Username("")` → `Err("Username: validation failed")`
- [x] E-7: `cargo test v975` — `v975_tests` モジュール全件通過

---

## Phase F: self-check + Bootstrap 検証

- [x] F-1: `cargo test checker_fav_wire_self_check` — self-check 通過
- [x] F-2: `cargo test bootstrap` — bytecode 維持確認
- [x] F-3: `cargo test` — 全件通過（目標: 1205 件以上）

---

## Phase G: バージョン更新

- [x] G-1: `fav/Cargo.toml` の version を `"9.7.5"` に更新
- [x] G-2: `fav/self/cli.fav` のバージョン文字列を `"9.7.5"` に更新
- [x] G-3: `versions/v9.7.5/tasks.md` 完了チェックを入れる（本ファイル）
- [x] G-4: `memory/MEMORY.md` に v9.7.5 完了を記録
- [x] G-5: commit

---

## 完了条件

| 条件 | 確認 |
|---|---|
| `type Name(Inner) where \|v\| pred` コンストラクタが `Ok`/`Err` を返す | |
| `type Name(Inner) with Iface` が parse エラーなし、既存機能に影響なし | |
| `expr?` が非 Result 型に使われると E0013 を返す | |
| checker.fav で `has_where` あり型のコンストラクタが `Result` 型として登録される | |
| `cargo test checker_fav_wire_self_check` 通過 | |
| `cargo test bootstrap` 維持 | |
| `cargo test` 全件通過（1205 件以上） | |

---

## スコープ外（v9.8.0 以降）

- `T!` 型後置（`Int!` = `Result<Int, String>`）— エフェクト注釈との曖昧性解決後
- `with Serialize/Deserialize/Show/Eq` 自動合成
- E0010 WrapperTypeMismatch
- E0011 UnknownInterface
