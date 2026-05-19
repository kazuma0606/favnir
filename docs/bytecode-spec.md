> **凍結宣言**: このファイルは v6.0.0 セルフホスト完成まで変更禁止。
> オペコード番号・名前を変更する場合は v6.0.0 以降のバージョンで行うこと。
> 新規オペコードを追加する場合は 0x55 以降を使用すること。

# Favnir バイトコード仕様書 (FVC)

作成日: 2026-05-20
対象バージョン: v5.1.0 以降、v6.0.0 まで有効

---

## ファイルフォーマット（FVC バイナリ）

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

---

## 定数エントリ形式

```
[1 byte] tag: 0x01=Int, 0x02=Float, 0x03=Str, 0x04=Bool
  tag=Int:   [8 bytes] i64 little-endian
  tag=Float: [8 bytes] f64 little-endian
  tag=Str:   [4 bytes] str_idx: u32
  tag=Bool:  [1 byte]  0x00=false, 0x01=true
```

---

## オペコード一覧（凍結）

| Hex  | 名前               | オペランド                      | 説明 |
|------|--------------------|---------------------------------|------|
| 0x01 | Const              | u32 const_idx                   | 定数をスタックに積む |
| 0x02 | ConstUnit          | なし                            | Unit をスタックに積む |
| 0x03 | ConstTrue          | なし                            | true をスタックに積む |
| 0x04 | ConstFalse         | なし                            | false をスタックに積む |
| 0x10 | LoadLocal          | u32 slot                        | ローカル変数をロード |
| 0x11 | StoreLocal         | u32 slot                        | ローカル変数にストア |
| 0x12 | LoadGlobal         | u32 global_idx                  | グローバルをロード |
| 0x13 | Pop                | なし                            | スタックトップを破棄 |
| 0x14 | Dup                | なし                            | スタックトップを複製 |
| 0x15 | Call               | u32 arg_count                   | 関数呼び出し |
| 0x16 | Return             | なし                            | 関数から戻る |
| 0x20 | Add                | なし                            | 加算 |
| 0x21 | Sub                | なし                            | 減算 |
| 0x22 | Mul                | なし                            | 乗算 |
| 0x23 | Div                | なし                            | 除算 |
| 0x24 | Eq                 | なし                            | 等値比較 |
| 0x25 | Ne                 | なし                            | 不等値比較 |
| 0x26 | Lt                 | なし                            | 小なり |
| 0x27 | Le                 | なし                            | 以下 |
| 0x28 | Gt                 | なし                            | 大なり |
| 0x29 | Ge                 | なし                            | 以上 |
| 0x2A | And                | なし                            | 論理 AND |
| 0x2B | Or                 | なし                            | 論理 OR |
| 0x30 | Jump               | i32 offset                      | 無条件ジャンプ |
| 0x31 | JumpIfFalse        | i32 offset                      | false ならジャンプ |
| 0x32 | MatchFail          | なし                            | パターンマッチ失敗（パニック）|
| 0x33 | ChainCheck         | なし                            | bind チェーン Ok/Some 確認 |
| 0x34 | JumpIfNotVariant   | u32 str_idx, i32 offset         | バリアント不一致ならジャンプ |
| 0x40 | GetField           | u32 str_idx                     | レコードフィールド取得 |
| 0x41 | BuildRecord        | u32 field_count                 | レコード構築 |
| 0x42 | MakeClosure        | u32 fn_idx, u32 capture_count   | クロージャ生成 |
| 0x43 | GetVariantPayload  | なし                            | バリアントのペイロード取得 |
| 0x50 | CollectBegin       | なし                            | リスト収集開始 |
| 0x51 | CollectEnd         | なし                            | リスト収集終了 |
| 0x52 | YieldValue         | なし                            | ストリームへの yield |
| 0x53 | EmitEvent          | なし                            | イベント emit |
| 0x54 | TrackLine          | u32 line                        | デバッグ行追跡 |

**このオペコード表は凍結。v6.0.0 以前に番号・名前を変更しない。**
新規オペコードを追加する場合は 0x55 以降を使用すること。

---

## 多引数バリアントの表現（v5.1.0 追加）

`Add(Expr, Expr)` のような複数引数を持つ tuple バリアントは、
実行時に引数を位置付きレコードとしてラップして格納する:

```
Variant("Add", Some(Record({"_0": v0, "_1": v1})))
```

パターンマッチ `Add(a, b)` は内部的に `{ _0: a, _1: b }` レコード分解として処理される。

---

## 参照実装

- バイナリ書き込み: `fav/src/backend/artifact.rs`
- コード生成: `fav/src/backend/codegen.rs`
- VM 実行: `fav/src/backend/vm.rs`
