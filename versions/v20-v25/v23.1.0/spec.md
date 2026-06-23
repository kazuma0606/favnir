# v23.1.0 仕様書 — `Bytes` 型

## 概要

生バイト列を Favnir から直接操作できる `Bytes` 型を実装する。
`Bytes` 型は vm.fav（v23.4〜v23.8）のバイトコードデコードで必要となるプリミティブであり、
v23.x「VM in Favnir」ロードマップの第一歩。

**テーマ**: 「Favnir の表現力を VM 実装レベルまで引き上げる最初のステップ」

---

## ロードマップ完了条件との対応

v23.1.0 は VM in Favnir ロードマップ（v23.1〜v24.0）の前提条件整備フェーズ。
ロードマップ v23.1「`Bytes` 型」を実装する。

完了条件: 「`Bytes` 型 + ビット演算 + `Mut<T>` が動作する」の第一段階

---

## 機能仕様

### Bytes 型の基本操作

```favnir
// 生成
bind data  <- Bytes.from_hex("464f4f")    // "FOO" (3 bytes)
bind data2 <- Bytes.from_str("hello")     // UTF-8 bytes

// アクセス
bind n    <- Bytes.len(data)              // Int: 3
bind byte <- Bytes.get(data, 0)           // Result<Int, String>: ok(70)
bind sl   <- Bytes.slice(data, 1, 3)      // Bytes: "OO"
bind cat  <- Bytes.concat(data, data)     // Bytes: "FOOFOO"

// 変換
bind s    <- Bytes.to_utf8(data)          // Result<String, String>
bind hex  <- Bytes.to_hex(data)           // String: "464f4f"

// バイナリ I/O（native のみ）
bind raw  <- Bytes.read_file("data.bin")  // Result<Bytes, String>
Bytes.write_file("out.bin", raw)          // Result<(), String>

// VM デコード用（Big-endian）
bind u16  <- Bytes.read_u16(data, 0)     // Result<Int, String>
bind u24  <- Bytes.read_u24(data, 0)     // Result<Int, String>
bind u32  <- Bytes.read_u32(data, 0)     // Result<Int, String>
```

### 関数一覧

| 関数 | シグネチャ | 説明 |
|---|---|---|
| `Bytes.from_hex` | `String -> Result<Bytes, String>` | 16進文字列からバイト列生成 |
| `Bytes.from_str` | `String -> Bytes` | UTF-8 文字列からバイト列生成 |
| `Bytes.len` | `Bytes -> Int` | バイト長 |
| `Bytes.get` | `Bytes, Int -> Result<Int, String>` | 指定インデックスのバイト値（0〜255）|
| `Bytes.slice` | `Bytes, Int, Int -> Bytes` | バイト列のスライス（範囲外はクランプ）|
| `Bytes.concat` | `Bytes, Bytes -> Bytes` | 結合 |
| `Bytes.to_utf8` | `Bytes -> Result<String, String>` | UTF-8 デコード |
| `Bytes.to_hex` | `Bytes -> String` | 16進文字列に変換（小文字）|
| `Bytes.read_file` | `String -> Result<Bytes, String>` | バイナリファイル読み込み（native）|
| `Bytes.write_file` | `String, Bytes -> Result<(), String>` | バイナリファイル書き込み（native）|
| `Bytes.read_u16` | `Bytes, Int -> Result<Int, String>` | Big-endian u16 読み取り |
| `Bytes.read_u24` | `Bytes, Int -> Result<Int, String>` | Big-endian u24 読み取り |
| `Bytes.read_u32` | `Bytes, Int -> Result<Int, String>` | Big-endian u32 読み取り |

---

## アーキテクチャ

### NaN-boxing 準拠の opaque handle 方式

`ArrowBatch` / `DbHandle` / `PgPool` と同じパターンを使用する。

> **注意**: ロードマップでは `VMValue::Bytes(Arc<Vec<u8>>)` と記載されているが、
> NaN-boxing アーキテクチャでは全ヒープ値を `HeapVal` 経由で格納するため、
> 実装は `VMValue::Bytes(u64)` opaque handle とする。

```
VMValue::Bytes(u64)  ←→  BYTES_STORE: HashMap<u64, Arc<Vec<u8>>>
```

### 変更ファイル一覧

| ファイル | 変更種別 | 内容 |
|---|---|---|
| `fav/src/backend/heap_val.rs` | 更新 | `HeapVal::Bytes(u64)` バリアント追加 + PartialEq |
| `fav/src/backend/nan_val.rs` | 更新 | VMValue::Bytes ↔ NanVal 変換を追加 |
| `fav/src/backend/vm.rs` | 更新 | `VMValue::Bytes(u64)` + thread-local BYTES_STORE + `vm_call_builtin` Bytes ハンドラ + `is_known_builtin_namespace` 更新 |
| `fav/src/middle/checker.rs` | 更新 | `"Bytes"` を namespace リストに追加 |
| `fav/src/middle/compiler.rs` | 更新 | `"Bytes.*"` を builtins リストに追加（13 関数） |
| `fav/src/driver.rs` | 更新 | `v230000_tests::version_is_23_0_0` に `#[ignore]` 追加、`v231000_tests` 5 件 |
| `fav/Cargo.toml` | 更新 | `version = "23.0.0"` → `"23.1.0"`（新規依存なし）|
| `CHANGELOG.md` | 更新 | v23.1.0 エントリ追加 |
| `benchmarks/v23.1.0.json` | 新規 | ベンチマーク記録 |
| `site/content/docs/runes/bytes.mdx` | 新規 | Bytes 型ドキュメント |

### 依存クレート

**新規追加なし**。`Arc` は `std::sync::Arc`、ファイル I/O は `std::fs` を使用。

---

## 実装詳細

### 1. `heap_val.rs` — `HeapVal::Bytes(u64)` 追加

```rust
/// v23.1.0: バイト列 opaque handle
Bytes(u64),
```

PartialEq の `_` アームで自動的にカバーされるが、明示的に追加:

```rust
(HeapVal::Bytes(a), HeapVal::Bytes(b)) => a == b,
```

### 2. `nan_val.rs` — VMValue::Bytes ↔ NanVal 変換

`From<VMValue>` の `VMValue::PgPool(id)` の直後に追加:

```rust
VMValue::Bytes(id) => NanVal::from_heap(HeapVal::Bytes(id)),
```

`to_vmvalue()` の `HeapVal::PgPool(id)` の直後に追加:

```rust
HeapVal::Bytes(id) => VMValue::Bytes(*id),
```

### 3. `vm.rs` — 主要変更

#### VMValue 列挙型

`PgPool(u64)` の直後に追加:

```rust
/// v23.1.0: 生バイト列 opaque handle
Bytes(u64),
```

PartialEq / Display / type_name_of() にも追加（既存パターンに倣う）。

#### thread-local ストア

```rust
// ── v23.1.0: Bytes スレッドローカルストア ───────────────────────────────────
thread_local! {
    static BYTES_STORE: std::cell::RefCell<
        std::collections::HashMap<u64, std::sync::Arc<Vec<u8>>>
    > = std::cell::RefCell::new(std::collections::HashMap::new());
    static NEXT_BYTES_ID: std::cell::Cell<u64> = const { std::cell::Cell::new(0) };
}

fn bytes_new(data: Vec<u8>) -> u64 {
    NEXT_BYTES_ID.with(|c| {
        let id = c.get();
        c.set(id + 1);
        BYTES_STORE.with(|m| m.borrow_mut().insert(id, std::sync::Arc::new(data)));
        id
    })
}

fn bytes_get(id: u64) -> Option<std::sync::Arc<Vec<u8>>> {
    BYTES_STORE.with(|m| m.borrow().get(&id).cloned())
}
```

#### `vm_call_builtin` の `"Bytes"` ハンドラ

```rust
"Bytes.from_hex" => { /* hex decode */ }
"Bytes.from_str" => { /* UTF-8 encode */ }
"Bytes.len"      => { /* length */ }
"Bytes.get"      => { /* bounds-checked index */ }
"Bytes.slice"    => { /* slice(start, end) */ }
"Bytes.concat"   => { /* concat */ }
"Bytes.to_utf8"  => { /* UTF-8 decode → Result<String, String> */ }
"Bytes.to_hex"   => { /* hex encode */ }
"Bytes.read_file"  => { /* std::fs::read */ }  // #[cfg(not(wasm32))] 内
"Bytes.write_file" => { /* std::fs::write */ } // #[cfg(not(wasm32))] 内
"Bytes.read_u16" => { /* big-endian u16 */ }
"Bytes.read_u24" => { /* big-endian u24 */ }
"Bytes.read_u32" => { /* big-endian u32 */ }
```

#### `is_known_builtin_namespace` に `"Bytes"` を追加

`"State"` の直後に追加:

```rust
| "Bytes"   // v23.1.0
```

### 4. `checker.rs` — namespace リスト

`"Arena"` の直後に追加:

```rust
"Bytes",
```

### 5. `compiler.rs` — builtins リスト

`"Arena.stats"` の近くに追加（13 エントリ）:

```rust
// v23.1.0 Bytes 型
"Bytes.from_hex", "Bytes.from_str",
"Bytes.len", "Bytes.get", "Bytes.slice", "Bytes.concat",
"Bytes.to_utf8", "Bytes.to_hex",
"Bytes.read_file", "Bytes.write_file",
"Bytes.read_u16", "Bytes.read_u24", "Bytes.read_u32",
```

---

## テスト一覧（v231000_tests、5 件）

| テスト名 | 内容 |
|---|---|
| `version_is_23_1_0` | Cargo.toml に `version = "23.1.0"` が含まれる |
| `bytes_from_hex_to_hex_roundtrip` | `from_hex("414243")` → `to_hex` → `"414243"` |
| `bytes_get_correct_byte` | `from_hex("ff00")` の `get(0)` → `255` |
| `bytes_concat_increases_length` | 2 bytes + 3 bytes → len 5 |
| `changelog_has_v23_1_0` | CHANGELOG.md に `[v23.1.0]` が含まれる |

---

## スコープ外（v23.1.0 では実装しない）

- ビット演算（`Int.bit_and` 等）→ v23.2.0
- `Mut<T>` 可変コレクション → v23.3.0
- Favnir 側での `Bytes` 型アノテーション（型チェッカーが `Bytes` 型を認識する） → 現行は `Named("Bytes", [])` として扱う
- WASM での `Bytes.read_file` / `Bytes.write_file`（cfg guard で除外）
- `Bytes.from_base64` / `Bytes.to_base64` → 将来バージョン

---

## 完了条件

| 確認項目 | 状態 |
|---|---|
| `HeapVal::Bytes(u64)` が `heap_val.rs` に追加される | [ ] |
| `nan_val.rs` の VMValue ↔ NanVal 変換に Bytes が追加される | [ ] |
| `VMValue::Bytes(u64)` と `BYTES_STORE` / `NEXT_BYTES_ID` が `vm.rs` に追加される | [ ] |
| `vm_call_builtin` に `"Bytes.*"` の 13 ハンドラが追加される | [ ] |
| `is_known_builtin_namespace` に `"Bytes"` が追加される | [ ] |
| `checker.rs` の namespace リストに `"Bytes"` が追加される | [ ] |
| `compiler.rs` の builtins リストに `"Bytes.*"` 13 エントリが追加される | [ ] |
| `Bytes.read_file` / `Bytes.write_file` に `#[cfg(not(target_arch = "wasm32"))]` が付く | [ ] |
| `cargo test v231000 --bin fav` — 5/5 PASS | [ ] |
| `cargo test --bin fav` — リグレッションなし（1886 件以上合格） | [ ] |
| `CHANGELOG.md` に v23.1.0 エントリ | [ ] |
| `benchmarks/v23.1.0.json` 作成済み | [ ] |
| `site/content/docs/runes/bytes.mdx` 作成済み | [ ] |
