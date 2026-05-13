# Favnir v2.1.0 仕様書 — 標準ライブラリ補完 + 論理演算子 + CLI ウェルカム

作成日: 2026-05-11

> **テーマ**: 数値計算・コレクション操作・文字列処理・対話入力の穴を埋め、
> 実用プログラムが書ける最低限を揃える。
> 論理演算子 `&&`/`||` を追加し、条件式の冗長さを解消する。
> `fav new` でプロジェクト雛形を生成し、CLI の第一印象を整える。
>
> **前提**: v2.0.0 完了（538 テスト通過）

---

## 1. スコープ概要

| Phase | テーマ | Done definition |
|---|---|---|
| 0 | バージョン更新 | `v2.1.0` がビルドされ HELP テキストに反映される |
| 1 | 標準ライブラリ補完 | Math/List/String/IO の全追加関数が動作する |
| 2 | 論理演算子 | `&&`/`||` が Bool 両辺に対して動作し、型エラーが適切に出る |
| 3 | `fav new` コマンド | 3 種のテンプレートでプロジェクト雛形が生成される |
| 4 | CLI ウェルカム画面 | `fav` 引数なしでドラゴンアイコンとコマンド一覧が表示される |
| 5 | テスト・ドキュメント | 全テスト通過、langspec v2.1.0 |

---

## 2. Phase 0 — バージョン更新

- `Cargo.toml`: `version = "2.1.0"`
- `src/main.rs`: HELP テキスト `v2.1.0`

---

## 3. Phase 1 — 標準ライブラリ補完

全関数は VM プリミティブ（Rust 側）として実装する。

### 3-1. Math モジュール（新規）

| 関数 | シグネチャ | 備考 |
|---|---|---|
| `Math.abs` | `Int -> Int` | 絶対値 |
| `Math.abs_float` | `Float -> Float` | |
| `Math.min` | `Int -> Int -> Int` | 小さい方を返す |
| `Math.max` | `Int -> Int -> Int` | 大きい方を返す |
| `Math.min_float` | `Float -> Float -> Float` | |
| `Math.max_float` | `Float -> Float -> Float` | |
| `Math.clamp` | `Int -> Int -> Int -> Int` | `clamp(value, lo, hi)` — 範囲内に収める |
| `Math.pow` | `Int -> Int -> Int` | 整数べき乗 (`pow(base, exp)`) |
| `Math.pow_float` | `Float -> Float -> Float` | |
| `Math.sqrt` | `Float -> Float` | 平方根 |
| `Math.floor` | `Float -> Int` | 切り捨て |
| `Math.ceil` | `Float -> Int` | 切り上げ |
| `Math.round` | `Float -> Int` | 四捨五入 |
| `Math.pi` | `Float` | 定数 3.141592653589793 |
| `Math.e` | `Float` | 定数 2.718281828459045 |

`Math.pi` / `Math.e` は引数なし（定数）。チェッカーでは `Math` モジュールに
定数フィールドとして登録し、`Math.pi` のフィールドアクセスを `Float` として解決する。

### 3-2. List 補完

| 関数 | シグネチャ | 備考 |
|---|---|---|
| `List.unique` | `List<T> -> List<T>` | 重複除去（初出順保持） |
| `List.flatten` | `List<List<T>> -> List<T>` | ネスト 1 段を平坦化 |
| `List.chunk` | `List<T> -> Int -> List<List<T>>` | 固定長分割（末尾は短くなる） |
| `List.sum` | `List<Int> -> Int` | 合計（空リスト = 0） |
| `List.sum_float` | `List<Float> -> Float` | 合計（空リスト = 0.0） |
| `List.min` | `List<Int> -> Option<Int>` | 最小値（空リスト = None） |
| `List.max` | `List<Int> -> Option<Int>` | 最大値（空リスト = None） |
| `List.count` | `List<T> -> (T -> Bool) -> Int` | 条件を満たす要素数 |

### 3-3. String 補完

| 関数 | シグネチャ | 備考 |
|---|---|---|
| `String.index_of` | `String -> String -> Option<Int>` | 部分文字列の先頭位置 |
| `String.pad_left` | `String -> Int -> String -> String` | `pad_left(s, width, fill)` |
| `String.pad_right` | `String -> Int -> String -> String` | `pad_right(s, width, fill)` |
| `String.reverse` | `String -> String` | 文字列を逆順にする |
| `String.lines` | `String -> List<String>` | `\n`/`\r\n` で分割 |
| `String.words` | `String -> List<String>` | 空白で分割（trim 後、空文字列除去） |

### 3-4. IO 補完

| 関数 | シグネチャ | エフェクト |
|---|---|---|
| `IO.read_line` | `() -> String` | `!Io` |

- `fav test` 実行時: `SUPPRESS_IO_OUTPUT` と同様の仕組みで空文字列を返す
  （テストが対話入力に依存しないよう保証する）
- VM での実装: `std::io::stdin().lock().lines().next()` で 1 行読み込む

---

## 4. Phase 2 — 論理演算子

### 4-1. 設計方針

現状は `a && b` を `if a { b } else { false }` と書く必要があり冗長。
`&&` / `||` をネイティブ二項演算子として追加する。

```favnir
// Before
if a { if b { true } else { false } } else { false }

// After
a && b
a || b
```

`===` は追加しない。Favnir は静的型付きのため `==` が既に厳密等値。

### 4-2. 優先順位

高い方から低い方へ:
```
?? > || > && > == / != / < / > / <= / >=
```

`??` は v1.9.0 で追加済み。`&&` / `||` は比較演算子より低い優先度。

### 4-3. 字句解析（`src/frontend/lexer.rs`）

- `&&` → `TokenKind::AmpAmp`（2文字トークン: `&` の先読みで分岐）
- `||` → `TokenKind::PipePipe`（2文字トークン: `|` の先読みで分岐）

単独の `&` / `|` は現在未使用のため衝突しない。

### 4-4. AST（`src/frontend/ast.rs`）

```rust
// BinOp enum に追加
And,  // &&
Or,   // ||
```

### 4-5. パーサー（`src/frontend/parser.rs`）

二項演算子の優先順位テーブルに追加:
- `||`: `??` の直下（次に低いレベル）
- `&&`: `||` の直下

左結合。`a && b && c` = `(a && b) && c`

### 4-6. 型チェッカー（`src/middle/checker.rs`）

- `BinOp::And` / `BinOp::Or`: 両辺が `Bool` であることを検査
- 結果型: `Bool`
- エラー:
  - `&&` で非 Bool → E070
  - `||` で非 Bool → E071

### 4-7. コンパイラ→VM

- `BinOp::And` → `IRBinOp::And` → opcode `And = 0x2A`
- `BinOp::Or`  → `IRBinOp::Or`  → opcode `Or  = 0x2B`
- VM: `And(Bool, Bool) -> Bool`, `Or(Bool, Bool) -> Bool`
- 短絡評価（short-circuit）は将来課題（v2.1.0 では両辺を評価する）

### 4-8. エラーコード

| コード | 条件 |
|---|---|
| E070 | `&&` の左辺または右辺が `Bool` でない |
| E071 | `\|\|` の左辺または右辺が `Bool` でない |

---

## 5. Phase 3 — `fav new` コマンド

### 5-1. CLI 仕様

```
fav new <name> [--template script|pipeline|lib]
```

デフォルトテンプレート: `script`

既存ディレクトリと同名の場合はエラーで終了する。

### 5-2. テンプレート別ディレクトリ構成

**`script`（デフォルト）— fn だけで書ける最小構成**:
```
<name>/
  fav.toml
  src/
    main.fav
```

**`pipeline` — stage/seq フルスタック構成**:
```
<name>/
  fav.toml
  rune.toml
  src/
    main.fav
    pipeline.fav
    stages/
      parse.fav
      validate.fav
      save.fav
```

**`lib` — rune ライブラリ開発用**:
```
<name>/
  fav.toml
  rune.toml
  src/
    lib.fav
    lib.test.fav
```

### 5-3. 生成ファイル内容

**`fav.toml`（全テンプレート共通）**:
```toml
[project]
name    = "<name>"
version = "0.1.0"
edition = "2026"
src     = "src"
```

**`rune.toml`（pipeline / lib のみ）**:
```toml
[dependencies]

[dev-dependencies]
```

**`src/main.fav`（script テンプレート）**:
```favnir
public fn main() -> Unit !Io {
    IO.println(greet("world"))
}

fn greet(name: String) -> String {
    $"Hello {name}!"
}
```

**`src/main.fav`（pipeline テンプレート）**:
```favnir
import "pipeline"

public fn main() -> Unit !Io {
    // seq MainPipeline を実行する
    IO.println("pipeline: ok")
}
```

**`src/lib.fav`（lib テンプレート）**:
```favnir
// <name> rune — public API をここに定義する

public fn hello() -> String {
    "hello from <name>"
}
```

### 5-4. 実装場所

- `src/driver.rs`: `cmd_new(name: &str, template: &str)` を追加
- `src/main.rs`: `new` サブコマンドを追加・HELP テキストに掲載

---

## 6. Phase 4 — CLI ウェルカム画面

### 6-1. 表示条件

- `fav`（引数なし）または `fav --help` 実行時に表示
- 既存の HELP 出力を `print_welcome()` 関数に分離して強化する

### 6-2. 表示内容

```
  🐉  Favnir v2.1.0 — The pipeline-first language

  fav run <file>          Run a .fav file
  fav check <file>        Type-check without running
  fav test <file>         Run tests
  fav new <name>          Create a new project
  fav fmt <file>          Format source code
  fav lint <file>         Run linter
  fav bench <file>        Run benchmarks
  fav migrate <file>      Migrate v1.x code to v2.x
  fav explain <file>      Show pipeline structure
  fav watch <file>        Watch and re-run on change

  fav help <command>      Show detailed help
```

### 6-3. アイコン表示

- `viuer` クレート（`versions/favnir.png` を `include_bytes!` で埋め込み）
  - kitty / iTerm2 / WezTerm: 実画像として表示
  - その他のターミナル: Unicode ブロック文字で近似表示
- `NO_COLOR` 環境変数が設定されている場合: 絵文字 🐉 のみ表示
- `supports-color` クレートでターミナルの色サポートを自動検出

### 6-4. 実装場所

- `src/main.rs`: `print_welcome()` 関数を追加
- `Cargo.toml`: `viuer`, `supports-color` 依存を追加

---

## 7. Phase 5 — テスト・ドキュメント

### 7-1. テスト要件

**Math モジュール**（`src/backend/vm_stdlib_tests.rs` または `src/middle/checker.rs`）:

| テスト | 期待値 |
|---|---|
| `Math.sqrt(4.0)` | `2.0` |
| `Math.abs(-5)` | `5` |
| `Math.abs(5)` | `5` |
| `Math.min(3, 7)` | `3` |
| `Math.max(3, 7)` | `7` |
| `Math.clamp(10, 0, 5)` | `5` |
| `Math.clamp(-1, 0, 5)` | `0` |
| `Math.pow(2, 10)` | `1024` |
| `Math.floor(3.7)` | `3` |
| `Math.ceil(3.2)` | `4` |
| `Math.round(3.5)` | `4` |
| `Math.pi` | `3.141...` (Float) |

**List 補完**:

| テスト | 期待値 |
|---|---|
| `List.unique([1, 2, 1, 3])` | `[1, 2, 3]` |
| `List.flatten([[1, 2], [3]])` | `[1, 2, 3]` |
| `List.chunk([1,2,3,4,5], 2)` | `[[1,2],[3,4],[5]]` |
| `List.sum([1, 2, 3])` | `6` |
| `List.sum([])` | `0` |
| `List.min([3, 1, 2])` | `Some(1)` |
| `List.min([])` | `None` |
| `List.max([3, 1, 2])` | `Some(3)` |
| `List.count([1,2,3,4], |x| x > 2)` | `2` |

**String 補完**:

| テスト | 期待値 |
|---|---|
| `String.pad_left("42", 5, "0")` | `"00042"` |
| `String.pad_right("hi", 5, ".")` | `"hi..."` |
| `String.reverse("abc")` | `"cba"` |
| `String.lines("a\nb\nc")` | `["a", "b", "c"]` |
| `String.words("  foo  bar  ")` | `["foo", "bar"]` |
| `String.index_of("hello", "ll")` | `Some(2)` |
| `String.index_of("hello", "zz")` | `None` |

**論理演算子**:

| テスト | 期待値 |
|---|---|
| `true && true` | `true` |
| `true && false` | `false` |
| `false && true` | `false` |
| `false || true` | `true` |
| `false || false` | `false` |
| `1 && true` | E070 型エラー |
| `true \|\| "x"` | E071 型エラー |
| `1 == 1 && 2 == 2` | `true`（`==` が `&&` より高優先） |

**IO.read_line**:
- `fav test` 実行時に空文字列を返し、テストがブロックされない

### 7-2. ドキュメント

- `versions/v2.1.0/langspec.md` — v2.1.0 言語仕様書（Math/論理演算子・fav new を追記）

---

## 8. エラーコード一覧（v2.1.0 追加分）

| コード | Phase | 条件 |
|---|---|---|
| E070 | 2 | `&&` の左辺または右辺が `Bool` でない |
| E071 | 2 | `\|\|` の左辺または右辺が `Bool` でない |

---

## 9. 後方互換性

v2.1.0 は加算リリース。v2.0.0 のコードはそのまま動く。

- 新規 VM プリミティブは追加のみ（既存関数の変更なし）
- `&&` / `||` はトークンとして予約済み扱いになる（元々演算子として使えなかったため問題なし）
- opcode `0x2A`/`0x2B` が新規割り当てになるため、v2.0.0 以前の `.fvc` は再コンパイルを推奨

---

## 10. 完了条件

- [ ] `Math.sqrt(2.0)` が正しい値を返す
- [ ] `List.unique([1, 2, 1, 3])` が `[1, 2, 3]` を返す
- [ ] `String.pad_left("42", 5, "0")` が `"00042"` を返す
- [ ] `IO.read_line()` が標準入力から 1 行読める
- [ ] `true && false` が `false` を返す
- [ ] `false || true` が `true` を返す
- [ ] `&&`/`||` の辺が `Bool` でない場合に E070/E071 が出る
- [ ] `fav new my-tool` でプロジェクト雛形が生成される
- [ ] `fav new my-pipeline --template pipeline` で stage/seq 構成が生成される
- [ ] `fav new my-rune --template lib` で lib 構成が生成される
- [ ] `fav`（引数なし）でドラゴンアイコンとウェルカムメッセージが表示される
- [ ] `NO_COLOR` 環境では絵文字フォールバックになる
- [ ] 既存テストが全て通る
- [ ] `cargo build` で警告ゼロ
- [ ] `Cargo.toml` バージョンが `"2.1.0"`

---

## 11. 先送り一覧

| 機能 | 理由 | 対応予定 |
|---|---|---|
| `&&`/`||` の短絡評価（short-circuit） | VM アーキテクチャの変更が必要 | v2.2.0 以降 |
| `!=` 演算子 | 現状 `==` の否定で代用可能、優先度低 | 将来候補 |
| `fav new` テンプレートのカスタマイズ | 使用実績を見てから | v2.6.0 以降 |
| `pipe match` / `pattern guard` | v2.2.0 スコープ | v2.2.0 |
