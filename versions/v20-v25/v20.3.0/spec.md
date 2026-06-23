# v20.3.0 Spec — NaN-boxing（VMValue の圧縮）

## 概要

v20.3.0 は VM スタックの値表現を **NaN-boxing** に切り替え、
`Vec<VMValue>` の要素サイズを 32〜40 bytes から **8 bytes** に圧縮する。

現在の `VMValue` は Rust の tagged union（enum）であり、最大バリアント
（`Str(String)` = 24 bytes）に合わせて全バリアントがパディングされる。
タイトループや大量レコード変換では `Vec<VMValue>` のキャッシュミスが支配的なコストになっている。

NaN-boxing は IEEE 754 f64 の "NaN 空間"（`exponent = all-1s && mantissa != 0`）を
型タグとして利用し、すべての VM 値を 8 bytes に収める技法である。

**テーマ**: Runtime Excellence シリーズ第3弾 — アーキテクチャレベルの値表現刷新

---

## 動機と期待効果

| ベンチマーク | v20.0.0 基準 | 期待改善 |
|---|---|---|
| `tight_loop_10m_iter_ms` | ~85ms（CI 推定）| **+2〜3x**（スタックキャッシュヒット率改善） |
| `record_transform_1m_ms` | ~210ms（CI 推定）| **+1.5〜2x** |
| `cold_start_precompiled_ms` | ~18ms | **< 10ms**（スタック初期化コスト削減） |

---

## 設計アーキテクチャ

### IEEE 754 NaN 空間

f64 の bit レイアウト:
```
[63]    [62:52]      [51:0]
sign    exponent     mantissa
 1 bit  11 bits      52 bits
```

- 通常の f64: exponent が all-1s でない、または all-1s かつ mantissa = 0（±Inf）
- NaN: exponent = all-1s（0x7FF）**かつ** mantissa ≠ 0
- Quiet NaN: bit 51 = 1（実行を継続するNaN）
- タグとして利用できる quiet NaN の範囲:
  - 正の quiet NaN: `0x7FF8_0000_0000_0000` 〜 `0x7FFF_FFFF_FFFF_FFFF`（upper 16 bits: 0x7FF8〜0x7FFF）
  - 負の quiet NaN: `0xFFF8_0000_0000_0000` 〜 `0xFFFF_FFFF_FFFF_FFFF`（upper 16 bits: 0xFFF8〜0xFFFF）

### NanVal エンコーディング（8 bytes = u64）

| パターン（upper 16 bits） | 型 | ペイロード（lower 48 bits） |
|---|---|---|
| 通常の f64（非 NaN） | Float | f64 bit pattern そのまま |
| `0x7FF8_0000_0000_0000` | Float NaN | − |
| `0x7FF9_xxxx_xxxx_xxxx` | Int | 48-bit 符号付き整数（`i48`、±140 兆範囲） |
| `0x7FFA_0000_0000_000b` | Bool | b=0 → false、b=1 → true |
| `0x7FFB_0000_0000_0000` | Unit | − |
| `0x7FFC_pppp_pppp_pppp` | Str | lower 48 bits = `Arc<String>` raw ptr |
| `0x7FFD_pppp_pppp_pppp` | List | lower 48 bits = `Arc<FavList>` raw ptr |
| `0x7FFE_pppp_pppp_pppp` | Record | lower 48 bits = `Arc<RecordMap>` raw ptr |
| `0x7FFF_pppp_pppp_pppp` | Misc heap | lower 48 bits = `Arc<HeapVal>` raw ptr |

> **注意**: x86-64 Linux/Windows の user-space アドレスは 47 bits 以内（上位 17 bits = 0）なので、
> lower 48 bits にポインタを格納できる。ARM64 (macOS) も同様に 48-bit VA。

### タグ定数

```rust
pub const TAG_FLOAT_NAN: u64 = 0x7FF8_0000_0000_0000;
pub const TAG_INT:        u64 = 0x7FF9_0000_0000_0000;
pub const TAG_BOOL:       u64 = 0x7FFA_0000_0000_0000;
pub const TAG_UNIT:       u64 = 0x7FFB_0000_0000_0000;
pub const TAG_STR:        u64 = 0x7FFC_0000_0000_0000;
pub const TAG_LIST:       u64 = 0x7FFD_0000_0000_0000;
pub const TAG_RECORD:     u64 = 0x7FFE_0000_0000_0000;
pub const TAG_HEAP:       u64 = 0x7FFF_0000_0000_0000;
pub const TAG_MASK:       u64 = 0xFFFF_0000_0000_0000;
pub const PTR_MASK:       u64 = 0x0000_FFFF_FFFF_FFFF;
pub const INT_SIGN_BIT:   u64 = 0x0000_8000_0000_0000; // bit 47 (i48 符号ビット)
// i48 → i64 符号拡張: bit 47 が 1 のとき上位 16 bits を 1 で埋める = (raw | TAG_MASK)
```

### HeapVal enum（TAG_HEAP が指すヒープオブジェクト）

```rust
pub enum HeapVal {
    Variant(String, Option<NanVal>),
    VariantCtor(String),
    CompiledFn(usize),
    Closure(usize, Vec<NanVal>),
    Builtin(String),
    Stream(Box<VMStream>),
    DbHandle(u64),
    TxHandle(u64),
    ArrowBatch(u64),
}
```

### type alias

```rust
pub type RecordMap = HashMap<String, NanVal>;
```

---

## NanVal 型定義と API

```rust
// src/backend/nan_val.rs

/// 8-byte NaN-boxed VM value.
/// Clone/Drop はヒープ型の Arc refcount を管理する。
/// 安全性保証: ptr_to_arc() は TAG_*_ptr で生成した値にのみ呼ぶこと。
///
/// NOTE: NanVal は Drop を手動実装するため Copy は不可（Rust コンパイラが禁止）。
///       Clone のみ実装する。
pub struct NanVal(u64);

// ── コンストラクタ ──────────────────────────────────────────────────
impl NanVal {
    pub fn from_float(f: f64) -> Self;          // NaN は TAG_FLOAT_NAN に変換
    pub fn from_int(n: i64) -> Self;            // i48 に収まらない場合は TAG_HEAP に退避
    pub fn from_bool(b: bool) -> Self;
    pub fn unit() -> Self;
    pub fn from_str(s: String) -> Self;         // Arc::new(s) してポインタ格納
    pub fn from_list(l: FavList) -> Self;       // Arc::new(l)
    pub fn from_record(r: RecordMap) -> Self;   // Arc::new(r)
    pub fn from_heap(h: HeapVal) -> Self;       // Arc::new(h)
}

// ── 型チェック ─────────────────────────────────────────────────────
impl NanVal {
    /// Float 判定: upper 16 bits が [0x7FF8, 0x7FFF] のタグ範囲外 OR TAG_FLOAT_NAN と一致。
    /// 負の f64（upper 16 bits > 0x7FFF）も正しく Float と判定する。
    ///   is_float = (upper16 < 0x7FF8) || (upper16 > 0x7FFF) || (self.0 == TAG_FLOAT_NAN)
    pub fn is_float(&self) -> bool;
    pub fn is_int(&self) -> bool;
    pub fn is_bool(&self) -> bool;
    pub fn is_unit(&self) -> bool;
    pub fn is_str(&self) -> bool;
    pub fn is_list(&self) -> bool;
    pub fn is_record(&self) -> bool;
    pub fn is_heap(&self) -> bool;
    pub fn tag(&self) -> u64;   // upper 16 bits
}

// ── デコーダ ──────────────────────────────────────────────────────
impl NanVal {
    pub fn as_float(&self) -> Option<f64>;
    pub fn as_int(&self) -> Option<i64>;
    pub fn as_bool(&self) -> Option<bool>;
    pub fn is_unit_val(&self) -> bool;
    pub fn as_str(&self) -> Option<&str>;           // 生存期間はself依存
    pub fn as_str_arc(&self) -> Option<Arc<String>>;
    pub fn as_list(&self) -> Option<&FavList>;
    pub fn as_list_arc(&self) -> Option<Arc<FavList>>;
    pub fn as_record(&self) -> Option<&RecordMap>;
    pub fn as_record_arc(&self) -> Option<Arc<RecordMap>>;
    pub fn as_heap(&self) -> Option<&HeapVal>;
    pub fn as_heap_arc(&self) -> Option<Arc<HeapVal>>;
}

// ── Clone / Drop（手動実装）────────────────────────────────────────
// ヒープポインタ型（TAG_STR/LIST/RECORD/HEAP）は Arc::clone で refcount++
// Drop 時は Arc::from_raw でポインタを Arc に戻して参照カウントを decrement

// ── VMValue との変換（レガシーブリッジ）──────────────────────────────
impl NanVal {
    pub fn to_vmvalue(self) -> VMValue;
    pub fn from_vmvalue(v: VMValue) -> Self;
}
```

### Int オーバーフロー処理

`i48` 範囲（±140,737,488,355,327）を超える `i64` は `TAG_HEAP` + `HeapVal::BigInt(i64)` として格納する。
実際の使用ではほぼ発生しないが、型システム上の正確性を保証するために必要。

---

## vm.rs の変更

### スタック型変更

```rust
// 変更前
stack: Vec<VMValue>

// 変更後
stack: Vec<NanVal>
```

### 変更が必要な主要関数

| 関数 | 変更内容 |
|---|---|
| `resume` loop（全 opcode ハンドラ） | `VMValue` → `NanVal` decode/encode |
| `apply_numeric_binop` | `VMValue::Int/Float` → `NanVal::as_int/as_float` |
| `compare_pair` | 同上 |
| `vmvalue_type_name` → `nanval_type_name` | NanVal の型名取得 |
| `constant_to_value` → `constant_to_nan` | `Constant` → `NanVal` |
| `vm_to_external_value` | `NanVal` → `Value`（外部公開型） |

### opcode ハンドラの変更パターン

```rust
// 変更前: VMValue::Int に直接マッチ
match (va, vb) {
    (VMValue::Int(a), VMValue::Int(b)) => VMValue::Int(a + b),
    (VMValue::Float(a), VMValue::Float(b)) => VMValue::Float(a + b),
    _ => return Err(...)
}

// 変更後: NanVal decode 関数経由
match (va.as_int(), vb.as_int()) {
    (Some(a), Some(b)) => NanVal::from_int(a + b),
    _ => match (va.as_float(), vb.as_float()) {
        (Some(a), Some(b)) => NanVal::from_float(a + b),
        _ => return Err(...)
    }
}
```

---

## `--legacy-value-repr` フラグ

```bash
fav run --legacy-value-repr src.fav   # 旧 VMValue enum を使用（フォールバック）
```

- コンパイルフラグ `#[cfg(feature = "legacy_value")]` または実行時フラグ
- 実装: driver.rs の `RunOptions` に `legacy_value_repr: bool` を追加
- VM 初期化時にフラグを確認し、`NanVal` ではなく `VMValue` の旧パスを使用

> `--legacy-value-repr` フラグは deprecation 予告なしで v21.0 以降に削除可能。
> パフォーマンスデバッグ・移行検証用途のみ。

---

## 変更ファイル

| ファイル | 変更内容 |
|---|---|
| `fav/src/backend/nan_val.rs` | **新規作成** — NanVal 型、タグ定数、encode/decode、Clone/Drop |
| `fav/src/backend/heap_val.rs` | **新規作成** — HeapVal enum |
| `fav/src/backend/vm.rs` | `Vec<VMValue>` → `Vec<NanVal>`、全 opcode ハンドラ更新 |
| `fav/src/backend/mod.rs` | `mod nan_val; mod heap_val;` 追加 |
| `fav/src/lib.rs` | 変更なし（vm.rs が内包） |
| `fav/src/driver.rs` | `--legacy-value-repr` フラグ追加、v203000_tests 追加 |
| `fav/Cargo.toml` | version `20.2.0` → `20.3.0` |

---

## テスト（v203000_tests、5件）

| テスト名 | 内容 |
|---|---|
| `version_is_20_3_0` | Cargo.toml に `"20.3.0"` が含まれる |
| `nan_val_size_is_8_bytes` | `size_of::<NanVal>() == 8`（NaN-boxing の根拠確認） |
| `nan_val_int_roundtrip` | `from_int(42).as_int() == Some(42)` 他 i32::MIN/MAX/負数 |
| `nan_val_float_roundtrip` | `from_float(3.14)` 誤差チェック + `from_float(-3.14)` + NaN |
| `nan_val_bool_roundtrip` | true/false/unit roundtrip |

---

## 完了条件

- [ ] `NanVal` 型が 8 bytes（`std::mem::size_of::<NanVal>() == 8`）
- [ ] Int/Float/Bool/Unit のエンコード・デコードが正確
- [ ] ヒープ型（Str/List/Record/HeapVal）の参照カウントが正しく動作
- [ ] `cargo test` — リグレッションなし（全既存テストが PASS）
- [ ] `cargo test v203000` — 5/5 PASS
- [ ] `--legacy-value-repr` フラグで旧動作にフォールバック可能
- [ ] `benchmarks/v20.3.0.json` が生成されている
- [ ] `tight_loop_10m_iter_ms` が v20.2.0 比 +50% 以上改善

---

## 技術ノート

### unsafe の封じ込め

`Arc::into_raw` / `Arc::from_raw` を使う unsafe ブロックは `nan_val.rs` 内に局所化する。
外部から見えるインターフェース（`from_str`, `as_str` など）はすべて safe。

### Clone / Drop の実装

```rust
impl Clone for NanVal {
    fn clone(&self) -> Self {
        let tag = self.0 & TAG_MASK;
        if matches!(tag, TAG_STR | TAG_LIST | TAG_RECORD | TAG_HEAP) {
            // Arc の refcount をインクリメント
            let ptr = (self.0 & PTR_MASK) as *const ();
            // tag ごとに適切な Arc 型で from_raw → clone → into_raw
            // 詳細は nan_val.rs の実装を参照
        }
        NanVal(self.0)  // u64 をそのままコピー（ポインタの bitwise copy）
    }
}

impl Drop for NanVal {
    fn drop(&mut self) {
        let tag = self.0 & TAG_MASK;
        if matches!(tag, TAG_STR | TAG_LIST | TAG_RECORD | TAG_HEAP) {
            // Arc::from_raw でポインタを Arc に戻し、refcount をデクリメント
        }
    }
}
```

### FavList の現在型と移行スコープ

`FavList` は vm.rs 内の型（内部に `Vec<VMValue>` を保持）。
v20.3.0 では **FavList 内部の `Vec<VMValue>` は変更しない**（将来の最適化として残す）。
`Arc<FavList>` でラップして NanVal にポインタ格納するが、
リスト **要素** は依然 `VMValue` のまま（キャッシュ改善の対象外）。
同様に `VMStream` 内の `VMValue` フィールドも v20.3.0 では変更しない。
`VMStream` は `HeapVal::Stream(Box<VMStream>)` として格納し、
内部の `seed: VMValue`・`next_fn: VMValue` 等はそのまま残す。

### BigInt fallback（i48 オーバーフロー）

```rust
// HeapVal に追加
BigInt(i64),  // i48 範囲を超えた Int 値の退避先
```

`NanVal::from_int(n)` 内で `n` が i48 範囲外なら `HeapVal::BigInt(n)` を使う。
`as_int()` はどちらのケースも `Some(i64)` を返す。

### ロードマップとのエンコーディング差異

ロードマップ（roadmap-v20.1-v21.0.md 行 155〜158）は概念図として
`0xFFF0_...` 系（負の NaN 空間）のタグを示しているが、
本 spec の正式エンコーディングは正の quiet NaN 空間（`0x7FF9_...〜0x7FFF_...`）を使う。
実装は本 spec のタグ定数が正式仕様であり、ロードマップの図は参考値。

### WASM ビルドへの影響

`Arc<T>` は WASM ターゲット（wasm32）でもコンパイル可能（`Atomics` 不要）。
`debug_assert!(ptr & TAG_MASK == 0)` は WASM32（32-bit VA）では常に true となり問題なし。
WASM64 は Favnir の対応外のため考慮不要。
WASM 向けに追加の `#[cfg]` 対応は不要。

### TCO との互換性

`try_apply_tco` は `Opcode::Call` の直後の `Opcode::Return` を検出する。
スタック型が変わっても TCO のロジックは opcode レベルで動作するため変更不要。
