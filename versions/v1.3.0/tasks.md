# Favnir v1.3.0 タスク一覧 — `abstract trf` / `abstract flw`

作成日: 2026-05-07

> **ゴール**: パイプライン構造そのものを抽象化し、型安全な依存注入を実現する
>
> **前提**: v1.2.0 完了（394 テスト通過）
>
> **スコープ管理が最優先。Done definition を超えない。**
>
> **注**: v1.3.0 では `abstract trf` / `abstract flw` キーワードを使用。v2.0.0 で `abstract stage` / `abstract seq` にリネーム予定。

---

## Phase 0: バージョン更新

### 0-1: バージョン更新

- [x] `Cargo.toml` の `version` を `"1.3.0"` に更新
- [x] `main.rs` の HELP テキストを `v1.3.0` に更新
- [x] `cargo build` が通ること

---

## Phase 1: AST + Lexer + Parser

### 1-1: ast.rs の変更

- [x] `AbstractTrfDef` 構造体を追加（`name`, `input_ty`, `output_ty`, `effects`, `span`）
- [x] `AbstractFlwDef` 構造体を追加（`name`, `type_params`, `slots`, `span`）
- [x] `FlwSlot` 構造体を追加（`name`, `input_ty`, `output_ty`, `effects`, `span`）
- [x] `FlwBindingDef` 構造体を追加（`name`, `template`, `type_args`, `bindings`, `span`）
- [x] `Item` enum に `AbstractTrfDef` / `AbstractFlwDef` / `FlwBindingDef` バリアントを追加
- [x] `Item` を参照している全箇所（compiler.rs, checker.rs, driver.rs 等）でコンパイルエラーがないこと

### 1-2: lexer.rs の変更

- [x] `Token::Abstract` バリアントを追加
- [x] キーワードマップに `"abstract" => Token::Abstract` を追加
- [x] 既存テストが全通過すること（`test_keywords` に `abstract` を含む）

### 1-3: parser.rs — `abstract trf` パース

- [x] `parse_abstract_trf_def(vis)` を実装
  - [x] `trf` → ident → `:` → type_expr → `->` → type_expr → effects の順にパース
  - [x] `AbstractTrfDef` を返す
- [x] パーサーテスト: `abstract trf FetchUser: UserId -> User? !Db` がパースできる
- [x] パーサーテスト: effects なし（`abstract trf Validate: User -> User!`）もパースできる

### 1-4: parser.rs — `abstract flw` パース

- [x] `parse_abstract_flw_def(vis)` を実装
  - [x] `flw` → ident → `<type_params>`（省略可） → `{` → slots → `}` の順にパース
  - [x] `parse_flw_slot()` を実装（`ident: type_expr -> type_expr effects`）
  - [x] 複数スロットのパースに対応（`;` 区切り）
- [x] パーサーテスト: 型パラメータなし `abstract flw` がパースできる
- [x] パーサーテスト: `abstract flw DataPipeline<Row> { parse: String -> List<Row>!; save: List<Row> -> Int !Db }` がパースできる（スロット 2 つ）
- [x] パーサーテスト: スロット 3 つ以上もパースできる

### 1-5: parser.rs — `flw` 束縛パース

- [x] `parse_flw_def_or_binding(vis)` を実装し `flw` の既存処理を置き換える
  - [x] ident 後に `:` → 既存 `FlwDef` パスに委譲
  - [x] ident 後に `=` (+ ident + `<` or `{`) → `parse_flw_binding_rest` に分岐
- [x] `parse_flw_binding_rest` を実装
  - [x] template_name → `<type_args>`（省略可） → `{` → `ident <- ident`* → `}` の順にパース
  - [x] `FlwBindingDef` を返す
- [x] パーサーテスト: `flw UserImport = DataPipeline<UserRow> { parse <- ParseCsv; save <- SaveUsers }` がパースできる
- [x] パーサーテスト: 型引数なし（`flw X = Template { slot <- Impl }`）もパースできる
- [x] パーサーテスト: 部分束縛（スロット 1 つのみ）もパースできる
- [x] `parse_item` に `Token::Abstract` の分岐を追加
  - [x] `Abstract` → `Trf` → `parse_abstract_trf_def`
  - [x] `Abstract` → `Flw` → `parse_abstract_flw_def`
  - [x] それ以外 → ParseError
- [x] `cargo test` が全通過すること

---

## Phase 2: 型検査統合

### 2-1: Checker の状態拡張

- [x] `Checker` 構造体に `abstract_trf_registry: HashMap<String, AbstractTrfDef>` を追加
- [x] `Checker` 構造体に `abstract_flw_registry: HashMap<String, AbstractFlwDef>` を追加
- [x] `Checker` 構造体に `flw_binding_info: HashMap<String, FlwBindingInfo>` を追加（explain 用）
  - [x] `FlwBindingInfo { template: String, bindings: Vec<(String, String)> }` を定義

### 2-2: 第1パス（first_pass）での登録

- [x] `first_pass` の `Item::AbstractTrfDef` で `abstract_trf_registry` に登録
  - [x] 型環境に `Type::AbstractTrf { input, output, effects }` を追加
- [x] `first_pass` の `Item::AbstractFlwDef` で `abstract_flw_registry` に登録
  - [x] 型環境に `Type::AbstractFlwTemplate(name)` を追加

### 2-3: `Type` enum の拡張

- [x] `Type::AbstractTrf { input, output, effects }` を追加
- [x] `Type::AbstractFlwTemplate(String)` を追加
- [x] `Type::PartialFlw { template: String, type_args: Vec<Type>, unbound_slots: Vec<String> }` を追加
- [x] `Type::display()` / `Type::is_compatible()` に新バリアントのハンドラを追加

### 2-4: E048–E051 エラーコードの定義

- [x] E048「abstract flw スロット型不一致」のエラーメッセージを定義
- [x] E049「未知スロット名（テンプレートに存在しない）」のエラーメッセージを定義
- [x] E050「PartialFlw を実行しようとした（必須スロット未束縛）」のエラーメッセージを定義
- [x] E051「abstract trf を直接実行しようとした」のエラーメッセージを定義

### 2-5: `check_flw_binding_def` の実装

- [x] テンプレート存在確認（未定義 → E002）
- [x] 型引数を型パラメータに代入して各スロット型を具体化（`TypeSubst` 利用）
- [x] 各束縛について:
  - [x] スロット名がテンプレートに存在するか確認（未知スロット → E049）
  - [x] 束縛実装の型を型環境から解決
  - [x] 期待型（具体化後）と実際の型を照合（不一致 → E048）
- [x] 未束縛スロット一覧を収集
  - [x] 未束縛なし → 完全束縛として型環境に `Type::Flw {...}` 相当を登録 + `flw_binding_info` に記録
  - [x] 未束縛あり → `Type::PartialFlw { ... }` として型環境に登録

### 2-6: Effect 推論

- [x] 完全束縛時に全スロットの effects を重複排除して合算
- [x] 推論した effects を型と `flw_binding_info` に記録
- [x] テスト: `test_flw_binding_effect_inference` が通ること

### 2-7: `abstract trf` 直接呼び出しの検査

- [x] `check_expr` の Apply/Call 処理で呼び出し先が `Type::AbstractTrf` なら E051 を出す

### 2-8: 型検査テスト

- [x] `test_flw_binding_type_ok`: 型一致スロット束縛が型検査を通る
- [x] `test_flw_binding_e048`: スロット型不一致で E048 が出る
- [x] `test_flw_binding_e049`: 未知スロット名で E049 が出る
- [x] `test_flw_binding_effect_inference`: 完全束縛の effect が正しく推論される
- [x] `test_flw_partial_type`: 部分束縛が `PartialFlw` 型になる
- [x] `test_abstract_trf_direct_call_e051`: abstract trf 直接呼び出しで E051 が出る
- [x] `test_flw_binding_type_params`: 型パラメータ付き `abstract flw` の束縛型検査が通る
- [x] `cargo test` が全通過すること

---

## Phase 3: IR + VM 実行

### 3-1: compiler.rs の変更

- [x] `Item::AbstractTrfDef` → IR 生成なし（グローバル登録のみ）
- [x] `Item::AbstractFlwDef` → IR 生成なし（グローバル登録のみ）
- [x] `Item::FlwBindingDef` で完全束縛の場合のみ `compile_flw_binding_def` を呼ぶ
- [x] `compile_flw_binding_def` を実装
  - [x] テンプレートのスロット順に従い bound 実装を直列合成した `IRFnDef` を生成
  - [x] 関数名: `FlwBindingDef.name`
  - [x] 本体: 各スロット実装を順次呼び出す IR
- [x] `PartialFlw` の場合は IR 生成しない（`fully_bound_flw_info` でガード）

### 3-2: driver.rs — PartialFlw の実行阻止

- [x] `ensure_no_partial_flw` を実装
- [x] `cmd_run` / `cmd_build` 前に呼び出し、`PartialFlw` があれば E050 でエラー終了

### 3-3: 実行テスト

- [x] `compile_flw_binding_exec_ok`: 完全束縛 flw の IR が正しく生成される
- [x] `compile_flw_binding_partial_skips_fn_codegen`: 部分束縛は IR 関数を生成しない
- [x] `ensure_no_partial_flw_e050`: `PartialFlw` を含むプログラムの実行で E050 が出る
- [x] `example_abstract_flw_basic_build_and_exec`: example ファイルがビルド+実行できる
- [x] `example_abstract_flw_inject_build_and_exec`: example ファイルがビルド+実行できる
- [x] `cargo test` が全通過すること

---

## Phase 4: `fav check` 部分束縛警告

### 4-1: check コマンドでの警告

- [x] `partial_flw_warnings` 関数を `driver.rs` に実装
  - [x] `find_partial_flw_bindings` でトップレベルの部分束縛を収集
  - [x] 警告コード W011「PartialFlw has unbound slots」を定義
- [x] `check_single_file` / `cmd_check` で `partial_flw_warnings` を呼び出し警告を表示
- [x] テスト: `partial_flw_warnings` が未束縛スロットを W011 として報告する

---

## Phase 5: `fav explain` 統合

### 5-1: abstract trf の表示

- [x] `ExplainPrinter` の出力に `ABSTRACT TRF` セクションを追加
  - [x] 書式: `Name: Input -> Output !Effects`
- [x] テスト: `explain_render_shows_abstract_trf_section` が `ABSTRACT TRF` を含む

### 5-2: abstract flw テンプレートの表示

- [x] `ExplainPrinter` の出力に `ABSTRACT FLW` セクションを追加
  - [x] 書式: テンプレート名（型パラメータ付き）+ 各スロット（名前・型・effects）
- [x] テスト: `explain_render_shows_abstract_flw_section` が `ABSTRACT FLW` を含む

### 5-3: 具体束縛 flw の表示

- [x] 具体束縛 `FlwBindingDef` のエントリにテンプレート名・バインディング先を表示
  - [x] 各スロット行に `<- ImplName` を追加表示
  - [x] `resolved:` 行に解決済みシグネチャを表示
  - [x] `effects:` 行に合成 effect を表示
- [x] テスト: `explain_render_shows_flw_binding_detail` が束縛情報を含む
- [x] `cargo test` が全通過すること

---

## Phase 6: テスト・ドキュメント

### 6-1: example ファイルの追加

- [x] `examples/abstract_flw_basic.fav` を作成
  - [x] `abstract flw` テンプレート定義 + 完全束縛 + `fav run` で動作確認
- [x] `examples/abstract_flw_inject.fav` を作成
  - [x] `fav run` で動作確認
  - 注: 仕様書が想定した「関数引数による動的注入パターン」ではなく、別テンプレートの静的束縛例として実装されている。動的注入（`fn make_import(save: ...) -> flw ...`）は v1.3.0 では先送り（先送り一覧参照）。

### 6-2: langspec.md の更新

- [x] `versions/v1.3.0/langspec.md` を新規作成
  - [x] `abstract trf` 構文・意味・E051
  - [x] `abstract flw` テンプレート構文
  - [x] スロット束縛のルール（E048・E049）
  - [x] `PartialFlw` の制約（W011・E050）
  - [x] E048–E051 エラーコード記載

### 6-3: README.md の更新

- [x] v1.3.0 セクションを追加（abstract trf/flw の紹介）

### 6-4: 全体確認

- [x] `cargo build` で Rust コンパイラ警告ゼロ
- [x] `cargo test` 全テスト通過（v1.2.0 継承 + 新規テスト）
- [x] `abstract flw DataPipeline<Row> { parse: String -> List<Row>!; save: List<Row> -> Int !Db }` が定義できる
- [x] `flw UserImport = DataPipeline<UserRow> { parse <- ParseCsv; save <- SaveUsers }` が型検査を通る
- [x] スロット型不一致で E048 が出る
- [x] `PartialFlw` が `fav run` で E050 になる
- [x] `fav explain` にテンプレート名・バインディング・effects が表示される
- [x] `Cargo.toml` バージョンが `"1.3.0"`

---

## 全体完了条件

- [x] `cargo build` で警告ゼロ
- [x] `cargo test` 全テスト通過
- [x] `abstract flw DataPipeline<Row> { parse: String -> List<Row>!; save: List<Row> -> Int !Db }` が定義できる
- [x] `flw UserImport = DataPipeline<UserRow> { parse <- ParseCsv; save <- SaveUsers }` が動く
- [x] スロット型不一致で E048 が出る
- [x] 部分束縛 `PartialFlw<...>` が `fav run`/`fav build` で E050 になる
- [x] `fav explain` にテンプレート名・具体バインディング・解決済み型が表示される
- [x] `Cargo.toml` バージョンが `"1.3.0"`

---

## 先送り一覧（守る）

| 制約 | バージョン |
|---|---|
| 関数引数による動的注入パターン（`fn f(save: ...) -> flw ...`） | v1.4.0 以降（trf を第一級値として渡す仕組みが必要） |
| スロット間の型連続性検査（parse 出力 = validate 入力） | v2.0.0 セルフホスト時 |
| `abstract flw` のネスト（flw のスロットが別 abstract flw） | v1.5.0 以降 |
| `fav graph` での abstract flw ノード描画 | v1.4.0 以降 |
| `PartialFlw` を型引数に取る関数 | v2.0.0 |
| `abstract flw` の継承（他の abstract flw を拡張） | v2.0.0 以降 |
| `abstract trf` のジェネリック型パラメータ | v1.4.0 以降 |
| `abstract seq` / `abstract stage` へのリネーム | v2.0.0 |
