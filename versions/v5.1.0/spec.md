# Favnir v5.1.0 仕様書 — セルフホスト前提条件

作成日: 2026-05-19

---

## 概要

v6.0.0 セルフホスト（Favnir コンパイラを Favnir で実装）を可能にするための
言語機能・VM primitive 補完バージョン。

変更カテゴリ:
- **A**: 再帰的 sum type の許容
- **B**: ファイル I/O VM primitive 追加
- **C**: ビット演算 VM primitive 追加
- **D**: バイトコード仕様書の作成・凍結
- **E**: `String.chars` 追加

---

## A. 再帰的 sum type

### 仕様

Sum type（variant 型）のバリアントが自型を直接参照することを許容する。

```favnir
// v5.1.0 以降: OK
type Expr =
  | Lit(Int)
  | Str(String)
  | Add(Expr, Expr)
  | If(Expr, Expr, Expr)
  | Call(String, List<Expr>)

// record の直接再帰: 引き続き E0251 エラー（無限サイズのため）
type Bad = { next: Bad }  // ← E0251: recursive type without indirection

// record の再帰は Option で回避（変更なし）
type Node = { next: Option<Node> }  // ← OK
```

### 根拠

VM の `VMValue::Variant(String, Option<Box<VMValue>>)` は実行時にポインタ経由で
再帰を自然にハンドルする。型チェッカーのコンパイル時制約のみを緩和する。

---

## B. ファイル I/O VM primitive

### 追加関数

```favnir
// ファイル全体を String として読み込む
IO.read_file_raw(path: String) -> Result<String, String> !Io

// String をファイルに書き込む（上書き）
IO.write_file_raw(path: String, content: String) -> Result<Unit, String> !Io

// バイト列（List<Int>、各要素 0-255）をバイナリファイルとして書き込む
IO.write_bytes_raw(path: String, bytes: List<Int>) -> Result<Unit, String> !Io

// ファイルの存在確認
IO.file_exists_raw(path: String) -> Bool !Io
```

### 動作仕様

- `read_file_raw`: UTF-8 テキストファイルを読む。失敗時は `Err(message)` を返す
- `write_file_raw`: 親ディレクトリが存在しない場合は `Err` を返す
- `write_bytes_raw`: 各 Int は `& 0xFF` でバイト正規化する（0-255 範囲外は下位 8bit）
- `file_exists_raw`: ディレクトリには `false` を返す（ファイルのみ）

### エフェクト

全て `!Io` エフェクト（既存 `BUILTIN_EFFECTS` に `"Io"` が登録済み）。

---

## C. ビット演算 VM primitive

### 追加関数

```favnir
Int.shl(x: Int, n: Int)  -> Int   // x << n  (左シフト)
Int.shr(x: Int, n: Int)  -> Int   // x >> n  (算術右シフト)
Int.band(x: Int, y: Int) -> Int   // x & y   (ビット AND)
Int.bor(x: Int, y: Int)  -> Int   // x | y   (ビット OR)
Int.bxor(x: Int, y: Int) -> Int   // x ^ y   (ビット XOR)
Int.bnot(x: Int)         -> Int   // !x      (ビット NOT / 補数)
Int.to_byte(x: Int)      -> Int   // x & 0xFF（バイト正規化）
```

### 命名根拠

`Int.and` / `Int.or` は論理演算子 `&&` / `||` と混同するため、
ビット演算であることを明示する `band` / `bor` / `bxor` / `bnot` とする。

### エフェクト

なし（純粋関数）。

### 用途

バイトコード生成でバイト列を組み立てる際に使用:

```favnir
// オペコード 1 バイト + u32 オペランド 4 バイトのエンコード例
fn encode_u32(n: Int) -> List<Int> {
  [
    Int.to_byte(Int.band(n, 0xFF)),
    Int.to_byte(Int.band(Int.shr(n, 8),  0xFF)),
    Int.to_byte(Int.band(Int.shr(n, 16), 0xFF)),
    Int.to_byte(Int.band(Int.shr(n, 24), 0xFF))
  ]
}
```

---

## D. バイトコード仕様書（`docs/bytecode-spec.md`）

`artifact.rs` / `codegen.rs` から抽出した仕様を文書化し凍結。

### ファイルフォーマット（FVC バイナリ）

```
[4 bytes] magic: "FVC\x01"
[1 byte]  version: 0x20 (= v2.0)
[3 bytes] reserved: 0x00 0x00 0x00
[4 bytes] str_count: u32 (little-endian)
[4 bytes] fn_count:  u32
[4 bytes] global_count: u32

── 文字列テーブル ──────────────────────────────
[str_count 件]
  [4 bytes] length: u32
  [N bytes] UTF-8 bytes

── 型情報セクション ────────────────────────────
[fn_count 件]
  [4 bytes] return_ty_str_idx: u32
  [4 bytes] effect_str_idx:    u32

── グローバルテーブル ──────────────────────────
[global_count 件]
  [4 bytes] name_idx: u32
  [1 byte]  kind: u8  (0=fn, 1=builtin, 2=variant_ctor)
  [4 bytes] fn_idx:   u32

── 関数テーブル ────────────────────────────────
[fn_count 件]
  [4 bytes] name_idx:           u32
  [4 bytes] param_count:        u32
  [4 bytes] local_count:        u32
  [4 bytes] source_line:        u32
  [4 bytes] return_ty_str_idx:  u32
  [4 bytes] effect_str_idx:     u32
  [4 bytes] constants_count:    u32
  [constants_count 件] constant entries (see below)
  [4 bytes] code_len:           u32
  [code_len bytes] bytecode

── オプションセクション ────────────────────────
  explain_json セクション（存在する場合）
  type_meta セクション（存在する場合）
```

### 定数エントリ形式

```
[1 byte] tag: 0x01=Int, 0x02=Float, 0x03=Str, 0x04=Bool
  tag=Int:   [8 bytes] i64 little-endian
  tag=Float: [8 bytes] f64 little-endian
  tag=Str:   [4 bytes] str_idx: u32
  tag=Bool:  [1 byte]  0x00=false, 0x01=true
```

### オペコード一覧（凍結）

| Hex  | 名前               | オペランド        | 説明 |
|------|--------------------|-------------------|------|
| 0x01 | Const              | u32 const_idx     | 定数をスタックに積む |
| 0x02 | ConstUnit          | なし              | Unit をスタックに積む |
| 0x03 | ConstTrue          | なし              | true をスタックに積む |
| 0x04 | ConstFalse         | なし              | false をスタックに積む |
| 0x10 | LoadLocal          | u32 slot          | ローカル変数をロード |
| 0x11 | StoreLocal         | u32 slot          | ローカル変数にストア |
| 0x12 | LoadGlobal         | u32 global_idx    | グローバルをロード |
| 0x13 | Pop                | なし              | スタックトップを破棄 |
| 0x14 | Dup                | なし              | スタックトップを複製 |
| 0x15 | Call               | u32 arg_count     | 関数呼び出し |
| 0x16 | Return             | なし              | 関数から戻る |
| 0x20 | Add                | なし              | 加算 |
| 0x21 | Sub                | なし              | 減算 |
| 0x22 | Mul                | なし              | 乗算 |
| 0x23 | Div                | なし              | 除算 |
| 0x24 | Eq                 | なし              | 等値比較 |
| 0x25 | Ne                 | なし              | 不等値比較 |
| 0x26 | Lt                 | なし              | 小なり |
| 0x27 | Le                 | なし              | 以下 |
| 0x28 | Gt                 | なし              | 大なり |
| 0x29 | Ge                 | なし              | 以上 |
| 0x2A | And                | なし              | 論理 AND |
| 0x2B | Or                 | なし              | 論理 OR |
| 0x30 | Jump               | i32 offset        | 無条件ジャンプ |
| 0x31 | JumpIfFalse        | i32 offset        | false ならジャンプ |
| 0x32 | MatchFail          | なし              | パターンマッチ失敗（パニック）|
| 0x33 | ChainCheck         | なし              | bind チェーン Ok/Some 確認 |
| 0x34 | JumpIfNotVariant   | u32 str_idx, i32 offset | バリアント不一致ならジャンプ |
| 0x40 | GetField           | u32 str_idx       | レコードフィールド取得 |
| 0x41 | BuildRecord        | u32 field_count   | レコード構築 |
| 0x42 | MakeClosure        | u32 fn_idx, u32 capture_count | クロージャ生成 |
| 0x43 | GetVariantPayload  | なし              | バリアントのペイロード取得 |
| 0x50 | CollectBegin       | なし              | リスト収集開始 |
| 0x51 | CollectEnd         | なし              | リスト収集終了 |
| 0x52 | YieldValue         | なし              | ストリームへの yield |
| 0x53 | EmitEvent          | なし              | イベント emit |
| 0x54 | TrackLine          | u32 line          | デバッグ行追跡 |

**このオペコード表は凍結。v6.0.0 以前に番号・名前を変更しない。**
新規オペコードを追加する場合は 0x55 以降を使用。

---

## E. `String.chars`

### 追加関数

```favnir
String.chars(s: String) -> List<String>
```

### 動作仕様

- 文字列を Unicode スカラー値単位で分割し、各文字を 1 文字の String として返す
- 空文字列の場合は空リストを返す
- `Char` 型は新設しない（単一文字の String で代替）

### 用途

```favnir
// レキサーでの文字単位処理
let chars = String.chars("hello");  // ["h", "e", "l", "l", "o"]
```

### エフェクト

なし（純粋関数）。
