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
$ fav new my-project
$ fav new my-project --template pipeline   # stage/seq 構成の雛形
$ fav new my-project --template lib        # rune ライブラリの雛形
```
- `fav.toml`（name/version/edition）を生成
- `src/main.fav`（hello world）を生成
- テンプレート別に `src/` 構成を変える
  - `pipeline`: main.fav + stages/parse.fav + stages/save.fav の骨格
  - `lib`: lib.fav + lib.test.fav の骨格

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
  fav.toml          ← プロジェクトルート
  src/
    main.fav
    models/
      user.fav
  runes/            ← rune 専用ディレクトリ（node_modules 相当）
    validate/
      validate.fav
```

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

**fav.toml**:
```toml
[package]
name = "my-project"
version = "0.1.0"

[runes]
path = "runes"   # デフォルト。変更時のみ記載

[dependencies]
validate = { rune = "std", version = "0.1.0" }
stat     = { rune = "std", version = "0.1.0" }
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
- `fav.toml` の `[dependencies]` に `validate = { rune = "std" }` と書けば使える
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

## v2.10.0 — rune レジストリ + `fav repl`

**テーマ**: エコシステムの基盤を動かし、試せる環境を用意する

### 追加するもの

**rune レジストリ（実装）**:
- `fav publish` でローカルレジストリへの登録が実際に動く（stub 解消）
- `fav install` でレジストリから取得・fav.lock 更新
- レジストリ API の最小仕様（ローカル HTTP サーバーとして動作確認）
- `fav search <keyword>` — レジストリ検索

**`fav repl`（対話的実行環境）**:
```
$ fav repl
Favnir v2.8.0 REPL. Type :help for commands.
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

- `fav repl` で式の評価と型表示ができる
- `fav publish` でローカルレジストリへ登録できる
- `fav install` でローカルレジストリから取得できる
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
| v2.10.0 | rune レジストリ + REPL | `fav repl`, `fav publish/install` 実装 |
| v3.0.0 | セルフホスト + 言語基盤安定化 | Favnir 製パーサー, E0xxx 体系, JSON スキーマ固定 |
| v3.1.0 | `fav docs` — ローカルリファレンス UI | Swagger UI 風ドキュメントサーバー |

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

v2.x 全体
  └─ v3.0.0 (selfhost + 安定化)
       └─ v3.1.0 (fav docs)  -- explain JSON スキーマ固定後に実装（スキーマ変更の影響を受けないため）
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

## 補助メモ（ロードマップ未確定）

### 将来候補（v3.x 以降）

| 機能 | メモ |
|---|---|
| `Set<T>` | `Map` が String キー固定のため汎用集合がない |
| 汎用 `Map<K, V>` | 現状はレコードを Map として使用（String キー固定） |
| デバッガー（ブレークポイント） | v2.4.0 のスタックトレースの延長線 |
| 型状態パターンのドキュメント化 | 実装より langspec での「推奨パターン」紹介が先 |
| named argument | `render(width: 800, height: 600)` |
| `IO.read_file` / `IO.write_file` | ファイル I/O |
| Veltra 連携（Phase C〜E） | notebook kernel, explain/trace API, .vnb |

### 設計ドキュメント

- `dev/post-v1/roadmap/favnir-post1-roadmap.md` — Phase A〜E 全体像
- `dev/post-v1/roadmap/favnir-selfhost-plan.md` — セルフホスト戦略（ハイブリッド方針）
- `dev/post-v1/ideas/favnir-next-candidates.md` — 次候補 5 件
- `dev/post-v1/ideas/favnir-open-questions.md` — must/later/maybe 分類
