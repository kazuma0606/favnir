# v16.8.0 Tasks — `tap` / `inspect` パイプライン演算子

Date: 2026-06-14
Branch: master

---

## Phase A — Cargo バージョン更新

- [x] A-1: `fav/Cargo.toml` の `version` を `"16.8.0"` に変更
- [x] A-2: `cargo build` → コンパイルエラーなし確認

---

## Phase B — AST: `FlwStep::Tap` / `FlwStep::Inspect` 追加（ast.rs）

- [x] B-1: `fav/src/ast.rs` の `FlwStep` enum に `Tap(Box<Expr>)` 追加
- [x] B-2: `FlwStep` enum に `Inspect` 追加
- [x] B-3: `cargo build` → exhaustive match エラーが出ることを確認（Phase F/G で対処）

---

## Phase C — Parser: `tap(expr)` / `inspect` パース（parser.rs）

- [x] C-1: `parse_flw_step` で `Ident("tap")` → `(` `<expr>` `)` → `FlwStep::Tap(expr)` パース追加
- [x] C-2: `parse_flw_step` で `Ident("inspect")` → `FlwStep::Inspect` パース追加
- [x] C-3: `cargo build` → コンパイルエラーなし確認

---

## Phase D — VM: `inspect_debug` プリミティブ追加（vm.rs）

- [x] D-1: `vm_call_builtin` に `"inspect_debug"` 追加（`vmvalue_repr` で標準出力）
- [x] D-2: `compiler.rs` グローバル builtin 名前テーブル（2 箇所）に `"inspect_debug"` 追加
- [x] D-3: `cargo build` → コンパイルエラーなし確認

---

## Phase E — CompileCtx: `no_tap` フィールド追加（compiler.rs）

- [x] E-1: `CompileCtx` に `pub no_tap: bool` フィールド追加
- [x] E-2: `CompileCtx` の全初期化箇所に `no_tap: false` 追加
- [x] E-3: `cargo build` → コンパイルエラーなし確認

---

## Phase F — Compiler: `FlwStep::Tap` / `FlwStep::Inspect` コンパイル（compiler.rs）

- [x] F-1: `flw_step_name` に `Tap(_) => "tap".to_string()` と `Inspect => "inspect".to_string()` 追加
- [x] F-2: `stage_names` / `display_str` 等の名前収集に `Tap(..) | Inspect => {}` 追加（スキップ）
- [x] F-3: `build_step_call` に `Tap(observer)` → `IRExpr::Block` 実装（`no_tap` 時は identity）
- [x] F-4: `build_step_call` に `Inspect` → `IRExpr::Block` + `inspect_debug` 実装（`no_tap` 時は identity）
- [x] F-5: `build_step_call_ctx` に同じ処理追加（ctx-aware 版）
- [x] F-6: `cargo build` → コンパイルエラーなし確認

---

## Phase G — lineage.rs / driver.rs / fmt.rs exhaustive match（FlwStep）

- [x] G-1: `lineage.rs` の `FlwStep` match に `Tap(..) | Inspect => {}` 追加
- [x] G-2: `driver.rs` の `FlwStep` match に `Tap(..) | Inspect => {}` 追加
- [x] G-3: `fmt.rs` の `FlwStep` match に `Tap(e) => format!(...)` / `Inspect => ...` 追加
- [x] G-4: `checker.rs` の `FlwStep` match があれば同様に追加
- [x] G-5: `cargo build` → コンパイルエラーなし確認

---

## Phase H — driver.rs / main.rs: `--no-tap` フラグ

- [x] H-1: `cmd_run` に `no_tap: bool` パラメータ追加
- [x] H-2: `cmd_run` から `compile_program_ctx` に `no_tap` を渡す（`CompileCtx.no_tap` に設定）
- [x] H-3: `main.rs` に `--no-tap` フラグ解析追加（`"--no-tap" => no_tap = true`）
- [x] H-4: `cargo build` → コンパイルエラーなし確認

---

## Phase I — テスト追加（v168000_tests）

- [x] I-1: `fav/src/driver.rs` に `v168000_tests` モジュール追加
- [x] I-2: `version_is_16_8_0` — `Cargo.toml` に `"16.8.0"` が含まれる
- [x] I-3: `tap_passes_value_through` — `tap` ステップ後に同じ値が次のステージに渡る
- [x] I-4: `tap_calls_observer` — オブザーバー関数が呼ばれる（副作用確認）
- [x] I-5: `inspect_prints_debug` — `inspect` がクラッシュせず値をそのまま通す
- [x] I-6: `no_tap_flag_skips_observer` — `--no-tap` 時にオブザーバーが呼ばれない
- [x] I-7: `cargo test v168000` → 5/5 PASS 確認

---

## Phase J — サイトドキュメント + コミット

- [x] J-1: `site/content/docs/language/pipeline.mdx` に tap/inspect セクション追加
  - `|> tap(fn)` 構文・セマンティクス・ユースケース例
  - `|> inspect` デバッグ用途説明
  - `--no-tap` フラグ（本番ゼロコスト）説明
- [x] J-2: `cargo test v168000` → 5/5 PASS 最終確認
- [x] J-3: `cargo test` → 全件 PASS（リグレッションなし）確認
- [x] J-4: コミット

---

## 完了条件

| 確認項目 | 状態 |
|---|---|
| `Cargo.toml version == "16.8.0"` | [ ] |
| `\|> tap(fn)` が値をそのまま通す | [ ] |
| `\|> tap(fn)` でオブザーバーが呼ばれる | [ ] |
| `\|> inspect` がクラッシュせず値を通す | [ ] |
| `--no-tap` 時にオブザーバーが呼ばれない | [ ] |
| `cargo test v168000` 全テストパス（5/5） | [ ] |
| `cargo test` 全件パス（リグレッションなし） | [ ] |
| `site/content/docs/language/pipeline.mdx` が更新されている | [ ] |

---

## 技術メモ

- **`tap` はソフトキーワード**: `TokenKind::Tap` は追加しない。`Ident("tap")` と `Ident("inspect")` を `parse_flw_step` で検出。
- **IRExpr::Block で実装**: `{ let __tap = input; observer(__tap); __tap }` — 新 VM opcode 不要。
- **`build_step_call` と `build_step_call_ctx` の両方に追加必須**: 片方だけだと ctx-aware pipeline（`fav.toml` プロジェクト）で動作しない。
- **`no_tap` は CompileCtx フィールド**: `cmd_run` → `compile_program_ctx` の呼び出し経路で渡す。
- **`inspect_debug` は compiler.rs の builtin テーブル 2 箇所に追加必須**: 未追加で `CallBuiltin` opcode が生成されず実行時エラー。
- **lineage.rs の FlwStep match**: `Tap(..) | Inspect` はリネージグラフに含めない（`=> {}` でスキップ）。
