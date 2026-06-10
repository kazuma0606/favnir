# v13.7.0 Tasks — seq pipeline + ctx 統合

Date: 2026-06-10
Branch: feat/v13-capability-context

---

## Phase A — AST 拡張

- [ ] A-1: `fav/src/ast.rs` — `FlwDef` 構造体に `ctx_param: Option<String>` フィールドを追加
- [ ] A-2: 既存の `FlwDef { name, steps, span }` 生成箇所を全件 `ctx_param: None` で補完
  - `grep -n "FlwDef {" fav/src` で全件確認
- [ ] A-3: `cargo build` でコンパイルエラーなし確認

---

## Phase B — パーサー

- [ ] B-1: `fav/src/frontend/parser.rs` — `seq` の parse 箇所を特定
  - `grep -n "Seq\|parse_flw\|FlwDef" parser.rs` で箇所確認
- [ ] B-2: `seq Name(ctx) = ...` のオプション `(ident)` をパースするロジックを追加
  - `(` があれば ident を取り込んで `ctx_param: Some(ident)` に
  - `(` がなければ `ctx_param: None`（後方互換）
- [ ] B-3: `cargo test` で既存の seq 関連テストが全件パスすること

---

## Phase C — コンパイラ

- [ ] C-1: `fav/src/middle/compiler.rs` — `compile_flw_def` の実装を読んで現行動作を把握
- [ ] C-2: `compile_flw_def` を ctx-aware / plain の2パスに分岐させる
  - 既存ロジックを `compile_flw_def_plain` にリネーム
  - `compile_flw_def_ctx_aware` を新規実装
- [ ] C-3: `build_step_call_ctx(step, ctx_slot, input, ctx)` ヘルパーを実装
  - `FlwStep::Stage(name)` → `Stage($ctx, $input)` を IRExpr で構築
  - `FlwStep::Par(_)` → コンパイルエラー相当（E0022 または panic!）
- [ ] C-4: ctx-aware FlwDef の `param_count = 2` でコンパイルされることを手動確認
- [ ] C-5: `cargo test` で既存の FlwDef/seq テストが全件パスすること

---

## Phase D — チェッカー + エラーカタログ

- [ ] D-1: `fav/src/error_catalog.rs` — E0022 エントリを追加
  ```
  "E0022" → "ctx-aware pipeline called with wrong number of arguments"
  ```
- [ ] D-2: `fav/src/middle/checker.rs` — `collect_flw_defs` ヘルパーを追加
  - `HashMap<String, Option<String>>`: pipeline 名 → ctx_param の存在
- [ ] D-3: `check_program` または call-site チェック関数に E0022 チェックを実装
  - ctx-aware FlwDef が 1 引数で呼ばれた場合 → E0022
  - （オプション）ctx なし FlwDef が 2 引数で呼ばれた場合 → E0022
- [ ] D-4: `fav/src/middle/ast_lower_checker.rs` — FlwDef の ctx_param が checker.fav IR に
  正しく反映されるか調査・修正
  - `lower_flw_def` または相当箇所を確認

---

## Phase E — E2E デモ書き換え（型チェックのみ）

- [ ] E-1: `infra/e2e-demo/fav2py/src/pipeline.fav` を seq Pipeline(ctx) 形式に書き換え
  - `seq Pipeline(ctx) = load_and_insert |> aggregate |> save_result` を追加
  - `fn main` 内の chain 3 行を `Pipeline(ctx, get_csv_path(IO.argv()))` に統合
  - 既存の `fn load_and_insert` / `fn aggregate` / `fn save_result` 定義はそのまま残す
- [ ] E-2: `infra/e2e-demo/airgap/src/analyze.fav` を seq AnalyzePipeline(ctx) 形式に書き換え
  - `seq AnalyzePipeline(ctx) = load_all |> validate |> write_output` を追加
  - `fn analyze_pipeline` と `fn main` の手動 bind を書き換え
  - `write_output` の戻り値型（Unit / Result）と SeqStageCheck の相性を事前確認
- [ ] E-3: 型チェック通過確認（cargo test の v137000_tests で確認）

---

## Phase F — テスト追加

- [ ] F-1: `fav/src/driver.rs` に `v137000_tests` モジュールを追加（v136000_tests の下に配置）
- [ ] F-2: 以下のテストを実装:
  - [ ] `version_is_13_7_0` — `CARGO_PKG_VERSION == "13.7.0"`
  - [ ] `seq_ctx_param_parsed` — `seq Pipeline(ctx) = A |> B` がパースエラーなし
  - [ ] `seq_ctx_compiles_param_count_2` — ctx-aware FlwDef が 2-param 関数にコンパイル
  - [ ] `seq_ctx_stage_gets_ctx_arg` — 各ステージ呼び出しが ctx を第1引数で受ける
  - [ ] `e0022_ctx_pipeline_called_without_ctx` — `Pipeline(data)` で E0022
  - [ ] `e2e_fav2py_seq_ctx_compiles` — pipeline.fav の seq Pipeline(ctx) が型チェックパス
  - [ ] `e2e_airgap_seq_ctx_compiles` — analyze.fav の seq AnalyzePipeline(ctx) が型チェックパス
  - [ ] `seq_no_ctx_backward_compat` — 既存 ctx なし seq が変わらず動作
- [ ] F-3: `cargo test v137000` で全件パス確認

---

## Phase G — バージョンバンプ + コミット

- [ ] G-1: `fav/Cargo.toml` → `version = "13.7.0"`
- [ ] G-2: `cargo test` 全件パス確認（リグレッション確認）
- [ ] G-3: self-check（任意）
  ```bash
  ./target/debug/fav check self/compiler.fav
  ./target/debug/fav check self/checker.fav
  ```
- [ ] G-4: `git add` + `git commit -m "feat: v13.7.0 — seq pipeline ctx threading + E0022"`
- [ ] G-5: `git push origin feat/v13-capability-context`

---

## 完了条件

| 確認項目 | 状態 |
|---|---|
| `seq Pipeline(ctx) = ...` 構文がパース可能 | |
| ctx-aware FlwDef が param_count=2 でコンパイル | |
| 各ステージが `Stage(ctx, intermediate)` で呼ばれる | |
| E0022: ctx-aware pipeline を1引数で呼ぶとエラー | |
| `fav2py/pipeline.fav` が seq(ctx) 形式で型チェックパス | |
| `airgap/analyze.fav` が seq(ctx) 形式で型チェックパス | |
| 既存 seq（ctx なし）テストが全件パス（後方互換） | |
| `cargo test v137000` 全件パス | |
| `CARGO_PKG_VERSION == "13.7.0"` | |
