# Favnir ロードマップ v2.1.0 → v3.0.0

作成日: 2026-05-09

v2.0.0 完了後の進化の方針。
各バージョンは前のバージョンの完了を前提にして順番に進める。

---

## 方針

- **v2.x**: 言語の書き心地・標準ライブラリ・ツールチェーンを整備し、「書けるけど書きたくない」状態を解消する。
  型システムの表現力（ADT・ジェネリクス・インターフェース・エフェクト・型状態）は v2.0.0 で十分整ったため、
  次の層は「型チェッカーが知っていることを書き手に還元する」エイリアス・糖衣構文の整備が中心。
- **v3.0.0**: セルフホスト・マイルストーン到達（Favnir 製パーサーが Rust VM 上で完全動作）と
  エラーコード体系刷新による言語基盤の安定化。

---

## v2.1.0 — 標準ライブラリ補完 + CLI ウェルカム画面

**テーマ**: 数値計算・コレクション操作・文字列処理・対話入力の穴を埋め、実用プログラムが書ける最低限を揃える。あわせて CLI の第一印象を整える。

### 追加するもの

**Math モジュール（新規）**:
- `Math.abs(Int) -> Int` / `Math.abs_float(Float) -> Float`
- `Math.min(Int, Int) -> Int` / `Math.max(Int, Int) -> Int`
- `Math.min_float(Float, Float) -> Float` / `Math.max_float(Float, Float) -> Float`
- `Math.clamp(Int, Int, Int) -> Int` — 範囲内に収める
- `Math.pow(Int, Int) -> Int` / `Math.pow_float(Float, Float) -> Float`
- `Math.sqrt(Float) -> Float`
- `Math.floor(Float) -> Int` / `Math.ceil(Float) -> Int` / `Math.round(Float) -> Int`
- `Math.pi -> Float` / `Math.e -> Float` — 数学定数

**List 補完**:
- `List.unique(List<T>) -> List<T>` — 重複除去（順序保持）
- `List.flatten(List<List<T>>) -> List<T>` — ネストを平坦化
- `List.chunk(List<T>, Int) -> List<List<T>>` — 固定長で分割
- `List.sum(List<Int>) -> Int` / `List.sum_float(List<Float>) -> Float`
- `List.min(List<Int>) -> Option<Int>` / `List.max(List<Int>) -> Option<Int>`
- `List.count(List<T>, T -> Bool) -> Int` — 条件を満たす要素数

**String 補完**:
- `String.index_of(String, String) -> Option<Int>` — 部分文字列の先頭位置
- `String.pad_left(String, Int, String) -> String` — 左埋め
- `String.pad_right(String, Int, String) -> String` — 右埋め
- `String.reverse(String) -> String`
- `String.lines(String) -> List<String>` — 改行で分割
- `String.words(String) -> List<String>` — 空白で分割（trim 後）

**`fav new` — プロジェクトテンプレート生成**:
```
$ fav new my-tool                          # デフォルト = script
$ fav new my-tool --template script        # 最小構成（fn のみ）
$ fav new my-pipeline --template pipeline  # フルスタック（stage/seq 構成）
$ fav new my-rune --template lib           # rune ライブラリ開発用
```

**`--template script`（デフォルト）** — fn だけで書ける最小構成:
```
my-tool/
  fav.toml      ← プロジェクト設定（エディション・src パス等）
  src/
    main.fav    ← public fn main() + ヘルパー fn の雛形
```
```favnir
public fn main() -> Unit !Io {
    IO.println(greet("world"))
}

fn greet(name: String) -> String {
    $"Hello {name}!"
}
```
用途: 自動化スクリプト・ユーティリティ・小規模ツール。stage/seq は不要。

**`--template pipeline`** — stage/seq フルスタック構成:
```
my-pipeline/
  fav.toml      ← プロジェクト設定
  rune.toml     ← 依存する rune の宣言（依存あり時のみ生成）
  rune.lock     ← 解決済み依存の固定（自動生成）
  src/
    main.fav        ← エントリポイント（seq 実行）
    pipeline.fav    ← seq 定義
    stages/
      parse.fav     ← stage 雛形（入力変換）
      validate.fav  ← stage 雛形（検証）
      save.fav      ← stage 雛形（出力保存）
```
```favnir
// src/pipeline.fav
import "stages/parse"
import "stages/validate"
import "stages/save"

seq MainPipeline = parse.ParseInput |> validate.Validate |> save.Save
```
用途: データ基盤・本番パイプライン・エフェクト管理が必要な開発。

**`--template lib`** — rune ライブラリ開発用:
```
my-rune/
  fav.toml      ← プロジェクト設定（name/version/edition）
  rune.toml     ← このライブラリ自身の依存宣言
  src/
    lib.fav         ← public stage/fn の実装
    lib.test.fav    ← テスト
```
用途: `import rune "my-rune"` で他プロジェクトから使うライブラリ開発。

共通:
- `fav.toml`（プロジェクト設定: name/version/edition/src/template種別）を生成
- `rune.toml`（依存管理: dependencies/dev-dependencies）は依存がある場合のみ生成
- テンプレート種別は `fav explain` や `fav docs` の表示にも反映

**論理演算子**:
- `&&` — 論理AND（`Bool -> Bool -> Bool`）
- `||` — 論理OR（`Bool -> Bool -> Bool`）
- 現状は `a && b` を `if a { b } else { false }` と書くしかなく、条件式が冗長になっている
- 優先順位: `??` > `||` > `&&` > 比較演算子（`==`/`!=`/`<`/`>`等）
- `===` は追加しない（静的型付きのため `==` が既に厳密等値）
- 実装: lexer に `AmpAmp`/`PipePipe` トークン追加、AST に `BinOp::And`/`Or`、
  opcode `And = 0x2A`/`Or = 0x2B` 追加、checker で両辺 `Bool` を検査

**IO 補完**:
- `IO.read_line() -> String !Io` — 標準入力から 1 行読み込む
  （`fav test` 実行時は SUPPRESS_IO_OUTPUT と同様の仕組みで空文字列を返す）

**CLI ウェルカム画面**:
- `fav`（引数なし）または `fav --help` 実行時にドラゴンアイコン + バージョン + コマンド一覧を表示
- アイコン表示は `viuer` クレートで `versions/favnir.png` をバイナリに埋め込み（`include_bytes!`）
  - kitty / iTerm2 / WezTerm: 実画像として表示
  - その他のターミナル: Unicode ブロック文字で近似表示
  - 非対応 / `NO_COLOR` 環境: 🐉 絵文字にフォールバック
- バージョンは `env!("CARGO_PKG_VERSION")` で自動取得
- `supports-color` クレートでターミナルの色サポートを自動検出

```
  🐉  Favnir v2.1.0 — The pipeline-first language

  fav run <file>      Run a .fav file
  fav check <file>    Type-check without running
  fav test <file>     Run tests
  fav docs            Browse standard library (v3.1.0)
  fav repl            Start interactive REPL (v2.8.0)

  fav help <command>  Show detailed help
```

### 完了条件

- `Math.sqrt(2.0)` が正しい値を返す
- `List.unique([1, 2, 1, 3])` が `[1, 2, 3]` を返す
- `String.pad_left("42", 5, "0")` が `"00042"` を返す
- `IO.read_line()` で標準入力から 1 行読める
- `true && false` が `false` を返す
- `false || true` が `true` を返す
- `&&`/`||` の両辺が `Bool` でない場合に型エラーが出る
- `fav` 実行時にドラゴンアイコンとウェルカムメッセージが表示される
- `NO_COLOR` 環境では絵文字フォールバックになる
- 既存テストが全て通る

---

## v2.2.0 — pipe match + pattern guard

**テーマ**: Favnir のパイプライン×型安全の核心部分を完成させる

v2.0.0 時点で「パイプライン×T! が不完全」「条件付き match が冗長」という 2 大問題が残っている。
これを解消することで、Favnir が目指す「データフロー記述言語」としての書き心地が整う。

### 追加するもの

**`pipe match`（`|> match { ... }`）**:
```favnir
fetch_user(id)
  |> match {
    Ok(user) => render(user)
    Err(e)   => default_view(e)
  }
```
- パーサーで `|> match { arms }` を `PipeMatch` ノードとして解析
- チェッカーで左辺の型からアームの網羅性を検査
- コンパイラで中間 bind + match に脱糖

**`pattern guard`（`where` 句）**:
```favnir
match user {
  { role: "admin", age } where age >= 18 => grant_access(user)
  { role: "admin" }                       => deny_underage()
  _                                       => deny_unauthorized()
}
```
- パーサーでアーム末尾の `where <expr>` を解析
- チェッカーでガード式の型を Bool と検査
- アーム実行時にガード条件を評価し、不成立なら次のアームへ

### 完了条件

- `result |> match { Ok(v) => v Err(_) => 0 }` が動く
- `match x { { age } where age >= 18 => "adult" _ => "minor" }` が動く
- ガード不成立時に次アームへフォールスルーする
- 既存テストが全て通る

---

## v2.3.0 — 分割 bind + 戻り型推論

**テーマ**: 型チェッカーが知っていることを書き手に還元し、定型コードを削減する

### 追加するもの

**分割 bind（destructuring bind）**:
```favnir
bind { name, age } <- fetch_user(id)
// 展開後: bind name <- fetch_user(id).name; bind age <- fetch_user(id).age

bind { name, age: user_age } <- fetch_user(id)
// エイリアス付き: age フィールドを user_age として束縛

bind { name, _ } <- fetch_user(id)
// 残余無視
```
- パーサーで `bind { fields } <- expr` を `BindDestructure` として解析
- チェッカーで右辺の型からフィールドを検索・型チェック
- compiler.rs で連続する `bind` に脱糖
- VM の ignored テスト 2 件（E: destructuring）を解消

**戻り型推論**:
```favnir
fn double(n: Int) = n * 2        // -> Int を省略可
fn greet(name: String) = $"Hello {name}!"  // -> String を省略可
```
- `fn name(params) = expr`（`->` なし）構文を許可
- チェッカーで本体式の型を推論して戻り型として採用
- 再帰関数は推論不可（明示アノテーション必須）

### 完了条件

- `bind { x, y } <- point` が `bind x <- point.x; bind y <- point.y` と等価に動く
- `fn add(a: Int, b: Int) = a + b` が型チェックを通る
- `fn id(x: Int) -> Int = x` との混在が可能
- 既存テストが全て通る

---

## v2.4.0 — スタックトレース + ランタイム品質改善

**テーマ**: エラーが起きたときに「どこで何が起きたか」を明確に示す

現状のランタイムエラーは関数名と命令位置のみ。本番利用で最も欠けている機能。

### 追加するもの

**スタックトレース**:
- コンパイラで呼び出し情報（ファイル名・行番号・関数名）を埋め込む
- VM でコールスタックを追跡（`CallFrame { fn_name, source_file, line }` のスタック）
- ランタイムエラー時にスタックトレースを表示:
  ```
  RuntimeError: division by zero
    at divide (math.fav:12)
    at process (pipeline.fav:34)
    at main (main.fav:5)
  ```
- パニック（VM の到達不能コード）でもトレースを表示

**`Unknown` フォールバックの削減**（post1-roadmap B-0）:
- チェッカーで `Unknown` 型になっているケースを列挙し、型エラー or 型推論強化で解消
- `fav check` で `Unknown` 型の変数があれば警告を出す

**ignored VM テスト 2 件の解消**（post1-roadmap B-0）:
- `#[ignore]` が付いたテストを特定し、実装 or テスト削除で解消

### 完了条件

- ランタイムエラー時に 3 段以上のスタックトレースが表示される
- `fav check` で `Unknown` 型の警告が出る
- `#[ignore]` テストが 0 件になる
- 既存テストが全て通る

---

## v2.5.0 — LSP 補完・定義ジャンプ

**テーマ**: エディタなしでは書けない状態を解消する

現状は hover と diagnostics のみ。補完と定義ジャンプがないと実用的な開発ができない。

### 追加するもの

**補完（`textDocument/completion`）**:
- `.` 入力後のフィールド・メソッド補完（`user.` → `name`, `age`, ...）
- グローバル関数・型名の補完
- `bind x <- ` 後の候補補完
- キーワード補完（`stage`, `seq`, `interface`, `match`, `if`, ...）
- スニペット補完（`fn`, `type`, `interface`, `match` の雛形）

**定義ジャンプ（`textDocument/definition`）**:
- 関数名・型名・フィールド名の定義位置へジャンプ
- `interface` と `impl` 間のジャンプ

**補完品質向上**:
- `CheckedDoc` に補完用インデックス（シンボルテーブル）を追加
- ドキュメントコメント（`//` 行）を hover に表示

### 完了条件

- VS Code で `user.` と入力するとフィールド候補が表示される
- 関数名上で F12（定義ジャンプ）が動く
- グローバル名が補完候補に出る
- 既存テストが全て通る

---

## v2.6.0 — モジュールシステム（import/export）

**テーマ**: 複数ファイルにわたる実用規模のプログラムを書けるようにする

現状は単一ファイル前提。これがないと rune（パッケージ）システムの基盤もない。

### 設計仕様

**プロジェクト構造**:
```
my-project/
  fav.toml          ← プロジェクト設定（ビルド設定・src パス・edition）
  rune.toml         ← 依存管理（package.json 相当）← 依存ありの場合のみ
  rune.lock         ← 解決済み依存の固定（自動生成）← git 管理対象
  src/
    main.fav
    models/
      user.fav
  runes/            ← rune インストール先（node_modules 相当）← .gitignore
    validate/
      validate.fav
```

**ファイル役割の分離**:
| ファイル | 役割 | git 管理 |
|---|---|---|
| `fav.toml` | プロジェクト設定（言語・ビルド） | ○ |
| `rune.toml` | 依存する rune の宣言 | ○ |
| `rune.lock` | 解決済み依存の固定 | ○ |
| `runes/` | インストール済み rune の実体 | ✕（.gitignore） |

**export は `public` で完結**（新キーワード不要）:
```favnir
// models/user.fav
public type User = { name: String  age: Int }   // 外部から参照可
public stage ParseUser: String -> User = ...     // 外部から参照可
fn internal_helper: String -> String = ...      // このファイル内のみ
```

**ローカルファイル import**（`src/` 起点）:
```favnir
import "models/user"           // namespace: user.User, user.ParseUser
import "models/user" as u      // namespace: u.User, u.ParseUser（alias）
```

**rune import**（`runes/` 起点。fav.toml の `[runes] path` で変更可）:
```favnir
import rune "validate"         // namespace: validate.Required, validate.Email
import rune "stat" as s        // namespace: s.int, s.float
```

**ファイル分離**:
```toml
# fav.toml — プロジェクト設定のみ
[project]
name    = "my-project"
version = "0.1.0"
edition = "2026"
src     = "src"

[runes]
path = "runes"   # インストール先（デフォルト。変更時のみ記載）
```

```toml
# rune.toml — 依存管理のみ（package.json 相当）
[dependencies]
validate = { version = "0.1.0" }
stat     = { version = "0.1.0" }

[dev-dependencies]
# 開発時のみ使用する rune
```

**namespace はファイル名（拡張子なし）から自動決定**:
- `import "models/user"` → namespace `user`
- `import "auth/user"` → namespace `user`（← 競合！）

**namespace 競合は E081 でエラー + `as` ヒント**:
```
E081: namespace conflict
  'user' is imported from both "models/user" and "auth/user"
  hint: use `as` to resolve
    import "models/user" as model_user
    import "auth/user"   as auth_user
```

**バレルファイル**（ディレクトリをまとめて公開）:
```favnir
// models/models.fav
public import "models/user"    // user.* を models.* として re-export
public import "models/post"

// 呼び出し側
import "models"                // models.user.User, models.post.Post
```

### 追加するもの

- パーサーに `import` 文を追加
- driver.rs でインポートグラフを解析・トポロジカルソートして順番にコンパイル
- チェッカーに cross-file シンボル解決を追加
- E080: 循環インポート検出（パスを表示）
- E081: namespace 競合検出（`as` ヒントを表示）
- `fav check --dir src/` でディレクトリ以下を一括チェック

### 完了条件

- `import "models/user"` で別ファイルの public 関数が `user.*` として使える
- `import rune "validate"` で `runes/validate/validate.fav` が読み込まれる
- 循環インポートで E080 が出る
- namespace 競合で E081 + `as` ヒントが出る
- `fav check --dir src/` が全 .fav ファイルをチェックする
- 既存テストが全て通る

---

## v2.7.0 — `validate` ルーン（Favnir 実装）

**テーマ**: バリデーションロジックを Favnir 自身で書いた最初の公式ルーンとして提供する

**設計原則**: ルーンは Rust の VM builtins に依存しない。
プリミティブ（List, String, Result 等）の上に純粋な Favnir で実装し、
`runes/validate/` ディレクトリに `.fav` ファイルとして管理する。

### 追加するもの

**`runes/validate/validate.fav`**（純粋 Favnir、VM 依存なし）:
```favnir
public type ValidationError = { path: String  code: String  message: String }

// フィールドレベル検証
public stage Required: String -> Result<String, ValidationError> = |s| {
    if String.is_empty(s) {
        Result.err(ValidationError { path: ""  code: "required"  message: "Field is required" })
    } else { Result.ok(s) }
}

public stage MinLen: String -> Result<String, ValidationError> = ...
public stage MaxLen: String -> Result<String, ValidationError> = ...
public stage Email:  String -> Result<String, ValidationError> = ...
public stage Range:  Int    -> Result<Int, ValidationError>    = ...

// パイプライン検証（複数ルールを seq で合成）
public stage ValidateAll: List<Result<T, ValidationError>> -> Result<T, List<ValidationError>> = ...
```

**`runes/validate/validate.test.fav`**:
- 各 stage の正常系・異常系テスト

**`import rune "validate"` のサポート**（v2.6.0 モジュールシステム上に追加）:
- `rune.toml` の `[dependencies]` に `validate = { version = "0.1.0" }` と書けば使える
- driver.rs が `runes/validate/validate.fav` をコンパイルして静的リンク

### 完了条件

- `fav run runes/validate/validate.test.fav` で全テストが通る
- `import rune "validate"` でユーザーコードから使える
- validate の実装に Rust コードが一行もない
- 既存テストが全て通る

---

## v2.8.0 — `stat` ルーン（Favnir 実装）

**テーマ**: 型駆動の乱数生成・統計推論ルーンを Favnir で実装する

**設計原則**: VM 側には `Random.int(min, max) -> Int !Random` と
`Random.float() -> Float !Random` の 2 プリミティブのみ追加し、
それ以上のロジックはすべて `runes/stat/stat.fav` に Favnir で書く。

> `stat` rune は乱数生成の**基盤プリミティブ層**。
> 型定義から合成データを自動生成する**型駆動層**は v3.5.0 の `gen` rune が担当する。

### 追加するもの

**VM プリミティブ（最小限、Rust 側）**:
- `Random.int(min: Int, max: Int) -> Int !Random`
- `Random.float() -> Float !Random`
- `effect Random` をエフェクトシステムに追加

**`runes/stat/stat.fav`**（Favnir 実装）:
```favnir
// 基本生成
public stage int:   Unit -> Int   !Random = |_| { Random.int(0, 100) }
public stage float: Unit -> Float !Random = |_| { Random.float() }
public stage bool:  Unit -> Bool  !Random = |_| { Random.int(0, 1) == 1 }

// 分布
public stage uniform: Int -> Int -> Int !Random = |min| |max| { Random.int(min, max) }
public stage choice: List<T> -> T !Random = |xs| {
    bind i <- Random.int(0, List.length(xs) - 1)
    Option.unwrap_or(List.first(List.drop(xs, i)), ...)
}

// リスト生成
public stage list_of: Int -> stage<Unit -> T !Random> -> List<T> !Random = ...

// プロファイル（invariant 適合率の計測）
public stage profile: List<T> -> ProfileReport = ...
```

**`runes/stat/stat.test.fav`**:
- seed 固定でのdeterministic生成テスト
- `choice` / `list_of` の動作確認

### 完了条件

- `fav run runes/stat/stat.test.fav` で全テストが通る
- `import rune "stat"` でユーザーコードから使える
- VM プリミティブは `Random.int` / `Random.float` の 2 つのみ
- 既存テストが全て通る

---

## v2.9.0 — `Stream<T>` + `collect` 内 `for`

**テーマ**: 遅延シーケンスと直感的なコレクション構築を追加する

### 追加するもの

**`Stream<T>`（遅延シーケンス）**:
```favnir
// 無限数列
bind nats <- Stream.from(0, |n| n + 1)
bind first10 <- Stream.take(nats, 10) |> Stream.collect
```
- `Type::Stream(Box<Type>)` を型システムに追加
- `Stream.from(seed, next)` — 状態駆動生成
- `Stream.of(list)` — List から Stream へ変換
- `Stream.map` / `Stream.filter` / `Stream.take` / `Stream.collect`
- `collect` との統合: `collect { for x in stream { yield transform(x); } }`

**`collect` 内 `for` サポート（E067 の解消）**:
```favnir
// 現状は E067 でブロックされている
bind evens <- collect {
  for x in List.range(0, 100) {
    if x % 2 == 0 { yield x; }
  }
}
```
- チェッカーで `collect` ブロック内の `for` を許可
- `for` 内の `yield` が外側の `collect` に帰属する

### 完了条件

- `Stream.from(0, |n| n + 1) |> Stream.take(5) |> Stream.collect` が `[0,1,2,3,4]` を返す
- `collect { for x in list { if cond { yield x; } } }` が E067 なしで動く
- 既存テストが全て通る

---

## v2.10.0 — パッケージマネージャ完成 + `fav repl`

**テーマ**: rune のインストール・管理・公開を完全に動かし、エコシステムの基盤を整える

### 設計方針

JS/TS の npm モデルを参考に、**依存管理は `rune.toml` に分離**して一か所で管理する：

```
my-project/
  fav.toml      ← プロジェクト設定（ビルド・edition・src）
  rune.toml     ← 依存の宣言（package.json 相当）
  rune.lock     ← 解決済み依存の固定（package-lock.json 相当）
  runes/        ← インストール先（node_modules 相当）← .gitignore
```

`rune.toml` と `rune.lock` をバージョン管理し、`runes/` は `fav install` で復元する運用。
`fav.toml` はプロジェクト設定のみ。依存は一切書かない。

### 追加するもの

**パッケージ管理 CLI**:
```bash
fav install                  # rune.toml の全依存を runes/ にインストール
fav install validate         # validate を追加して rune.toml に記録
fav install validate@0.2.0   # バージョン指定
fav remove validate          # rune.toml から削除 + runes/ から除去
fav tidy                     # import されていない rune を rune.toml から削除（go mod tidy 相当）
fav update                   # 全 rune を最新互換バージョンに更新
fav search <keyword>         # レジストリ検索
```

**ファイル構成**:
```toml
# fav.toml — プロジェクト設定のみ
[project]
name    = "my-pipeline"
version = "0.1.0"
edition = "2026"
src     = "src"

[runes]
path = "runes"   # インストール先（デフォルト）
```

```toml
# rune.toml — 依存管理のみ
[dependencies]
validate = { version = "0.1.0" }
stat     = { version = "0.2.0" }

[dev-dependencies]
# 開発・テスト時のみ使用する rune
```

**レジストリサーバー（段階的整備）**:

| ステップ | 内容 | タイミング |
|---|---|---|
| Step 1 | GitHub リリースをレジストリとして使用（npm 初期と同様） | v2.10.0 |
| Step 2 | 簡易セルフホストレジストリ（VPS 1台、REST API） | 収益化後 |
| Step 3 | 本格レジストリサービス（Veltra と統合） | Phase D 以降 |

v2.10.0 では Step 1（GitHub ベース）で動作確認し、サーバー運用コストを最小化する。
レジストリサーバーの本格運用は収益が発生してから投資する。

**`fav repl`（対話的実行環境）**:
```
$ fav repl
Favnir v2.10.0 REPL. Type :help for commands.
> bind x <- 42
x: Int = 42
> x * 2
84: Int
> :type List.map
List.map: List<A> -> (A -> B) -> List<B>
> :quit
```
- 式を入力すると型と値を表示
- `bind` で REPL スコープに変数を追加
- `:type <expr>` で型だけ表示
- `:help` / `:quit` コマンド
- 複数行入力（`{` で開いたら `}` まで待つ）

### 完了条件

- `fav install validate` で runes/validate/ にインストールされ rune.toml が更新される
- `fav tidy` で未使用 rune が rune.toml と runes/ から削除される
- `rune.toml` + `rune.lock` をコミットすれば `fav install` で環境を再現できる
- `fav.toml` には依存が一切書かれない（設定のみ）
- `fav repl` で式の評価と型表示ができる
- 既存テストが全て通る

---

## v2.11.0 — マルチソースパイプライン

**テーマ**: 複数のデータソースを型安全に合流・分岐・ゲートできるパイプライン構文を追加する

現状の `seq` は「1入力 → n変換 → 1出力」の線形のみ。
複数ソースの統合やフォールバック処理が `fn` による命令的コードに逃げるしかなく、
データパイプライン専用言語としての表現力に欠けている。

### 追加するもの

**`&` — 並列ペア化（fan-in）**:

```favnir
seq DualPipeline =
    (SourceA |> Sampling) & (SourceB |> Sampling)
    |> Merge<Row>
    |> Normalise
    |> PrintLn
```

- 2つのパイプラインブランチを `(List<T>, List<U>)` のタプルにまとめる
- 常にペア化（空でも続行）
- `Merge<T>` の前段として使用

**`&&` — 両方揃ったら進む（gate fan-in）**:

```favnir
seq BothRequired =
    (SourceA |> CleanA) && (SourceB |> CleanB)
    |> Merge<Row>
    |> Normalise
    |> PrintLn
```

- `&` と同じ型構造 `(List<T>, List<T>)` を返すが、どちらかが空なら中断
- 「両方のソースが揃ったときだけ処理したい」ユースケース向け

**`||` — フォールバック（fallback fan-in）**:

```favnir
seq FallbackPipeline =
    (SourceA |> Sampling) || (SourceB |> Sampling)
    |> Normalise
    |> PrintLn
```

- `List<T> || List<T> → List<T>` — 左が空なら右を使う
- `Merge` 不要、型が `List<T>` のまま続く

**`Tuple(StageA, StageB)` — タプルレーンへの個別stage適用**:

```favnir
seq SplitPipeline =
    Source
    |> Sampling
    |> Partition(|row| row.region == "EU")   // List<Row> -> (List<Row>, List<Row>)
    |> Tuple(ProcessEU, ProcessUS)           // (List<Row>, List<Row>) -> (List<Row>, List<Row>)
    |> Merge<Row>
    |> Normalise
    |> PrintLn
```

- `(List<T>, List<U>)` の左右レーンにそれぞれ別の stage を適用する組み込みコンビネータ

**`Join(pred)` — キーベース結合**:

```favnir
// DB（マスター）+ CSV（トランザクション）を id で結合
seq EnrichPipeline =
    (DbSource |> FromDb) & (CsvSource |> FromCsv |> chain)
    |> Join(|master, txn| master.id == txn.master_id)   // List<(MasterRow, TransactionRow)>
    |> Map(|(m, t)| EnrichedRow { id: m.id  name: m.name  value: t.value })
    |> Normalise
    |> PrintLn
```

- `(List<A>, List<B>) -> List<(A, B)>` — SQL の INNER JOIN に相当
- 述語でキー条件を宣言する（位置ではなく意味で結合）
- 異種ソース統合（DB + CSV など）の主要パターン
- `Zip<A, B>` は位置ベースの補助的な結合として残す

**組み込みステージ**:

| ステージ | 型 | 意味 |
|---|---|---|
| `Merge<T>` | `(List<T>, List<T>) -> List<T>` | 同型2レーンを結合（concat） |
| `Join(pred)` | `(List<A>, List<B>) -> List<(A, B)>` | キー条件で異型2レーンを結合（INNER JOIN） |
| `Zip<A, B>` | `(List<A>, List<B>) -> List<(A, B)>` | 異型2レーンを位置で結合（補助的） |
| `Partition(pred)` | `List<T> -> (List<T>, List<T>)` | 述語で1ストリームを2分岐 |
| `guard(pred)` | `(List<T>, List<T>) -> (List<T>, List<T>)` | 条件偽なら中断、真なら素通し |

**`guard` — 条件付き中断ステージ**:

```favnir
// (PipeA) != (PipeB) 相当：差分があるときだけ処理続行
seq DiffOnly =
    (SourceA |> Sampling) & (SourceB |> Sampling)
    |> guard |(a, b)| a != b
    |> Merge<Row>
    |> Normalise
    |> PrintLn
```

- `Bool` を返す述語を受け取り、`false` なら中断（後続ステージを実行しない）
- `!=` 相当の「差分ゲート」はこのパターンで表現する

**`abstract seq` への multi-input 拡張**:

```favnir
abstract seq DualSource<T, U> {
    left:    List<T> -> List<T>
    right:   List<U> -> List<U>
    combine: (List<T>, List<U>) -> List<T>
    process: List<T> -> Unit
}

seq MyPipeline = DualSource<Row, Row> {
    left    <- Sampling
    right   <- Sampling
    combine <- Merge<Row>
    process <- Normalise |> PrintLn
}
```

**型安全の保証**:
- `Merge<T>` は左右が同型でなければチェッカーがエラー（型の取り違えを防ぐ）
- `Tuple(S, S)` は `(List<T>, List<U>)` のタプルにしか適用できない
- `Zip<A, B>` は `A != B` でも使えるが下流は `List<(A, B)>` として型が確定する
- `&&` / `||` を `Bool` 以外（`List<T>`）に使う場合はパイプライン文脈のみ有効

### 完了条件

- `(SourceA |> Sampling) & (SourceB |> Sampling) |> Merge<Row>` が型チェックを通る
- `&&` でどちらかが空のとき後続ステージが実行されない
- `||` で左が空のとき右のソースにフォールバックする
- `Tuple(StageA, StageB)` がタプルの左右に個別 stage を適用する
- `Partition(pred)` で1ストリームが2レーンに分岐する
- `guard(pred)` で条件偽のとき中断する
- `Merge<T>` で左右の型が異なるとき型エラーが出る
- `Join(pred)` でキー条件が一致する要素がペアリングされる
- `Join` の結果が `List<(A, B)>` として型チェックを通る
- 既存テストが全て通る

---

## v3.0.0 — セルフホスト完成 + 言語基盤安定化

**テーマ**: Favnir のパーサーを Favnir 自身で書き、エラーコード体系を刷新して言語を安定させる

### 追加・変更するもの

**セルフホスト Step 1 完成（パーサー移植）**:
- `selfhost/lexer/` — v2.0.0 で着手済みのレキサーを完成させる
  - 全トークン種別に対応（現状は算術演算子のみ）
- `selfhost/parser/` — Favnir 製パーサーの実装
  - 式・文・型定義・関数定義を Favnir のデータ型（AST）として返す
  - `fav run selfhost/parser/parser.fav` で Favnir コードを解析できる
- `fav explain compiler` — コンパイル工程（字句解析→構文解析→型検査→コード生成）を可視化

**エラーコード体系の刷新**（roadmap-v2 v2.0.0 より繰り越し）:
- `E001`–`E099` 体系 → `E0100`–`E0999` 体系への移行
- エラーコードをカテゴリ別に整理（E01xx: 構文, E02xx: 型, E03xx: エフェクト, E04xx: runtime）
- `fav explain-error E0201` でエラーの詳細説明・修正例を表示

**explain/trace/artifact JSON スキーマ固定**（post1-roadmap B-2）:
- `fav explain --format json` の出力スキーマをバージョン付きで固定
- `schema_version: "3.0"` フィールドを追加
- breaking change があれば migration ツールを同梱

**ドキュメント**:
- langspec.md v3.0.0 全面改訂
- セルフホスト戦略ドキュメント更新（Step 1 完了 → Step 2 計画）
- RELEASE_NOTES.md v3.0.0

### 完了条件

- `fav run selfhost/parser/parser.fav` が Favnir コードを解析して AST を返す
- レキサー・パーサー合わせて 100 件以上の selfhost テストが通る
- `fav explain-error E0201` 等でエラー説明が表示される
- エラーコードが E0xxx 体系に移行している
- explain JSON に `schema_version: "3.0"` が含まれる
- 既存テストが全て通る

---

## バージョンと機能の対応表

| バージョン | テーマ | 主な追加 |
|---|---|---|
| v2.0.0 | 破壊的変更 + selfhost 着手 | `stage/seq/interface` リネーム, selfhost lexer |
| v2.1.0 | 標準ライブラリ補完 + CLI ウェルカム | Math モジュール, List/String 補完, IO.read_line, ドラゴンアイコン, `fav new` |
| v2.2.0 | pipe match + pattern guard | `\|> match {}`, `where` 句 |
| v2.3.0 | 分割 bind + 戻り型推論 | `bind { a, b } <- record`, `fn f(x) = x` |
| v2.4.0 | スタックトレース + 品質改善 | コールスタック, Unknown 削減, ignored テスト解消 |
| v2.5.0 | LSP 補完・定義ジャンプ | フィールド補完, F12 ジャンプ |
| v2.6.0 | モジュールシステム | `import "path"`, 循環検出, rune import 基盤 |
| v2.7.0 | `validate` ルーン | Favnir 製バリデーション, VM 依存ゼロ |
| v2.8.0 | `stat` ルーン | Favnir 製乱数生成, Random primitive 2つのみ |
| v2.9.0 | `Stream<T>` + collect 内 for | 遅延シーケンス, E067 解消 |
| v2.10.0 | パッケージマネージャ完成 + REPL | `fav install/remove/tidy/update/search`, `fav repl`, GitHub レジストリ |
| v2.11.0 | マルチソースパイプライン | `&`/`&&`/`||` ブランチ合成, `Tuple`, `Merge<T>`, `Join`, `Zip<A,B>`, `Partition`, `guard` |
| v3.0.0 | セルフホスト + 言語基盤安定化 | Favnir 製パーサー, E0xxx 体系, JSON スキーマ固定 |
| v3.1.0 | `fav docs` — ローカルリファレンス UI | Swagger UI 風ドキュメントサーバー |
| v3.2.0 | `csv` + `json` rune | `Csv.parse<T>`, `#[col(n)]`, `Schema.adapt<T>`, `Json.parse<T>` |
| v3.3.0 | `db` rune | `DB.query<T>`, `DB.execute`, PostgreSQL/SQLite 対応, DB+CSV Join 完成 |
| v3.4.0 | `fav infer` — スキーマ自動生成 | CSV/DBから型定義を自動生成、`Option<T>` nullable対応 |
| v3.5.0 | `gen` rune — 型駆動データ生成 | `Gen.one<T>`, `Gen.list<T>`, `Gen.simulate`, `Gen.profile`, `fav check --sample` |
| v3.6.0 | 増分処理（Incremental） | `Checkpoint`, `DB.query_since`, `DB.upsert`, 冪等性サポート |
| v3.7.0 | `http` + `parquet` rune | REST / GraphQL API, `Http.get/post/serve/serve_graphql`, `fav build --graphql`, `Parquet.read/write` |
| v3.8.0 | `grpc` rune | `Grpc.serve<S>`, `Stream<T>`↔サーバーストリーミング, `fav build --proto`, `fav infer --proto` |

---

## 実装順序の依存関係

```
v2.1.0 (stdlib)        -- 独立。どこでも実施可能
  └─ v2.2.0 (pipe match / pattern guard)  -- stdlib 補完後が望ましい
       └─ v2.3.0 (分割 bind / 戻り型推論) -- pipe match と組み合わせて書き心地を確認

v2.4.0 (スタックトレース)  -- 独立して実施可能（品質改善）

v2.5.0 (LSP 補完)      -- v2.3.0 後が望ましい（推論結果を補完に使うため）

v2.6.0 (モジュールシステム)  -- v2.5.0 後（LSP がモジュールを認識する必要があるため）
  ├─ v2.7.0 (validate ルーン) -- import rune が前提
  │    └─ v2.8.0 (stat ルーン) -- validate と同様、rune import が前提
  └─ v2.9.0 (Stream<T>)       -- モジュール分割で書きやすくなるため
       └─ v2.10.0 (rune / REPL) -- モジュールシステムが前提
            └─ v2.11.0 (マルチソースパイプライン) -- Stream<T> と論理演算子（v2.1.0/v2.9.0）が前提

v2.x 全体
  └─ v3.0.0 (selfhost + 安定化)
       └─ v3.1.0 (fav docs)  -- explain JSON スキーマ固定後に実装（スキーマ変更の影響を受けないため）
            └─ v3.2.0 (csv + json rune) -- v2.11.0 の Join/Merge + v2.6.0 の rune import が前提
                 └─ v3.3.0 (db rune) -- Schema.adapt<T>（v3.2.0）が前提、DB+CSV Join を完成させる
                      └─ v3.4.0 (fav infer) -- csv/db rune が揃ってからスキーマ推論を実装
                           └─ v3.5.0 (gen rune) -- fav infer（v3.4.0）+ stat rune（v2.8.0）+ Gen interface が前提
                                └─ v3.6.0 (増分処理) -- DB書き込み（v3.3.0）と型定義基盤（v3.4.0）が前提
                                     └─ v3.7.0 (http + parquet rune) -- csv/db/incremental が揃ってから出力先を完成させる
                                          └─ v3.8.0 (grpc rune) -- Stream<T>（v2.9.0）+ interface system（v2.0.0）+ http rune（v3.7.0）が前提
```

---

## 補助メモ（ロードマップ未確定）

## v3.1.0 — `fav docs` ローカルリファレンス UI

**テーマ**: Swagger UI 風のドキュメントサーバーをCLIから起動し、関数・型・標準ライブラリを視覚的に参照できるようにする

**v3.0.0 後に配置する理由**: `fav explain --format json` の出力スキーマが v3.0.0 で固定されるため、
その後に実装することで selfhost 進展によるスキーマ変更の影響を受けない。

### 設計方針

- **HTTPサーバー → Rust 永続**（インフラ層。selfhost 後も変わらない）
- **データ → explain JSON**（生成者が Rust → Favnir に変わっても、スキーマが同じなら UI は無変更）
- **UI → 埋め込み HTML/JS**（`include_str!` で単一バイナリに同梱）

### 追加するもの

**`fav docs [file]` コマンド**:
```
$ fav docs                 # 標準ライブラリのリファレンスを表示
$ fav docs main.fav        # プロジェクト固有の関数も含めて表示
$ fav docs --port 8080     # ポート指定（デフォルト: 7777）
```

**UI の構成**:
- 左ペイン: モジュール別ナビゲーション（`List`, `String`, `Math`, `Option`, `Result`, ...）
- 右ペイン: 関数シグネチャ・引数型・戻り値型・エフェクト・invariant
- `//` コメントをドキュメントコメントとして表示
- 検索ボックス（関数名・型名でフィルタ）
- ブラウザを自動で開く（`--no-open` で抑制）

**データフロー**:
```
fav docs main.fav
  → fav explain --format json（内部呼び出し）
  → JSON を HTTP で配信
  → 埋め込み HTML/JS が描画
```

### selfhost との関係

| 部分 | selfhost 後の変化 |
|---|---|
| HTTP サーバー起動・配信 | 変わらない（Rust インフラ） |
| HTML/JS UI | 変わらない |
| explain JSON 生成 | Rust → Favnir に変わるが、スキーマが同じなら透過的 |

### 完了条件

- `fav docs` で `http://localhost:7777` が起動しブラウザが開く
- 標準ライブラリ（List・String・Math 等）の全関数が一覧表示される
- `fav docs main.fav` でプロジェクト関数も表示される
- 検索ボックスで関数名を絞り込める
- 既存テストが全て通る

---

## v3.2.0 — `csv` + `json` rune（データフォーマット入出力）

**テーマ**: 現場で最も多い「CSVとJSONを型安全に読み書きする」を公式 rune として提供する

CSV はデータエンジニアリングの現場で最も普遍的なソース形式。
型宣言をスキーマ契約として使う `Schema.adapt<T>` パターンの基盤もここで整備する。

### 追加するもの

**`runes/csv/csv.fav`**:

```favnir
// ヘッダーあり CSV → 型付きリスト
public stage parse<T>: String -> Result<List<T>, SchemaError> = Schema.adapt<T>

// ヘッダーなし CSV（位置ベース、#[col(n)] アノテーション使用）
public stage parse_positional<T>: String -> Result<List<T>, SchemaError> = Schema.adapt_positional<T>

// 型付きリスト → CSV 文字列
public stage write<T>: List<T> -> String = Schema.to_csv<T>
```

`#[col(n)]` アノテーション（ヘッダーなし CSV 用）:
```favnir
type Row = {
    #[col(0)] id:     Int
    #[col(1)] name:   String
    #[col(2)] value:  Float
}
```

**`Schema.adapt<T>`（VM プリミティブ、Rust 側）**:
- フィールド名の照合（ヘッダーあり）または位置照合（ヘッダーなし）
- 文字列 → 各フィールドの型へのパース・変換
- 変換失敗時は `SchemaError { field, expected, got }` を返す
- 下流はすべて型安全な `List<T>` として流れる

**`runes/json/json.fav`**:
```favnir
public stage parse<T>: String -> Result<T, SchemaError>
public stage write<T>: T -> String
```

**利用例（DB + CSV 異種ソース統合）**:
```favnir
import rune "csv"
import rune "db"   // v3.3.0 で追加

seq HeterogeneousPipeline =
    (DbSource |> db.query<MasterRow>) &
    (CsvSource |> csv.parse<TransactionRow> |> chain)
    |> Join(|m, t| m.id == t.master_id)
    |> Map(|(m, t)| EnrichedRow { id: m.id  name: m.name  value: t.value })
    |> Normalise
    |> PrintLn
```

### 完了条件

- `csv.parse<Row>` でヘッダーありCSVが `Result<List<Row>, SchemaError>` として読み込める
- `csv.parse_positional<Row>` で `#[col(n)]` アノテーションによる位置マッピングが動く
- カラム型変換失敗時に `SchemaError` が返る（`Any` に逃げない）
- `csv.write<Row>` で `List<Row>` をCSV文字列に書き出せる
- `json.parse<T>` / `json.write<T>` が動く
- `chain` との組み合わせで変換失敗時にパイプラインが中断する
- 既存テストが全て通る

---

## v3.3.0 — `db` rune（データベース接続）

**テーマ**: SQLデータベースへの型安全なアクセスを提供し、DB + CSV の異種ソース統合を完成させる

### 追加するもの

**`runes/db/db.fav`（Favnir 側インターフェース）+ Rust VM プリミティブ**:

```favnir
// SELECT: 結果を型 T のリストとして返す
public stage query<T>: DbConnection -> String -> Result<List<T>, DbError>

// INSERT / UPDATE / DELETE
public stage execute: DbConnection -> String -> Result<Int, DbError>  // Int = affected rows

// トランザクション
public stage transaction<T>: DbConnection -> (DbConnection -> Result<T, DbError>) -> Result<T, DbError>
```

**接続設定**:
```favnir
type DbConnection = { driver: String  host: String  port: Int  name: String  user: String }

// fav.toml の [env] からシークレットを読む（ハードコード禁止）
bind conn <- DB.connect(DbConnection {
    driver: "postgres"
    host:   Env.get("DB_HOST")
    port:   5432
    name:   Env.get("DB_NAME")
    user:   Env.get("DB_USER")
})
```

**対応ドライバ（段階的）**:
| ドライバ | 優先度 |
|---|---|
| PostgreSQL | 高（データエンジニアのデファクト） |
| SQLite | 高（開発・テスト用途） |
| MySQL / MariaDB | 中 |

**VM プリミティブ（Rust 側）**:
- `DB.connect(conn_str)` — 接続確立
- `DB.query_raw(conn, sql)` → `List<Map<String, String>>` — 生結果
- `Schema.adapt<T>(raw_rows)` — v3.2.0 の `Schema.adapt<T>` でそのまま型付け

### 完了条件

- `db.query<Row>(conn, "SELECT id, name FROM users")` が `Result<List<Row>, DbError>` を返す
- PostgreSQL と SQLite で動作する
- `db.execute` で INSERT / UPDATE / DELETE が実行できる
- `db.transaction` でロールバックが動く
- DB + CSV の `Join` パターンが end-to-end で動く
- 接続情報をコードにハードコードするとリンタ警告が出る（L005 追加）
- 既存テストが全て通る

---

## v3.4.0 — `fav infer`（スキーマ自動生成）

**テーマ**: 既存のCSV・DBテーブルから Favnir の型定義を自動生成し、パイプライン構築の最初の一歩を軽くする

現状はパイプラインを書く前に型定義を手書きする必要がある。
外部ソースのカラムが多いと宣言コストが高く、「既存データからはじめる」障壁になっている。

### 追加するもの

**`fav infer` コマンド**:

```bash
fav infer data.csv                      # CSV からヘッダーと値を見て型を推測
fav infer data.csv --out types.fav      # ファイルに書き出し
fav infer --db postgres://...           # DBの全テーブルを一括推測
fav infer --db postgres://... users     # 特定テーブルのみ
fav infer --db postgres://... --out schema/
# schema/users.fav, schema/orders.fav ... を生成
```

**CSV からの推論**:

```
// data.csv
id,name,value,region
1,Alice,3.14,EU
2,Bob,2.71,US
```

```bash
$ fav infer data.csv
```

```favnir
// auto-generated by `fav infer data.csv`
// Review and adjust before use.
type Row = {
    id:     Int
    name:   String
    value:  Float
    region: String
}
```

**型推論ルール**:
| 値の例 | 推論される型 |
|---|---|
| `1`, `42`, `-3` | `Int` |
| `3.14`, `2.0` | `Float` |
| `true`, `false` | `Bool` |
| `2026-01-01` | `String`（Date は将来候補） |
| それ以外 | `String` |
| 空値あり | `Option<T>` |

**DBテーブルからの推論**:

```bash
$ fav infer --db postgres://user:pass@localhost/mydb users --out schema/users.fav
```

```favnir
// auto-generated by `fav infer --db ... users`
// Review and adjust before use.
type User = {
    id:         Int
    name:       String
    email:      String
    created_at: String
    deleted_at: Option<String>
}
```

- DBのカラム型（INTEGER, VARCHAR, NULLABLE）をそのまま Favnir 型にマッピング
- nullable カラム → `Option<T>`
- 生成ファイルには「自動生成」コメントを付与して手動修正を促す

**推論の限界と設計方針**:
- あくまで「最初の型定義の草案」として出力する
- 推論結果は必ず人間がレビューしてから使う（コメントで明示）
- `fav infer` が出すのは `Result<T, SchemaError>` ではなく生の型定義 — 実行時バリデーションは `Schema.adapt<T>` が担当

### 完了条件

- `fav infer data.csv` でヘッダーと値から型定義を生成できる
- `fav infer --db postgres://...` でDBテーブルから型定義を生成できる
- nullable カラムが `Option<T>` になる
- 生成した型定義を `Schema.adapt<T>` にそのまま渡せる
- `--out` でファイルに書き出せる
- 既存テストが全て通る

---

## v3.5.0 — `gen` rune（型駆動データ生成）

**テーマ**: Favnir の型定義から合成データを自動生成し、実データなしで PoC を即開始できるようにする

`stat` rune（v2.8.0）は乱数生成の基盤。`gen` rune はその上に乗り、
**型定義そのものがデータ仕様書になる**という考え方を実現する。
中小企業向けデータ基盤 PoC の「実データ不要」という価値命題を言語レベルで支える。

### 設計方針

- `runes/gen/gen.fav` として Favnir で実装（Rust 依存なし）
- `stat` rune の `Random.int` / `Random.float` プリミティブを内部で使用
- `Gen` interface を実装した型なら `Gen.one<T>` / `Gen.list<T>` で生成できる
- 組み込み型（`Int`, `Float`, `String`, `Bool`）は `Gen` を自動実装
- フィールドが全て `Gen` を持つ record 型は `impl Gen for T` を自動合成

### 追加するもの

**`Gen` interface**:

```favnir
// runes/gen/gen.fav
interface Gen {
    generate: Unit -> Self !Random
}

// 組み込み型の自動実装（VM側）
// impl Gen for Int   → Random.int(-1000, 1000)
// impl Gen for Float → Random.float()
// impl Gen for Bool  → Random.int(0, 1) == 1
// impl Gen for String → ランダムな英数字列
```

**`Gen.one<T>` / `Gen.list<T>` — 型から生成**:

```favnir
// fav infer で生成した型をそのまま使える
type UserRow = {
    id:     Int
    name:   String
    email:  String
    age:    Int
    region: String
}

// 1件生成
bind user <- Gen.one<UserRow>()            // UserRow !Random

// 大量生成（seed固定でdeterministic）
bind users <- Gen.list<UserRow>(1000, seed: 42)  // List<UserRow> !Random
```

**`impl Gen` でカスタム生成ロジック**:

```favnir
// ビジネスルールを反映した生成
impl Gen for UserRow {
    generate = |_| {
        bind age    <- Random.int(18, 80)
        bind region <- Gen.choice(["EU", "US", "JP", "APAC"])
        bind name   <- Gen.choice(["Alice", "Bob", "Carol", "Dave"])
        UserRow {
            id:     Random.int(1, 999999)
            name:   name
            email:  $"{String.lower(name)}@example.com"
            age:    age
            region: region
        }
    }
}
```

**`Gen.simulate` — ノイズ混入（汚れたデータの再現）**:

```favnir
// noise: 0.1 → 約10%のフィールドに意図的な異常値を混入
bind dirty_users <- Gen.simulate<UserRow>(1000, noise: 0.1, seed: 42)

// → 電話番号フォーマット不統一、null混入、範囲外の値などを再現
// data-basis の dirty.py がやっていることを型定義から自動導出
```

**`Gen.profile<T>` — 実データの invariant 適合率計測**:

```favnir
// 実データを食わせてどれくらい「きれいか」を計測
bind report <- Gen.profile<UserRow>(real_data)
// ProfileReport { total: 1000  valid: 832  invalid: 168  rate: 0.832 }
// → どのフィールドが何件壊れているかをフィールド別に報告
```

**`fav check --sample N` との統合**:

```bash
fav check pipeline.fav --sample 100
# → Gen.list<T>(100) で合成データを生成してパイプラインを試し実行
# → 実データなしでパイプラインの型安全性を確認できる
```

**ファイル構成**:

```
runes/gen/
  gen.fav        ← Gen interface + Gen.one/list/simulate/profile/choice の実装
  gen.test.fav   ← テスト（seed固定でdeterministic確認）
```

### `fav infer` → `gen` rune の連携フロー

```
実データ（CSV / DB）
    ↓ fav infer（v3.4.0）
型定義ファイル（schema/user_row.fav）
    ↓ impl Gen for UserRow（カスタム生成ロジックを追加）
    ↓ Gen.list<UserRow>(1000, seed: 42)
合成データ → PoC 即開始
    ↓ Gen.simulate(noise: 0.1)
汚れたデータ → クレンジングパイプラインの検証
```

### 完了条件

- `Gen.one<T>()` で全フィールドが `Gen` を持つ型の値を生成できる
- `Gen.list<T>(N, seed: K)` で deterministic な大量生成ができる
- `impl Gen for T` でカスタム生成ロジックを定義できる
- `Gen.simulate<T>(N, noise: 0.1)` でノイズ混入データを生成できる
- `Gen.profile<T>(data)` で invariant 適合率を計測できる
- `fav check --sample N` との統合が動く
- `runes/gen/gen.fav` は Rust コードを一行も含まない
- 既存テストが全て通る

---

## v3.6.0 — 増分処理（Incremental Processing）

**テーマ**: 「前回実行からの差分だけ処理する」をパイプライン宣言として書けるようにする

本番ETLで最も重要なパターン。現状は全件処理のみで、毎回全データを読み直す必要がある。
大規模データでは実行時間・コストの問題になる。

### 設計方針

**チェックポイント（Checkpoint）**:
前回処理した位置を記録し、次回実行時にそこから再開する概念。

```favnir
// チェックポイントを使った増分処理
seq IncrementalPipeline =
    DB.query_since<Row>(conn, "SELECT * FROM events", Checkpoint.last("events"))
    |> Validate
    |> Normalise
    |> DB.upsert<Row>(dest_conn, "processed_events")
    |> Checkpoint.save("events")   // 処理済み位置を記録
```

**`Checkpoint` 組み込みステージ**:

| ステージ | 型 | 意味 |
|---|---|---|
| `Checkpoint.last(name)` | `String -> Option<CheckpointVal>` | 前回の終了位置を取得 |
| `Checkpoint.save(name)` | `List<T> -> List<T>` | 処理後に位置を記録して素通し |

**増分の種類**:

```favnir
// タイムスタンプベース（updated_at >= 前回実行時刻）
DB.query_since<Row>(conn, "SELECT * FROM events", Checkpoint.last("events"))

// オフセットベース（id > 前回処理した最大id）
DB.query_after<Row>(conn, "SELECT * FROM events", Checkpoint.last("events"))

// CSVの場合（新しいファイルのみ）
Csv.new_files<Row>("data/", Checkpoint.last("csv_load"))
```

**チェックポイントの保存先**:
```toml
# fav.toml
[checkpoint]
backend = "file"          # デフォルト：.fav_checkpoints/ ディレクトリ
# backend = "postgres"   # DB に保存（本番向け）
# backend = "sqlite"     # SQLite ファイルに保存
```

**冪等性（Idempotency）のサポート**:
```favnir
// upsert: 重複実行しても結果が変わらない
|> DB.upsert<Row>(conn, "processed_events", on_conflict: "id")
```

- 同じデータを2回処理しても壊れない設計を言語レベルで誘導する
- `DB.insert`（重複でエラー）と `DB.upsert`（重複で上書き）を明確に分ける

### 完了条件

- `Checkpoint.last` / `Checkpoint.save` でチェックポイントが記録・参照できる
- タイムスタンプベース増分処理が動く
- チェックポイント保存先を `fav.toml` で設定できる（file / sqlite）
- `DB.upsert` で冪等な書き込みができる
- パイプライン中断後の再実行で重複処理が起きない
- 既存テストが全て通る

---

## v3.7.0 — `http` + `parquet` rune（REST + GraphQL + DWH 出力）

**テーマ**: 「読んで変換してAPIとして公開する」を完成させ、レガシーシステムの非破壊モダン化を実現する

v3.5.0 までで E（Extract）と T（Transform）は揃う。
v3.6.0 で L（Load）の主要出力先を整備し、ETL としての完成度を高める。
REST だけでなく GraphQL も一級の出力先として提供する。
Favnir の型定義がそのまま GraphQL スキーマになるため、スキーマ管理の二重化が不要。

### `http` rune — REST API の入出力

**`Http.get<T>` — 外部 API からデータを取得（Extract）**:

```favnir
// 外部 REST API をデータソースとして使う
seq ApiSourcePipeline =
    Http.get<ProductRow>("https://api.example.com/products")
    |> chain                        // エラー時は中断
    |> Normalise
    |> DB.upsert<ProductRow>(conn, "products")
```

**`Http.post<T>` — 外部 API にデータを送る（API sink）**:

```favnir
// 加工済みデータを外部 API に送る
seq ApiSinkPipeline =
    Source |> Clean |> Normalise
    |> Http.post<Row>("https://api.partner.com/ingest")
```

**`Http.serve<T>` — 自分が API になる（レガシー統合の核心）**:

```favnir
// SAP や古い DB の前に Favnir を置いて API として公開する
seq LegacyExposePipeline =
    SapCsvExport
    |> Csv.parse<SapRow> |> chain
    |> Normalise
    |> Http.serve<SapRow>("/api/v1/products")  // GET /api/v1/products として公開
```

**これが実現すること**:
- SAP・古いオンプレDB・CSV定期エクスポートなど、APIを持たないシステムをそのまま公開できる
- ソース側は読み取り権限のみ。一切の変更不要
- `fav` バイナリ1本 + `.fav` ファイルだけで稼働
- 大規模移行プロジェクト・JVM・Python環境が不要

### `http` rune — GraphQL API

REST は URL・クエリパラメータが文字列依存で、型チェッカーが関与できる余地が少ない。
GraphQL はスキーマが Favnir の型定義そのものになるため、型安全性を完全に保ったまま API を公開できる。

**Favnir 型 → GraphQL スキーマ（自動）**:

```favnir
// 型定義がそのまま GraphQL スキーマになる
type User = { id: Int  name: String  email: Email }

// リゾルバ = stage（型シグネチャが SDL の定義そのもの）
stage UserById:  Int  -> Result<User, DbError>       !Db = |id| { DB.query_one<User>(conn, ...) }
stage AllUsers:  Unit -> Result<List<User>, DbError> !Db = |_|  { DB.query<User>(conn, ...) }

// スキーマ = interface（Query / Mutation の宣言）
interface UserQuerySchema {
    user:  Int  -> Result<User, DbError>
    users: Unit -> Result<List<User>, DbError>
}

impl UserQuerySchema {
    user  = UserById
    users = AllUsers
}

// GraphQL エンドポイントとして公開
Http.serve_graphql<UserQuerySchema>(schema_impl, "/graphql")
// → POST /graphql で GraphQL クエリを受け付ける
```

**なぜ REST より GraphQL が Favnir と相性が良いか**:

| | REST | GraphQL |
|---|---|---|
| スキーマ管理 | 手動（OpenAPI 等） | **Favnir 型定義から自動生成** |
| コンパイル時検証 | 難しい | **リゾルバ型をチェッカーが検査** |
| 過剰取得 / 不足取得 | 発生しやすい | クライアントがフィールド指定 |

**Mutation のサポート**:

```favnir
interface UserMutationSchema {
    create_user: UserInput -> Result<User, DbError> !Db
    delete_user: Int       -> Result<Unit, DbError> !Db
}
```

**自動生成できるもの（`fav build --graphql`）**:
```bash
fav build --graphql src/main.fav --out schema.graphql
# → schema.graphql（SDL 形式）を生成。クライアントコード生成ツールと連携可能
```

### `parquet` rune — DWH 向け列指向フォーマット出力

```favnir
// BigQuery / Snowflake / Spark が直接読める形式で保存
seq DwhExportPipeline =
    Source |> Clean |> Normalise
    |> Parquet.write<Row>("output/result.parquet")

// 読み込みも可能
seq ParquetSourcePipeline =
    Parquet.read<Row>("data/snapshot.parquet")
    |> Normalise
    |> PrintLn
```

- Apache Parquet 形式（列指向・型情報あり・高圧縮）
- BigQuery・Snowflake・Spark・DuckDB が直接読み込める
- CSV より大容量データに適する
- v3.2.0 の `Csv.write<T>` と対称的な位置づけ

### 完了条件

- `Http.get<T>` で外部 REST API のレスポンスを型付きで取得できる
- `Http.post<T>` で加工済みデータを外部 API に送信できる
- `Http.serve<T>` で GET エンドポイントを公開できる
- CSV → `Http.serve` の end-to-end がバイナリ1本で動く
- `Http.serve_graphql<S>` で GraphQL エンドポイントを公開できる
- `interface` + `impl` の型シグネチャからリゾルバの型ミスマッチがコンパイル時に検出される
- `fav build --graphql` で `.graphql`（SDL）ファイルを生成できる
- `Parquet.write<T>` で Parquet ファイルを書き出せる
- `Parquet.read<T>` で Parquet ファイルを読み込める
- 既存テストが全て通る

---

## v3.8.0 — `grpc` rune

**テーマ**: gRPC サービスを Favnir の型定義から直接公開し、高速バイナリ通信とストリーミングを提供する

GraphQL（v3.7.0）が「クライアント主導のクエリ」に強い一方、
gRPC は「サービス間通信・ストリーミング」に強く、マイクロサービス構成や大量データ転送に適する。
Favnir の `interface` + `Stream<T>` がそのまま Protobuf サービス定義にマッピングされる。

**なぜ gRPC が Favnir と相性が良いか**:

| | REST | GraphQL | gRPC |
|---|---|---|---|
| スキーマ | なし | 型定義から自動 | **型定義から自動** |
| ストリーミング | SSE 等 | Subscription | **Stream<T> がネイティブ対応** |
| 転送効率 | テキスト | テキスト | **バイナリ（Protobuf）** |
| サービス間通信 | △ | △ | **◎** |

### 追加するもの

**Favnir 型 → Protobuf メッセージ（自動）**:

```favnir
// Favnir 型定義 = Protobuf message（手動 .proto 不要）
type GetUserRequest  = { id: Int }
type GetUserResponse = { id: Int  name: String  email: String }
type UserList        = { users: List<GetUserResponse> }
```

**サービス定義 = `interface`**:

```favnir
// interface のメソッドシグネチャ = RPC 定義
interface UserService {
    get_user:   GetUserRequest -> Result<GetUserResponse, RpcError> !Rpc   // Unary RPC
    list_users: Unit           -> Stream<GetUserResponse>           !Rpc   // Server Streaming
    watch:      GetUserRequest -> Stream<GetUserResponse>           !Rpc   // Server Streaming
}

impl UserService {
    get_user   = GetUserImpl
    list_users = |_| { DB.query_stream<GetUserResponse>(conn, "SELECT ...") }
    watch      = WatchUserImpl
}
```

**gRPC サーバーとして公開**:

```favnir
// バイナリ1本で gRPC サーバーが立ち上がる
Grpc.serve<UserService>(impl, port: 50051)
```

**`Stream<T>` → サーバーストリーミング RPC**:

- `fn` / `stage` の戻り型が `Stream<T>` のとき、自動的にサーバーストリーミング RPC にマッピング
- クライアントは受け取りながら処理できる（大量データ転送に有効）
- `Stream<T>` は v2.9.0 で実装済みのため、新しい概念を追加しない

**Protobuf 出力（既存システムとの interop）**:

```bash
fav build --proto src/main.fav --out schema.proto
# → schema.proto を生成。既存 gRPC クライアント（Go/Python/Java 等）と接続可能
```

**`fav infer --proto` — 既存 .proto からの型インポート**:

```bash
fav infer --proto users.proto --out schema/users.fav
# → .proto の message/service 定義を Favnir 型定義に変換
# → 既存 gRPC サービスを Favnir で書き直す入口
```

**VM プリミティブ（Rust 側、最小限）**:
- `Grpc.bind(port)` — gRPC サーバーソケット確立
- `Grpc.send_stream_item<T>(item)` — ストリームアイテム送信
- Protobuf シリアライズ / デシリアライズ（`prost` クレート依存）
- TLS 設定（本番向け）

### 完了条件

- `Grpc.serve<UserService>` で gRPC サーバーが起動する
- Unary RPC（`T -> Result<U, RpcError>`）が動く
- サーバーストリーミング RPC（`T -> Stream<U>`）が動く
- `fav build --proto` で `.proto` ファイルを生成できる
- `fav infer --proto` で既存 `.proto` から Favnir 型定義を生成できる
- リゾルバ（`impl` のメソッド）の型ミスマッチがコンパイル時に検出される
- 既存テストが全て通る

---

## 補助メモ（ロードマップ未確定）

### 将来候補（v3.x 以降）

**言語・標準ライブラリ**:
| 機能 | メモ |
|---|---|
| `Set<T>` | `Map` が String キー固定のため汎用集合がない |
| 汎用 `Map<K, V>` | 現状はレコードを Map として使用（String キー固定） |
| デバッガー（ブレークポイント） | v2.4.0 のスタックトレースの延長線 |
| 型状態パターンのドキュメント化 | 実装より langspec での「推奨パターン」紹介が先 |
| named argument | `render(width: 800, height: 600)` |
| `IO.read_file` / `IO.write_file` | ファイル I/O |
| エフェクト推論 | `!Io` 等を呼び出し先から自動推論。宣言コスト削減 |
| 構造的部分型（row polymorphism） | フィールド名で互換性判定。`Merge<CommonRow>` に RowA/RowB を渡せる |

**エコシステム Rune**（データエンジニア向け競合対応）:
| Rune | 対抗 | 内容 | 状況 |
|---|---|---|---|
| `csv` / `json` | — | データフォーマット変換 Rune | **v3.2.0 に昇格** |
| `db` | — | DB接続・クエリ | **v3.3.0 に昇格** |
| `http` | — | REST / GraphQL、`Http.get/post/serve/serve_graphql`、`fav build --graphql` | **v3.7.0 に昇格** |
| `parquet` | — | カラム型フォーマット対応、DWH出力 | **v3.7.0 に昇格** |
| `grpc` | — | `Grpc.serve<S>`、`Stream<T>`↔ストリーミング、`fav build --proto` | **v3.8.0 に昇格** |
| `orchestration` | Airflow / Prefect | `seq` を DAG としてスケジュール実行 | 将来候補 |

**製品・インフラ**:
| 項目 | メモ |
|---|---|
| Veltra 連携（Phase C〜E） | notebook kernel, explain/trace API, .vnb |
| オンライン Playground | ブラウザで Favnir を試せる環境 |
| レジストリサーバー本格運用 | 収益化後に VPS → Veltra 統合 |

### 設計ドキュメント

- `dev/post-v1/roadmap/favnir-post1-roadmap.md` — Phase A〜E 全体像
- `dev/post-v1/roadmap/favnir-selfhost-plan.md` — セルフホスト戦略（ハイブリッド方針）
- `dev/post-v1/ideas/favnir-next-candidates.md` — 次候補 5 件
- `dev/post-v1/ideas/favnir-open-questions.md` — must/later/maybe 分類
