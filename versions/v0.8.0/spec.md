# Favnir v0.8.0 仕様書 — CLI + Tooling

更新日: 2026-04-30（Codex レビュー反映）

---

## 概要

v0.8.0 は二層構造のリリース。

1. **クリーンアップ層**（Phase 0）: v0.7.x で蓄積した技術的負債を解消
2. **Tooling 層**（Phase 1-5）: 開発体験を整える CLI コマンドを追加

---

## Phase 0: クリーンアップ

### 0-A: バージョン文字列の統一

| 場所 | 現状 | 修正後 |
|---|---|---|
| `Cargo.toml` / `version` | `"0.1.0"` | `"0.8.0"` |
| `src/main.rs` / HELP テキスト | `"v0.6.0"` | `"v0.8.0"` |

### 0-B: コンパイラ警告の解消（31件）

警告の分類と対応方針：

| カテゴリ | 件数 | 方針 |
|---|---|---|
| Span フィールド未参照（`ast.rs`） | ~13件 | Phase 3 のエラー改善で使う予定 → `#[allow(dead_code)]` |
| 未使用関数（`compose_effects`, `merge_effect`, `instantiate`） | 3件 | 削除 |
| 未使用 AST ノード（`EmitUnion`, `NamespaceDecl`, `UseDecl` など） | 4件 | `#[allow(dead_code)]` |
| 未使用フィールド（`type_params`, `subst`） | 2件 | 削除 |
| `eval::run`, `eval_item`（暫定実行系の遺物） | 2件 | Phase 0-D で解消 |

### 0-C: 共有 Value 型の切り出し

`eval::Value` を `vm.rs` が参照している設計臭を解消する。

```
src/value.rs   ← 新規（共有ルートに配置）
```

- `eval::Value` enum と全 impl を `value.rs` に移動
- `eval.rs` は `pub use crate::value::Value;` を残す（後方互換）
- `backend/vm.rs`: `use crate::eval::Value` → `use crate::value::Value`

### 0-D: eval.rs の廃止（段階的）

Codex 指摘に従い「切替 → 枯らす → 削除」の 2 ステップで実施。

```
Step 1: fav run の実行パスを VM に切り替える
  fav run → parse → typecheck → compile → codegen → vm.rs

Step 2: eval.rs を dead code 化し、確認後に削除
  - eval.rs は残したまま driver.rs から参照を切る
  - cargo build で eval.rs が完全に未参照になったことを確認
  - 削除
```

**前提**: 0-C（value.rs 切り出し）完了後に実施

**完了条件**
- `fav run examples/hello.fav` の出力が変更後も同一
- `cargo test` 全通過

---

## Phase 1: fav test（テストランナー）

### 構文

```favnir
test "add two numbers" {
    assert_eq(1 + 2, 3)
}

test "string predicate" {
    assert(String.starts_with("hello world", "hello"))
}

test "option unwrap" {
    bind v <- Option.to_result(some(42), "none")
    assert_eq(v, 42)
}
```

### 文法定義

`test` はトップレベル Item として追加。

```
TestDef ::= "test" StringLit Block
```

- `test` は新キーワード。`TokenKind::Test` を追加。
- Block は既存コードと同じ（`bind`, `chain`, `match` 等が使える）
- 戻り値は `Unit`
- テスト本体でのエフェクトは `!Io` / `!File` を許可

### 組み込みアサーション

| 関数 | 型 | 動作 |
|---|---|---|
| `assert(cond: Bool)` | `Unit` | `false` で TestFailure |
| `assert_eq(a: T, b: T)` | `Unit` | 不一致で TestFailure（Eq cap 利用） |
| `assert_ne(a: T, b: T)` | `Unit` | 一致で TestFailure（Eq cap 利用） |

### テストファイル規約

TypeScript スタイルのテストファイル分離をサポート。

```
*.test.fav    — テスト専用ファイル（test ブロックのみ記述可）
*.spec.fav    — 同上（仕様記述スタイルとして使う）
```

- `fav test` は `.fav` / `.test.fav` / `.spec.fav` を全て検索する
- `test` キーワードは通常の `.fav` ファイルにも記述可能（既存コードとの混在を許可）
- `*.test.fav` / `*.spec.fav` は `fav build` / `fav run` / `fav check` の対象外

### CLI

```
fav test [OPTIONS] [file|dir]

OPTIONS:
    --filter <pattern>   名前に pattern を含むテストのみ実行
    --fail-fast          最初の失敗で停止
    --trace              実行ログを表示（!Trace 出力を含む）

引数なし: fav.toml がある場合は src/**/*.{fav,test.fav,spec.fav} の全テストを実行
```

### 出力形式

```
running 3 tests in examples/hello.fav

test "add two numbers"      ok
test "string predicate"     ok
test "option unwrap"        FAILED

---- failures ----
test "option unwrap"
  assert_eq failed: left=42, right=43
  --> examples/hello.fav:12:5

test result: FAILED. 2 passed; 1 failed
```

### 実装方針

- `ast.rs`: `Item::TestDef(TestDef)` を追加
- `parser.rs`: `parse_test_item()` を追加
- `checker.rs`: テスト本体の型検査（エフェクト許可）
- `driver.rs`: `cmd_test()` を追加（VM で実行）
- テストは `fav build` / `fav exec` の対象外（test item はコンパイルしない）

---

## Phase 2: fav fmt（フォーマッタ）

### 動作

```
fav fmt [--check] [file]

--check  フォーマット差分があれば exit 1（ファイルは変更しない）
引数なし: fav.toml の src/**/*.fav を整形
```

### フォーマット規則（MVP スコープ）

Codex 指摘: `fn/trf/flw/type/block/expr` を優先し、`cap/impl/test` は後続。

| 要素 | 規則 |
|---|---|
| インデント | スペース 4 つ |
| トップレベル定義間 | 空行 1 行 |
| 演算子前後 | スペースあり（`\|>`, `<-`, `->`, 算術・比較） |
| ブロック `{ ... }` | 開き `{` は同行、閉じ `}` は独立行 |
| `match` アーム | `\| pattern => expr` を各行に |
| コメント | 位置を保持（ベストエフォート） |

**MVP 対象**: `fn`, `trf`, `flw`, `type`, `bind`, `match`, `if`, `expr`
**MVP 対象外**: `cap`, `impl`, `test`（後続タスク）

### 実装方針

- `src/fmt.rs` を新規作成（AST から直接 pretty-print）
- `driver.rs`: `cmd_fmt()` を追加

---

## Phase 3: エラーメッセージの改善

### 現状

```
error: type mismatch
  --> examples/hello.fav:5:1
```

### 目標

```
error[E001]: type mismatch: expected `Int`, found `String`
  --> examples/hello.fav:5:12
   |
 5 |     bind x <- "hello"
   |               ^^^^^^^ expected `Int`
   |
   = note: function `add` expects its first argument to be `Int`
```

### 実装内容

- `TypeError` の `span` フィールドをエラー出力で活用
- `driver.rs` に `format_diagnostic(source, span, label)` ヘルパー追加
- エラーコード `E001-E036` を `error[EXXX]:` 形式で表示

---

## Phase 4: fav explain 強化

### 現状

```
VIS      NAME              TYPE                              EFFECTS
public   fn main           () -> Unit                        !Io
```

### 追加する表示

```
VIS      NAME              TYPE                              EFFECTS       DEPS
public   fn main           () -> Unit                        !Io           IO.println
public   trf ParseCsv      String -> List<Row>               Pure          String.split, List.map
```

### DEPS の実装方針（IR ベース）

Codex 指摘: AST 走査より typed IR (middle::ir) の `IRExpr::Builtin` / `IRExpr::Var` を参照する方が
v0.6.0 以降の設計と整合する。

```rust
// IRProgram を走査して呼び出し先の名前を収集
fn collect_deps(fn_def: &IRFnDef) -> Vec<String>
```

- `IRExpr::Builtin(name)` → name を DEPS に追加
- `IRExpr::Var(name)` のうち top-level fn → name を DEPS に追加

### --full フラグ

`fav explain --full [file]`: 関数本体のブロックを展開表示（型付きのまま）

---

## Phase 5: fav lint（MVP）

Codex 指摘に従い「薄く切る」。最初から完全なスタイルチェックは目指さない。

### MVP スコープ（v0.8.0）

| チェック | 内容 |
|---|---|
| L001 | `pub fn` が型注釈（戻り値型）を持たない |
| L002 | 未使用の `bind` 束縛（値が以降の式で参照されない） |
| L003 | `fn` 名がスネークケースでない |
| L004 | `type` 名がパスカルケースでない |

### CLI

```
fav lint [file]

OPTIONS:
    --warn-only   エラーでなく警告として出力（exit 0）
```

### 実装方針

- `src/lint.rs` 新規作成（AST ウォーカー）
- `driver.rs`: `cmd_lint()` を追加
- `main.rs`: `"lint"` コマンドをディスパッチに追加
- チェックは AST レベルで実装（型情報不要な項目のみ MVP 対象）

---

## 完了条件（v0.8.0 全体）

- `cargo build` で警告ゼロ
- `cargo test` 全テスト通過（eval.rs 廃止後も同数以上）
- `fav test examples/test_sample.fav` が動く
- `fav fmt examples/hello.fav --check` が差分なし
- エラー出力に `^^^` アンダーラインが表示される
- `fav lint examples/hello.fav` が L001-L004 を検出できる
- `Cargo.toml` バージョンが `"0.8.0"`

---

## 対象外（v0.9.0 以降）

- WASM バックエンド — v0.9.0
- LSP サーバ — v1.0.0
- lint の完全なルールセット — v0.9.0+
