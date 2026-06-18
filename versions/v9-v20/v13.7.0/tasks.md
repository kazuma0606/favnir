# v13.7.0 Tasks — seq pipeline + ctx 統合

Date: 2026-06-10
Branch: feat/v13-capability-context
Completed: 2026-06-11

---

## Phase A — AST 拡張

- [x] A-1: `fav/src/ast.rs` — `FlwDef` 構造体に `ctx_param: Option<String>` フィールドを追加
- [x] A-2: 既存の `FlwDef { name, steps, span }` 生成箇所を全件 `ctx_param: None` で補完
  - `grep -n "FlwDef {" fav/src` で全件確認
- [x] A-3: `cargo build` でコンパイルエラーなし確認

---

## Phase B — パーサー

- [x] B-1: `fav/src/frontend/parser.rs` — `seq` の parse 箇所を特定
  - `grep -n "Seq\|parse_flw\|FlwDef" parser.rs` で箇所確認
- [x] B-2: `seq Name(ctx) = ...` のオプション `(ident)` をパースするロジックを追加
  - `(` があれば ident を取り込んで `ctx_param: Some(ident)` に
  - `(` がなければ `ctx_param: None`（後方互換）
- [x] B-3: `cargo test` で既存の seq 関連テストが全件パスすること

---

## Phase C — コンパイラ

- [x] C-1: `fav/src/middle/compiler.rs` — `compile_flw_def` の実装を読んで現行動作を把握
- [x] C-2: `compile_flw_def` を ctx-aware / plain の2パスに分岐させる
  - `compile_flw_def_ctx_aware` を新規実装（plain は既存ロジックを維持）
- [x] C-3: `build_step_call_ctx(step, ctx_slot, input, ctx)` ヘルパーを実装
  - `FlwStep::Stage(name)` → `Stage($ctx, $input)` を IRExpr で構築
- [x] C-4: ctx-aware FlwDef の `param_count = 2` でコンパイルされることを確認
- [x] C-5: `cargo test` で既存の FlwDef/seq テストが全件パスすること

---

## Phase D — チェッカー + エラーカタログ

- [x] D-1: `fav/src/error_catalog.rs` — E0022 エントリを追加
- [x] D-2: `fav/src/middle/checker.rs` — `check_ctx_pipeline_arity` パスを追加
  - `HashMap<String, bool>`: pipeline 名 → ctx_param あり/なし
- [x] D-3: `check_program` から E0022 チェックを呼び出す
  - ctx-aware FlwDef が 1 引数で呼ばれた場合 → E0022
- [x] D-4: `fav/src/middle/ast_lower_checker.rs` — `lower_flw_def` に `ctx_param` フィールドを追加
- [x] D-5: `seq_unwrap_result` ヘルパーを追加（SeqChain が Result<T,E>→T を unwrap する動作をチェッカーで追跡）

---

## Phase E — E2E デモ書き換え（型チェックのみ）

- [x] E-1: `infra/e2e-demo/fav2py/src/pipeline.fav` を seq Pipeline(ctx) 形式に書き換え
  - `seq Pipeline(ctx) = load_and_insert |> aggregate |> save_result` を追加
  - `fn main` を `Pipeline(ctx, get_csv_path(IO.argv()))` に統合
- [x] E-2: `infra/e2e-demo/airgap/src/analyze.fav` を seq AnalyzePipeline(ctx) 形式に書き換え
  - `seq AnalyzePipeline(ctx) = load_all |> validate |> write_output` を追加
  - `fn analyze_pipeline` と手動 bind を削除
- [x] E-3: 型チェック通過確認（cargo test の v137000_tests で確認）

---

## Phase F — テスト追加

- [x] F-1: `fav/src/driver.rs` に `v137000_tests` モジュールを追加（v136000_tests の下に配置）
- [x] F-2: 以下のテストを実装:
  - [x] `version_is_13_7_0` — `CARGO_PKG_VERSION >= "13.7.0"`
  - [x] `seq_ctx_param_parsed` — `seq Pipeline(ctx) = A |> B` がパースエラーなし
  - [x] `seq_no_ctx_backward_compat` — 既存 ctx なし seq が変わらず動作
  - [x] `seq_ctx_compiles_param_count_2` — ctx-aware FlwDef が 2-param 関数にコンパイル
  - [x] `seq_plain_compiles_param_count_1` — plain FlwDef が 1-param 関数にコンパイル
  - [x] `seq_ctx_called_correctly_no_e0022` — `Pipeline(ctx, data)` で E0022 なし
  - [x] `e0022_ctx_pipeline_called_without_ctx` — `Pipeline(data)` で E0022
  - [x] `e2e_fav2py_seq_ctx_compiles` — pipeline.fav の seq Pipeline(ctx) が型チェックパス
  - [x] `e2e_airgap_seq_ctx_compiles` — analyze.fav の seq AnalyzePipeline(ctx) が型チェックパス
- [x] F-3: `cargo test v137000` で全件パス確認（9/9）

---

## Phase G — バージョンバンプ + コミット

- [x] G-1: `fav/Cargo.toml` → `version = "13.7.0"`
- [x] G-2: `cargo test` 全件パス確認（1478 passed, 0 failed）
- [x] G-4: `git commit -m "feat: v13.7.0 — seq pipeline ctx threading + E0022"`

---

## 完了条件

| 確認項目 | 状態 |
|---|---|
| `seq Pipeline(ctx) = ...` 構文がパース可能 | ✓ |
| ctx-aware FlwDef が param_count=2 でコンパイル | ✓ |
| 各ステージが `Stage(ctx, intermediate)` で呼ばれる | ✓ |
| E0022: ctx-aware pipeline を1引数で呼ぶとエラー | ✓ |
| `fav2py/pipeline.fav` が seq(ctx) 形式で型チェックパス | ✓ |
| `airgap/analyze.fav` が seq(ctx) 形式で型チェックパス | ✓ |
| 既存 seq（ctx なし）テストが全件パス（後方互換） | ✓ |
| `cargo test v137000` 全件パス（9/9） | ✓ |
| `CARGO_PKG_VERSION >= "13.7.0"` | ✓ |

---

## 実装ノート

- **`seq_unwrap_result`**: SeqChain opcode は実行時に `Result<T,E>` → `T` を unwrap する。checker の `check_flw_def` はこの動作を追跡するため `seq_unwrap_result` を適用しないと E0103（型不一致）が発生した。
- **E0022 の実装**: `check_ctx_pipeline_arity` が全 fn/trf ボディをスキャンし、FlwDef 名 → has_ctx の HashMap を参照して call-site の引数数を検証する。
- **後方互換**: ctx なし `seq P = A |> B` は `param_count=1` でコンパイルされ既存の動作を維持。
