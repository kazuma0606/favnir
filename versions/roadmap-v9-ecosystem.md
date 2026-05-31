# Favnir ロードマップ v9.1.0 → v10.0.0 — Favnir ファースト：言語・エコシステム強化

作成日: 2026-05-31

v9.0.0（セルフホスト完成宣言）以降のエコシステム進化の方針。
v10.0.0 を「OSS 公開準備完了」という次の大きなマイルストーンとして区切る。

---

## 前提：v9.0.0 完了時点の状態

- **セルフホスト完成**: `fav check` / `fav run`（全経路）が Favnir 実装経由で動作
  - 型チェッカー: `fav/self/checker.fav`（v8.1.0〜）
  - コンパイラ: `fav/self/compiler.fav`（v8.5.0〜）
  - CLI: `fav/self/cli.fav`（v7.6.0〜）
- **Bootstrap 検証**: `bytecode_A == bytecode_B` 維持
- **Rune エコシステム**: AWS / DuckDB / SQL / fs / slack / queue / cache / email / http（基本）
- **stdlib Favnir 化**: `intersperse` / `capitalize` / `indent` の 3 関数（v8.2.0）
- **テスト**: 1136 件通過
- **`--legacy` フラグ**: 非推奨化済み

### Rust に残るもの（今後も変更しない）

| コンポーネント | 理由 |
|---|---|
| VM（バイトコード実行エンジン） | メモリ安全・性能・設計上の決定 |
| ファイル I/O・ネットワーク primitive | OS インターフェース層 |
| パーサー（ほぼ確定）| 新構文追加時のみ最小変更 |

---

## 方針

**v9.x では Rust を原則触らず、Favnir 自身で Favnir を育てる。**
各バージョンは 1〜2 週間で完了できる粒度を目安とする。

```
v9.1〜v9.4   : 基盤強化（stdlib / fmt / lint / json・csv Rune）
v9.5〜v9.7   : コネクタ拡充 + 型システム強化（http / llm / with / T? bugfix）
v9.8〜v9.9   : 開発体験（fav doc / fav profile / fav watch）
v9.10〜v9.11 : インタラクティブ開発（fav repl / LSP 補完）
v9.12〜v9.13 : 言語強化（ユーザー定義インターフェース / par 並列実行）
v10.0.0      : OSS 公開準備完了（CI / fav new / GitHub Public 化）
```

v9.x のバージョン番号は v9.9.0 に限定しない。
v10.0.0 は機能追加ではなく「外部に見せられる状態への整備」マイルストーン。

---

## v9.1.0 — stdlib 拡充（List / String / Map / Result / Option）

**テーマ**: 純粋 Favnir で実装できる標準ライブラリ関数を一気に追加する。
データパイプライン記述に必要な「部品」を揃え、v9.2.0 以降のツール実装を楽にする。

**背景**

現在の stdlib Favnir 化は `intersperse` / `capitalize` / `indent` の 3 関数のみ。
`List.chunk` / `List.flat_map` / `Result.all` 等の実用的な関数が不足しており、
ユーザーコードで手書きする必要がある。

**やること**

List:
- `List.chunk(xs, n)` — `[[1,2],[3,4],[5]]` のように n 件ずつ分割
- `List.flat_map(f, xs)` — モナド的バインド（`List.map` + `List.concat`）
- `List.group_by(key_fn, xs)` — キー関数で分類、`List<{key, values}>` を返す
- `List.zip_with(f, xs, ys)` — 2 リストを f で合成
- `List.take_while(pred, xs)` / `List.drop_while(pred, xs)`
- `List.unique(xs)` — 順序保持で重複除去
- `List.count(pred, xs)` / `List.sum(xs)` / `List.min(xs)` / `List.max(xs)`

String:
- `String.pad_left(s, n, ch)` / `String.pad_right(s, n, ch)` — 桁揃え
- `String.truncate(s, n, suffix)` — `"Hello..."` のように末尾を省略
- `String.repeat(s, n)` — 文字列の繰り返し
- `String.trim_start(s)` / `String.trim_end(s)`
- `String.replace(s, from, to)` — 部分文字列の置換
- `String.starts_with(s, prefix)` / `String.ends_with(s, suffix)`

Map:
- `Map.merge_with(f, m1, m2)` — 同一キーは f で解決
- `Map.filter(pred, m)` / `Map.map_values(f, m)`
- `Map.from_list(pairs)` / `Map.to_list(m)` — List ↔ Map 変換

Result / Option:
- `Result.map_err(f, r)` — エラー側を変換
- `Result.and_then(f, r)` — モナド的バインド（flatMap）
- `Result.all(results)` — `List<Result<A,E>>` → `Result<List<A>,E>`
- `Option.map(f, opt)` / `Option.and_then(f, opt)`
- `Option.unwrap_or(default, opt)` / `Option.is_some(opt)` / `Option.is_none(opt)`

**correctness fix — E0012 非ジェネリック関数引数数チェック**

v8.8.0 の実装メモで「スコープ外」とされた未検出バグを修正する。
現状 `checker.fav` の `env` には非ジェネリック関数の戻り型のみ保存しており、
`fn foo(x: Int) -> String` を `foo(1, 2)` と呼んでもエラーにならない。

`checker.fav` への追加:
- 関数スキーム文字列に引数型列を含める（`"Int|String"` = 引数 Int、戻り String）
- `check_fn_call_arity(env, name, arg_count)` を追加
- **E0012 — ArgCountMismatch**: 非ジェネリック関数の引数数不一致
  → `"E0012: foo expects 1 argument(s), got 2"`

**correctness fix — マルチパラメータクロージャの self-hosted 対応**

`parser.fav` の `ELambda` は `ELambda(String, Expr)` — 引数が 1 つのみ。
`|x, y| x + y` のような多引数クロージャが self-hosted pipeline で動作しない可能性がある。
`List.zip_with` 等を使う際に必要になる根本的な制限。

`parser.fav` への追加:
- `ELambda` を `ELambda(List<String>, Expr)` に変更（引数リスト化）
- `|x, y, z| body` の複数引数パースに対応
- 単引数の既存コードとの後方互換性を維持

`checker.fav` への追加:
- マルチパラメータクロージャの型推論対応

`compiler.fav` への追加:
- マルチパラメータクロージャのコード生成対応（カリー化または多引数命令）

**`rvm` — 独立した Rust VM バイナリ**

`fav` はフルツールチェーン（コンパイラ + VM + CLI）だが、
本番 executor には VM だけあれば十分。独立した `rvm` バイナリを追加することで：
- executor イメージに `fav` を含める必要がなくなる（イメージが軽量化）
- VM のバグフィックスを `fav` 全体の再ビルドなしにリリースできる
- VM バージョンを言語バージョンとは独立して採番できる

```bash
rvm --version          # Favnir VM 1.0.0
rvm file.fvc           # バイトコードを直接実行
rvm --db <url> file.fvc  # DB 接続付きで実行
```

VM バージョンは言語バージョンとは独立した採番（`VM_VERSION` 定数を `vm.rs` に定義）。
ECS / EKS / Lambda の executor イメージは `fav` ではなく `rvm` だけを含める構成にできる。

Rust への変更:
- `src/bin/rvm.rs` を追加（`fav exec` と同等のエントリポイント）
- `VM_VERSION: &str = "1.0.0"` を定数として定義
- `Cargo.toml` に `[[bin]] name = "rvm"` を追加
- `--version` / `--db` / `--help` フラグのみ実装（軽量）

**完了条件**
- 上記全関数が `fav/self/stdlib/*.fav` に実装されている
- 各関数の型シグネチャが `checker.fav` / `checker.rs` に登録されている
- E0012 が非ジェネリック関数の引数数不一致を検出できる
- `|x, y| x + y` が `fav run`（Favnir pipeline）で動作する
- `List.zip_with(|x, y| x + y, xs, ys)` が動作する
- `rvm --version` が `Favnir VM 1.0.0` を表示する
- `rvm file.fvc` が `fav exec file.fvc` と同じ結果を返す
- 統合テスト 24 件以上（stdlib 15 件 + E0012 3 件 + マルチパラムクロージャ 4 件 + rvm 2 件）

---

## v9.2.0 — fav fmt（コードフォーマッタ）

**テーマ**: `compiler.fav` の AST を使ってコードフォーマットを実現する。
Rust に触れずに開発できる最初の CLI 拡張。

**背景**

`compiler.fav` は既にソースコードを AST に変換する機能を持っている。
その AST から整形済みテキストを出力する pretty-printer を Favnir で実装し、
`cli.fav` にサブコマンドとして追加する。

**やること**

`compiler.fav` への追加:
- `fn pretty_expr(expr: Expr, indent: Int) -> String`
  - `let` / `if` / `match` / `fn call` / `binary op` の整形ルール
  - 演算子前後スペース、インデント幅 2
- `fn pretty_stmt(stmt: Stmt, indent: Int) -> String`
  - `stage` / `seq` / `fn` / `type` 定義の整形
- `fn pretty_program(prog: Program) -> String`
  - トップレベル間の空行ルール（定義間は 2 行）

`cli.fav` への追加:
- `fn cmd_fmt(path: String) -> Unit !Io`
  - ファイル読み込み → parse → pretty_print → 上書き保存
- `--check` フラグ: 上書きせず差分があれば終了コード 1（CI 用）
- `fav fmt src/pipeline.fav` / `fav fmt --check src/` が動作すること

**完了条件**
- `fav fmt` を 2 回通しても差分が出ない（冪等性）
- `fav fmt fav/self/compiler.fav` が `compiler.fav` 自身に適用できる
- 統合テスト 3 件以上

---

## v9.3.0 — fav lint（静的解析ルールエンジン）

**テーマ**: 型エラー（E0xxx）以外の警告・改善提案を `checker.fav` に追加する。
「型は正しいが設計上疑問がある」コードをユーザーに伝える。

**背景**

現在の `checker.fav` は型エラーのみを検出する。
データパイプライン特有のアンチパターン（副作用のない `Unit` 関数・未使用バインディング等）を
警告として伝える仕組みがない。

**やること**

`checker.fav` への追加:
- `type LintWarning = { code: String, message: String, name: String }`
- `fn lint_program(prog: Program) -> List<LintWarning>`

組み込みルール:
- **W001 — EffectlessSink**: `stage` の戻り型が `Unit` かつエフェクトなし
  → `"stage FetchData: String -> Unit に副作用がありません"`
- **W002 — NoWriteInSeq**: `seq` の最終 `stage` に `!Db` / `!AWS` がない
  → `"seq Pipeline は外部書き込みなしで終了します"`
- **W003 — UnusedBinding**: `let x = ...` で `x` が一度も参照されない
  → `"変数 x は定義されていますが使用されていません"`
- **W004 — TooManyArgs**: `stage` の引数型が 4 個以上（タプル化を検討）
- **W005 — WildcardOnlyMatch**: `match` の腕が `_` のみ
  → `"match 式の腕が _ のみです。網羅的なパターンを検討してください"`

`cli.fav` への追加:
- `fn cmd_lint(path: String) -> Unit !Io`
- `fav lint src/pipeline.fav` が動作すること
- `--warn-as-error` フラグ（CI 用、警告があれば終了コード 1）

**完了条件**
- 上記 5 ルールが動作する
- `fav lint fav/self/compiler.fav` が実行できる
- 統合テスト 5 件以上

---

## v9.4.0 — json・csv・gen Rune（データ I/O + ID 生成）

**テーマ**: データエンジニアが日常的に扱う JSON・CSV を型安全に読み書きできる Rune を追加する。
合わせて既存 `gen` Rune に UUID 生成を追加し、ID 採番をパイプラインに自然に組み込めるようにする。
`http` / `llm` Rune（v9.5.0〜）の基盤にもなる。

**背景**

現状、JSON / CSV の読み書きには `IO.read_file_raw` + 手動パースが必要で冗長。
型パラメータ付き `json.decode<Order>` / `csv.read<Order>` が使えると、
パイプライン記述が大幅に簡潔になる。
また、新規レコードの ID 採番・相関 ID 付与に UUID は頻出だが、
現在の `gen` Rune には UUID 生成が含まれていない。
v9.7.0 で導入する名目型ラッパー（`type UserId(String)`）との組み合わせで
「生成 → 型でラップ」がパイプラインに自然に入る。

**やること**

`json` Rune (`runes/json/`):
- `json.encode<T>(value: T) -> String`
- `json.decode<T>(s: String) -> Result<T, String>`
- `json.pretty(s: String) -> String`
- `rune.toml` + `json.fav` を作成

`csv` Rune (`runes/csv/`):
- `csv.read<T>(path: String) -> Result<List<T>, String> !Io`
  - ヘッダ行を型 T のフィールド名にマッピング
- `csv.write<T>(path: String, rows: List<T>) -> Unit !Io`
- `csv.parse<T>(s: String) -> Result<List<T>, String>`
  - ファイルなし・文字列から直接パース（テスト・WASM 向け）
- `rune.toml` + `csv.fav` を作成

既存 `gen` Rune への追加 (`runes/gen/`):
- `gen.uuid() -> String !Gen` — UUID v4（ランダム）
- `gen.uuid_v7() -> String !Gen` — UUID v7（タイムスタンプ付き・DB インデックス効率良）
- `gen.nano_id(n: Int) -> String !Gen` — URL-safe ランダム文字列（n 文字）

使用例:
```favnir
import rune "csv"
import rune "json"
import rune "gen"

stage LoadOrders: String -> List<Order> !Io = |path| {
  csv.read<Order>(path)
}

stage Serialize: List<Order> -> String = |orders| {
  json.encode(orders)
}

// UUID 採番 + 名目型ラップ（v9.7.0 以降）
stage CreateOrder: OrderInput -> Order !Gen = |input| {
  bind id <- gen.uuid_v7()
  Order { id: id  item: input.item  amount: input.amount }
}
```

**完了条件**
- `csv.read<Order>` / `json.decode<Order>` が型付きで動作する
- `fav check` で型パラメータの不一致を検出できる
- `gen.uuid()` / `gen.uuid_v7()` / `gen.nano_id(n)` が `!Gen` エフェクトで動作する
- 統合テスト 8 件以上（CSV 読み込み・JSON ラウンドトリップ・UUID 生成等）

---

## v9.5.0 — http Rune（HTTP クライアント + `!Http` エフェクト）

**テーマ**: `!Http` エフェクトを導入し、HTTP アクセスを型レベルで追跡できるようにする。
「どの `stage` が外部 API を呼ぶか」がエフェクトで静的に見えるようになる。

**背景**

現在 HTTP 通信には `IO.http_get_raw` primitive が存在するが、
エフェクト型は `!Io` に混在しており、HTTP アクセスとファイル I/O が区別できない。
`!Http` を独立したエフェクトとして分離することで `fav explain` のリネージ情報が充実する。

**やること**

`http` Rune (`runes/http/`):
- `http.get(url: String) -> Result<String, String> !Http`
- `http.get_json<T>(url: String) -> Result<T, String> !Http`
  - 内部で `json.decode<T>` を使用
- `http.post(url: String, body: String) -> Result<String, String> !Http`
- `http.post_json<T, R>(url: String, body: T) -> Result<R, String> !Http`
- `rune.toml` + `http.fav` を作成

`!Http` エフェクト登録:
- `checker.fav` の既知エフェクトリストに `Http` を追加
- `checker.rs` の `BUILTIN_EFFECTS` に追加

`fav explain --lineage` への反映:
- `!Http` エフェクトを持つ `stage` を Sources として表示

使用例:
```favnir
import rune "http"

stage FetchOrders: String -> List<Order> !Http = |api_url| {
  http.get_json<List<Order>>(api_url)
}

// fav explain --lineage で:
// Sources:  !Http → api_url
// Sinks:    !Db   → orders_table
```

**完了条件**
- `http.get` / `http.post` / `http.get_json<T>` が動作する
- `!Http` が型チェッカーで追跡される（エフェクト宣言なしでエラー）
- `fav explain --lineage` が `!Http` を Sources に表示する
- 統合テスト 3 件以上

---

## v9.6.0 — llm Rune（`!Llm` エフェクト + Claude / OpenAI 対応）

**テーマ**: LLM 呼び出しを `!Llm` エフェクトとして型レベルで追跡できるようにする。
「どの `stage` が AI を使うか」がコードから一目でわかるようになる。

**背景**

LLM API（Claude / OpenAI）は `http.post` で呼べるが、
それでは「AI を使っている stage」と「普通の HTTP 通信をしている stage」が区別できない。
`!Llm` エフェクトを独立させることで、パイプラインの AI 依存度が静的に可視化される。

**やること**

`llm` Rune (`runes/llm/`):
- `llm.complete(prompt: String) -> Result<String, String> !Llm`
  - 環境変数 `ANTHROPIC_API_KEY` / `OPENAI_API_KEY` を自動参照
  - `LLM_PROVIDER=anthropic`（default）/ `openai` で切り替え
- `llm.chat(messages: List<{role: String, content: String}>) -> Result<String, String> !Llm`
- `llm.extract<T>(prompt: String, data: String) -> Result<T, String> !Llm`
  - LLM に JSON 形式で構造化データを返させ、`json.decode<T>` で受け取る
- `rune.toml` + `llm.fav` を作成

`!Llm` エフェクト登録（`!Http` と同様）

使用例:
```favnir
import rune "llm"

stage SummarizeReport: String -> String !Llm = |text| {
  llm.complete("3行で要約してください:\n" + text)
}

// fav explain --lineage で:
// Effects: !Db(read: orders), !Llm, !AWS(S3 write)
// → 「DB を読んで AI で要約して S3 に書く」が静的に保証される
```

**完了条件**
- `llm.complete` / `llm.chat` が Claude API で動作する
- `!Llm` が型チェッカーで追跡される
- 統合テスト 2 件以上（モック可）

---

## v9.7.0 — 名目型ラッパー + バリデーション + with（型システム強化）

**テーマ**: 意味的に異なる値を型レベルで区別し、バリデーションを型定義に内包する。
さらに `with` キーワードでインターフェース実装を自動合成する。

**背景**

現在 `type UserId = Int` は型エイリアスであり、`UserId` と `Int` は型チェッカーで区別されない。
また `Eq` / `Show` / `Serialize` などの実装は手書きが必要でボイラープレートが多い。
さらにバリデーション（「Percent は 0〜100 の Float」）はパイプラインの各 stage に散在しがちで、
入口で一度だけ確認するという保証が言語レベルでできない。

**構文設計**

```favnir
// 名目型ラッパー: type Name(InnerType)
// エイリアス（既存）と括弧の有無で視覚的に区別
type UserId(Int)
type Email(String)

// バリデーション付き: where |v| pred
// コンストラクタが Result<T, String> を返すようになる
type Email(String)    where |v| String.contains(v, "@")
type Percent(Float)   where |v| v >= 0.0 && v <= 100.0
type NonEmpty(String) where |v| String.length(v) > 0

// with: レコード型・名目型ラッパー両方にインターフェースを自動合成
type UserId(Int)      with Eq, Show
type Order with Eq, Show, Serialize = { id: Int  item: String  amount: Float }
type Email(String)    with Eq, Show  where |v| String.contains(v, "@")
```

**コンストラクタの型規則**

```
where なし → T を直接返す
where あり → Result<T, String> を返す
```

```favnir
// where なし: 直接 T
let id = UserId(42)                    // UserId

// where あり: Result<T, String> → bind で unwrap
bind pct <- Percent(50.0)             // OK: Percent
bind pct <- Percent(150.0)            // Result.err("Percent: validation failed")
bind em  <- Email("a@b.com")          // OK: Email
bind em  <- Email("invalid")          // Result.err("Email: validation failed")
```

`bind x <- expr` が既に `Result` の unwrap に使われているため、
バリデーション付きコンストラクタとの相性が自然。

**パイプラインでの使用例**

```favnir
// 入口で一度だけ検証 → 下流は型が保証
stage ParsePercent: String -> Percent !Io = |s| {
  bind raw <- Float.parse(s)
  bind pct <- Percent(raw)
  pct
}

// パターンマッチで分解
match pct {
  Percent(v) -> v * 0.01
}
```

**やること**

**【バグ修正】self-hosted pipeline の `T?` / `T!` / `??` 未対応を解消**

精査で判明した実装漏れ。Rust パイプラインでは完全動作するが、
`fav run`（Favnir pipeline）では `compiler.fav` の自前 lexer/parser が使われるため未対応。

`lexer.fav` への追加:
- `TkQuestion` / `TkQuestionQuestion` トークンを追加
- `?` → `TkQuestion`、`??` → `TkQuestionQuestion` のスキャンルールを追加

`parser.fav` への追加:
- 型パース関数 `parse_type_expr` に `T?` → `TeOption(T)` の後置処理を追加
- `T!`（エフェクト注釈でない Bang）→ `TeResult(T, TeSimple("String"))` の処理を追加
- `??` 演算子をパース（null-coalesce 二項演算子として `OpQuestionQuestion` で追加）

`compiler.fav` への追加:
- **`expr?` エラー伝播演算子** の脱糖（Rust 変更不要、compiler.fav で変換）
  - `expr?` → `match expr { Ok(v) -> v  Err(e) -> return Err(e) }` に変換
  - 戻り型が `Result` でない関数での使用は E0013 でエラー

パーサー（Rust — 名目型ラッパー構文）:
- `type Name(InnerType)` 構文を追加
- `where |v| pred` 節を AST に追加
- `type T with Iface1, Iface2 = { ... }` の `with` 節を AST に追加
- AST: `WrapperDef { name, inner, validator, with_impls }`

`checker.fav` への追加:
- `type Name(Inner)` 定義を型環境 `env` に登録
- コンストラクタ呼び出し `Name(x)` の型推論
  - `where` なし: `Inner -> Name`
  - `where` あり: `Inner -> Result<Name, String>`
- パターンマッチ `Name(n)` の分解型規則
- `Name` と `Inner` の型不一致を E0010 として検出

`compiler.fav` への追加:
- `where` あり: コンストラクタに述語チェックコードを挿入
  - 失敗時は `Result.err("<Name>: validation failed")` を返すコードを生成
- `with` 自動合成（レコード型・名目型ラッパー共通）:
  - `Eq` — `eq(a: T, b: T) -> Bool` を合成
  - `Show` — `show(t: T) -> String` を合成
  - `Serialize` — `to_json(t: T) -> String` を合成
  - `Deserialize` — `from_json(s: String) -> Result<T, String>` を合成
  - 未知のインターフェース名は E0011（未定義インターフェース）でエラー

**完了条件**
- `T?` / `T!` / `??` が `fav run`（Favnir pipeline）で正しく動作する
- `fav check` と `fav run` の挙動が `T?` に関して一致する
- `expr?` が `Result` を返す関数内で使える（E0013 で誤用検出）
- `type Name(Inner)` がコンストラクタ・パターンマッチで使える
- `where` あり型のコンストラクタが `Result<T, String>` を返す
- `with Eq, Show, Serialize` の自動合成が `compiler.fav` で動作する
- 型の取り違えを E0010 でコンパイル時に検出できる
- 統合テスト 12 件以上

---

## v9.8.0 — fav doc（ドキュメント自動生成）

**テーマ**: ソースコードの `///` コメントと型シグネチャから Markdown を自動生成する。
`fav doc fav/self/` で Favnir 自身のドキュメントを Favnir が書く。

**背景**

現在ドキュメントはすべて手書き。`compiler.fav` が AST に `///` コメントを保持し、
型シグネチャと合わせて Markdown を出力する。OSS 公開（v10.0.0）に向けた準備でもある。

**やること**

パーサー（Rust 最小変更）:
- `///` ドキュメントコメントを AST に保持
- `stage` / `fn` / `seq` / `type` 定義にコメントを紐付け

`compiler.fav` への追加:
- `fn doc_item(name, comment, sig, effects) -> String` — Markdown 断片生成
- `fn doc_program(prog: Program) -> String` — ファイル全体のドキュメント生成

`cli.fav` への追加:
- `fn cmd_doc(src_dir: String, out_dir: String) -> Unit !Io`
- `fav doc src/ --out docs/api/` が動作すること
- 出力: `docs/api/<filename>.md`

**完了条件**
- `fav doc fav/self/` が `compiler.fav` / `checker.fav` のドキュメントを生成する
- `stage` / `fn` の型シグネチャとエフェクトがドキュメントに含まれる
- 統合テスト 2 件以上

---

## v9.9.0 — fav profile + fav watch（実行時間計測 + ファイル監視）

**テーマ**: `compiler.fav` が計測コードを自動挿入してボトルネックを可視化する。
合わせてファイル変更監視 + 自動再実行（`fav watch`）を追加し、開発体験を改善する。

**背景**

大規模パイプラインで「どの stage が遅いか」を特定するには
現在手動で計測コードを書く必要がある。`--profile` フラグ一つで自動計測できると
本番パイプラインの最適化が大幅に楽になる。

また、パイプラインを iterative に開発するには毎回 `fav run` を手動実行する必要があり煩雑。
`fav watch` があれば保存するたびに自動で再実行・再テストでき、フィードバックループが縮まる。

**やること**

`compiler.fav` への追加（fav profile）:
- `fn instrument_stage_call(name: String, expr: Expr) -> Expr`
  - `stage` 呼び出しの前後に `Env.now_ms()` を挿入するコード変換
- `--profile` フラグ時のみ変換を適用（通常ビルドに性能影響なし）

`cli.fav` への追加（fav profile）:
- `fav run --profile pipeline.fav` が動作すること
- 実行後にステージ別実行時間をテーブル形式で表示

```
=== Pipeline Profile ===
Stage FetchOrders :  1,203 ms  (58%)
Stage Summarize   :    421 ms  (20%)  [!Llm]
Stage SaveToS3    :    432 ms  (21%)  [!AWS]
Total             :  2,056 ms
```

`cli.fav` への追加（fav watch）:
- `fn cmd_watch(path: String, mode: String) -> Unit !Io`
  - ファイルのタイムスタンプをポーリング（500ms 間隔）
  - 変更を検知したら `cmd_run` / `cmd_test` を自動実行
  - エラーがあっても継続（クラッシュしない）
- `fav watch pipeline.fav` — 変更のたびに `fav run`
- `fav watch --test pipeline.fav` — 変更のたびに `fav test`
- `fav watch --check pipeline.fav` — 変更のたびに `fav check`
- Rust 変更なし（`IO.now_ms` / `IO.read_file_raw` のタイムスタンプ比較で実装）

使用例:
```
$ fav watch --test src/pipeline.fav
[watch] monitoring src/pipeline.fav ...
[watch] change detected — running tests
  test "summarize empty"  PASS
  test "summarize orders" PASS
[watch] 2/2 passed
[watch] monitoring ...
```

**完了条件**
- `--profile` で各 stage の実行時間が計測される
- 計測コードを使わないビルドに性能影響がない
- `fav watch` がファイル変更を検知して自動再実行する
- エラー発生後も watch が継続する
- 統合テスト 3 件以上

---

## v9.10.0 — fav repl（インタラクティブ REPL）

**テーマ**: 式・関数・stage を対話的に評価できる REPL を実装する。
新規ユーザーのオンボーディングと探索的なパイプライン開発を支援する。

**背景**

現在 Favnir を試すには `.fav` ファイルを作成して `fav run` する必要がある。
REPL があれば、式の型確認・小さな変換の動作確認・Rune の動作探索を
ファイルを作らずにできる。セルフホスト完成後はほぼ Favnir だけで実装できる。

**やること**

`cli.fav` への追加:
- `fn cmd_repl() -> Unit !Io` — メインループ
  - `> ` プロンプトを表示し 1 行ずつ読み込む
  - 入力を checker.fav → compiler.fav → Rust VM の経路で評価
  - 結果を `show(result)` で文字列化して表示

対応する入力形式:
- **式**: `1 + 2` → `3`、`List.map([1,2,3], |x| x * 2)` → `[2, 4, 6]`
- **型確認**: `:type List.first([1,2,3])` → `Option<Int>`
- **定義の累積**: `fn double(x: Int) -> Int = x * 2` → 以降の入力で使える
- **stage 定義**: `stage Trim: String -> String = |s| String.trim(s)` → 登録
- **メタコマンド**: `:help` / `:quit` / `:reset`（定義をクリア）/ `:env`（登録済み定義一覧）

実装上の設計:
- セッション内の定義を `List<Item>` として累積し、各入力時に結合してコンパイル
- 式のみの入力は暗黙的に `fn _repl_result() -> _ = <expr>` として扱い実行
- エラーは `E0xxx` を表示してプロンプトに戻る（クラッシュしない）

使用例:
```
$ fav repl
Favnir v9.10.0 — type :help for commands
> 1 + 2
3
> fn greet(name: String) -> String = "Hello, " + name
defined: greet
> greet("world")
"Hello, world!"
> :type List.first([1,2])
Option<Int>
> :quit
```

**完了条件**
- 式・関数定義・stage 定義が対話的に評価できる
- セッション内で定義が累積される
- `:type` で型を確認できる
- エラーがあってもセッションが継続する
- 統合テスト 3 件以上（式評価・定義累積・エラー回復）

---

## v9.11.0 — LSP 補完 + go-to-definition

**テーマ**: 既存の LSP（hover / diagnostics）に補完とジャンプ機能を追加する。
VSCode 上での Favnir 開発体験を大幅に改善する。

**背景**

現在の LSP は hover（型表示）と diagnostics（エラー表示）のみ。
`orders.` と打ったときにフィールド候補が出ず、`List.` と打っても補完が出ない。
補完と定義ジャンプを追加することで、OSS 公開時の第一印象が大きく変わる。

**やること**

**Completion（補完）**:
- **フィールド補完**: `record.` の後にフィールド名候補を表示
  - checker.fav / checker.rs のシンボルテーブルから型情報を参照
- **モジュール補完**: `List.` / `String.` / `Map.` 等の後に関数候補を表示
  - 型シグネチャと docstring を候補に付加
- **Rune 補完**: `import rune` 後に既知 Rune 名を候補表示
- **型名補完**: `:` の後に登録済み型名を候補表示

**Go-to-definition（定義ジャンプ）**:
- ユーザー定義 `fn` / `stage` / `type` の名前をクリックで定義行へジャンプ
- Rune 関数の場合は `runes/<name>/<name>.fav` の該当行へジャンプ
- Span 情報（行・列）を AST に保持する必要あり（Rust 最小変更）

**Signature help（引数ヒント）**:
- `foo(` を打った時点で引数の型シグネチャを表示
  - `stage ParseCsv: String -> List<Row> !Io` のような情報

実装方針:
- LSP サーバー（Rust）はすでに存在する。今回は補完応答の実装を追加
- 補完候補の生成ロジックは `checker.fav` のシンボルテーブルを活用
- Rust 変更: LSP の `textDocument/completion` ハンドラ追加、Span 情報保持

**完了条件**
- `record.` でフィールド補完が動作する
- `List.` / `String.` で関数補完が動作する（型シグネチャ付き）
- ユーザー定義関数・型の定義ジャンプが動作する
- 関数呼び出し時の Signature help が表示される
- VSCode 拡張でこれらが動作することを手動確認

---

## v9.12.0 — ユーザー定義インターフェース（`interface` キーワード）

**テーマ**: `with Eq, Show` の組み込みインターフェースに加えて、
ユーザーが独自インターフェースを定義・実装できるようにする。
v9.7.0 の `with` 自動合成を基盤として拡張する。

**背景**

v9.7.0 では `Eq` / `Show` / `Serialize` / `Deserialize` の 4 つの組み込みインターフェースを実装した。
しかしデータパイプラインでは「この型はバリデーション可能」「この型は変換可能」など
ドメイン固有の能力を型レベルで表現したいケースがある。

**構文設計**

```favnir
// インターフェース定義
interface Validatable {
  fn validate(self) -> Result<Unit, String>
}

interface Transformer<B> {
  fn transform(self) -> B
}

// ユーザー定義型での実装
type Order with Eq, Validatable = {
  id:     Int
  amount: Float
}

// with Validatable を指定した場合、compiler.fav は
// fn validate(self: Order) -> Result<Unit, String> の実装を要求する
// → 実装がなければ E0014: missing interface implementation

// 明示的な実装ブロック（with の自動合成ではカバーできない場合）
impl Validatable for Order {
  fn validate(self) -> Result<Unit, String> = {
    if self.amount > 0.0 { Result.ok(()) }
    else { Result.err("amount must be positive") }
  }
}
```

**やること**

パーサー（Rust 最小変更）:
- `interface Name { fn ... }` 構文を追加
- `impl Interface for Type { ... }` 構文を追加

`checker.fav` への追加:
- インターフェース定義を環境に登録
- `with CustomInterface` 使用時に `impl` ブロックの存在チェック
- **E0014 — MissingImpl**: `with` で指定したインターフェースの実装がない

`compiler.fav` への追加:
- `impl` ブロックのコード生成
- `with` で指定したインターフェースの実装を型定義に紐付け

**完了条件**
- `interface` でユーザー定義インターフェースを宣言できる
- `impl Interface for Type` で実装を提供できる
- 実装漏れを E0014 でコンパイル時に検出できる
- 組み込みインターフェース（Eq/Show 等）と共存する
- 統合テスト 5 件以上

---

## v9.13.0 — par 並列 stage 実行

**テーマ**: 独立した `stage` を並列で実行する `par` 構文を追加する。
「複数ソースから並列取得 → マージ → 保存」というパターンを言語ネイティブに表現できる。

**背景**

現在の `|>` は逐次実行のみ。例えば「S3 からデータ取得」と「DB からデータ取得」を
並列で行い結果をマージするパターンは手書きで並列処理コードが必要。
`par` を言語プリミティブにすることで、エフェクト追跡との組み合わせも自然になる：
「どの並列 stage が `!AWS` を使っているか」が `fav explain` で静的に見える。

**構文設計**

```favnir
stage FetchOrders: String -> List<Order> !Db  = |conn| { ... }
stage FetchPrices: String -> List<Price> !AWS = |bucket| { ... }
stage Merge:      (List<Order>, List<Price>) -> Report = |pair| { ... }
stage Save:        Report -> Unit !Db = |r| { ... }

// par: 複数 stage を並列実行し、結果をタプルで次 stage に渡す
seq FullReport = par [FetchOrders, FetchPrices] |> Merge |> Save

// fav explain --lineage で:
// par[FetchOrders(!Db), FetchPrices(!AWS)] → Merge → Save(!Db)
// → DB と AWS を並列で読み、DB に書く
```

**やること**

パーサー（Rust 変更あり）:
- `par [StageA, StageB]` 構文を `seq` 定義内で解析
- AST に `SeqNode::Par(Vec<String>)` を追加

VM（Rust 変更あり）:
- `par` 実行時に複数 stage を Rust の `tokio::spawn` で並列実行
- 結果をタプル `(A, B)` として次 stage に渡す

`checker.fav` への追加:
- `par [StageA, StageB]` の型チェック
  - 各 stage の入力型が同一であることを確認
  - 出力をタプル型として次 stage の入力型と照合
- エフェクトの和集合を伝播（`!Db` と `!AWS` の両方を宣言したことになる）

`fav explain --lineage` への反映:
- `par` ブロックを視覚的に区別（並列分岐として表示）

**完了条件**
- `par [StageA, StageB] |> Merge` が並列実行される
- 各 stage のエフェクトが型チェッカーで追跡される
- `fav explain` が並列構造を表示する
- 直列実行より実際に速いことを `fav profile` で確認できる
- 統合テスト 4 件以上

---

## v10.0.0 — OSS 公開準備完了

**テーマ**: v9.x シリーズで積み上げたエコシステムを整えて GitHub Public 化する。
「型安全なデータパイプライン専用言語」として世界に発信する準備を整える。

**背景**

v9.1〜v9.11 で言語・エコシステム・開発体験が大幅に成長する。
v10.0.0 はそれを受けて「外部に見せられる状態」に整えるマイルストーン。
機能追加は行わず、整備・公開に専念する。

**やること**

ドキュメント整備:
- `CONTRIBUTING.md` 作成（開発環境セットアップ・PR ガイドライン）
- `CHANGELOG.md` 初版（v4.0.0〜v9.11.0 サマリー）
- `site/` ドキュメントを v9.x の新機能（http/llm Rune・fmt・lint・doc）で更新
- `fav doc` で生成した API ドキュメントをサイトに組み込む

CI/CD 整備:
- GitHub Actions: `cargo test` → `fav check fav/self/` → `fav lint fav/self/` → `fav fmt --check fav/self/`
- `fav fmt` / `fav lint` の CI 強制（`--warn-as-error`）

`fav new` スキャフォールディング:
- `fav new <name>` でプロジェクトテンプレートを生成
  ```
  <name>/
    fav.toml       # プロジェクト設定
    src/main.fav   # エントリポイントテンプレート
    runes/         # rune モジュール配置先
    .gitignore
  ```
- `cli.fav` に `fn cmd_new(name: String) -> Unit !Io` を追加
- Rust 変更なし（ファイル生成は `IO.write_file_raw` で実装）

公開:
- GitHub リポジトリを Public に変更
- `LICENSE`（MIT）確認・配置
- 発表準備（ブログ下書き・connpass LT 登録）

**完了条件**
- GitHub Public リポジトリとして公開されている
- CI が main ブランチで green になっている
- `fav doc fav/self/` が自動生成するドキュメントがサイトに組み込まれている

---

## 全体スケジュール概観

| バージョン | テーマ | Rust 変更 | フェーズ |
|---|---|---|---|
| v9.1.0 | stdlib 拡充（約 30 関数）+ E0012 + マルチパラムクロージャ修正 + `rvm` 独立バイナリ | `rvm` バイナリ追加のみ | 基盤強化 |
| v9.2.0 | fav fmt — コードフォーマッタ（冪等性保証） | なし | 基盤強化 |
| v9.3.0 | fav lint — 静的解析（W001 EffectlessSink 〜 W005 WildcardOnlyMatch） | なし | 基盤強化 |
| v9.4.0 | json・csv Rune — 型安全データ I/O、gen Rune に UUID v4/v7/nano_id 追加 | なし | データ I/O |
| v9.5.0 | http Rune — `!Http` エフェクト追加 | `!Http` 登録のみ | コネクタ拡充 |
| v9.6.0 | llm Rune — `!Llm` エフェクト（Claude / OpenAI） | `!Llm` 登録のみ | コネクタ拡充 |
| v9.7.0 | 名目型ラッパー + `where` + `with` + `T?`/`T!`/`??`/`expr?` self-hosted 修正（bugfix） | パーサーのみ | 型システム |
| v9.8.0 | fav doc — `///` コメントから Markdown 自動生成 | `///` 保持のみ | 開発体験 |
| v9.9.0 | fav profile — パイプライン実行時間計測 | なし | 開発体験 |
| v9.10.0 | fav repl — 対話的 REPL（式・関数・stage 評価、定義累積、:type） | なし | インタラクティブ開発 |
| v9.11.0 | LSP 補完 + go-to-definition（フィールド・モジュール・Signature help） | LSP ハンドラのみ | インタラクティブ開発 |
| v9.12.0 | ユーザー定義インターフェース（`interface` / `impl` / E0014） | パーサーのみ | 型システム |
| v9.13.0 | `par` 並列 stage 実行（`par [A, B] \|> Merge`、VM 並列化） | VM + パーサー | パイプライン強化 |
| **v10.0.0** | **OSS 公開準備完了（CI / CONTRIBUTING / fav new / GitHub Public 化）** | なし | **公開** |

---

## 設計原則

**Rust は触らない（原則）**
新機能は `checker.fav` / `compiler.fav` / `cli.fav` / `runes/` の Favnir コードに追加する。
パーサーへの新構文追加（`newtype`・`///` コメント）のみ例外として許容する。

**セルフホストの一貫性を保つ**
- `fav check fav/self/compiler.fav` が常に通ること（self-check）
- Bootstrap 検証（`bytecode_A == bytecode_B`）を維持すること
- 新しいツール（fmt / lint / doc）は自分自身に適用できること

**エフェクトで境界を引く**
新しい副作用は必ず専用エフェクト（`!Http`・`!Llm` 等）として型レベルで表現する。
`!Io` に混在させない。`fav explain` のリネージ情報を常に充実させる方向で設計する。

**ドキュメントは実装と同じバージョンで完成させる**
各バージョンの完了条件にサイトドキュメント更新を含める。
v9.8.0（fav doc）以降は Favnir 製ドキュメント生成を CI に組み込む。
