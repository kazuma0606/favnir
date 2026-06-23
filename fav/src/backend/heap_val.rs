/// NaN-boxing ヒープ値（v20.3.0）
///
/// `NanVal` の TAG_HEAP が指すヒープオブジェクト。
/// Arc<HeapVal> として格納され、NanVal の Clone/Drop で参照カウント管理される。

use crate::backend::nan_val::NanVal;
use crate::backend::vm::VMStream;

/// ヒープ割り当ての VM 値。
///
/// TAG_STR / TAG_LIST / TAG_RECORD で直接格納できない値はすべてここに入る。
/// `Stream` は内部が VMValue のまま（v20.3.0 スコープ外）なので
/// PartialEq の比較対象外（常に false を返す）。
#[derive(Debug)]
pub(crate) enum HeapVal {
    Variant(String, Option<NanVal>),
    VariantCtor(String),
    CompiledFn(usize),
    Closure(usize, Vec<NanVal>),
    Builtin(String),
    /// Stream の内部は VMValue のまま（v20.3.0 では変更しない）
    Stream(Box<VMStream>),
    DbHandle(u64),
    TxHandle(u64),
    ArrowBatch(u64),
    /// v20.8.0: DB コネクションプール opaque handle
    PgPool(u64),
    /// v23.1.0: 生バイト列 opaque handle
    Bytes(u64),
    /// v23.3.0: 可変リスト opaque handle
    MutList(u64),
    /// v23.3.0: 可変マップ opaque handle
    MutMap(u64),
    /// i48 範囲（±140 兆）を超えた Int 値の退避先
    BigInt(i64),
}

impl PartialEq for HeapVal {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (HeapVal::Variant(n1, p1), HeapVal::Variant(n2, p2)) => n1 == n2 && p1 == p2,
            (HeapVal::VariantCtor(a), HeapVal::VariantCtor(b)) => a == b,
            (HeapVal::CompiledFn(a), HeapVal::CompiledFn(b)) => a == b,
            (HeapVal::Closure(a, ca), HeapVal::Closure(b, cb)) => a == b && ca == cb,
            (HeapVal::Builtin(a), HeapVal::Builtin(b)) => a == b,
            // Stream は比較不可（内部の VMValue が PartialEq を持つが意味論的に不一致）
            (HeapVal::Stream(_), HeapVal::Stream(_)) => false,
            (HeapVal::DbHandle(a), HeapVal::DbHandle(b)) => a == b,
            (HeapVal::TxHandle(a), HeapVal::TxHandle(b)) => a == b,
            (HeapVal::ArrowBatch(a), HeapVal::ArrowBatch(b)) => a == b,
            (HeapVal::PgPool(a),     HeapVal::PgPool(b))     => a == b,
            (HeapVal::Bytes(a),      HeapVal::Bytes(b))      => a == b,
            (HeapVal::MutList(a),    HeapVal::MutList(b))    => a == b,
            (HeapVal::MutMap(a),     HeapVal::MutMap(b))     => a == b,
            (HeapVal::BigInt(a), HeapVal::BigInt(b)) => a == b,
            _ => false,
        }
    }
}
