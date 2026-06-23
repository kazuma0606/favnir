// ── v20.8.0: DB コネクションプール ─────────────────────────────────────────
//
// PgPoolInner: tokio_postgres::Client の Vec プール（Mutex 保護）
// 統計カウンターは AtomicUsize × 5 で管理（Mutex のネストによるデッドロック回避）
//
// 接続の background task は pg_pool_runtime()（専用長寿命 tokio runtime）で
// spawn するため、block_on 終了後も接続が維持される。

use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicUsize, Ordering};
use tokio_postgres::NoTls;

// ── PgPoolStats ────────────────────────────────────────────────────────────

#[derive(Debug, Default, Clone)]
pub struct PgPoolStats {
    pub borrow_count:  usize,  // pool hit（既存接続を再利用）
    pub miss_count:    usize,  // pool miss（新規接続が必要）
    pub return_count:  usize,  // 接続を pool に返却した回数
    pub error_count:   usize,  // 接続・クエリエラー回数
    pub idle_count:    usize,  // 現在の idle 接続数
}

// ── PgPoolInner ────────────────────────────────────────────────────────────

pub(crate) struct PgPoolInner {
    pub(crate) conn_str:  String,
    pub(crate) pool_size: usize,
    idle:          Mutex<Vec<tokio_postgres::Client>>,
    // 統計は AtomicUsize で管理（idle Mutex 保持中に取得しないためデッドロックなし）
    borrow_count:  AtomicUsize,
    miss_count:    AtomicUsize,
    return_count:  AtomicUsize,
    error_count:   AtomicUsize,
    idle_count:    AtomicUsize,
}

impl PgPoolInner {
    pub(crate) fn new(conn_str: &str, pool_size: usize) -> Arc<Self> {
        Arc::new(PgPoolInner {
            conn_str: conn_str.to_string(),
            pool_size,
            idle:         Mutex::new(Vec::new()),
            borrow_count: AtomicUsize::new(0),
            miss_count:   AtomicUsize::new(0),
            return_count: AtomicUsize::new(0),
            error_count:  AtomicUsize::new(0),
            idle_count:   AtomicUsize::new(0),
        })
    }

    /// pool から接続を取得する。idle あり → pool hit / idle なし → 新規接続。
    pub(crate) fn acquire(&self) -> Result<tokio_postgres::Client, String> {
        {
            let mut idle = self.idle.lock().unwrap_or_else(|e| e.into_inner());
            if let Some(client) = idle.pop() {
                self.borrow_count.fetch_add(1, Ordering::Relaxed);
                self.idle_count.store(idle.len(), Ordering::Relaxed);
                return Ok(client);
            }
        }
        // pool miss: 新規接続
        self.miss_count.fetch_add(1, Ordering::Relaxed);

        let conn_str = self.conn_str.clone();
        // NOTE: pg_pool_runtime は fav-pgpool 専用 tokio runtime。
        // VM のメインスレッドが別の tokio runtime 内で動いている場合でも
        // block_on のネストは起きない（異なる runtime なので安全）。
        let result = crate::backend::vm::pg_pool_runtime()
            .block_on(async move {
                let (client, connection) = tokio_postgres::connect(&conn_str, NoTls)
                    .await
                    .map_err(|e| format!("PgPool: connection failed: {e}"))?;
                // 接続 background task を長寿命 runtime でスポーン
                tokio::spawn(async move {
                    if let Err(e) = connection.await {
                        eprintln!("[fav PgPool] connection error: {e}");
                    }
                });
                Ok::<tokio_postgres::Client, String>(client)
            });

        match result {
            Ok(client) => Ok(client),
            Err(e) => {
                self.error_count.fetch_add(1, Ordering::Relaxed);
                Err(e)
            }
        }
    }

    /// 使い終わった接続を pool に返却する。pool が満杯の場合はドロップ。
    pub(crate) fn release(&self, client: tokio_postgres::Client) {
        let new_idle = {
            let mut idle = self.idle.lock().unwrap_or_else(|e| e.into_inner());
            if idle.len() < self.pool_size {
                idle.push(client);
                let n = idle.len();
                n  // idle ロックをここで解放してから stats を更新
            } else {
                drop(idle);   // idle ロックを先に解放
                drop(client); // 満杯なので接続をクローズ
                return;
            }
        };
        // idle Mutex 解放後に atomic 更新（デッドロックなし）
        self.return_count.fetch_add(1, Ordering::Relaxed);
        self.idle_count.store(new_idle, Ordering::Relaxed);
    }

    pub(crate) fn stats_snapshot(&self) -> PgPoolStats {
        PgPoolStats {
            borrow_count: self.borrow_count.load(Ordering::Relaxed),
            miss_count:   self.miss_count.load(Ordering::Relaxed),
            return_count: self.return_count.load(Ordering::Relaxed),
            error_count:  self.error_count.load(Ordering::Relaxed),
            idle_count:   self.idle_count.load(Ordering::Relaxed),
        }
    }
}
