# Favnir v6.4.0 仕様書 — Playground 改善

作成日: 2026-05-26

---

## テーマ

Playground を「データエンジニア向けデモ」として機能させる。

現状の課題:
- WASM バイナリのビルド・デプロイ手順が自動化されていない
- WASM バックエンドが `Int` / `Float` / `Bool` / `String` / `Unit` しか扱えない
  - `List<T>` や Record 型を返す関数は `UnsupportedType(W001)` エラーになる
- サンプルコードが `Int` 演算のみで、Favnir の強みである pipeline が伝わらない

---

## Phase A — WASM ビルドパイプライン整備

### 背景

`site/public/wasm/` に `favnir.js` / `favnir_bg.wasm` 等が存在するが、
`fav/src/wasm_entry.rs`（またはそれに相当するクレート）を変更した際に
自動的に再ビルドされる仕組みがない。

### 仕様

`scripts/build-wasm.sh` を作成する:

```bash
#!/usr/bin/env bash
set -euo pipefail

# fav-wasm クレートを wasm32-unknown-unknown ターゲットでビルド
cd "$(git rev-parse --show-toplevel)/fav"
wasm-pack build --target web --out-dir ../site/public/wasm fav-wasm/
# または cargo build --target wasm32-unknown-unknown の直接呼び出し
```

`deploy-site` スキル（`scripts/deploy-site.sh`）の冒頭で `build-wasm.sh` を呼ぶ。

完了条件:
- `scripts/build-wasm.sh` を実行すると `site/public/wasm/` が最新バイナリで更新される
- `deploy-site` スキル実行時に WASM が自動ビルドされる

---

## Phase B — WASM バックエンド: List 型対応

### 設計方針

WebAssembly の線形メモリ上にリストをヒープ表現する。
既存の bump-alloc (`HEAP_PTR_GLOBAL_IDX`) を使う。

#### リストのメモリレイアウト

```
[  tag: i32  ][  head_ptr: i32  ]   (Cons セル, 8 bytes)
[  tag: i32  ]                      (Nil セル, 4 bytes)
```

`tag` は `0 = Nil`, `1 = Cons` とする。

#### 型マッピング

| Favnir 型 | WASM 表現 |
|-----------|----------|
| `List<Int>` | `i32` (heap pointer) |
| `List<String>` | `i32` (heap pointer) |
| `List<T>` 一般 | `i32` (heap pointer) |

`favnir_type_to_wasm_results` / `favnir_type_to_wasm_params` を拡張:

```rust
Type::List(_) => Ok(vec![ValType::I32]),  // heap pointer
```

#### 対応する IR 式

| IR | WASM 操作 |
|----|----------|
| `IRExpr::RecordConstruct` (リストコンストラクタ) | bump_alloc → store tag + head |
| `List.singleton(x)` builtin | Cons セル + Nil セル を alloc |
| `List.first` / `List.rest` builtin | load head_ptr / rest_ptr |

#### ホスト関数追加

Playground の出力のため `io_println_list_int` を `wasm_exec.rs` に追加:

```rust
"fav_host", "io_println_list_int" : (ptr: i32) → walk linked list and print
```

### 完了条件

- `List<Int>` を返す関数が WASM コンパイルできる
- `List.singleton` / `List.first` / `List.rest` が WASM で動作する
- `cargo test wasm_list` が通る

---

## Phase C — WASM バックエンド: Record 型対応

### 設計方針

Record をヒープ上の固定オフセット構造体として表現する。
フィールド順は checker が決定した順序に従う。

#### レコードのメモリレイアウト

各フィールドを `i64`（8 bytes）で格納（Int/Float/Bool/ポインタを統一表現）。

```
offset 0:  field_0 as i64
offset 8:  field_1 as i64
...
```

#### 型マッピング

| Favnir 型 | WASM 表現 |
|-----------|----------|
| `{name: String, age: Int}` (record) | `i32` (heap pointer) |
| Record 型一般 | `i32` (heap pointer) |

`favnir_type_to_wasm_results`:

```rust
Type::Record(_) => Ok(vec![ValType::I32]),  // heap pointer
```

#### IR 式への対応

| IR | WASM 操作 |
|----|----------|
| `IRExpr::RecordConstruct(fields, ty)` | bump_alloc(size) → store each field |
| `IRExpr::FieldAccess(record, field, ty)` | i64.load at computed offset |

### 完了条件

- `{name: String, age: Int}` を返す関数が WASM コンパイルできる
- フィールドアクセスが正しいオフセットで動作する
- `cargo test wasm_record` が通る

---

## Phase D — Playground サンプルコード更新

### 現状

`site/app/playground/page.tsx` の `EXAMPLE_CODE`:
- `clamp(value, lo, hi)` — Int 演算のみ
- `stage` / `seq` を使っておらず、Favnir の強みが伝わらない

### 変更後サンプル

```favnir
// Favnir Playground — stage/seq パイプライン例

stage Double: Int -> Int = |n| { n * 2 }
stage AddOne: Int -> Int = |n| { n + 1 }
stage Square: Int -> Int = |n| { n * n }

seq Transform = Double |> AddOne |> Square

public fn main() -> Unit !Io {
  bind result <- Transform(3)
  IO.println_int(result)
}
```

`Transform(3)` = `square(add_one(double(3)))` = `square(7)` = `49`

### 「非対応」メッセージの更新

List/Record は Phase B/C で対応するため、残存する非対応型（Option/Result/Sum 等）
に絞ってメッセージを更新する。

### 完了条件

- Playground でサンプルを実行すると `49` が出力される
- stage/seq を使った例が動くことをブラウザで確認できる

---

## 完了条件まとめ

1. `scripts/build-wasm.sh` が存在し、`deploy-site` から呼ばれる
2. `List<Int>` を使うプログラムが Playground でコンパイル・実行できる
3. Record 型を使うプログラムが Playground でコンパイル・実行できる
4. Playground のデフォルトサンプルが `stage`/`seq` を使ったパイプライン例になっている
5. `cargo test` 全テスト通過（List/Record の WASM テストを含む）
