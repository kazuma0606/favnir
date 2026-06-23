# v20.3.0 実装計画 — NaN-boxing（VMValue の圧縮）

## 実装順序

```
T1: nan_val.rs — NanVal 型 + タグ定数 + encode/decode       ← 最初（他すべてが依存）
T2: heap_val.rs — HeapVal enum                              ← T1 と並列可
T3: nan_val.rs — Clone / Drop（Arc refcount 管理）          ← T1/T2 完了後
T4: vm.rs — スタック型切り替え + 全 opcode ハンドラ更新     ← T1/T2/T3 完了後
T5: vm.rs — ヘルパー関数更新                                ← T4 と並列可
T6: driver.rs — --legacy-value-repr フラグ + v203000_tests  ← T4/T5 完了後
T7: Cargo.toml バージョン更新                               ← 任意
T8: CHANGELOG.md 更新                                       ← T6 完了後
```

**変更ファイル一覧:**
- `fav/src/backend/nan_val.rs`（T1/T3）— 新規作成
- `fav/src/backend/heap_val.rs`（T2）— 新規作成
- `fav/src/backend/mod.rs`（T1）
- `fav/src/backend/vm.rs`（T4/T5）
- `fav/src/driver.rs`（T6）
- `fav/Cargo.toml`（T7）

---

## T1: `nan_val.rs` — NanVal 型 + タグ定数 + encode/decode

`fav/src/backend/nan_val.rs` を新規作成する。

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
pub const INT_SIGN_BIT:   u64 = 0x0000_8000_0000_0000; // bit 47（i48 符号ビット）
// 符号拡張マスク: TAG_MASK（0xFFFF_0000_0000_0000）を使う。
// bit 47 が 1 のとき `raw | TAG_MASK` で上位 16 bits をすべて 1 にする。
```

### NanVal 構造体

```rust
/// 8-byte NaN-boxed VM value.
///
/// 安全性不変条件:
/// 1. ポインタ型（TAG_STR/LIST/RECORD/HEAP）の lower 48 bits は
///    Arc::into_raw() で取得した有効なポインタである。
/// 2. ポインタ型は Clone 時に refcount++ され、Drop 時に refcount-- される。
/// 3. TAG_FLOAT_NAN 以外の NaN 空間（0x7FF8...0x7FFF）は本実装の予約。
///    外部から生の u64 で NanVal を作成しないこと。
#[repr(transparent)]
pub struct NanVal(u64);
```

### コンストラクタ

```rust
impl NanVal {
    /// f64 値から NanVal を作る。NaN → TAG_FLOAT_NAN。
    #[inline]
    pub fn from_float(f: f64) -> Self {
        let bits = f.to_bits();
        // NaN (exponent all-1s AND mantissa != 0) → 正規化
        if f.is_nan() {
            NanVal(TAG_FLOAT_NAN)
        } else {
            NanVal(bits)
        }
    }

    /// i64 値から NanVal を作る。
    /// i48 範囲内なら TAG_INT にインライン格納。
    /// 範囲外なら TAG_HEAP + HeapVal::BigInt(n)。
    #[inline]
    pub fn from_int(n: i64) -> Self {
        const I48_MIN: i64 = -(1 << 47);
        const I48_MAX: i64 = (1 << 47) - 1;
        if n >= I48_MIN && n <= I48_MAX {
            // 下位 48 bits に格納（負数は 2's complement のまま）
            NanVal(TAG_INT | (n as u64 & PTR_MASK))
        } else {
            NanVal::from_heap(crate::backend::heap_val::HeapVal::BigInt(n))
        }
    }

    #[inline]
    pub fn from_bool(b: bool) -> Self {
        NanVal(TAG_BOOL | b as u64)
    }

    #[inline]
    pub fn unit() -> Self {
        NanVal(TAG_UNIT)
    }

    /// String を Arc に包み、ポインタを格納する。
    pub fn from_str(s: String) -> Self {
        let ptr = std::sync::Arc::into_raw(std::sync::Arc::new(s)) as u64;
        debug_assert!(ptr & TAG_MASK == 0, "pointer exceeds 48 bits");
        NanVal(TAG_STR | (ptr & PTR_MASK))
    }

    pub fn from_list(l: crate::backend::vm::FavList) -> Self {
        let ptr = std::sync::Arc::into_raw(std::sync::Arc::new(l)) as u64;
        debug_assert!(ptr & TAG_MASK == 0);
        NanVal(TAG_LIST | (ptr & PTR_MASK))
    }

    pub fn from_record(r: crate::backend::nan_val::RecordMap) -> Self {
        let ptr = std::sync::Arc::into_raw(std::sync::Arc::new(r)) as u64;
        debug_assert!(ptr & TAG_MASK == 0);
        NanVal(TAG_RECORD | (ptr & PTR_MASK))
    }

    pub fn from_heap(h: crate::backend::heap_val::HeapVal) -> Self {
        let ptr = std::sync::Arc::into_raw(std::sync::Arc::new(h)) as u64;
        debug_assert!(ptr & TAG_MASK == 0);
        NanVal(TAG_HEAP | (ptr & PTR_MASK))
    }
}
```

### 型チェック / デコーダ

```rust
impl NanVal {
    #[inline]
    pub fn tag(&self) -> u64 { self.0 & TAG_MASK }

    #[inline]
    pub fn is_float(&self) -> bool {
        // upper 16 bits がタグ範囲 [0x7FF8, 0x7FFF] の外にあるか、
        // または canonical NaN (TAG_FLOAT_NAN) に一致する場合は Float。
        // 負の f64 は upper 16 bits が 0x8000〜0xFFF7 などになり > 0x7FFF になるため
        // `upper16 > 0x7FFF` で正しく Float と判定される。
        let upper16 = (self.0 >> 48) as u16;
        upper16 < 0x7FF8 || upper16 > 0x7FFF || self.0 == TAG_FLOAT_NAN
    }

    #[inline]
    pub fn is_int(&self) -> bool { self.tag() == TAG_INT }
    #[inline]
    pub fn is_bool(&self) -> bool { self.tag() == TAG_BOOL }
    #[inline]
    pub fn is_unit(&self) -> bool { self.0 == TAG_UNIT }

    #[inline]
    pub fn as_float(&self) -> Option<f64> {
        if self.0 == TAG_FLOAT_NAN {
            Some(f64::NAN)
        } else if self.is_float() {
            Some(f64::from_bits(self.0))
        } else {
            None
        }
    }

    #[inline]
    pub fn as_int(&self) -> Option<i64> {
        if self.tag() == TAG_INT {
            let raw = self.0 & PTR_MASK;
            // 符号拡張: bit 47 が 1 なら上位 16 bits を 1 で埋める
            // i48 → i64 符号拡張: bit 47 が 1 ならば上位 16 bits をすべて 1 に
            let n = if raw & INT_SIGN_BIT != 0 {
                (raw | TAG_MASK) as i64  // 0xFFFF_0000_0000_0000 で上位埋め
            } else {
                raw as i64
            };
            Some(n)
        } else if self.tag() == TAG_HEAP {
            // BigInt fallback
            let arc = unsafe {
                let ptr = (self.0 & PTR_MASK) as *const crate::backend::heap_val::HeapVal;
                std::mem::ManuallyDrop::new(std::sync::Arc::from_raw(ptr))
            };
            if let crate::backend::heap_val::HeapVal::BigInt(n) = arc.as_ref() {
                Some(*n)
            } else {
                None
            }
        } else {
            None
        }
    }

    #[inline]
    pub fn as_bool(&self) -> Option<bool> {
        if self.tag() == TAG_BOOL {
            Some((self.0 & 1) == 1)
        } else {
            None
        }
    }

    pub fn as_str_arc(&self) -> Option<std::sync::Arc<String>> {
        if self.tag() == TAG_STR {
            let ptr = (self.0 & PTR_MASK) as *const String;
            // ManuallyDrop: from_raw でポインタを Arc に復元するが drop しない
            // （refcount を増やさずに参照だけ借りる）
            let arc = unsafe {
                std::mem::ManuallyDrop::new(std::sync::Arc::from_raw(ptr))
            };
            // clone() で refcount++ し、その新しい Arc を返す
            Some(std::sync::Arc::clone(&arc))
        } else {
            None
        }
    }

    pub fn as_str(&self) -> Option<&str> {
        if self.tag() == TAG_STR {
            let ptr = (self.0 & PTR_MASK) as *const String;
            // 安全性: Arc が生きている間 (&self の lifetime) は有効
            Some(unsafe { &*ptr })
        } else {
            None
        }
    }

    // as_list, as_list_arc, as_record, as_record_arc, as_heap, as_heap_arc は同パターン
}
```

### PartialEq

```rust
impl PartialEq for NanVal {
    fn eq(&self, other: &Self) -> bool {
        // インライン型: ビット比較（Float NaN は IEEE 754 準拠で NaN != NaN）
        if self.tag() < TAG_STR && other.tag() < TAG_STR {
            if self.0 == TAG_FLOAT_NAN || other.0 == TAG_FLOAT_NAN {
                return false; // NaN != NaN
            }
            return self.0 == other.0;
        }
        // ヒープ型: デコードして比較（Str/List/Record のみ等価性あり）
        match (self.tag(), other.tag()) {
            (TAG_STR, TAG_STR) => self.as_str() == other.as_str(),
            (TAG_LIST, TAG_LIST) => self.as_list() == other.as_list(),
            (TAG_RECORD, TAG_RECORD) => self.as_record() == other.as_record(),
            (TAG_HEAP, TAG_HEAP) => {
                // HeapVal::Variant などは内容で比較
                self.as_heap() == other.as_heap()
            }
            _ => false,
        }
    }
}
```

---

## T2: `heap_val.rs` — HeapVal enum

`fav/src/backend/heap_val.rs` を新規作成する。

```rust
// fav/src/backend/heap_val.rs

use crate::backend::nan_val::NanVal;
use crate::backend::vm::VMStream;

#[derive(PartialEq)]
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
    BigInt(i64),          // i48 範囲を超えた Int の退避先
}
```

> `Stream(Box<VMStream>)` は `PartialEq` に `derive` できないため、
> `HeapVal::Stream` の eq は常に `false` を返す手動実装にする。

---

## T3: `nan_val.rs` — Clone / Drop（Arc refcount 管理）

```rust
// Clone / Drop の共通パターン: ManuallyDrop を使って from_raw の drop を抑制する。
// この統一パターンにより forget の意図が明確になる。

/// ヘルパー: ポインタを Arc に変換せずに refcount++ する（ManuallyDrop パターン）
/// 安全性: ptr は TAG_T で生成した有効な Arc<T> のポインタであること。
macro_rules! arc_increment_refcount {
    ($ptr:expr, $T:ty) => {
        unsafe {
            let arc = std::mem::ManuallyDrop::new(
                std::sync::Arc::<$T>::from_raw($ptr as *const $T)
            );
            let _ = std::sync::Arc::clone(&arc); // refcount++（clone した Arc はすぐ drop）
        }
    };
}

impl Clone for NanVal {
    fn clone(&self) -> Self {
        let tag = self.tag();
        let ptr = (self.0 & PTR_MASK) as usize;
        match tag {
            TAG_STR    => arc_increment_refcount!(ptr, String),
            TAG_LIST   => arc_increment_refcount!(ptr, crate::backend::vm::FavList),
            TAG_RECORD => arc_increment_refcount!(ptr, crate::backend::nan_val::RecordMap),
            TAG_HEAP   => arc_increment_refcount!(ptr, crate::backend::heap_val::HeapVal),
            _ => { /* インライン型（Int/Float/Bool/Unit）: u64 コピーのみ */ }
        }
        NanVal(self.0) // u64 をそのままコピー
    }
}

impl Drop for NanVal {
    fn drop(&mut self) {
        let tag = self.tag();
        match tag {
            TAG_STR => {
                let ptr = (self.0 & PTR_MASK) as *const String;
                unsafe { drop(std::sync::Arc::from_raw(ptr)); } // refcount--
            }
            TAG_LIST   => { /* Arc<FavList> */ }
            TAG_RECORD => { /* Arc<RecordMap> */ }
            TAG_HEAP   => { /* Arc<HeapVal> */ }
            _ => {}
        }
    }
}
```

> **注意**: `Clone` と `Drop` は対称性が重要。`from_heap(h)` で `into_raw` した Arc は、
> `clone()` で refcount++ され、`drop()` で `from_raw` して refcount-- される。
> ダブルフリーを防ぐため、`NanVal(raw_bits)` を直接コピーする前に必ず `clone()` を使うこと。

---

## T4: `vm.rs` — スタック型切り替え + opcode ハンドラ更新

### 4-1. スタック型変更

```rust
// VM struct 内
// 変更前:
stack:          Vec<VMValue>,
globals:        Vec<VMValue>,     // グローバル変数テーブル
collect_frames: Vec<Vec<VMValue>>, // Collect opcode 用バッファ
emit_log:       Vec<VMValue>,     // !Emit エフェクト記録

// 変更後:
stack:          Vec<NanVal>,
globals:        Vec<NanVal>,
collect_frames: Vec<Vec<NanVal>>,
emit_log:       Vec<NanVal>,
```

`CallFrame` struct は `VMValue` フィールドを持たないため変更不要。
（`fn_idx: usize, ip: usize, base: usize, n_locals: usize, line: u32`）

> **VMStream の扱い**: `VMStream` 内の `seed: VMValue`, `next_fn: VMValue` 等は
> v20.3.0 では変更しない。`HeapVal::Stream(Box<VMStream>)` として NanVal に格納し、
> Stream に対する操作は内部で `VMValue` ↔ `NanVal` の変換を行う。

### 4-2. opcode ハンドラ変換パターン

以下の変換パターンを全ハンドラに適用する。

#### パターン A: 数値演算（Add/Sub/Mul/Div）

```rust
// 変更前
let vb = vm.stack.pop()...;
let va = vm.stack.pop()...;
match (va, vb) {
    (VMValue::Int(a), VMValue::Int(b)) => vm.stack.push(VMValue::Int(a + b)),
    (VMValue::Float(a), VMValue::Float(b)) => vm.stack.push(VMValue::Float(a + b)),
    ...
}

// 変更後
let vb = vm.stack.pop()...;
let va = vm.stack.pop()...;
match (va.as_int(), vb.as_int()) {
    (Some(a), Some(b)) => vm.stack.push(NanVal::from_int(a + b)),
    _ => match (va.as_float(), vb.as_float()) {
        (Some(a), Some(b)) => vm.stack.push(NanVal::from_float(a + b)),
        _ => return Err(vm.error(artifact, "add: numeric operands required")),
    }
}
```

#### パターン B: 条件分岐（JumpIfFalse）

```rust
// 変更前
if let VMValue::Bool(b) = condition { if !b { ... } }

// 変更後
if let Some(b) = condition.as_bool() { if !b { ... } }
// または
if condition == NanVal::from_bool(false) { ... }
```

#### パターン C: 文字列操作

```rust
// 変更前
if let VMValue::Str(s) = val { ... }

// 変更後
if let Some(s) = val.as_str() { ... }
```

#### パターン D: リスト操作

```rust
// 変更前
if let VMValue::List(l) = val { ... }

// 変更後（所有権が必要な場合）
if let Some(l_arc) = val.as_list_arc() { ... }
// 参照のみ必要な場合
if let Some(l) = val.as_list() { ... }
```

#### パターン E: VMValue 生成（Const opcode など）

```rust
// 変更前: constant_to_value(c) → VMValue
// 変更後: constant_to_nan(c) → NanVal
fn constant_to_nan(constant: Constant) -> NanVal {
    match constant {
        Constant::Int(n)    => NanVal::from_int(n),
        Constant::Float(f)  => NanVal::from_float(f),
        Constant::Str(s)    => NanVal::from_str(s),
        Constant::Bool(b)   => NanVal::from_bool(b),
        Constant::Unit      => NanVal::unit(),
    }
}
```

### 4-3. VMValue が残る箇所

以下は意図的に `VMValue` を残す（NaN-boxing の外側のインターフェース）:

1. `vm_to_external_value(NanVal) -> Value` — テスト・API のためのブリッジ
2. `--legacy-value-repr` パスの旧ロジック
3. `Value` enum（`src/value.rs`）— 変更なし

---

## T5: `vm.rs` — ヘルパー関数更新

### `apply_numeric_binop` の更新

```rust
// 変更前
fn apply_numeric_binop(
    va: VMValue, vb: VMValue,
    int_op: impl Fn(i64, i64) -> i64,
    float_op: impl Fn(f64, f64) -> f64,
    op_name: &str,
    ...
) -> Result<VMValue, ...>

// 変更後
fn apply_numeric_binop_nan(
    va: NanVal, vb: NanVal,
    int_op: impl Fn(i64, i64) -> i64,
    float_op: impl Fn(f64, f64) -> f64,
    op_name: &str,
    ...
) -> Result<NanVal, ...> {
    if let (Some(a), Some(b)) = (va.as_int(), vb.as_int()) {
        return Ok(NanVal::from_int(int_op(a, b)));
    }
    if let (Some(a), Some(b)) = (va.as_float(), vb.as_float()) {
        return Ok(NanVal::from_float(float_op(a, b)));
    }
    // Int × Float の混合（Int → Float に昇格）
    if let (Some(a), Some(b)) = (va.as_int().map(|n| n as f64), vb.as_float()) {
        return Ok(NanVal::from_float(float_op(a, b)));
    }
    if let (Some(a), Some(b)) = (va.as_float(), vb.as_int().map(|n| n as f64)) {
        return Ok(NanVal::from_float(float_op(a, b)));
    }
    Err(format!("{op_name}: numeric operands required"))
}
```

### `compare_pair` の更新

同パターンで `VMValue` → `NanVal` に変更。

### `nanval_type_name` 追加（旧 `vmvalue_type_name` を置き換え）

```rust
pub fn nanval_type_name(v: &NanVal) -> &'static str {
    match v.tag() {
        TAG_INT  | t if t == TAG_HEAP && v.as_int().is_some() => "Int",
        t if t < TAG_FLOAT_NAN || v.is_float() => "Float",
        TAG_BOOL => "Bool",
        TAG_UNIT => "Unit",
        TAG_STR  => "String",
        TAG_LIST => "List",
        TAG_RECORD => "Record",
        TAG_HEAP => {
            if let Some(h) = v.as_heap() {
                match h {
                    HeapVal::Variant(..)    => "Variant",
                    HeapVal::VariantCtor(_) => "VariantCtor",
                    HeapVal::CompiledFn(_)  => "CompiledFn",
                    HeapVal::Closure(..)    => "Closure",
                    HeapVal::Builtin(_)     => "Builtin",
                    HeapVal::Stream(_)      => "Stream",
                    HeapVal::DbHandle(_)    => "DbHandle",
                    HeapVal::TxHandle(_)    => "TxHandle",
                    HeapVal::ArrowBatch(_)  => "ArrowBatch",
                    HeapVal::BigInt(_)      => "Int",
                }
            } else {
                "Unknown"
            }
        }
        _ => "Unknown",
    }
}
```

---

## T6: `driver.rs` — `--legacy-value-repr` + `v203000_tests`

### `--legacy-value-repr` フラグ

`RunOptions` struct に `legacy_value_repr: bool` フィールドを追加。
`fav run --legacy-value-repr` を受け付ける。
VM 初期化時に `VM::run_legacy` ルートへ分岐する（旧 `VMValue` パス）。

> 旧 `VMValue` の `resume` ループは削除せず `#[allow(dead_code)]` + `--legacy-value-repr` でのみ使用。

### v203000_tests モジュール

```rust
// ── v203000_tests (v20.3.0) — NaN-boxing ────────────────────────────────────
#[cfg(test)]
mod v203000_tests {
    use crate::backend::nan_val::{NanVal, TAG_INT, TAG_BOOL, TAG_UNIT};

    #[test]
    fn version_is_20_3_0() {
        let cargo = include_str!("../Cargo.toml");
        assert!(cargo.contains("20.3.0"), "Cargo.toml should have version 20.3.0");
    }

    #[test]
    fn nan_val_size_is_8_bytes() {
        assert_eq!(std::mem::size_of::<NanVal>(), 8, "NanVal must be 8 bytes");
    }

    #[test]
    fn nan_val_int_roundtrip() {
        for n in [-1_i64, 0, 1, 42, i32::MAX as i64, i32::MIN as i64] {
            let v = NanVal::from_int(n);
            assert_eq!(v.as_int(), Some(n), "int roundtrip failed for {n}");
        }
    }

    #[test]
    fn nan_val_float_roundtrip() {
        let v = NanVal::from_float(3.14);
        assert!((v.as_float().unwrap() - 3.14).abs() < 1e-10);
        let nan = NanVal::from_float(f64::NAN);
        assert!(nan.as_float().unwrap().is_nan());
    }

    #[test]
    fn nan_val_bool_roundtrip() {
        assert_eq!(NanVal::from_bool(true).as_bool(), Some(true));
        assert_eq!(NanVal::from_bool(false).as_bool(), Some(false));
        assert_eq!(NanVal::unit().as_bool(), None);
    }
}
```

---

## T7/T8: Cargo.toml / CHANGELOG.md 更新

`version = "20.2.0"` → `"20.3.0"`

CHANGELOG エントリ:
```markdown
## [v20.3.0] — 2026-06-XX — NaN-boxing（VMValue の圧縮）

### Changed
- `VMValue` enum（32〜40 bytes/値）を `NanVal`（8 bytes/値）に置き換え
- Int/Bool/Float/Unit はインライン格納（ヒープ割り当て不要）
- Str/List/Record/その他ヒープ型は `Arc<T>` 経由でポインタ格納

### Added
- `fav/src/backend/nan_val.rs` — NanVal 型、タグ定数、encode/decode
- `fav/src/backend/heap_val.rs` — HeapVal enum
- `fav run --legacy-value-repr` — 旧 VMValue 表現へのフォールバック

### Performance
- `tight_loop_10m_iter`: +2〜3x（スタックキャッシュヒット率改善）
- `record_transform_1m`: +1.5〜2x
```

---

## 注意点

### 変更ファイル数と戦略

vm.rs は 16,000 行。全ての `VMValue::` マッチを機械的に変換する必要がある。
戦略:
1. まず `VMValue` の型別名を作成して `type VMValue = NanVal;` でコンパイルを通す（T4 初期）
2. 実際にデコーダを使う変換を段階的に行う（T4 後半）
3. `cargo check` で進捗確認を繰り返す

### Arc vs Box の選択

- `String`: `Arc<String>`（複数の NanVal が同じ String を参照するケースがある）
- `FavList`: `Arc<FavList>`（同上）
- `RecordMap`: `Arc<RecordMap>`（レコードのフィールドアクセスで参照が生まれる）
- `HeapVal`: `Arc<HeapVal>`（Closure capture で Vec<NanVal> を参照カウント）

全て `Arc` に統一することで Clone の実装がシンプルになる。
シングルスレッド VM では `Rc` の方が高速だが、将来の並列 VM を見越して `Arc` を使う。
（`par` stage は既に rayon でスレッドを使うため、`Rc` は安全でない）

### FavList の公開

vm.rs の `FavList` 型を `nan_val.rs` から参照するため、`pub` に変更する必要がある。
または `type alias` を別ファイルに切り出す。

### `vmvalue_type_name` の移行

`nanval_type_name` に変更。`vmvalue_type_name` を呼んでいる箇所を全て更新する。
