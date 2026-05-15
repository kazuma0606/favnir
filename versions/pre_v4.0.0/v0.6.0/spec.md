# Favnir v0.6.0 仕様書

更新日: 2026-04-29

## 概要

v0.6.0 は **「artifact/VM 版」** — tree-walking インタープリタを Typed IR → bytecode → 小型 VM のパイプラインに置き換えることをテーマとするバージョン。

- **Typed IR**: AST の型注釈付き中間表現。名前解決・型情報を持つ。
- **Bytecode**: Typed IR からコンパイルされるスタックベースの命令列。
- **小型 VM**: bytecode を実行する仮想機械。tree-walking インタープリタと同一の出力を保証する。
- **`.fvc` artifact**: bytecode + メタデータを格納するポータブルなバイナリ形式。
- **`fav build` / `fav exec`**: ビルド・実行 CLI コマンド。

> **スコープ注記**: rune / namespace / use（モジュール・パッケージ配布機能）は本版の対象外。
> これらは将来の「パッケージ/モジュール版」で扱う。v0.6.0 は単一ファイルの artifact 生成・実行に特化する。

---

## スコープ

### v0.6.0 で追加するもの

- `src/ir.rs` — Typed IR の定義（型注釈付き AST 的な中間表現）
- `src/compiler.rs` — AST → Typed IR のコンパイラ
- `src/codegen.rs` — Typed IR → bytecode のコードジェネレータ
- `src/vm.rs` — スタックベース VM（bytecode 実行）
- `src/artifact.rs` — `.fvc` フォーマットの読み書き（シリアライズ / デシリアライズ）
- `fav build <file> [-o <out.fvc>]` CLI コマンド
- `fav exec <file.fvc>` CLI コマンド
- v0.1.0〜v0.5.0 の全機能を VM 上で実行できること

### v0.6.0 では含まないもの

- 最適化パス（定数畳み込み、デッドコード除去、インライン展開）
- 末尾呼び出し最適化（TCO）
- ガベージコレクション（Rc のまま）
- JIT コンパイル
- 並行・並列実行
- 外部関数インタフェース（FFI）
- 完全なデバッグ情報（ソースマップ）— 関数名と行番号のみ保持
- WASM バックエンド（v0.9.0 以降）

---

## Typed IR

### 概念

Typed IR (以下 IR) は、型チェック後の AST を名前解決・脱糖した中間表現。

- 変数はシンボル名の代わりに **スロット番号**（`u16`）で参照する。
- グローバル関数・組み込みは名前で参照する（グローバルテーブルに登録）。
- 型情報が各ノードに付いている（`Type` は v0.4.0 の型）。
- `chain` は `IRStmt::Chain` として明示的に残す。
- `collect / yield` は `IRExpr::Collect` / `IRStmt::Yield` として残す。
- `|> match` デシュガーはパーサで済んでいるので IR レベルでは不要。

### IR 定義 (`src/ir.rs`)

```rust
// ── Typed IR ────────────────────────────────────────────────────────────────

pub struct IRProgram {
    pub globals: Vec<IRGlobal>,   // グローバルテーブル（関数・型コンストラクタ）
    pub fns:     Vec<IRFnDef>,
}

pub struct IRGlobal {
    pub name: String,
    pub kind: IRGlobalKind,
}

pub enum IRGlobalKind {
    Fn(usize),        // fns テーブルのインデックス
    Builtin,          // 組み込み（実行時に解決）
    VariantCtor,      // ADT コンストラクタ
}

pub struct IRFnDef {
    pub name:       String,
    pub param_count: usize,
    pub local_count: usize,   // params を含む全ローカル変数数
    pub effects:    Vec<Effect>,
    pub return_ty:  Type,
    pub body:       IRExpr,
}

// ── IRExpr ──────────────────────────────────────────────────────────────────

pub enum IRExpr {
    Lit(Lit, Type),
    Local(u16, Type),           // ローカル変数スロット
    Global(u16, Type),          // グローバルテーブルインデックス
    Call(Box<IRExpr>, Vec<IRExpr>, Type),
    Block(Vec<IRStmt>, Box<IRExpr>, Type),
    If(Box<IRExpr>, Box<IRExpr>, Box<IRExpr>, Type),
    Match(Box<IRExpr>, Vec<IRArm>, Type),
    FieldAccess(Box<IRExpr>, String, Type),
    BinOp(BinOp, Box<IRExpr>, Box<IRExpr>, Type),
    Closure(Vec<u16>, Box<IRExpr>, Type),   // キャプチャスロット一覧 + body
    Collect(Box<IRExpr>, Type),             // collect ブロック（body は IRExpr::Block）
    Emit(Box<IRExpr>, Type),
    RecordConstruct(Vec<(String, IRExpr)>, Type),
}

impl IRExpr {
    pub fn ty(&self) -> &Type { /* ... */ }
}

// ── IRStmt ──────────────────────────────────────────────────────────────────

pub enum IRStmt {
    Bind(u16, IRExpr),    // ローカルスロット + 右辺
    Chain(u16, IRExpr),   // chain 束縛（失敗で脱出）
    Yield(IRExpr),        // collect スタックへ追加
    Expr(IRExpr),         // 副作用のみの式文
}

// ── IRArm ───────────────────────────────────────────────────────────────────

pub struct IRArm {
    pub pattern: IRPattern,
    pub guard:   Option<IRExpr>,
    pub body:    IRExpr,
}

pub enum IRPattern {
    Wildcard,
    Lit(Lit),
    Bind(u16),                            // スロット番号
    Variant(String, Option<Box<IRPattern>>),
    Record(Vec<(String, IRPattern)>),
}
```

### IR コンパイラ (`src/compiler.rs`)

AST → IR の変換。型チェック済みの AST を受け取る。

主な処理:

1. **名前解決**: 各識別子がローカルスロット・グローバルインデックスのいずれかを確定する。
2. **ローカル変数スロットの割り当て**: 関数ごとに変数名 → スロット番号のマップを構築する。
3. **クロージャのキャプチャ解析**: クロージャが参照する外部スロットを特定し、`Closure.captures` に記録する。
4. **パターン変数のスロット化**: パターン内の束縛変数にスロットを割り当てる。
5. **型情報の引き渡し**: Checker が推論した型を各 IRExpr に付与する（型推論ありの場合は型環境から引く）。

```
compile_program(program: &Program, checker_env: &TyEnv) -> IRProgram
compile_fn_def(fd: &FnDef, ctx: &mut CompileCtx) -> IRFnDef
compile_expr(expr: &Expr, ctx: &mut CompileCtx) -> IRExpr
compile_stmt(stmt: &Stmt, ctx: &mut CompileCtx) -> IRStmt
compile_pattern(pat: &Pattern, ctx: &mut CompileCtx) -> IRPattern
```

---

## Bytecode

### 設計方針

- **スタックベース**: オペランドスタックに値をプッシュ・ポップして演算する。
- **固定幅オペコード**: オペコード 1 バイト + オペランド（0〜4 バイト）。
- **1 関数 = 1 命令列**: 関数呼び出しはスタック上の `Value::Closure` (またはグローバル関数参照) を通じて行う。
- **定数プール**: `CONST_INT` などは定数プールのインデックスを参照する。

### 命令セット

| オペコード | バイト | オペランド | 動作 |
|---|---|---|---|
| `NOP` | 0x00 | — | 何もしない |
| **定数** | | | |
| `CONST` | 0x01 | idx: u16 | 定数プール[idx] をプッシュ |
| `CONST_UNIT` | 0x02 | — | Unit をプッシュ |
| `CONST_TRUE` | 0x03 | — | Bool(true) をプッシュ |
| `CONST_FALSE` | 0x04 | — | Bool(false) をプッシュ |
| **ローカル変数** | | | |
| `LOAD_LOCAL` | 0x10 | slot: u16 | ローカル変数スロット[slot] をプッシュ |
| `STORE_LOCAL` | 0x11 | slot: u16 | スタックトップをスロット[slot] に格納（ポップしない） |
| **グローバル** | | | |
| `LOAD_GLOBAL` | 0x12 | idx: u16 | グローバルテーブル[idx] をプッシュ（関数・組み込み） |
| **スタック操作** | | | |
| `POP` | 0x20 | — | スタックトップを捨てる |
| `DUP` | 0x21 | — | スタックトップを複製 |
| **算術・比較** | | | |
| `ADD` | 0x30 | — | pop b, pop a → push a+b |
| `SUB` | 0x31 | — | pop b, pop a → push a-b |
| `MUL` | 0x32 | — | pop b, pop a → push a*b |
| `DIV` | 0x33 | — | pop b, pop a → push a/b（ゼロ除算は実行時エラー） |
| `EQ` | 0x34 | — | pop b, pop a → push a==b |
| `NEQ` | 0x35 | — | pop b, pop a → push a!=b |
| `LT` | 0x36 | — | pop b, pop a → push a<b |
| `GT` | 0x37 | — | pop b, pop a → push a>b |
| `LEQ` | 0x38 | — | pop b, pop a → push a<=b |
| `GEQ` | 0x39 | — | pop b, pop a → push a>=b |
| **制御フロー** | | | |
| `JUMP` | 0x40 | offset: i16 | IP += offset（相対ジャンプ） |
| `JUMP_IF_FALSE` | 0x41 | offset: i16 | pop bool → false なら IP += offset |
| `JUMP_IF_NOT_VARIANT` | 0x42 | name_idx: u16, offset: i16 | スタックトップの Variant タグが name でなければジャンプ |
| **関数呼び出し** | | | |
| `CALL` | 0x50 | n_args: u8 | スタックから n_args 個 + callee をポップ、呼び出し、結果をプッシュ |
| `RETURN` | 0x51 | — | スタックトップを返り値としてフレームを破棄 |
| `MAKE_CLOSURE` | 0x52 | fn_idx: u16, n_cap: u8 | n_cap 個のキャプチャをポップして Closure を作成 |
| **コレクション構築** | | | |
| `BUILD_LIST` | 0x60 | n: u16 | スタックから n 個ポップして List を作成（順序: 先頭が底） |
| `BUILD_RECORD` | 0x61 | n: u8, names_idx: u16 | スタックから n 個ポップしてフィールド名と合わせ Record を作成 |
| **ADT** | | | |
| `BUILD_VARIANT` | 0x70 | name_idx: u16, has_payload: u8 | スタックトップ（has_payload=1 の場合）を payload として Variant を作成 |
| `GET_VARIANT_PAYLOAD` | 0x71 | — | pop Variant → push payload（payload なしは Unit） |
| **フィールドアクセス** | | | |
| `GET_FIELD` | 0x72 | name_idx: u16 | pop Record → push field |
| **v0.5.0 機能** | | | |
| `CHAIN_CHECK` | 0x80 | escape_offset: i16 | スタックトップを検査: `ok(v)`/`some(v)` なら v をプッシュ, `err(e)`/`none` ならスタックをクリアしてジャンプ |
| `COLLECT_BEGIN` | 0x81 | — | COLLECT_STACK に新フレームをプッシュ |
| `COLLECT_END` | 0x82 | — | COLLECT_STACK からフレームをポップして List をプッシュ |
| `YIELD_VALUE` | 0x83 | — | pop value → COLLECT_STACK トップに追加、Unit をプッシュ |
| **エフェクト** | | | |
| `EMIT_EVENT` | 0x90 | — | pop value → EMIT_LOG に追加、Unit をプッシュ |
| `BUILTIN_CALL` | 0x91 | ns_idx: u16, method_idx: u16, n_args: u8 | 組み込み関数を呼び出す |
| **デバッグ** | | | |
| `MATCH_FAIL` | 0xFF | — | 非網羅的マッチで実行時エラー |

### 定数プール

定数プールは各関数の bytecode に付随する配列。型タグ 1 バイト + ペイロード。

| タグ | 型 | ペイロード |
|---|---|---|
| 0x01 | Int | i64 (8 bytes, little-endian) |
| 0x02 | Float | f64 (8 bytes, little-endian) |
| 0x03 | Str | len: u32 + UTF-8 bytes |
| 0x04 | Name | len: u16 + UTF-8 bytes（フィールド名・バリアント名・関数名） |

---

## `.fvc` ファイルフォーマット

`.fvc` (Favnir Compiled) は関数バイトコード + メタデータを格納するバイナリ形式。

### 全体構造

```
[Header]
[String table]
[Types section]       ← 将来拡張: v0.6.0 では最小実装
[Globals table]
[Functions section]
```

### Header (16 bytes)

```
magic:    [u8; 4]  = b"FVC\x01"
version:  u8       = 0x06           ← v0.6.0
flags:    u8       = 0x00           ← 予約（将来: debug info あり/なし など）
reserved: [u8; 2]  = [0x00, 0x00]
str_count:  u32                     ← String table のエントリ数
glob_count: u32                     ← Globals table のエントリ数
fn_count:   u32                     ← Functions section のエントリ数
```

### String Table

全ての文字列（関数名、フィールド名、バリアント名、ソースファイル名）を集約する。
各セクションは文字列インデックス (`u32`) で参照する。

```
[str_count entries]
  len: u32
  data: [u8; len]   ← UTF-8
```

### Globals Table

```
[glob_count entries]
  name_idx: u32     ← String table インデックス
  kind: u8          ← 0=Fn, 1=Builtin, 2=VariantCtor
  fn_idx: u32       ← kind=Fn の場合のみ使用（Functions section インデックス）
```

### Types Section（v0.6.0 最小実装）

関数シグネチャのみ保持（デバッグ情報・エラーメッセージ用）。

```
[fn_count entries]
  return_ty_str_idx: u32    ← 戻り型の文字列表現（"Int" "Result<Int,String>" など）
  effect_str_idx:    u32    ← エフェクトの文字列表現（"Pure" "!Io !Db" など）
```

> **将来の方針**: v0.6.0 では文字列化シグネチャのみを保存するが、将来的には型の構造体（ADT バリアント、フィールド名と型、型パラメータ）を構造化データとして格納する予定。これにより `fav explain --artifact`（artifact から型情報を表示）や、バージョン間の互換性検証（同じシグネチャを持つ関数かをバイナリ比較）が実現できる。

### Functions Section

```
[fn_count entries]
  name_idx:    u32      ← String table
  param_count: u8
  local_count: u16      ← params を含む全ローカル数
  source_line: u32      ← 定義開始行（デバッグ用）
  pool_size:   u16      ← 定数プールエントリ数
  code_len:    u32      ← bytecode バイト数
  [pool_size entries]   ← 定数プール（tag + payload）
  [code_len bytes]      ← bytecode
```

---

## VM の実装

### データ構造 (`src/vm.rs`)

```rust
pub struct VM {
    globals: Vec<Value>,         // グローバルテーブル（関数・組み込み）
    stack:   Vec<Value>,         // オペランドスタック
    frames:  Vec<CallFrame>,     // 呼び出しフレームスタック
}

struct CallFrame {
    fn_idx:  usize,             // 実行中の関数インデックス
    ip:      usize,             // 命令ポインタ
    base:    usize,             // このフレームのスタックベース（locals の開始位置）
}
```

ローカル変数はスタック上に確保する（フレームベース + スロット番号でアドレス計算）。

```
stack: [ ... | param0 | param1 | local2 | local3 | ... | (作業領域) ]
              ↑ frame.base
              local[0] = stack[frame.base + 0]
              local[1] = stack[frame.base + 1]
```

### 実行ループ

```rust
pub fn run(&mut self, fn_idx: usize, args: Vec<Value>) -> Result<Value, VMError>
```

1. 初期フレームを積む（`fn_idx`, `ip=0`, `base=0`）。
2. args をスタックに積む（locals の先頭 = params）。
3. ループ: `frames.last()` からオペコードを 1 バイト読み、dispatch する。
4. `RETURN` で値をスタックに残してフレームを破棄。フレームがなくなれば終了。

### ChainEscape の扱い

`CHAIN_CHECK` 命令がスタックトップを検査:

- `ok(v)` または `some(v)` → payload `v` をプッシュして継続
- `err(e)` または `none` → **ChainEscape** として扱う

ChainEscape は専用のシグナル値（特殊な `Value` バリアント、または VM の別フィールド）として伝播し、`CALL` / `RETURN` の境界でキャッチして通常の返り値にする。

```rust
pub enum VMSignal {
    Normal(Value),
    Escape(Value),   // chain の早期脱出値
}
```

`CALL` を実行する際:
- 新フレームで実行
- `VMSignal::Escape(v)` が返ってきたら → `RETURN` せずに `Ok(v)` として上位フレームに返す

### COLLECT_STACK

tree-walking インタープリタと同じスレッドローカル `COLLECT_STACK` を VM でも利用する（`src/eval.rs` から `pub` に昇格させて共有、または `vm.rs` に移動）。

`COLLECT_BEGIN` / `COLLECT_END` / `YIELD_VALUE` はインタープリタの `collect_push_frame` / `collect_pop_frame` / `collect_yield` と対応する。

> **実装方針**: v0.6.0 では `eval.rs` の `COLLECT_STACK` 実装を流用するのが現実的（VM 追加コストを最小化）。将来的には VM の呼び出しフレームスタックに collect フレームを統合した VM 専用実装へ移行する予定（スレッドローカルへの依存を排除し、並行実行対応への布石とする）。

### VMError

```rust
pub struct VMError {
    pub message: String,
    pub fn_name: String,
    pub ip:      usize,    // エラー発生時の命令ポインタ
}
```

---

## コードジェネレータ (`src/codegen.rs`)

Typed IR → bytecode の変換。

### 概要

```rust
pub struct Codegen {
    code:       Vec<u8>,        // 生成中の命令列
    constants:  Vec<Constant>,  // 定数プール
    str_table:  Vec<String>,    // 文字列テーブル（グローバル共有）
    locals:     Vec<String>,    // ローカル変数名 → スロット（デバッグ用）
}
```

主なコンパイル関数:

```
emit_expr(expr: &IRExpr, cg: &mut Codegen)
emit_stmt(stmt: &IRStmt, cg: &mut Codegen)
emit_pattern_match(pat: &IRPattern, scrutinee_slot: u16, fail_label: Label, cg: &mut Codegen)
```

### ジャンプパッチング

前向きジャンプ（`if` / `match` の分岐）は「プレースホルダ」として 0 を書き込み、後からパッチする方式。

```rust
fn emit_jump(&mut self) -> usize { /* offset 位置を返す */ }
fn patch_jump(&mut self, pos: usize) { /* 現在位置を計算して書き戻す */ }
```

### match コンパイル

各アームを順に試みる直列コード:

```
push scrutinee
dup
JUMP_IF_NOT_VARIANT "ok" → next_arm
GET_VARIANT_PAYLOAD
STORE_LOCAL slot_n
[guard code if any]
JUMP_IF_FALSE next_arm
[body code]
JUMP arm_end
next_arm:
dup
...
MATCH_FAIL   ← 全アーム失敗
arm_end:
POP          ← scrutinee を捨てる
```

---

## CLI の変更

### `fav build`

```
fav build <file> [-o <out.fvc>]
```

- `<file>` をパース・型チェック・IR コンパイル・コード生成して `.fvc` を書き出す。
- `-o` を省略した場合: ファイル名の拡張子を `.fvc` に変えたものを同じディレクトリに出力。
- エラーがあれば型チェックエラーを表示して非 0 で終了。

```bash
fav build examples/chain.fav          # → examples/chain.fvc
fav build examples/chain.fav -o dist/chain.fvc
```

### `fav exec`

```
fav exec <file.fvc> [--db <path>]
```

- `.fvc` ファイルを読み込み、VM で `main` を実行する。
- `--db` は `fav run` と同じく SQLite DB へのパス（`:memory:` デフォルト）。
- パース・型チェックは行わない（artifact に含まれる情報のみ使う）。

```bash
fav exec examples/chain.fvc
fav exec myapp.fvc --db myapp.db
```

### `fav run` への変更

`fav run` は引き続き tree-walking インタープリタを使う（変更なし）。
bytecode を経由するパスは `fav build` + `fav exec` として分離する。

---

## 実装フェーズ

### Phase 1: Typed IR (`src/ir.rs` + `src/compiler.rs`)

- IR の定義（`IRProgram` / `IRFnDef` / `IRExpr` / `IRStmt` / `IRArm` / `IRPattern`）
- IR コンパイラ: 変数スロット割り当て + グローバルテーブル構築
- IR の Debug 表示（`-dump-ir` フラグ、オプション）

### Phase 2: Bytecode + コードジェネレータ (`src/codegen.rs`)

- 命令セットの定義（`Opcode` enum）
- `Codegen` 構造体 + 定数プール管理
- 式・文・パターンのコンパイル
- ジャンプパッチング

### Phase 3: `.fvc` フォーマット (`src/artifact.rs`)

- Header の読み書き
- String table / Globals / Types / Functions セクションの読み書き
- `FvcWriter` / `FvcReader` の実装

### Phase 4: VM (`src/vm.rs`)

- スタックと呼び出しフレームの管理
- 命令ディスパッチループ
- 組み込み関数の呼び出し（eval.rs の `eval_builtin` 相当をポート）
- ChainEscape の伝播とキャッチ
- COLLECT_STACK の利用

### Phase 5: CLI + 統合

- `fav build` の実装（main.rs）
- `fav exec` の実装（main.rs）
- tree-walking インタープリタと出力が一致することの確認

### Phase 6: テストと例

- 命令生成の単体テスト
- VM 実行の単体テスト（`fav run` と `fav exec` で同じ出力が出ること）
- 既存の examples 全て（`fav build` → `fav exec` が通ること）

---

## エラーコード

v0.6.0 では新しいコンパイルエラー・実行時エラーを追加する。

| コード | フェーズ | 内容 |
|--------|----------|------|
| E031 | build | `.fvc` ファイルの書き込みに失敗した |
| E032 | exec | `.fvc` ファイルが見つからない / 読み込めない |
| E033 | exec | `.fvc` ファイルのマジックバイトが不正（非 FVC ファイル） |
| E034 | exec | `.fvc` ファイルのバージョンが非互換 |
| E035 | exec | `main` 関数が artifact に存在しない |

VM 実行時エラー（`VMError`）はコードなし。ファイル名と命令ポインタを含めてメッセージを出力する。

---

## 例

### コンパイルと実行

```bash
# 型チェック
fav check examples/pipe_match.fav

# bytecode にコンパイル
fav build examples/pipe_match.fav
# → examples/pipe_match.fvc を生成

# bytecode で実行
fav exec examples/pipe_match.fvc
# tree-walking インタープリタと同じ出力
```

### 生成される bytecode のイメージ

```favnir
// 元のソース
fn classify(n: Int) -> String {
    n |> match {
        x where x > 0 => "positive"
        x where x < 0 => "negative"
        _              => "zero"
    }
}
```

```
// classify 関数の bytecode（概念図）
LOAD_LOCAL 0        ; n をプッシュ
DUP                 ; scrutinee を複製（match の各アームで使う）
STORE_LOCAL 1       ; x = n (arm1)
LOAD_LOCAL 1        ; x
CONST 0             ; 0
GT                  ; x > 0
JUMP_IF_FALSE 8     ; → arm2
POP                 ; scrutinee を捨てる
CONST 1             ; "positive"
JUMP 20             ; → end

; arm2
STORE_LOCAL 1       ; x = n
LOAD_LOCAL 1        ; x
CONST 0             ; 0
LT                  ; x < 0
JUMP_IF_FALSE 8     ; → arm3
POP
CONST 2             ; "negative"
JUMP 4              ; → end

; arm3 (wildcard)
POP
CONST 3             ; "zero"

; end
RETURN
```

### `.fvc` の確認（`fav exec --info`）

`fav disasm`（逆アセンブル）は v0.6.0 に含まない。代わりに `fav exec --info` でメタデータのみ表示する。

```bash
fav exec --info examples/pipe_match.fvc
```

出力フォーマット（固定）:

```
FVC v0.6  <glob_count> globals  <fn_count> functions

<関数名>  <パラメータ型リスト> -> <戻り型>  <エフェクト列>
...
```

出力例:

```
FVC v0.6  4 globals  3 functions

main      () -> Unit         !Io
classify  (Int) -> String    Pure
helper    (Int, Int) -> Bool Pure
```

- **パラメータ型リスト**: `(型1, 型2, ...)` — 引数なしは `()`
- **戻り型**: Types Section の `return_ty_str_idx` から取得
- **エフェクト列**: Types Section の `effect_str_idx` から取得（Pure は明示的に `Pure` と表示）
- bytecode の中身（命令列）は表示しない（`fav disasm` の領域）

---

## 既存コードへの影響

| ファイル | 変更 |
|---|---|
| `src/main.rs` | `fav build` / `fav exec` サブコマンドを追加 |
| `src/eval.rs` | `COLLECT_STACK` / `collect_*` 関数を `pub` に変更して VM からも使えるようにする（または `vm.rs` に移動）|
| `src/checker.rs` | 変更なし（checker の出力を IR コンパイラに渡すアダプタを追加するのみ）|
| `src/ast.rs` | 変更なし |
| `src/parser.rs` | 変更なし |

新規追加:
- `src/ir.rs`
- `src/compiler.rs`
- `src/codegen.rs`
- `src/vm.rs`
- `src/artifact.rs`

---

## 完了条件

- `fav build examples/hello.fav` が成功し `.fvc` ファイルが生成される
- `fav exec examples/hello.fvc` が `fav run examples/hello.fav` と同じ出力を返す
- 既存の全 examples（`hello`, `pipeline`, `adt_match`, `users`, `chain`, `collect`, `pipe_match`, `generics`, `cap_sort`, `cap_user`）が `fav build` → `fav exec` で通る
- 既存 202 テストが全パス（デグレなし）
- VM 専用の単体テスト（各命令・各機能）が全パス
