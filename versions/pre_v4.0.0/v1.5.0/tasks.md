# Favnir v1.5.0 タスク一覧 — CI/CD 統合 + 静的解析強化

作成日: 2026-05-08

> **ゴール**: explain 差分比較・fn 依存グラフ・ユーザー定義エフェクト・lint 強化により、
> CI/CD ワークフローへの統合と静的解析の完成度を高める。
>
> **前提**: v1.4.0 完了（441 テスト通過）
>
> **先送り残件の解消**:
> - `fav explain diff`（v1.4.0 先送り）
> - `fav graph --focus fn`（v1.4.0 先送り）
>
> **スコープ管理が最優先。Done definition を超えない。**

---

## Phase 0: バージョン更新

### 0-1: バージョン更新

- [x] `Cargo.toml` の `version` を `"1.5.0"` に更新
- [x] `main.rs` の HELP テキストを `v1.5.0` に更新
- [x] HELP に `explain diff` サブコマンドと `graph --focus fn` の説明を追加
- [x] `cargo build` が通ること

---

## Phase 1: `fav explain diff`

### 1-1: データ構造の定義

- [x] `ExplainDiff` 構造体（`from_label`, `to_label`, `fn_changes`, `trf_changes`, `flw_changes`, `type_changes`, `effects_added`, `effects_removed`, `breaking_changes`）を `driver.rs` に追加
- [x] `CategoryDiff` 構造体（`added: Vec<serde_json::Value>`, `removed`, `changed: Vec<ChangedEntry>`）を追加
- [x] `ChangedEntry` 構造体（`name: String`, `diffs: Vec<String>`）を追加

### 1-2: 比較ロジックの実装

- [x] `diff_explain_json(from_label, from: &Value, to_label, to: &Value) -> ExplainDiff` を実装
  - [x] `fns` / `trfs` / `flws` / `types` の各カテゴリを比較
  - [x] `effects_used` の差分を計算
- [x] `diff_category(from: &Value, to: &Value, key: &str) -> CategoryDiff` を実装
  - [x] `added`: `to` にあって `from` にない名前
  - [x] `removed`: `from` にあって `to` にない名前
  - [x] `changed`: 両方にあって `return_type`/`input_type`/`output_type`/`effects`/`params` が変わっている名前
- [x] `diff_entry(from: &Value, to: &Value) -> Vec<String>` を実装（フィールドごとの差分文字列）
- [x] `detect_breaking_changes(category: &str, diff: &CategoryDiff) -> Vec<String>` を実装
  - [x] fn/trf の削除 → 破壊的
  - [x] return_type / effects / params の変更 → 破壊的
  - [x] type フィールドの削除 → 破壊的
  - [x] 追加のみ → 非破壊的

### 1-3: テキスト/JSON レンダリング

- [x] `render_diff_text(diff: &ExplainDiff) -> String` を実装
  - [x] 変更なし → `"No changes detected."` を出力
  - [x] 変更あり → `+`/`-`/`~` プレフィックスでカテゴリごとに表示
  - [x] `[summary]` セクションに added/removed/changed カウントと breaking_changes を表示
- [x] `render_diff_json(diff: &ExplainDiff) -> String` を実装
  - [x] `schema_version`, `from`, `to`, `changes`, `summary` を含む
  - [x] `summary.breaking_changes` リストを含む

### 1-4: `load_explain_json` ヘルパー

- [x] `load_explain_json(path: &str) -> serde_json::Value` を実装
  - [x] `.json` → ファイル読み込み + パース
  - [x] `.fvc` → `read_artifact_from_path` + `explain_json_from_artifact`
  - [x] `.fav` → `load_and_check_program` + `compile_program` + `render_json`

### 1-5: `cmd_explain_diff` の実装

- [x] `pub fn cmd_explain_diff(from_path: &str, to_path: &str, format: &str)` を実装
- [x] `diff_explain_json` を呼び出して `ExplainDiff` を取得
- [x] `format == "json"` → `render_diff_json`、それ以外 → `render_diff_text`

### 1-6: CLI への統合

- [x] `main.rs` の `explain` コマンド処理で第2引数が `"diff"` の場合に `cmd_explain_diff` へルーティング
  - [x] `fav explain diff <from> <to>` を処理
  - [x] `--format <text|json>` フラグをパース

### 1-7: テスト

- [x] テスト: `explain_diff_no_changes` — 同じソース同士で "No changes detected." が出る
- [x] テスト: `explain_diff_fn_added_shows_in_diff` — fn 追加が `added` に入る（`explain_diff_fn_added` として実装）
- [x] テスト: `explain_diff_fn_removed_is_breaking` — fn 削除が `breaking_changes` に入る（`explain_diff_breaking` として実装）
- [x] テスト: `explain_diff_fn_changed_return_type` — return_type 変更が `changed` に入る（`explain_diff_breaking` 内でカバー）
- [x] テスト: `explain_diff_breaking_changes_list` — 複数の破壊的変更が正確にリストされる
- [x] テスト: `explain_diff_json_valid` — `--format json` が有効な JSON を出力する（`explain_diff_json_format` として実装）
- [x] `cargo test` が全通過すること

---

## Phase 2: `fav graph --focus fn`

### 2-1: fn 呼び出しグラフ構築

- [x] fn 呼び出しグラフを構築する機能を実装
  - [x] `driver.rs` の `render_graph_text_with_opts` 内で `crate::middle::ir::collect_deps` を使って inline 実装（仕様では `reachability.rs` への独立関数追加を想定していたが、既存 `collect_deps` を直接活用）

### 2-2: text レンダラーの拡張

- [x] `render_graph_text_with_opts(program, focus, entry, max_depth) -> String` を追加
  - [x] `focus == "fn"` の場合に fn 依存グラフを描画
  - [x] 出力形式: `fn X:\n  -> dep1\n  -> dep2` のフラット形式（仕様のツリー形式 `├──` ではなく簡素化）
  - [x] `--depth N` で表示深さを制限
  - [x] 循環参照があってもパニックしない

### 2-3: mermaid レンダラーの拡張

- [x] `render_graph_mermaid_with_opts(program, focus, entry, max_depth) -> String` を追加
  - [x] `focus == "fn"` の場合に fn 呼び出し辺 `main --> helper` を生成
  - [x] `flowchart LR` ヘッダで開始
  - [x] エフェクトを持つ fn ノードを赤系でスタイリング

### 2-4: CLI への統合

- [x] `cmd_graph` シグネチャに `entry: Option<&str>` と `depth: Option<usize>` を追加
- [x] `main.rs` の `graph` コマンドに `--entry <name>` フラグを追加
- [x] `main.rs` の `graph` コマンドに `--depth <n>` フラグを追加
- [x] `--focus flw`（デフォルト）の既存動作が変わらないこと

### 2-5: テスト

- [x] テスト: `graph_fn_text_shows_calls` — text 形式で fn 呼び出し依存が表示される
- [x] テスト: `graph_fn_mermaid_valid` — mermaid 形式が `flowchart LR` を含む有効な構文を出力する
- [x] テスト: `graph_fn_depth_limit` — `--depth 1` で直接呼び出しのみが表示される
- [x] テスト: `graph_fn_cycle_safe` — 循環参照があってもパニックしない
- [x] `cargo test` が全通過すること

---

## Phase 3: ユーザー定義エフェクト

### 3-1: `ast.rs` の変更

- [x] `EffectDef` 構造体（`visibility: Option<Visibility>`, `name: String`, `span: Span`）を追加
- [x] `Item` に `EffectDef(EffectDef)` バリアントを追加
- [x] `Item::span()` に `Item::EffectDef(e) => &e.span` を追加

### 3-2: `lexer.rs` の変更

- [x] `TokenKind::Effect` を追加
- [x] キーワードマップに `"effect" => TokenKind::Effect` を追加
- [x] `test_keywords` に `("effect", TokenKind::Effect)` を追加

### 3-3: `parser.rs` の変更

- [x] `parse_item` に `TokenKind::Effect` の分岐を追加（`parse_effect_def` を呼ぶ）
- [x] `parse_effect_def(&mut self, visibility: Option<Visibility>) -> Result<EffectDef, ParseError>` を実装
- [x] パーサーテスト: `effect Payment` がパースできる
- [x] パーサーテスト: `public effect Notification` がパースできる

### 3-4: `checker.rs` の変更

- [x] `Checker` 構造体に `effect_registry: HashSet<String>` フィールドを追加
- [x] `new()` / `new_with_resolver()` で初期化
- [x] `first_pass` で `Item::EffectDef` を `effect_registry` に登録
- [x] `check_effects_declared(&mut self, effects: &[Effect], span: &Span)` を実装
  - [x] 組み込みエフェクト定数を定義
    - 実装値: `["Pure", "Io", "Db", "Network", "File", "Trace", "Emit"]`
    - 仕様値: `["Io", "Db", "File", "Trace", "Emit"]`（`"Pure"` と `"Network"` は追加拡張）
  - [x] 未宣言エフェクトで E052 を発行
- [x] fn/trf/flw/abstract trf/abstract flw の型注釈チェック時に `check_effects_declared` を呼ぶ

### 3-5: `driver.rs` の変更（explain JSON）

- [x] `ExplainPrinter::render_json` の出力に `custom_effects` フィールドを追加
  - [x] `Item::EffectDef` を収集して `{ "name": ..., "public": ... }` のリストを生成

### 3-6: テスト

- [x] テスト: `effect_def_parses` — `effect Payment` がパースできる
- [x] テスト: `effect_def_registered` — `first_pass` 後に `effect_registry` に登録される
- [x] テスト: `effect_custom_in_trf_ok` — 宣言済みエフェクトを trf 注釈で使ってもエラーなし
- [x] テスト: `effect_unknown_e052` — 未宣言エフェクトで E052 が発生する
- [x] テスト: `effect_builtin_no_error` — 組み込みエフェクト（`Io`, `Db` など）は宣言不要
- [x] テスト: `explain_json_custom_effects` — `custom_effects` フィールドが explain JSON に含まれる
- [x] `cargo test` が全通過すること

---

## Phase 4: `fav lint` 強化

### 4-1: `collect_trf_flw_uses` の実装

- [x] `lint.rs` に `collect_trf_flw_uses(program: &Program) -> HashSet<String>` を追加
  - [x] `FlwDef.steps` の名前を収集
  - [x] `FlwBindingDef.template` と `bindings` の実装名を収集
  - [x] fn/trf 本体の AST を走査して `Ident` 参照名を収集

### 4-2: L005 の実装

- [x] 未参照 かつ private な `TrfDef` に L005 を発行
- [x] 未参照 かつ private な `AbstractTrfDef` に L005 を発行
- [x] 未参照 かつ private な `FlwDef` に L005 を発行
- [x] 未参照 かつ private な `AbstractFlwDef` に L005 を発行
- [x] 未参照 かつ private な `FlwBindingDef` に L005 を発行

### 4-3: L006 の実装

- [x] `TrfDef.name` が `is_pascal_case` でない場合 L006 を発行

### 4-4: L007 の実装

- [x] `EffectDef.name` が `is_pascal_case` でない場合 L007 を発行

### 4-5: テスト

- [x] テスト: `lint_l005_unused_trf` — 未参照 private trf に L005 が出る
- [x] テスト: `lint_l005_public_trf_ignored` — public trf に L005 が出ない
- [x] テスト: `lint_l005_unused_flw` — 未参照 private flw に L005 が出る
- [x] テスト: `lint_l005_used_trf_no_warning` — 参照済み trf に L005 が出ない
- [x] テスト: `lint_l006_trf_not_pascal` — trf 名が非 PascalCase で L006 が出る
- [x] テスト: `lint_l006_trf_pascal_ok` — trf 名が PascalCase で L006 が出ない
- [x] テスト: `lint_l007_effect_not_pascal` — effect 名が非 PascalCase で L007 が出る
- [x] `cargo test` が全通過すること

---

## Phase 5: テスト・ドキュメント

### 5-1: example ファイルの追加

- [x] `examples/custom_effects.fav` を作成
- [x] `examples/diff_demo/old.fav` を作成
- [x] `examples/diff_demo/new.fav` を作成

### 5-2: langspec.md の作成

- [x] `versions/v1.5.0/langspec.md` を新規作成

### 5-3: README.md の更新

- [x] v1.5.0 セクションを追加

### 5-4: 全体確認

- [x] `cargo build` で Rust コンパイラ警告ゼロ
- [x] `cargo test` 全テスト通過（462 passed）
- [x] `fav explain diff` が差分を出力する
- [x] `fav graph --focus fn` が fn 依存グラフを出力する
- [x] ユーザー定義エフェクトが型検査で動作する
- [x] L005/L006/L007 が lint で動作する
- [x] `Cargo.toml` バージョンが `"1.5.0"`

---

## 全体完了条件

- [x] `cargo build` で警告ゼロ
- [x] `cargo test` 全テスト通過（462 passed）
- [x] `fav explain diff old.json new.json` が差分を text で出力する
- [x] `fav explain diff old.fav new.fav --format json` が差分 JSON を出力する
- [x] 破壊的変更が `breaking_changes` に正確に分類される
- [x] `fav graph src/main.fav --focus fn` が fn 呼び出し依存グラフを出力する
- [x] `fav graph src/main.fav --focus fn --format mermaid` が mermaid 形式を出力する
- [x] `effect Payment` がパースされチェッカーに登録される
- [x] 未宣言エフェクトで E052 が発生する
- [x] 組み込みエフェクト（`Io`, `Db`, `File`, `Trace`, `Emit`, `Pure`, `Network`）は宣言不要
- [x] 未参照の private trf/flw に L005 が発生する
- [x] trf 名の非 PascalCase に L006 が発生する
- [x] effect 名の非 PascalCase に L007 が発生する
- [x] v1.4.0 の全テストが引き続き通る
- [x] `Cargo.toml` バージョンが `"1.5.0"`

---

## 先送り一覧（守る）

| 制約 | バージョン |
|---|---|
| artifact の explain metadata 圧縮（gzip） | v2.0.0 |
| `PartialFlw` を型引数に取る関数 | v2.0.0 |
| `abstract flw` 継承 | v2.0.0 以降 |
| `abstract seq` / `abstract stage` / JSON キー renaming | v2.0.0 |
| Veltra との直接統合 | v2.0.0 以降 |
| `fav explain result`（Lineage Tracking） | v2.0.0 以降 |
| エフェクトの `use` による再エクスポート | v2.0.0 |
| エフェクト階層（`effect Foo extends Bar`） | v2.0.0 以降 |
| `fav lint` カスタムルールプラグイン | v2.0.0 以降 |

---

## 実装差異メモ（仕様との相違点）

- **2-1 `collect_fn_calls_from_ir`**: 仕様では `reachability.rs` への独立関数追加を想定していたが、`driver.rs` の `render_graph_text_with_opts` 内で既存の `crate::middle::ir::collect_deps` を直接活用して inline 実装。機能的に同等。
- **2-2 fn グラフのテキスト形式**: 仕様の `├──` ツリー形式ではなく、`fn X:\n  -> dep1` のフラット形式で実装。テストはこの形式に合わせて記述済み。
- **3-4 組み込みエフェクト**: 仕様の5個（`Io`, `Db`, `File`, `Trace`, `Emit`）に加えて `"Pure"` と `"Network"` が組み込みとして追加されている。完了条件の組み込みエフェクト一覧を更新済み。
- **テスト名の相違**:
  - `explain_diff_fn_added_shows_in_diff` → `explain_diff_fn_added`
  - `explain_diff_fn_removed_is_breaking` → `explain_diff_breaking`
  - `explain_diff_json_valid` → `explain_diff_json_format`
