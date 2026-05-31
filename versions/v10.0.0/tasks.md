# v10.0.0 — Favnir ファースト開発：言語・エコシステム強化

Date: 2026-05-31

## テーマ

セルフホスト完成（v9.0.0）を受け、**コンパイラ・型チェッカー・CLI をすべて Favnir 自身で拡張する**フェーズ。
Rust は原則触らず、`checker.fav` / `compiler.fav` / `cli.fav` / `runes/*.fav` / `stdlib/*.fav` への
追記・追加によって言語とエコシステムを育てる。

### 実装方針
- **Rust 不要**: stdlib 追加・Rune 追加・checker.fav / compiler.fav 拡張
- **Rust 最小変更**: `newtype` キーワードなど新構文はパーサーへの小さな追加のみ
- **後回し（v11 以降）**: `where` 制約・`par seq`・`derive` など大きな構文変更

---

## Phase 1 — stdlib 拡充（純粋 Favnir、最速）

> 既存の Favnir コードから即座に使える関数を追加する。すべて `.fav` で実装。

### 1-A: List stdlib
- [ ] `List.chunk(xs: List<A>, n: Int) -> List<List<A>>`
  - `[1,2,3,4,5]` を n=2 で `[[1,2],[3,4],[5]]` に分割
- [ ] `List.flat_map(f: A -> List<B>, xs: List<A>) -> List<B>`
  - モナド的バインド。`List.map` + `List.concat` の合成
- [ ] `List.group_by(key: A -> K, xs: List<A>) -> List<{key: K, values: List<A>}>`
  - SQL の GROUP BY 相当
- [ ] `List.zip_with(f: A -> B -> C, xs: List<A>, ys: List<B>) -> List<C>`
  - 2リストを f で合成
- [ ] `List.take_while(pred: A -> Bool, xs: List<A>) -> List<A>`
- [ ] `List.drop_while(pred: A -> Bool, xs: List<A>) -> List<A>`
- [ ] `List.unique(xs: List<A>) -> List<A>`
  - 重複除去（順序保持）
- [ ] `List.count(pred: A -> Bool, xs: List<A>) -> Int`
- [ ] `List.sum(xs: List<Int>) -> Int`
- [ ] `List.min(xs: List<Int>) -> Option<Int>`
- [ ] `List.max(xs: List<Int>) -> Option<Int>`

### 1-B: String stdlib
- [ ] `String.pad_left(s: String, n: Int, ch: String) -> String`
  - `"42"` → `"  42"` (n=4, ch=" ")
- [ ] `String.pad_right(s: String, n: Int, ch: String) -> String`
- [ ] `String.truncate(s: String, n: Int, suffix: String) -> String`
  - `"Hello World"` → `"Hello..."` (n=8)
- [ ] `String.repeat(s: String, n: Int) -> String`
- [ ] `String.trim_start(s: String) -> String`
- [ ] `String.trim_end(s: String) -> String`
- [ ] `String.replace(s: String, from: String, to: String) -> String`
- [ ] `String.starts_with(s: String, prefix: String) -> Bool`
- [ ] `String.ends_with(s: String, suffix: String) -> Bool`

### 1-C: Map stdlib
- [ ] `Map.merge_with(f: V -> V -> V, m1: Map<K,V>, m2: Map<K,V>) -> Map<K,V>`
  - 同一キーは f で解決
- [ ] `Map.filter(pred: K -> V -> Bool, m: Map<K,V>) -> Map<K,V>`
- [ ] `Map.map_values(f: V -> W, m: Map<K,V>) -> Map<K,W>`
- [ ] `Map.from_list(pairs: List<{key: K, value: V}>) -> Map<K,V>`
- [ ] `Map.to_list(m: Map<K,V>) -> List<{key: K, value: V}>`

### 1-D: Result stdlib
- [ ] `Result.map_err(f: E -> F, r: Result<A,E>) -> Result<A,F>`
- [ ] `Result.and_then(f: A -> Result<B,E>, r: Result<A,E>) -> Result<B,E>`
  - モナド的バインド（flatMap）
- [ ] `Result.all(results: List<Result<A,E>>) -> Result<List<A>,E>`
  - 最初のエラーで止まる
- [ ] `Result.ok_or(default: A, opt: Option<A>) -> A`

### 1-E: Option stdlib
- [ ] `Option.map(f: A -> B, opt: Option<A>) -> Option<B>`
- [ ] `Option.and_then(f: A -> Option<B>, opt: Option<A>) -> Option<B>`
- [ ] `Option.unwrap_or(default: A, opt: Option<A>) -> A`
- [ ] `Option.is_some(opt: Option<A>) -> Bool`
- [ ] `Option.is_none(opt: Option<A>) -> Bool`

**実装ノート:**
- `self/stdlib/list_stdlib.fav`, `string_stdlib.fav`, `map_stdlib.fav`, `result_stdlib.fav` に追記
- `vm.rs` の dispatch テーブルに関数名を追加（既存パターン踏襲）
- `checker.rs` / `checker.fav` の型シグネチャ登録も忘れず

---

## Phase 2 — `fav fmt` フォーマッタ（compiler.fav 拡張）

> AST を持つ compiler.fav に pretty-printer を追加し、コードフォーマットを実現。

### 2-A: compiler.fav に pretty-print モジュール追加
- [ ] `fn pretty_expr(expr: Expr, indent: Int) -> String`
  - let / if / match / fn call / binary op の整形ルール
- [ ] `fn pretty_stmt(stmt: Stmt, indent: Int) -> String`
  - stage / seq / fn / type 定義の整形
- [ ] `fn pretty_program(prog: Program) -> String`
  - トップレベルの整形・空行ルール

### 2-B: cli.fav に `fmt` サブコマンド追加
- [ ] `fn cmd_fmt(path: String) -> Unit !Io`
  - ファイル読み込み → parse → pretty_print → 上書き保存
- [ ] `fav fmt src/pipeline.fav` が動作すること
- [ ] `fav fmt --check src/` (差分検出のみ、CI 用) オプション

### 2-C: テスト
- [ ] `fmt` を通したコードを再度 `fmt` しても差分が出ないこと（冪等性）
- [ ] 統合テスト 3 件以上

---

## Phase 3 — `fav lint` ルールエンジン（checker.fav 拡張）

> 型エラー以外の警告・改善提案を checker.fav に追加する。

### 3-A: ルールエンジン基盤
- [ ] `type LintRule = { code: String, message: String, line: Int }`
- [ ] `fn lint_program(prog: Program) -> List<LintRule>`
  - check_items を呼び出した後に lint_items を走らせる

### 3-B: 組み込みルール
- [ ] **W001**: `stage` の戻り型が `Unit` かつエフェクトなし → 「副作用のない Unit 関数」警告
- [ ] **W002**: `seq` の最終 stage が `!Db` / `!AWS` を持たない → 「書き込みなし」警告
- [ ] **W003**: 未使用の `let` バインディング（変数が定義後に参照されない）
- [ ] **W004**: `stage` の引数が 4 個以上 → 「タプル化を検討」提案
- [ ] **W005**: `match` 式で `_` ワイルドカードのみ → 「網羅性の確認」提案

### 3-C: cli.fav に `lint` サブコマンド追加
- [ ] `fav lint src/pipeline.fav` が動作すること
- [ ] `--warn-as-error` フラグ（CI 用）

---

## Phase 4 — Rune 拡充（`http` / `json` / `csv`）

> データエンジニアが必要とするコネクタを Favnir 製 Rune として提供。

### 4-A: `json` Rune
- [ ] `json.encode<T>(value: T) -> String !Io`
- [ ] `json.decode<T>(s: String) -> Result<T, String> !Io`
- [ ] `json.pretty(s: String) -> String !Io`
- [ ] `runes/json/` に `rune.toml` + `json.fav` を作成

### 4-B: `csv` Rune
- [ ] `csv.read<T>(path: String) -> List<T> !Io`
  - ヘッダ行を型 T のフィールド名にマッピング
- [ ] `csv.write<T>(path: String, rows: List<T>) -> Unit !Io`
- [ ] `csv.parse<T>(s: String) -> List<T>`
  - ファイルなし・文字列から直接パース
- [ ] `runes/csv/` に `rune.toml` + `csv.fav` を作成

### 4-C: `http` Rune
- [ ] `http.get(url: String) -> Result<String, String> !Http`
- [ ] `http.get_json<T>(url: String) -> Result<T, String> !Http`
- [ ] `http.post(url: String, body: String) -> Result<String, String> !Http`
- [ ] `http.post_json<T, R>(url: String, body: T) -> Result<R, String> !Http`
- [ ] `runes/http/` に `rune.toml` + `http.fav` を作成
- [ ] `!Http` エフェクトを checker.fav / checker.rs に登録

**実装ノート:**
- 各 Rune は既存の `IO.http_get_raw` 等の primitive をラップする形で実装
- 型パラメータのシリアライゼーションは `json` Rune を内部使用

### 4-D: `llm` Rune（Claude API）
- [ ] `llm.complete(prompt: String) -> Result<String, String> !Llm`
- [ ] `llm.chat(messages: List<{role: String, content: String}>) -> Result<String, String> !Llm`
- [ ] `!Llm` エフェクト登録
- [ ] `runes/llm/` に `rune.toml` + `llm.fav` を作成
- [ ] 環境変数 `ANTHROPIC_API_KEY` / `OPENAI_API_KEY` を自動参照

---

## Phase 5 — `newtype` ラッパー（型安全強化）

> 意味的に異なる値を型レベルで区別できるようにする。Rust 側はパーサーのみ最小変更。

### 5-A: パーサー対応（Rust 最小変更）
- [ ] `newtype UserId = Int` 構文をパーサーに追加
- [ ] AST に `NewtypeDef { name, inner_type }` を追加

### 5-B: checker.fav に newtype 型規則追加
- [ ] `newtype` 定義を `env` に登録
- [ ] `UserId` と `Int` の型不一致を E0009 系エラーで検出
- [ ] コンストラクタ `UserId(42)` の型推論
- [ ] パターンマッチ `UserId(n)` の分解

### 5-C: テスト
- [ ] `newtype UserId = Int; fn f(id: UserId) -> Int` が正しく型チェックされること
- [ ] `UserId` と `Int` を混在させると型エラーになること

---

## Phase 6 — `fav doc` ドキュメント生成（cli.fav 拡張）

> コメントと型情報から自動ドキュメントを生成。OSS 公開準備にも直結。

### 6-A: コメント付き AST 拡張
- [ ] `///` ドキュメントコメントを AST に保持（パーサー対応）
- [ ] `stage` / `fn` / `seq` / `type` 定義にコメントを紐付け

### 6-B: compiler.fav に doc 生成モジュール追加
- [ ] `fn doc_stage(name, comment, sig) -> String`
  - Markdown 形式のドキュメント断片を生成
- [ ] `fn doc_program(prog: Program) -> String`
  - ファイル全体のドキュメント生成

### 6-C: cli.fav に `doc` サブコマンド追加
- [ ] `fav doc src/ --out docs/` が動作すること
- [ ] Markdown 形式出力（HTML は後回し）

---

## Phase 7 — `fav profile` パイプライン計測（compiler.fav 拡張）

> 各 stage の実行時間を自動計測するコードを compiler.fav が挿入する。

### 7-A: compiler.fav に instrumentation 変換追加
- [ ] `--profile` フラグ時、各 stage 呼び出しの前後に `Env.now_ms()` を挿入
- [ ] 結果を `Map<String, Int>` として集計

### 7-B: cli.fav に `--profile` フラグ追加
- [ ] `fav run --profile pipeline.fav` が動作すること
- [ ] 実行後にステージ別実行時間を表示

---

## 完了条件サマリー

| Phase | 内容 | Rust 変更 | 優先度 |
|---|---|---|---|
| 1 | stdlib 拡充（List/String/Map/Result/Option） | なし | ★★★ |
| 2 | `fav fmt` フォーマッタ | なし | ★★★ |
| 3 | `fav lint` ルールエンジン | なし | ★★★ |
| 4 | `json` / `csv` / `http` / `llm` Rune | なし | ★★★ |
| 5 | `newtype` ラッパー | パーサーのみ | ★★ |
| 6 | `fav doc` ドキュメント生成 | コメント保持のみ | ★★ |
| 7 | `fav profile` 計測 | なし | ★ |

---

## 実装ノート

### セルフホスト開発の流れ（v10.0.0 以降の標準）

```
1. fav/self/ 配下の .fav ファイルを編集
2. fav check self/compiler.fav  → checker.fav で型チェック
3. fav run self/compiler.fav    → compiler.fav でコンパイル
4. cargo test                   → Rust 側統合テストを実行（最低限）
```

### 各 Phase の実装場所

| 対象 | ファイル |
|---|---|
| stdlib 関数追加 | `fav/self/stdlib/list_stdlib.fav` 等 |
| vm dispatch 追加 | `fav/src/vm.rs`（stdlib 追加時のみ） |
| checker 型登録 | `fav/self/checker.fav` + `fav/src/checker.rs` |
| compiler 拡張 | `fav/self/compiler.fav` |
| CLI コマンド追加 | `fav/self/cli.fav` |
| 新 Rune | `runes/<name>/` |

### 参考: 既存 stdlib Favnir 化の実装パターン（v8.2.0）

```favnir
// self/stdlib/list_stdlib.fav
pub fn List.chunk(xs: List<Int>, n: Int) -> List<List<Int>> = |xs, n| {
  // ... Favnir 実装 ...
}
```

```rust
// vm.rs — dispatch テーブルに追加
"List.chunk" => call_list_stdlib("List.chunk", args),
```

### テスト追加指針
- 各 Phase 最低 3 件の統合テスト（`fav/tests/` 配下）
- `fav check self/compiler.fav` の self-check が通り続けること
- Bootstrap 検証（bytecode_A == bytecode_B）を維持すること
