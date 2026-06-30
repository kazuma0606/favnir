/// NaN-boxing によるコンパクト VM 値表現（v20.3.0）
///
/// IEEE 754 の quiet NaN 空間（upper 16 bits = 0x7FF8〜0x7FFF）を型タグとして使い、
/// すべての VM 値を 8 bytes（u64）に収める。
///
/// # エンコーディング
///
/// | upper 16 bits | 型          | ペイロード（lower 48 bits）         |
/// |---------------|-------------|-------------------------------------|
/// | 0 〜 0x7FF7   | Float（正）  | f64 bit pattern そのまま            |
/// | 0x7FF8        | Float NaN   | −（正規化済み NaN）                  |
/// | 0x7FF9        | Int (i48)   | 48-bit 符号付き整数                  |
/// | 0x7FFA        | Bool        | 0=false, 1=true                     |
/// | 0x7FFB        | Unit        | −                                   |
/// | 0x7FFC        | Str         | Arc<String> raw ptr                 |
/// | 0x7FFD        | List        | Arc<FavList> raw ptr                |
/// | 0x7FFE        | Record      | Arc<RecordMap> raw ptr              |
/// | 0x7FFF        | Heap        | Arc<HeapVal> raw ptr                |
/// | 0x8000〜      | Float（負）  | f64 bit pattern そのまま            |
///
/// # 安全性不変条件
///
/// 1. ポインタ型（TAG_STR/LIST/RECORD/HEAP）の lower 48 bits は
///    `Arc::into_raw()` で取得した有効なポインタである。
/// 2. ポインタ型は `Clone` 時に refcount++ され、`Drop` 時に refcount-- される。
/// 3. `NanVal` は `Drop` を手動実装するため `Copy` は実装できない（Rust コンパイラが禁止）。
/// 4. 外部から生の u64 で `NanVal` を作成しないこと（`from_raw_bits` は unsafe のみ）。

use std::collections::HashMap;
use std::fmt;
use std::mem::ManuallyDrop;
use std::sync::Arc;

use crate::backend::heap_val::HeapVal;
use crate::backend::vm::FavList;

// ── タグ定数 ─────────────────────────────────────────────────────────────────

pub const TAG_FLOAT_NAN: u64 = 0x7FF8_0000_0000_0000;
pub const TAG_INT:       u64 = 0x7FF9_0000_0000_0000;
pub const TAG_BOOL:      u64 = 0x7FFA_0000_0000_0000;
pub const TAG_UNIT:      u64 = 0x7FFB_0000_0000_0000;
pub const TAG_STR:       u64 = 0x7FFC_0000_0000_0000;
pub const TAG_LIST:      u64 = 0x7FFD_0000_0000_0000;
pub const TAG_RECORD:    u64 = 0x7FFE_0000_0000_0000;
pub const TAG_HEAP:      u64 = 0x7FFF_0000_0000_0000;
/// upper 16 bits をすべて 1 にするマスク（TAG 抽出 / i48 符号拡張に使用）
pub const TAG_MASK:      u64 = 0xFFFF_0000_0000_0000;
/// lower 48 bits マスク（ポインタ / i48 ペイロード抽出）
pub const PTR_MASK:      u64 = 0x0000_FFFF_FFFF_FFFF;
/// i48 の符号ビット（bit 47）
pub const INT_SIGN_BIT:  u64 = 0x0000_8000_0000_0000;

/// レコード型の実体。HashMap<String, NanVal> の別名。
pub type RecordMap = HashMap<String, NanVal>;

// ── NanVal 構造体 ─────────────────────────────────────────────────────────────

/// 8-byte NaN-boxed VM 値。
///
/// `Drop` を手動実装しているため `Copy` trait は実装不可。
/// Clone は `Arc::increment_strong_count` で参照カウントを正しく管理する。
#[repr(transparent)]
pub struct NanVal(pub(crate) u64);

// ── コンストラクタ ────────────────────────────────────────────────────────────

impl NanVal {
    /// f64 から NanVal を作る。NaN はすべて TAG_FLOAT_NAN に正規化する。
    #[inline]
    pub fn from_float(f: f64) -> Self {
        if f.is_nan() {
            NanVal(TAG_FLOAT_NAN)
        } else {
            NanVal(f.to_bits())
        }
    }

    /// i64 から NanVal を作る。
    /// i48 範囲（±140,737,488,355,327）に収まる場合はインライン格納。
    /// 範囲外は `HeapVal::BigInt` に退避する。
    #[inline]
    pub fn from_int(n: i64) -> Self {
        const I48_MIN: i64 = -(1_i64 << 47);
        const I48_MAX: i64 = (1_i64 << 47) - 1;
        #[allow(clippy::manual_range_contains)]
        if n >= I48_MIN && n <= I48_MAX {
            // 下位 48 bits に 2's complement のまま格納
            NanVal(TAG_INT | (n as u64 & PTR_MASK))
        } else {
            NanVal::from_heap(HeapVal::BigInt(n))
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
    #[allow(clippy::should_implement_trait)]
    pub fn from_str(s: String) -> Self {
        let ptr = Arc::into_raw(Arc::new(s)) as u64;
        debug_assert!(ptr & TAG_MASK == 0, "pointer exceeds 48 bits");
        NanVal(TAG_STR | (ptr & PTR_MASK))
    }

    /// FavList を Arc に包み、ポインタを格納する。
    pub(crate) fn from_list(l: FavList) -> Self {
        let ptr = Arc::into_raw(Arc::new(l)) as u64;
        debug_assert!(ptr & TAG_MASK == 0, "pointer exceeds 48 bits");
        NanVal(TAG_LIST | (ptr & PTR_MASK))
    }

    /// RecordMap を Arc に包み、ポインタを格納する。
    pub fn from_record(r: RecordMap) -> Self {
        let ptr = Arc::into_raw(Arc::new(r)) as u64;
        debug_assert!(ptr & TAG_MASK == 0, "pointer exceeds 48 bits");
        NanVal(TAG_RECORD | (ptr & PTR_MASK))
    }

    /// HeapVal を Arc に包み、ポインタを格納する。
    pub(crate) fn from_heap(h: HeapVal) -> Self {
        let ptr = Arc::into_raw(Arc::new(h)) as u64;
        debug_assert!(ptr & TAG_MASK == 0, "pointer exceeds 48 bits");
        NanVal(TAG_HEAP | (ptr & PTR_MASK))
    }
}

// ── 型チェック ────────────────────────────────────────────────────────────────

impl NanVal {
    /// upper 16 bits のみを返す（タグ部）。
    #[inline]
    pub fn tag(&self) -> u64 {
        self.0 & TAG_MASK
    }

    /// Float 判定。
    ///
    /// - upper 16 bits がタグ範囲 \[0x7FF8, 0x7FFF\] の外 → 通常の f64（正・負）
    /// - `self.0 == TAG_FLOAT_NAN` → 正規化済み NaN
    ///
    /// 負の f64 は upper 16 bits が 0x8000 以上になるため `upper16 > 0x7FFF` で正しく判定される。
    #[inline]
    pub fn is_float(&self) -> bool {
        let upper16 = (self.0 >> 48) as u16;
        #[allow(clippy::manual_range_contains)]
        let in_nan_range = upper16 >= 0x7FF8 && upper16 <= 0x7FFF;
        !in_nan_range || self.0 == TAG_FLOAT_NAN
    }

    #[inline]
    pub fn is_int(&self) -> bool {
        self.0 & TAG_MASK == TAG_INT
    }

    #[inline]
    pub fn is_bool(&self) -> bool {
        self.0 & TAG_MASK == TAG_BOOL
    }

    #[inline]
    pub fn is_unit(&self) -> bool {
        self.0 == TAG_UNIT
    }

    #[inline]
    pub fn is_str(&self) -> bool {
        self.0 & TAG_MASK == TAG_STR
    }

    #[inline]
    pub fn is_list(&self) -> bool {
        self.0 & TAG_MASK == TAG_LIST
    }

    #[inline]
    pub fn is_record(&self) -> bool {
        self.0 & TAG_MASK == TAG_RECORD
    }

    #[inline]
    pub fn is_heap(&self) -> bool {
        self.0 & TAG_MASK == TAG_HEAP
    }
}

// ── デコーダ ──────────────────────────────────────────────────────────────────

impl NanVal {
    /// Float 値を取得する。TAG_FLOAT_NAN の場合は `f64::NAN` を返す。
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

    /// Int 値を取得する。BigInt fallback 込み。
    #[inline]
    pub fn as_int(&self) -> Option<i64> {
        if self.0 & TAG_MASK == TAG_INT {
            let raw = self.0 & PTR_MASK;
            // i48 → i64 符号拡張: bit 47 が 1 なら上位 16 bits をすべて 1 にする
            let n = if raw & INT_SIGN_BIT != 0 {
                (raw | TAG_MASK) as i64
            } else {
                raw as i64
            };
            Some(n)
        } else if self.0 & TAG_MASK == TAG_HEAP {
            // BigInt fallback: i48 範囲外の Int
            let ptr = (self.0 & PTR_MASK) as *const HeapVal;
            let arc = unsafe { ManuallyDrop::new(Arc::from_raw(ptr)) };
            if let HeapVal::BigInt(n) = arc.as_ref() {
                Some(*n)
            } else {
                None
            }
        } else {
            None
        }
    }

    /// Bool 値を取得する。
    #[inline]
    pub fn as_bool(&self) -> Option<bool> {
        if self.0 & TAG_MASK == TAG_BOOL {
            Some((self.0 & 1) == 1)
        } else {
            None
        }
    }

    /// 文字列への参照を返す（lifetime は self に依存）。
    pub fn as_str(&self) -> Option<&str> {
        if self.0 & TAG_MASK == TAG_STR {
            let ptr = (self.0 & PTR_MASK) as *const String;
            // 安全性: Arc が NanVal の生存期間中は解放されないため有効
            Some(unsafe { (*ptr).as_str() })
        } else {
            None
        }
    }

    /// 文字列の Arc クローンを返す（refcount++ する）。
    pub fn as_str_arc(&self) -> Option<Arc<String>> {
        if self.0 & TAG_MASK == TAG_STR {
            let ptr = (self.0 & PTR_MASK) as *const String;
            let arc = unsafe { ManuallyDrop::new(Arc::from_raw(ptr)) };
            Some(Arc::clone(&arc))
        } else {
            None
        }
    }

    /// リストへの参照を返す（lifetime は self に依存）。
    pub(crate) fn as_list(&self) -> Option<&FavList> {
        if self.0 & TAG_MASK == TAG_LIST {
            let ptr = (self.0 & PTR_MASK) as *const FavList;
            Some(unsafe { &*ptr })
        } else {
            None
        }
    }

    /// リストの Arc クローンを返す（refcount++ する）。
    pub(crate) fn as_list_arc(&self) -> Option<Arc<FavList>> {
        if self.0 & TAG_MASK == TAG_LIST {
            let ptr = (self.0 & PTR_MASK) as *const FavList;
            let arc = unsafe { ManuallyDrop::new(Arc::from_raw(ptr)) };
            Some(Arc::clone(&arc))
        } else {
            None
        }
    }

    /// レコードへの参照を返す（lifetime は self に依存）。
    pub fn as_record(&self) -> Option<&RecordMap> {
        if self.0 & TAG_MASK == TAG_RECORD {
            let ptr = (self.0 & PTR_MASK) as *const RecordMap;
            Some(unsafe { &*ptr })
        } else {
            None
        }
    }

    /// レコードの Arc クローンを返す（refcount++ する）。
    pub fn as_record_arc(&self) -> Option<Arc<RecordMap>> {
        if self.0 & TAG_MASK == TAG_RECORD {
            let ptr = (self.0 & PTR_MASK) as *const RecordMap;
            let arc = unsafe { ManuallyDrop::new(Arc::from_raw(ptr)) };
            Some(Arc::clone(&arc))
        } else {
            None
        }
    }

    /// HeapVal への参照を返す（lifetime は self に依存）。
    pub(crate) fn as_heap(&self) -> Option<&HeapVal> {
        if self.0 & TAG_MASK == TAG_HEAP {
            let ptr = (self.0 & PTR_MASK) as *const HeapVal;
            Some(unsafe { &*ptr })
        } else {
            None
        }
    }

    /// HeapVal の Arc クローンを返す（refcount++ する）。
    pub(crate) fn as_heap_arc(&self) -> Option<Arc<HeapVal>> {
        if self.0 & TAG_MASK == TAG_HEAP {
            let ptr = (self.0 & PTR_MASK) as *const HeapVal;
            let arc = unsafe { ManuallyDrop::new(Arc::from_raw(ptr)) };
            Some(Arc::clone(&arc))
        } else {
            None
        }
    }
}

// ── Clone ─────────────────────────────────────────────────────────────────────

impl Clone for NanVal {
    fn clone(&self) -> Self {
        let tag = self.0 & TAG_MASK;
        let ptr = (self.0 & PTR_MASK) as usize;
        // ポインタ型は Arc の参照カウントをインクリメントする
        unsafe {
            match tag {
                TAG_STR    => Arc::increment_strong_count(ptr as *const String),
                TAG_LIST   => Arc::increment_strong_count(ptr as *const FavList),
                TAG_RECORD => Arc::increment_strong_count(ptr as *const RecordMap),
                TAG_HEAP   => Arc::increment_strong_count(ptr as *const HeapVal),
                _ => {} // インライン型（Float/Int/Bool/Unit）はコピーのみ
            }
        }
        NanVal(self.0) // u64 を bitwise コピー
    }
}

// ── Drop ──────────────────────────────────────────────────────────────────────

impl Drop for NanVal {
    fn drop(&mut self) {
        let tag = self.0 & TAG_MASK;
        let ptr = (self.0 & PTR_MASK) as usize;
        if ptr == 0 {
            return; // null ポインタガード（通常は発生しない）
        }
        // ポインタ型は Arc の参照カウントをデクリメントする（0 になれば解放）
        unsafe {
            match tag {
                TAG_STR    => Arc::decrement_strong_count(ptr as *const String),
                TAG_LIST   => Arc::decrement_strong_count(ptr as *const FavList),
                TAG_RECORD => Arc::decrement_strong_count(ptr as *const RecordMap),
                TAG_HEAP   => Arc::decrement_strong_count(ptr as *const HeapVal),
                _ => {} // インライン型: 何もしない
            }
        }
    }
}

// ── PartialEq ─────────────────────────────────────────────────────────────────

impl PartialEq for NanVal {
    fn eq(&self, other: &Self) -> bool {
        // TAG_FLOAT_NAN は IEEE 754 準拠で NaN != NaN
        if self.0 == TAG_FLOAT_NAN || other.0 == TAG_FLOAT_NAN {
            return false;
        }
        // インライン型（Float/Int/Bool/Unit）はビット比較
        let tag = self.0 & TAG_MASK;
        let other_tag = other.0 & TAG_MASK;
        if tag < TAG_STR && other_tag < TAG_STR {
            return self.0 == other.0;
        }
        // ヒープ型: デコードして比較
        match (tag, other_tag) {
            (TAG_STR, TAG_STR)       => self.as_str() == other.as_str(),
            (TAG_LIST, TAG_LIST)     => self.as_list() == other.as_list(),
            (TAG_RECORD, TAG_RECORD) => self.as_record() == other.as_record(),
            (TAG_HEAP, TAG_HEAP)     => self.as_heap() == other.as_heap(),
            _ => false,
        }
    }
}

// ── Debug ─────────────────────────────────────────────────────────────────────

impl fmt::Debug for NanVal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.0 == TAG_FLOAT_NAN {
            return write!(f, "NanVal(Float(NaN))");
        }
        if let Some(n) = self.as_float() {
            return write!(f, "NanVal(Float({n}))");
        }
        if self.is_int() {
            if let Some(n) = self.as_int() {
                return write!(f, "NanVal(Int({n}))");
            }
        }
        if self.is_bool() {
            if let Some(b) = self.as_bool() {
                return write!(f, "NanVal(Bool({b}))");
            }
        }
        if self.is_unit() {
            return write!(f, "NanVal(Unit)");
        }
        if self.is_str() {
            if let Some(s) = self.as_str() {
                return write!(f, "NanVal(Str({s:?}))");
            }
        }
        if self.is_list() {
            return write!(f, "NanVal(List(...))");
        }
        if self.is_record() {
            return write!(f, "NanVal(Record(...))");
        }
        if self.is_heap() {
            if let Some(h) = self.as_heap() {
                return write!(f, "NanVal(Heap({h:?}))");
            }
        }
        write!(f, "NanVal(0x{:016X})", self.0)
    }
}

// ── VMValue ブリッジ（T4/T5）──────────────────────────────────────────────────

impl NanVal {
    /// VMValue（旧型システム）から NanVal に変換するブリッジ。
    /// call_value/call_builtin の境界で使用する（slow path）。
    pub(crate) fn from_vmvalue(v: super::vm::VMValue) -> Self {
        use super::heap_val::HeapVal;
        use super::vm::VMValue;
        match v {
            VMValue::Int(n)    => NanVal::from_int(n),
            VMValue::Float(f)  => NanVal::from_float(f),
            VMValue::Bool(b)   => NanVal::from_bool(b),
            VMValue::Unit      => NanVal::unit(),
            VMValue::Str(s)    => NanVal::from_str(s),
            VMValue::List(l)   => NanVal::from_list(l),
            VMValue::Record(r) => NanVal::from_record(
                r.into_iter().map(|(k, v)| (k, NanVal::from_vmvalue(v))).collect(),
            ),
            VMValue::Variant(name, payload) => NanVal::from_heap(HeapVal::Variant(
                name,
                payload.map(|b| NanVal::from_vmvalue(*b)),
            )),
            VMValue::VariantCtor(name) => NanVal::from_heap(HeapVal::VariantCtor(name)),
            VMValue::CompiledFn(idx)   => NanVal::from_heap(HeapVal::CompiledFn(idx)),
            VMValue::Closure(idx, caps) => NanVal::from_heap(HeapVal::Closure(
                idx,
                caps.into_iter().map(NanVal::from_vmvalue).collect(),
            )),
            VMValue::Builtin(name)  => NanVal::from_heap(HeapVal::Builtin(name)),
            VMValue::Stream(s)      => NanVal::from_heap(HeapVal::Stream(s)),
            VMValue::DbHandle(id)   => NanVal::from_heap(HeapVal::DbHandle(id)),
            VMValue::TxHandle(id)   => NanVal::from_heap(HeapVal::TxHandle(id)),
            VMValue::ArrowBatch(id) => NanVal::from_heap(HeapVal::ArrowBatch(id)),
            VMValue::PgPool(id)     => NanVal::from_heap(HeapVal::PgPool(id)),
            VMValue::Bytes(id)      => NanVal::from_heap(HeapVal::Bytes(id)),
            VMValue::MutList(id)    => NanVal::from_heap(HeapVal::MutList(id)),
            VMValue::MutMap(id)     => NanVal::from_heap(HeapVal::MutMap(id)),
        }
    }

    /// NanVal から VMValue に変換するブリッジ（self を消費する）。
    /// call_value/call_builtin の境界で使用する（slow path）。
    #[allow(clippy::wrong_self_convention)]
    pub(crate) fn to_vmvalue(self) -> super::vm::VMValue {
        use super::heap_val::HeapVal;
        use super::vm::VMValue;
        if let Some(f) = self.as_float() { return VMValue::Float(f); }
        if self.is_int() {
            if let Some(n) = self.as_int() { return VMValue::Int(n); }
        }
        if self.is_bool() { return VMValue::Bool(self.as_bool().unwrap()); }
        if self.is_unit() { return VMValue::Unit; }
        if self.is_str() {
            if let Some(s) = self.as_str() { return VMValue::Str(s.to_string()); }
        }
        if self.is_list() {
            if let Some(l) = self.as_list() { return VMValue::List(l.clone()); }
        }
        if self.is_record() {
            if let Some(r) = self.as_record() {
                return VMValue::Record(
                    r.iter().map(|(k, v)| (k.clone(), v.clone().to_vmvalue())).collect(),
                );
            }
        }
        if self.is_heap() {
            if let Some(h) = self.as_heap() {
                return match h {
                    HeapVal::Variant(name, payload) => VMValue::Variant(
                        name.clone(),
                        payload.as_ref().map(|v| Box::new(v.clone().to_vmvalue())),
                    ),
                    HeapVal::VariantCtor(name) => VMValue::VariantCtor(name.clone()),
                    HeapVal::CompiledFn(idx)   => VMValue::CompiledFn(*idx),
                    HeapVal::Closure(idx, caps) => VMValue::Closure(
                        *idx,
                        caps.iter().map(|v| v.clone().to_vmvalue()).collect(),
                    ),
                    HeapVal::Builtin(name)  => VMValue::Builtin(name.clone()),
                    HeapVal::Stream(s)      => VMValue::Stream(s.clone()),
                    HeapVal::DbHandle(id)   => VMValue::DbHandle(*id),
                    HeapVal::TxHandle(id)   => VMValue::TxHandle(*id),
                    HeapVal::ArrowBatch(id) => VMValue::ArrowBatch(*id),
                    HeapVal::PgPool(id)     => VMValue::PgPool(*id),
                    HeapVal::Bytes(id)      => VMValue::Bytes(*id),
                    HeapVal::MutList(id)    => VMValue::MutList(*id),
                    HeapVal::MutMap(id)     => VMValue::MutMap(*id),
                    HeapVal::BigInt(n)      => VMValue::Int(*n),
                };
            }
        }
        VMValue::Unit
    }
}

// ── 内部テスト（T3 Clone/Drop 対称性確認）────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    #[test]
    fn clone_drop_symmetry_str() {
        // NanVal::from_str が Arc::into_raw でポインタを格納することを利用して
        // Clone/Drop の対称性（refcount の増減）を確認する。
        let v1 = NanVal::from_str("hello".to_string());

        // raw ptr を取得（v1 は依然として有効なので refcount = 1）
        let ptr = (v1.0 & PTR_MASK) as *const String;

        // ManuallyDrop で Arc をピーク（refcount を変化させずに確認）
        let peeked = ManuallyDrop::new(unsafe { Arc::from_raw(ptr) });
        assert_eq!(Arc::strong_count(&peeked), 1, "initial refcount should be 1");

        // clone で refcount++
        let v2 = v1.clone();
        assert_eq!(Arc::strong_count(&peeked), 2, "after clone, refcount should be 2");

        // drop clone で refcount--
        drop(v2);
        assert_eq!(Arc::strong_count(&peeked), 1, "after drop clone, refcount should be 1");

        // drop original で refcount-- → 0（解放）
        // drop 後は peeked 経由のアクセスは UB なので確認しない
        drop(v1);
        // ここで panic や二重解放がなければ対称性確認完了
    }

    #[test]
    fn int_roundtrip_edge_cases() {
        // i48 境界値
        const I48_MIN: i64 = -(1_i64 << 47);
        const I48_MAX: i64 = (1_i64 << 47) - 1;

        let v_min = NanVal::from_int(I48_MIN);
        assert_eq!(v_min.as_int(), Some(I48_MIN));

        let v_max = NanVal::from_int(I48_MAX);
        assert_eq!(v_max.as_int(), Some(I48_MAX));

        // i48 範囲外 → HeapVal::BigInt
        let big = NanVal::from_int(I48_MAX + 1);
        assert_eq!(big.as_int(), Some(I48_MAX + 1));
        assert!(big.is_heap());
    }
}
