# Favnir v1.4.0 タスク一覧 — `fav explain --format json` + `fav bundle` + 残件

作成日: 2026-05-07

> **ゴール**: コードの意味を機械可読 JSON で出力し、最小実行 artifact を生成する。
> v1.3.0 の先送り残件（動的注入・`fav graph`・`abstract trf` ジェネリック）を合わせて解消する。
>
> **前提**: v1.3.0 完了
>
> **スコープ管理が最優先。Done definition を超えない。**

---

## Phase 0: バージョン更新

### 0-1: バージョン更新

- [x] `Cargo.toml` の `version` を `"1.4.0"` に更新
- [x] `main.rs` の HELP テキストを `v1.4.0` に更新
- [x] `main.rs` の HELP に `bundle` / `graph` コマンドを追加
- [x] `cargo build` が通ること

---

## Phase 1: `fav explain --format json`

### 1-1: CLI 拡張

- [x] `main.rs` の `explain` コマンドに `--format <text|json>` フラグを追加
- [x] `main.rs` の `explain` コマンドに `--focus <all|fns|trfs|flws|types>` フラグを追加
- [x] `driver.rs` の `cmd_explain` シグネチャに `format: &str` と `focus: Option<&str>` を追加
- [x] 既存の text 出力は `--format text`（デフォルト）として変わらず動作すること

### 1-2: `ExplainJson` 構造体の定義

- [x] `FnEntry` 構造体（`name`, `kind`, `params`, `return_type`, `effects`, `calls`, `reachable_from_entry`）
- [x] `TrfEntry` 構造体（`name`, `kind`, `input_type`, `output_type`, `effects`, `calls`, `reachable_from_entry`）
- [x] `FlwEntry` 構造体（`name`, `kind`, `input_type`, `output_type`, `effects`, `steps`, `template`, `type_args`, `bindings`, `type_params`, `slots`, `reachable_from_entry`）
- [x] `TypeEntry` 構造体（`name`, `kind`, `fields`, `variants`, `invariants`）
- [x] `SlotEntry` / `FieldEntry` / `ParamEntry` 補助構造体
- [x] `ExplainJson` トップレベル構造体（`schema_version`, `favnir_version`, `entry`, `source`, `fns`, `trfs`, `flws`, `types`, `effects_used`, `emits`, `runes_used`）
- [x] 全構造体に `serde::Serialize` を derive

### 1-3: `render_json` の実装

- [x] `ExplainPrinter` または別モジュールに `render_json(program, reachability, focus) -> String` を追加
- [x] `build_fn_entries(program, reachability)` を実装
  - [x] `Item::FnDef` からエントリを生成
  - [x] `calls` は IR または AST からの参照先収集で埋める（`collect_calls` ヘルパー）
- [x] `build_trf_entries(program, reachability)` を実装
  - [x] `Item::TrfDef` → `kind: "trf"`
  - [x] `Item::AbstractTrfDef` → `kind: "abstract_trf"` （calls は None）
- [x] `build_flw_entries(program, reachability)` を実装
  - [x] `Item::FlwDef` → `kind: "flw"`
  - [x] `Item::FlwBindingDef` → `kind: "flw_binding"`（`template`, `bindings` を含む）
  - [x] `Item::AbstractFlwDef` → `kind: "abstract_flw"`（`type_params`, `slots` を含む）
- [x] `build_type_entries(program)` を実装
  - [x] `TypeBody::Record` → `kind: "record"`, `fields` を含む
  - [x] `TypeBody::Sum` → `kind: "sum"`, `variants` を含む
  - [x] `invariants` フィールドを `format_expr_compact` で文字列化
- [x] `--focus` 絞り込みを実装（該当フィールド以外を空配列にする）
- [x] `reachability` が `None` の場合は全て `reachable_from_entry: true` とする

### 1-4: `cmd_explain` への統合

- [x] `--format json` 時に `render_json` の出力を stdout に表示
- [x] `--format text` 時は既存の text 出力（変更なし）
- [x] テスト: `explain_json_valid_schema` — `--format json` の出力が有効な JSON である
- [x] テスト: `explain_json_has_all_sections` — `fns`/`trfs`/`flws`/`types` が全て含まれる
- [x] テスト: `explain_json_focus_trfs` — `--focus trfs` で `trfs` のみが含まれる
- [x] テスト: `explain_json_kinds` — `kind` フィールドが正確な値を持つ
- [x] テスト: `explain_json_reachable_flag` — `reachable_from_entry` フラグが正確に設定される
- [x] `cargo test` が全通過すること

---

## Phase 2: 到達可能性解析

### 2-1: `src/middle/reachability.rs` の新規作成

- [x] `ReachabilityResult` 構造体（`included: HashSet<String>`, `excluded: HashSet<String>`, `effects_required: Vec<String>`, `emits: Vec<String>`）を定義
- [x] `reachability_analysis(entry: &str, program: &IRProgram) -> ReachabilityResult` を実装
  - [x] BFS/DFS で `entry` から呼び出しグラフを走査
  - [x] `IRExpr::Global` / IR 関数呼び出しを依存として追跡
  - [x] 到達不能な関数を `excluded` に収集
  - [x] 到達可能な関数の effects を集約して `effects_required` に格納
- [x] `src/middle/mod.rs` に `pub mod reachability` を追加

### 2-2: `collect_calls_in_ir` の実装

- [x] `collect_calls_in_ir(fn_def: &IRFnDef) -> Vec<String>` を実装
  - [x] `IRExpr` / `IRStmt` を再帰的にスキャン
  - [x] `IRExpr::Global(name)` / `IRExpr::Call { func }` を収集
- [x] 既存の `collect_deps`（`ir.rs`）と重複する場合は統合または再利用

### 2-3: explain.json への反映

- [x] `render_json` に `reachability: Option<&ReachabilityResult>` を渡す
- [x] `reachability` が `Some` の場合に各エントリの `reachable_from_entry` を正確に設定

### 2-4: 到達可能性テスト

- [x] `test_reachability_simple`: main → fn A → fn B の到達性が正確
- [x] `test_reachability_excluded`: 未使用 fn が `excluded` に入る
- [x] `test_reachability_effects_required`: included 関数の effects が `effects_required` に集約される
- [x] `test_explain_json_reachable_flag`: explain JSON の `reachable_from_entry` が正確（`driver.rs` の `explain_json_reachable_flag`）
- [x] `cargo test` が全通過すること

---

## Phase 3: `fav bundle`

### 3-1: `cmd_bundle` の実装（`driver.rs`）

- [x] `pub fn cmd_bundle(file: &str, out: Option<&str>, entry: &str, manifest: bool, explain: bool)` を実装
- [x] ソースのパース・型検査・IR 生成を行う
- [x] `reachability_analysis` を呼び出す
- [x] `filter_ir_program(ir, included)` を実装（到達不能関数を除外した IR を生成）
- [x] filtered IR から artifact をビルドして出力パスに書き出す
- [x] 出力パスのデフォルト: `dist/<basename>.fvc`（ディレクトリが存在しなければ作成）

### 3-2: `--manifest` フラグ

- [x] `build_manifest_json(source, artifact_path, artifact_bytes, reachability) -> String` を実装
  - [x] `schema_version`, `favnir_version`, `entry`, `source`, `artifact`, `artifact_size` を含む
  - [x] `included`（ソート済み）, `excluded`（ソート済み）を含む
  - [x] `effects_required`, `emits`, `runes_used` を含む
  - [x] `built_at` を現在時刻で埋める（`chrono` 依存を `Cargo.toml` に追加、またはプレースホルダー文字列）
- [x] `manifest.json` を `<out>.manifest.json` に書き出す

### 3-3: CLI への統合

- [x] `main.rs` に `bundle` コマンドを追加
  - [x] `-o <path>` フラグ
  - [x] `--entry <name>` フラグ（デフォルト `"main"`）
  - [x] `--manifest` フラグ
  - [x] `--explain` フラグ

### 3-4: bundle テスト

- [x] `test_bundle_produces_artifact`: bundle で有効な .fvc が生成される（`bundle_writes_artifact_manifest_and_explain`）
- [x] `test_bundle_excludes_dead_code`: unreachable 関数が excluded に入る（`bundle_filter_excludes_dead_code`）
- [x] `test_bundle_manifest_json`: `--manifest` で manifest.json が生成され、`included`/`excluded` が正確（`bundle_manifest_json_has_reachability` + `bundle_writes_artifact_manifest_and_explain`）
- [x] `test_bundle_artifact_smaller_than_build`: bundle 後の artifact が build より小さい（`bundle_writes_artifact_manifest_and_explain` 内で `helper` 除外を確認）
- [x] `cargo test` が全通過すること

---

## Phase 4: `fav bundle --explain` + artifact explain

### 4-1: `.fvc` フォーマット拡張（`artifact.rs`）

- [x] `FvcArtifact` に `explain_json: Option<String>` フィールドを追加
- [x] `FvcWriter` に `write_explain_section(json: &str)` を実装
  - [x] セクションマジック `EXPL` (4 bytes) + 長さ (4 bytes) + JSON バイト列の形式
- [x] `FvcArtifact` の読み込みで `EXPL` セクションを検出して `explain_json` に格納
- [x] `EXPL` セクションがない artifact（旧形式）は `explain_json = None` で正常読み込みできること

### 4-2: `fav bundle --explain` での統合

- [x] `cmd_bundle` の `--explain` パスで `render_json` を呼び出す
- [x] `artifact.explain_json` に JSON をセットしてから `write_artifact_to_path` で書き出す
- [x] `<out>.explain.json` ファイルも別途生成する

### 4-3: `fav explain dist/app.fvc`

- [x] `cmd_explain` でファイル拡張子が `.fvc` の場合の分岐を追加
  - [x] `--format json` かつ `explain_json` あり → stdout に JSON を出力
  - [x] `--format text` → スケルトン出力（artifact の fn/trf/flw 一覧のみ）
  - [x] `--format json` かつ `explain_json` なし → エラーメッセージ
- [x] テスト: `test_bundle_explain_embedded`: bundle --explain で埋め込まれた JSON を artifact から読み出せる（`bundle_explain_embedded_round_trip`）
- [x] テスト: `test_explain_fvc_format_json`: `fav explain app.fvc --format json` が JSON を出力する（`bundle_explain_embedded_round_trip` でカバー）
- [x] `cargo test` が全通過すること

---

## Phase 5: `fav graph`（v1.3.0 残件）

### 5-1: `cmd_graph` の実装（`driver.rs`）

- [x] `pub fn cmd_graph(file: &str, format: &str, focus: Option<&str>)` を実装
- [x] `main.rs` に `graph` コマンドを追加
  - [x] `--format <text|mermaid>` フラグ（デフォルト `text`）
  - [x] `--focus <flw-name>` フラグ（特定 flw に絞る）

### 5-2: text レンダラー

- [x] `render_graph_text(program, focus) -> String` を実装
  - [x] `FlwBindingDef`: テンプレート名・各スロット名・実装名・型シグネチャを表示
  - [x] `FlwDef`: ステップ一覧を矢印で表示
  - [x] `focus` 指定時は該当 flw のみ表示
- [x] テスト: `graph_text_shows_flw_structure`: text 形式で flw 構造が表示される

### 5-3: mermaid レンダラー

- [x] `render_graph_mermaid(program, focus) -> String` を実装
  - [x] `flowchart LR` ヘッダで開始
  - [x] スロット間を `-->` で接続
  - [x] 実装名をノードラベルに含める
- [x] テスト: `graph_mermaid_valid_syntax`: mermaid 形式が `flowchart LR` を含む有効な構文を出力する
- [x] `cargo test` が全通過すること

---

## Phase 6: trf 第一級値 + 動的注入（v1.3.0 残件）

### 6-1: `TypeExpr` の拡張（`ast.rs`）

- [x] `TypeExpr::TrfFn { input, output, effects }` バリアントを追加
- [x] `TypeExpr` を参照している全箇所でコンパイルエラーがないこと

### 6-2: パーサーの拡張（`parser.rs`）

- [x] `parse_fn_param_type` でパラメータ型として `A -> B !Fx` 形式（`TypeExpr::TrfFn`）を受け入れる
  - [x] 既存の型式に `->` が続く場合に `TrfFn` に変換
  - [x] effects（`!Effect` 列）のパースも対応
- [x] パーサーテスト: `fn f(save: UserRow -> Int !Io)` がパースできる
- [x] パーサーテスト: 複数の trf 型引数を持つ関数がパースできる

### 6-3: `SlotImpl` の追加（`ast.rs`）

- [x] `pub enum SlotImpl { Global(String), Local(String) }` を定義
- [x] `FlwBindingDef.bindings` の型を `Vec<(String, SlotImpl)>` に変更
- [x] パーサーでは `SlotImpl::Global(name)` として保持（チェッカーで解決）

### 6-4: チェッカーの変更（`checker.rs`）

- [x] `check_flw_binding_def` で各 binding を `resolve_slot_impl(name)` で `Global`/`Local` に変換
  - [x] `env.is_local(name)` でローカル変数か判定
- [x] `Local` スロットの型照合: ローカル変数の型を型環境から取得して E048 条件を適用
- [x] `TypeExpr::TrfFn` → `Type::Trf(input, output, effects)` として解決する `resolve_type_expr` の拡張
- [x] テスト: `test_dynamic_injection_type_ok`: `fn f(save: A -> B)` + `bind p <- T { slot <- save }` が型検査を通る
- [x] テスト: `test_dynamic_injection_type_e048`: trf 型不一致の引数注入で E048 が出る

### 6-5: IR の拡張（`ir.rs`）

- [x] `IRExpr::TrfRef(idx: u16, ty: Type)` バリアントを追加（グローバル trf への参照値）
- [x] `IRExpr::CallTrfLocal { local: u16, arg: Box<IRExpr>, ty: Type }` バリアントを追加（ローカル trf 経由の呼び出し）

### 6-6: コンパイラの変更（`compiler.rs`）

- [x] `compile_flw_binding_def` で `SlotImpl::Local` の場合 `IRExpr::CallTrfLocal` を生成
- [x] 関数定義で trf 型パラメータを受け取る場合、ローカルスロットとして IR に展開

### 6-7: VM / コードジェン の拡張

- [x] `IRExpr::TrfRef` → `Opcode::LoadGlobal`（既存の `VMValue::CompiledFn` を流用。仕様の `VMValue::TrfRef(String)` より合理的）
- [x] `IRExpr::CallTrfLocal { local, arg }` → `LoadLocal` + `Call`（既存のコールパスを再利用）
- [x] テスト: `exec_artifact_main_runs_local_callable_param_source`（= `test_dynamic_injection_exec_ok`: 注入された trf が実行時に正しく呼ばれる）
- [x] `cargo test` が全通過すること

---

## Phase 7: `abstract trf` ジェネリック型パラメータ（v1.3.0 残件）

### 7-1: `AbstractTrfDef` の変更（`ast.rs`）

- [x] `AbstractTrfDef` に `type_params: Vec<String>` フィールドを追加
- [x] 既存の `AbstractTrfDef` コンストラクタ・参照箇所を更新（デフォルト `vec![]`）

### 7-2: パーサーの変更（`parser.rs`）

- [x] `parse_abstract_trf_def` で `<type_params>` のオプションパースを追加
- [x] パーサーテスト: `test_parse_abstract_trf_generic`（`abstract trf Fetch<T>: Id -> T? !Db` がパースできる）
- [x] パーサーテスト: `abstract trf Transform<A, B>: A -> B` がパースできる

### 7-3: チェッカーの変更（`checker.rs`）

- [x] `abstract_trf_registry` のジェネリック `AbstractTrfDef` をそのまま格納（型パラメータ保持）
- [x] `first_pass` で `AbstractTrfDef` の型環境登録時に型パラメータを考慮
- [x] `check_flw_binding_def` でスロット型が `AbstractTrfDef<T>` の場合:
  - [x] `FlwBindingDef.type_args` の対応要素を `T` に代入して期待型を具体化
  - [x] 既存の E048 照合ロジックを再利用
- [x] テスト: `test_abstract_trf_generic_parse`: ジェネリック abstract trf がパースできる（parser.rs の `test_parse_abstract_trf_generic`）
- [x] テスト: `test_abstract_trf_generic_binding_ok`: ジェネリック abstract trf をスロット型に使った束縛が通る
- [x] テスト: `test_abstract_trf_generic_binding_e048`: 型不一致で E048 が出る
- [x] `cargo test` が全通過すること

---

## Phase 8: テスト・ドキュメント

### 8-1: example ファイルの追加

- [x] `examples/bundle_demo.fav` を作成
  - [x] unreachable な `fn` を含む（`unused_helper` 等）
  - [x] `fav bundle` で到達不能コードが除外されることを確認
- [x] `examples/dynamic_inject.fav` を作成
  - [x] `abstract trf` ジェネリックをスロット型に使うパターン（fn引数渡し動的注入は `exec_artifact_main_runs_local_callable_param_source` でテスト済み）

### 8-2: langspec.md の更新

- [x] `versions/v1.4.0/langspec.md` を新規作成（v1.3.0 langspec を起点に追加）
  - [x] `fav explain --format json` スキーマ仕様（全フィールド一覧、型定義）
  - [x] v2.0.0 でのリネーム予告（`kind: "trf"` → `"stage"` 等）
  - [x] `fav bundle` / `--manifest` / `--explain` コマンド
  - [x] `manifest.json` スキーマ
  - [x] `fav graph` コマンド
  - [x] trf 第一級値の構文・制約（`fn f(save: A -> B !Fx)`）
  - [x] `abstract trf` ジェネリック構文

### 8-3: README.md の更新

- [x] v1.4.0 セクションを追加（`fav bundle` / explain JSON / `fav graph` / 動的注入の紹介）

### 8-4: 全体確認

- [x] `cargo build` で Rust コンパイラ警告ゼロ
- [x] `cargo test` 全テスト通過（441 passed）
- [x] `fav explain main.fav --format json` が有効な JSON を出力する
- [x] `fav bundle main.fav --manifest` で `manifest.json` が生成される
- [x] `fav bundle main.fav --explain` で `.fvc` + `explain.json` が生成される
- [x] `fav explain dist/app.fvc --format json` が artifact から JSON を出力する
- [x] `fav graph main.fav` が text 形式でグラフを出力する
- [x] `fn f(save: A -> B) -> ...` が型検査・実行できる
- [x] `abstract trf Fetch<T>: Id -> T? !Db` がスロット型として使える
- [x] `Cargo.toml` バージョンが `"1.4.0"`

---

## 全体完了条件

- [x] `cargo build` で警告ゼロ
- [x] `cargo test` 全テスト通過（441 passed）
- [x] `fav explain main.fav --format json` が仕様スキーマに準拠した JSON を出力する
- [x] `fav bundle main.fav -o dist/app.fvc --explain` が `.fvc` + `manifest.json` + `explain.json` を生成する
- [x] `included` / `excluded` が正確に到達可能性を反映している
- [x] `effects_required` が実行環境の capability チェックに使える
- [x] `fav explain dist/app.fvc` が artifact から explain を出力できる
- [x] `fav graph` が abstract flw 構造を text/mermaid で出力する
- [x] trf 第一級値として関数引数に渡せる
- [x] `abstract trf` にジェネリック型パラメータを付けられる
- [x] `Cargo.toml` バージョンが `"1.4.0"`

---

## 先送り一覧（守る）

| 制約 | バージョン |
|---|---|
| `fav explain diff` 専用コマンド（CI 差分比較） | v1.5.0 以降 |
| `fav graph --focus fn`（fn 依存グラフ） | v1.5.0 以降 |
| artifact の explain metadata 圧縮（gzip） | v2.0.0 |
| `PartialFlw` を型引数に取る関数 | v2.0.0 |
| `abstract flw` 継承 | v2.0.0 以降 |
| `abstract seq` / `abstract stage` / JSON キー renaming | v2.0.0 |
| Veltra との直接統合 | v2.0.0 以降 |
| `fav explain result`（Lineage Tracking, 計算グラフ由来追跡） | v2.0.0 以降 |

---

## 実装差異メモ（仕様との相違点）

- **`VMValue::TrfRef(String)`**: 仕様では新バリアント追加を想定していたが、実装では既存の `VMValue::CompiledFn` を `Opcode::LoadGlobal` で流用。機能的に同等かつシンプル。
- **テスト名の相違**:
  - `test_dynamic_injection_exec_ok` → `exec_artifact_main_runs_local_callable_param_source`（driver.rs）
  - `test_explain_fvc_format_json` → `bundle_explain_embedded_round_trip`（driver.rs）でカバー
  - `test_bundle_artifact_smaller_than_build` → `bundle_writes_artifact_manifest_and_explain` 内で `helper` 除外を確認
  - `test_abstract_trf_generic_parse` → `test_parse_abstract_trf_generic`（parser.rs）
- **`examples/dynamic_inject.fav`**: fn引数渡しの動的注入ではなく `abstract trf` ジェネリック束縛パターンで示している。fn引数渡しの動的注入は `exec_artifact_main_runs_local_callable_param_source` テストで検証済み。
