// ── v20.7.0: Arena アロケータ — Vec プール + bumpalo Bump ────────────────────
//
// ChunkArena は `__streaming_pipeline` の chunk ごとの Vec<VMValue> alloc/free を
// pool 再利用に置き換える。bumpalo::Bump はチャンク境界マーカー + 将来の
// 文字列インターン基盤として導入（v20.7 では VMValue への直接割り当ては行わない）。

use crate::backend::vm::VMValue;

// ── ArenaStats ────────────────────────────────────────────────────────────────

/// Arena アロケーション統計。`Arena.stats()` primitive で観測できる。
#[derive(Debug, Default, Clone)]
pub struct ArenaStats {
    /// pool から Vec を取得した回数（pool hit）
    pub acquire_count: usize,
    /// 新規 malloc が必要だった回数（pool miss）
    pub alloc_count: usize,
    /// chunk 境界でリセットした回数（`end_chunk` 呼び出し数）
    pub reset_count: usize,
    /// Vec の最大 capacity（要素数）
    pub peak_capacity: usize,
}

// ── ChunkArena ────────────────────────────────────────────────────────────────

/// ストリーミングパイプライン向け Vec プール + bumpalo Bump アロケータ。
///
/// `FAV_ARENA_ENABLED=0` 環境変数で無効化可能（デバッグ用）。
/// テストでは `ChunkArena::new_with_enabled(bool)` を使う（`set_var` 不要）。
pub struct ChunkArena {
    /// チャンク境界マーカー + 将来の文字列インターン基盤
    bump: bumpalo::Bump,
    /// 再利用可能な Vec<VMValue> プール
    pool: Vec<Vec<VMValue>>,
    /// アロケーション統計
    stats: ArenaStats,
    /// arena 有効フラグ
    enabled: bool,
}

// bumpalo::Bump は Debug/Clone を実装しないため手動実装する
impl std::fmt::Debug for ChunkArena {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ChunkArena")
            .field("pool_size", &self.pool.len())
            .field("stats", &self.stats)
            .field("enabled", &self.enabled)
            .finish()
    }
}

impl Clone for ChunkArena {
    fn clone(&self) -> Self {
        ChunkArena {
            bump: bumpalo::Bump::new(),
            pool: self.pool.clone(),
            stats: self.stats.clone(),
            enabled: self.enabled,
        }
    }
}

impl ChunkArena {
    /// `FAV_ARENA_ENABLED` 環境変数を読み込んで ChunkArena を生成する。
    /// 環境変数が `"0"` の場合は arena が無効になる（pool を使わず通常 alloc）。
    pub fn new() -> Self {
        let enabled = std::env::var("FAV_ARENA_ENABLED")
            .map(|v| v != "0")
            .unwrap_or(true);
        Self::new_with_enabled(enabled)
    }

    /// テスト用コンストラクタ（環境変数不要）。
    pub fn new_with_enabled(enabled: bool) -> Self {
        ChunkArena {
            bump: bumpalo::Bump::new(),
            pool: Vec::new(),
            stats: ArenaStats::default(),
            enabled,
        }
    }

    /// pool から Vec を取得する（pool hit）か、新規 Vec を生成する（pool miss）。
    ///
    /// `enabled=true` かつ pool に Vec がある場合は pool.pop() して clear() して返す。
    /// それ以外は `Vec::with_capacity(capacity)` を生成して返す。
    pub(crate) fn acquire(&mut self, capacity: usize) -> Vec<VMValue> {
        if self.enabled {
            if let Some(mut buf) = self.pool.pop() {
                buf.clear();
                self.stats.acquire_count += 1;
                return buf;
            }
        }
        self.stats.alloc_count += 1;
        Vec::with_capacity(capacity)
    }

    /// Vec を pool に返却する（`enabled=true` 時）。
    ///
    /// `peak_capacity` を更新し、`enabled=true` なら clear して pool に push する。
    /// `reset_count` をインクリメントする。
    pub(crate) fn release(&mut self, buf: Vec<VMValue>) {
        if buf.capacity() > self.stats.peak_capacity {
            self.stats.peak_capacity = buf.capacity();
        }
        self.stats.reset_count += 1;
        if self.enabled {
            let mut buf = buf;
            buf.clear();
            self.pool.push(buf);
        }
    }

    /// chunk 処理開始マーカー（将来の文字列インターン用、現在は no-op）。
    pub fn start_chunk(&mut self) {
        // no-op: 将来の文字列インターン用マーカー
    }

    /// chunk 処理完了: result_val を out に追加し、backing Vec を pool に返却し、bump をリセットする。
    ///
    /// `VMValue::List(fl)` の場合:
    /// - `Arc::try_unwrap` で独占所有権を取れた場合 → `drain` して out に追加し、
    ///   空になった Vec を pool に返却（pool 再利用 = malloc なし）。
    /// - Arc が共有されている場合 → clone して out に追加（pool 返却なし）。
    ///
    /// その他の値は `out.push(result_val)` する。
    /// 最後に `self.bump.reset()` でバンプアロケータをリセットし、
    /// `self.stats.reset_count` をインクリメントする。
    pub(crate) fn end_chunk(&mut self, result_val: VMValue, out: &mut Vec<VMValue>) {
        match result_val {
            VMValue::List(fl) => {
                let offset = fl.1;
                // Arc の独占所有権を取れれば Vec を drain して pool に返却する。
                // stage 関数が Arc を clone していなければ try_unwrap は成功する。
                match std::sync::Arc::try_unwrap(fl.0) {
                    Ok(mut v) => {
                        // 独占所有 — drain で値を out に移し、空の Vec を pool へ
                        out.extend(v.drain(offset..));
                        if v.capacity() > self.stats.peak_capacity {
                            self.stats.peak_capacity = v.capacity();
                        }
                        if self.enabled {
                            self.pool.push(v);
                        }
                    }
                    Err(arc) => {
                        // Arc が共有されている — clone して out に追加（pool 返却なし）
                        out.extend(arc[offset..].iter().cloned());
                    }
                }
            }
            other => out.push(other),
        }
        self.bump.reset();
        self.stats.reset_count += 1;
    }

    /// bumpalo Bump アロケータをリセットする（chunk ループ後に呼ぶ）。
    pub fn reset_bump(&mut self) {
        self.bump.reset();
    }

    /// 現在の統計を返す。
    pub fn stats(&self) -> &ArenaStats {
        &self.stats
    }
}
