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
v9.1〜v9.4 : 基盤強化（stdlib / fmt / lint / json・csv Rune）
v9.5〜v9.7 : コネクタ拡充（http / llm Rune / newtype）
v9.8〜v9.9 : 開発体験（fav doc / fav profile）
v10.0.0    : OSS 公開準備完了（CONTRIBUTING / CI / GitHub Public 化）
```

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

**完了条件**
- 上記全関数が `fav/self/stdlib/*.fav` に実装されている
- 各関数の型シグネチャが `checker.fav` / `checker.rs` に登録されている
- E0012 が非ジェネリック関数の引数数不一致を検出できる
- 統合テスト 18 件以上（stdlib 15 件 + E0012 3 件）

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

## v9.9.0 — fav profile（パイプライン実行時間計測）

**テーマ**: `compiler.fav` が各 `stage` の計測コードを自動挿入し、
ボトルネックを可視化する。Rust 不要でプロファイリングを実現。

**背景**

大規模パイプラインで「どの stage が遅いか」を特定するには
現在手動で計測コードを書く必要がある。`--profile` フラグ一つで自動計測できると
本番パイプラインの最適化が大幅に楽になる。

**やること**

`compiler.fav` への追加:
- `fn instrument_stage_call(name: String, expr: Expr) -> Expr`
  - `stage` 呼び出しの前後に `Env.now_ms()` を挿入するコード変換
- `--profile` フラグ時のみ変換を適用（通常ビルドに性能影響なし）

`cli.fav` への追加:
- `fav run --profile pipeline.fav` が動作すること
- 実行後にステージ別実行時間をテーブル形式で表示

```
=== Pipeline Profile ===
Stage FetchOrders :  1,203 ms  (58%)
Stage Summarize   :    421 ms  (20%)  [!Llm]
Stage SaveToS3    :    432 ms  (21%)  [!AWS]
Total             :  2,056 ms
```

**完了条件**
- `--profile` で各 stage の実行時間が計測される
- 計測コードを使わないビルドに性能影響がない
- 統合テスト 2 件以上

---

## v10.0.0 — OSS 公開準備完了

**テーマ**: v9.x シリーズで積み上げたエコシステムを整えて GitHub Public 化する。
「型安全なデータパイプライン専用言語」として世界に発信する準備を整える。

**背景**

v9.1〜v9.9 で言語・エコシステムが大幅に成長する。
v10.0.0 はそれを受けて「外部に見せられる状態」に整えるマイルストーン。

**やること**

ドキュメント整備:
- `CONTRIBUTING.md` 作成（開発環境セットアップ・PR ガイドライン）
- `CHANGELOG.md` 初版（v4.0.0〜v9.9.0 サマリー）
- `site/` ドキュメントを v9.x の新機能（http/llm Rune・fmt・lint・doc）で更新
- `fav doc` で生成した API ドキュメントをサイトに組み込む

CI/CD 整備:
- GitHub Actions: `cargo test` → `fav check fav/self/` → `fav lint fav/self/` → `fav fmt --check fav/self/`
- `fav fmt` / `fav lint` の CI 強制（`--warn-as-error`）

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
| v9.1.0 | stdlib 拡充（約 30 関数）+ E0012 非ジェネリック引数数チェック（correctness fix） | なし | 基盤強化 |
| v9.2.0 | fav fmt — コードフォーマッタ（冪等性保証） | なし | 基盤強化 |
| v9.3.0 | fav lint — 静的解析（W001 EffectlessSink 〜 W005 WildcardOnlyMatch） | なし | 基盤強化 |
| v9.4.0 | json・csv Rune — 型安全データ I/O、gen Rune に UUID v4/v7/nano_id 追加 | なし | データ I/O |
| v9.5.0 | http Rune — `!Http` エフェクト追加 | `!Http` 登録のみ | コネクタ拡充 |
| v9.6.0 | llm Rune — `!Llm` エフェクト（Claude / OpenAI） | `!Llm` 登録のみ | コネクタ拡充 |
| v9.7.0 | 名目型ラッパー + `where` + `with` + `T?`/`T!`/`??`/`expr?` self-hosted 修正（bugfix） | パーサーのみ | 型システム |
| v9.8.0 | fav doc — `///` コメントから Markdown 自動生成 | `///` 保持のみ | 開発体験 |
| v9.9.0 | fav profile — パイプライン実行時間計測 | なし | 開発体験 |
| **v10.0.0** | **OSS 公開準備完了（CI / CONTRIBUTING / GitHub Public 化）** | なし | **公開** |

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
