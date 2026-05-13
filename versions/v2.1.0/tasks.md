# Favnir v2.1.0 タスクリスト

作成日: 2026-05-11

---

## Phase 0 — バージョン更新

- [x] `Cargo.toml`: `version = "2.1.0"` に変更
- [x] `src/main.rs`: HELP テキストを `v2.1.0` に更新
- [x] `src/main.rs`: `new` コマンドを HELP テキストに追加

---

## Phase 1 — 標準ライブラリ補完

### Math モジュール（`src/backend/vm.rs`）

- [x] `Math.abs(Int) -> Int` を実装
- [x] `Math.abs_float(Float) -> Float` を実装
- [x] `Math.min(Int, Int) -> Int` を実装
- [x] `Math.max(Int, Int) -> Int` を実装
- [x] `Math.min_float(Float, Float) -> Float` を実装
- [x] `Math.max_float(Float, Float) -> Float` を実装
- [x] `Math.clamp(Int, Int, Int) -> Int` を実装
- [x] `Math.pow(Int, Int) -> Int` を実装
- [x] `Math.pow_float(Float, Float) -> Float` を実装
- [x] `Math.sqrt(Float) -> Float` を実装
- [x] `Math.floor(Float) -> Int` を実装
- [x] `Math.ceil(Float) -> Int` を実装
- [x] `Math.round(Float) -> Int` を実装
- [x] `Math.pi` (定数 Float) を実装
- [x] `Math.e` (定数 Float) を実装

### Math モジュール（`src/middle/compiler.rs`）

- [x] Math の全関数をビルトイン型テーブルに登録
- [x] `Math.pi` / `Math.e` を引数なし定数として登録

### List 補完（`src/backend/vm.rs`）

- [x] `List.unique(List<T>) -> List<T>` を実装（初出順保持）
- [x] `List.flatten(List<List<T>>) -> List<T>` を実装
- [x] `List.chunk(List<T>, Int) -> List<List<T>>` を実装
- [x] `List.sum(List<Int>) -> Int` を実装
- [x] `List.sum_float(List<Float>) -> Float` を実装
- [x] `List.min(List<Int>) -> Option<Int>` を実装
- [x] `List.max(List<Int>) -> Option<Int>` を実装
- [x] `List.count(List<T>, T -> Bool) -> Int` を実装

### List 補完（`src/middle/compiler.rs`）

- [x] 上記 List 関数の型シグネチャをビルトイン型テーブルに追加

### String 補完（`src/backend/vm.rs`）

- [x] `String.index_of(String, String) -> Option<Int>` を実装
- [x] `String.pad_left(String, Int, String) -> String` を実装
- [x] `String.pad_right(String, Int, String) -> String` を実装
- [x] `String.reverse(String) -> String` を実装
- [x] `String.lines(String) -> List<String>` を実装（\r\n / \n 対応）
- [x] `String.words(String) -> List<String>` を実装（trim + 空文字除去）

### String 補完（`src/middle/compiler.rs`）

- [x] 上記 String 関数の型シグネチャをビルトイン型テーブルに追加

### IO 補完（`src/backend/vm.rs`）

- [x] `IO.read_line() -> String !Io` を実装
  - [x] `SUPPRESS_IO_OUTPUT` フラグが立っている場合は空文字列を返す
  - [x] 末尾の `\n` / `\r\n` を除去する

### IO 補完（`src/middle/compiler.rs`）

- [x] `IO.read_line` を `Fn([], String, effects=[Io])` として登録

### テスト（`src/backend/vm_stdlib_tests.rs`）

- [x] `math_abs_positive`: `Math.abs(5)` → `5`
- [x] `math_abs_negative`: `Math.abs(-5)` → `5`
- [x] `math_sqrt`: `Math.sqrt(4.0)` → `2.0`
- [x] `math_clamp_above`: `Math.clamp(10, 0, 5)` → `5`
- [x] `math_clamp_below`: `Math.clamp(-1, 0, 5)` → `0`
- [x] `math_clamp_inside`: `Math.clamp(3, 0, 5)` → `3`
- [x] `math_pow`: `Math.pow(2, 10)` → `1024`
- [x] `math_floor`: `Math.floor(3.7)` → `3`
- [x] `math_ceil`: `Math.ceil(3.2)` → `4`
- [x] `math_round`: `Math.round(3.5)` → `4`
- [x] `math_pi_is_float`: `Math.pi` が `Float` を返す
- [x] `list_unique_removes_duplicates`: `[1, 2, 1, 3]` → `[1, 2, 3]`
- [x] `list_unique_preserves_order`: 初出順が維持される
- [x] `list_flatten_one_level`: `[[1, 2], [3]]` → `[1, 2, 3]`
- [x] `list_chunk_even`: `[1,2,3,4], 2` → `[[1,2],[3,4]]`
- [x] `list_chunk_remainder`: `[1,2,3,4,5], 2` → `[[1,2],[3,4],[5]]`
- [x] `list_sum`: `[1, 2, 3]` → `6`
- [x] `list_sum_empty`: `[]` → `0`
- [x] `list_min`: `[3, 1, 2]` → `Some(1)`
- [x] `list_min_empty`: `[]` → `None`
- [x] `list_max`: `[3, 1, 2]` → `Some(3)`
- [x] `list_count`: `[1,2,3,4], |x| x > 2` → `2`
- [x] `string_index_of_found`: `"hello", "ll"` → `Some(2)`
- [x] `string_index_of_not_found`: `"hello", "zz"` → `None`
- [x] `string_pad_left`: `"42", 5, "0"` → `"00042"`
- [x] `string_pad_right`: `"hi", 5, "."` → `"hi..."`
- [x] `string_reverse`: `"abc"` → `"cba"`
- [x] `string_lines`: `"a\nb\nc"` → `["a", "b", "c"]`
- [x] `string_words`: `"  foo  bar  "` → `["foo", "bar"]`
- [x] `io_read_line_suppressed_returns_empty`: suppress モード時に空文字列

---

## Phase 2 — 論理演算子

### レキサー（`src/frontend/lexer.rs`）

- [x] `TokenKind::AmpAmp` を追加
- [x] `TokenKind::PipePipe` を追加
- [x] `&&` を `AmpAmp` としてトークン化する（`&` の先読みで分岐）
- [x] `||` を `PipePipe` としてトークン化する（`|` の先読み、既存の `|` と分岐）

### AST（`src/frontend/ast.rs`）

- [x] `BinOp::And` を追加
- [x] `BinOp::Or` を追加

### パーサー（`src/frontend/parser.rs`）

- [x] `&&` の優先順位を追加（比較演算子より低く `||` より高い）
- [x] `||` の優先順位を追加（`&&` より低く `??` より低い）
- [x] `AmpAmp` → `BinOp::And` の変換を追加
- [x] `PipePipe` → `BinOp::Or` の変換を追加

### 型チェッカー（`src/middle/checker.rs`）

- [x] `BinOp::And`: 左右辺が `Bool` でなければ E070
- [x] `BinOp::Or`: 左右辺が `Bool` でなければ E071
- [x] 結果型を `Bool` として返す

### IR（`src/backend/ir.rs`）

- [x] `IRBinOp::And` を追加
- [x] `IRBinOp::Or` を追加

### コンパイラ（`src/middle/compiler.rs`）

- [x] `BinOp::And` → `IRBinOp::And` のマッピングを追加
- [x] `BinOp::Or` → `IRBinOp::Or` のマッピングを追加

### opcode（`src/backend/opcode.rs` または同等ファイル）

- [x] 既存 opcode との衝突がないことを確認する
- [x] `And = 0x2A`（または空き番号）を追加
- [x] `Or  = 0x2B`（または空き番号）を追加

### コード生成（`src/backend/codegen.rs`）

- [x] `IRBinOp::And` → `Opcode::And` の emit を追加
- [x] `IRBinOp::Or`  → `Opcode::Or` の emit を追加

### VM（`src/backend/vm.rs`）

- [x] `Opcode::And`: スタックから 2 Bool をポップして AND した結果をプッシュ
- [x] `Opcode::Or`: スタックから 2 Bool をポップして OR した結果をプッシュ

### テスト

- [x] `logical_and_true_true`: `true && true` → `true`（checker + vm）
- [x] `logical_and_true_false`: `true && false` → `false`
- [x] `logical_and_false_any`: `false && true` → `false`
- [x] `logical_or_false_true`: `false || true` → `true`
- [x] `logical_or_false_false`: `false || false` → `false`
- [x] `logical_and_non_bool_left_e070`: `1 && true` で E070
- [x] `logical_and_non_bool_right_e070`: `true && "x"` で E070
- [x] `logical_or_non_bool_e071`: `true || 1` で E071
- [x] `logical_and_precedence`: `1 == 1 && 2 == 2` が `(1==1) && (2==2)` として評価される
- [x] `logical_or_precedence`: `false || 1 == 1` が `false || (1==1)` として評価される

---

## Phase 3 — `fav new` コマンド

### ドライバー（`src/driver.rs`）

- [x] `cmd_new(name: &str, template: &str)` を実装
- [x] `create_script_project(name)` を実装
  - [x] `fav.toml` を生成
  - [x] `src/main.fav` を生成（greet 関数雛形）
- [x] `create_pipeline_project(name)` を実装
  - [x] `fav.toml` を生成
  - [x] `rune.toml` を生成
  - [x] `src/main.fav` を生成
  - [x] `src/pipeline.fav` を生成（seq MainPipeline 雛形）
  - [x] `src/stages/parse.fav` を生成
  - [x] `src/stages/validate.fav` を生成
  - [x] `src/stages/save.fav` を生成
- [x] `create_lib_project(name)` を実装
  - [x] `fav.toml` を生成
  - [x] `rune.toml` を生成
  - [x] `src/lib.fav` を生成
  - [x] `src/lib.test.fav` を生成
- [x] 既存ディレクトリと同名の場合にエラーで終了する
- [x] 生成後に `cd <name>` / `fav run src/main.fav` のガイドを出力する

### CLI（`src/main.rs`）

- [x] `"new"` サブコマンドの解析を追加
  - [x] `--template <name>` オプション（デフォルト `"script"`）
  - [x] 位置引数（プロジェクト名）
- [x] `driver::cmd_new` に接続する
- [x] 引数なしで `Usage:` メッセージを出力する

### テスト（`src/driver.rs`）

- [x] `fav_new_script_creates_files`: `cmd_new("_test_script", "script")` でファイルが生成される
- [x] `fav_new_pipeline_creates_files`: pipeline テンプレートで全ファイルが生成される
- [x] `fav_new_lib_creates_files`: lib テンプレートで全ファイルが生成される
- [x] `fav_new_fails_on_existing_dir`: 既存ディレクトリ名でエラー
- [x] `fav_new_invalid_template_fails`: 不明テンプレート名でエラー
- [x] テスト後に生成ディレクトリを削除するクリーンアップを実装

---

## Phase 4 — CLI ウェルカム画面

### 依存追加（`Cargo.toml`）

- [x] `viuer = "0.7"` を追加
- [x] `supports-color = "3"` を追加
- [x] `image` クレートが必要な場合は追加

### 実装（`src/main.rs`）

- [x] `print_welcome()` 関数を実装
  - [x] `NO_COLOR` 環境変数チェック → 絵文字フォールバック
  - [x] `versions/favnir.png` を `include_bytes!` で埋め込む
  - [x] `viuer::print` でターミナルに画像を表示する
  - [x] コマンド一覧テキストを出力する
- [x] `main()` の引数なし（`args.len() == 1`）で `print_welcome()` を呼ぶ
- [x] `--help` フラグでも `print_welcome()` を呼ぶ

### 動作確認

- [x] `fav` 実行時にドラゴンアイコンとバージョンが表示される
- [x] `fav --help` でも同じ画面が表示される
- [x] `NO_COLOR=1 fav` で絵文字のみの表示になる
- [x] コマンド一覧に `fav new` が含まれている

---

## Phase 5 — テスト・ドキュメント

### 最終テスト確認

- [x] `cargo build` で警告ゼロを確認
- [x] `cargo test` で全テスト通過を確認（v2.0.0 の 538 以上）
- [x] `fav check examples/stage_seq_demo.fav` が通ることを確認
- [x] `fav run` で `true && false` を含むサンプルが動くことを確認
- [x] `fav new demo_proj` でプロジェクトが生成され `fav run demo_proj/src/main.fav` が動くことを確認

### ドキュメント作成

- [x] `versions/v2.1.0/langspec.md` を作成
  - [x] Math モジュール全関数の一覧と例
  - [x] List/String 補完関数の一覧と例
  - [x] `IO.read_line` の説明
  - [x] `&&` / `||` の構文・優先順位・エラーコード
  - [x] `fav new` コマンドの説明（3テンプレート）
  - [x] CLI ウェルカム画面の説明
  - [x] エラーコード一覧（E070 / E071 追加）

---

## 完了条件チェック

- [x] `Math.sqrt(2.0)` が正しい値を返す
- [x] `List.unique([1, 2, 1, 3])` が `[1, 2, 3]` を返す
- [x] `String.pad_left("42", 5, "0")` が `"00042"` を返す
- [x] `IO.read_line()` が標準入力から 1 行読める
- [x] `true && false` が `false` を返す
- [x] `false || true` が `true` を返す
- [x] `&&`/`||` の辺が `Bool` でない場合に E070/E071 が出る
- [x] `fav new my-tool` でプロジェクト雛形が生成される
- [x] `fav new my-pipeline --template pipeline` で stage/seq 構成が生成される
- [x] `fav new my-rune --template lib` で lib 構成が生成される
- [x] `fav`（引数なし）でドラゴンアイコンとウェルカムメッセージが表示される
- [x] `NO_COLOR` 環境では絵文字フォールバックになる
- [x] `cargo test` 全テスト通過
- [x] `cargo build` 警告ゼロ
- [x] `Cargo.toml` バージョンが `"2.1.0"`
- [x] `versions/v2.1.0/langspec.md` 作成済み
