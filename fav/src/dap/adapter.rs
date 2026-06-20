use std::sync::{Arc, Condvar, Mutex};
use super::session::DapSession;

/// VM から DAP セッションに送るフック
#[derive(Debug, Clone)]
pub enum DapHook {
    StageEnter {
        name: String,
        source: String,
        line: u32,
        locals: Vec<(String, String, String)>, // (name, type, value)
    },
    StageExit {
        name: String,
        result: String, // vmvalue_repr の結果
    },
    Output(String),
}

/// VM と DAP サーバーを繋ぐアダプター
#[derive(Clone, Debug)]
pub struct DapAdapter {
    pub session: Arc<Mutex<DapSession>>,
    pub step_mode: bool, // true = ステップモード（各 stage で停止）
    /// VM スレッドをブレークポイント停止中にブロックする Condvar。
    /// (Mutex<bool>, Condvar): bool = true のとき VM スレッドは wait する。
    /// DAP サーバーが next/continue を受信したら false にして notify する。
    pub vm_block: Arc<(Mutex<bool>, Condvar)>,
}

impl DapAdapter {
    pub fn new() -> Self {
        DapAdapter {
            session: Arc::new(Mutex::new(DapSession::new())),
            step_mode: false,
            vm_block: Arc::new((Mutex::new(false), Condvar::new())),
        }
    }

    /// VM から呼ばれるフック処理。ブレークポイントヒット時は vm_block を true にセットし、
    /// VM スレッドが `wait_if_stopped` で待機できるようにする。
    pub fn on_hook(&self, hook: DapHook) {
        let mut sess = self.session.lock().unwrap_or_else(|e| e.into_inner());
        match hook {
            DapHook::StageEnter {
                name,
                source,
                line,
                locals,
            } => {
                let is_bp = sess.is_breakpoint(&source, line);
                let reason = if is_bp { "breakpoint" } else { "step" };
                if is_bp || self.step_mode {
                    sess.stop_at(&source, line, reason, &name, locals);
                    // VM スレッドをブロックするフラグをセット
                    let (lock, _cvar) = &*self.vm_block;
                    let mut blocked = lock.lock().unwrap_or_else(|e| e.into_inner());
                    *blocked = true;
                }
            }
            // TODO(v21.2): StageExit フックを resume ループの Return 処理で発火する
            DapHook::StageExit { .. } => {}
            // TODO(v21.2): Output イベントを DAP クライアントへ送信する
            DapHook::Output(_msg) => {}
        }
    }

    /// ブレークポイント停止中は VM スレッドをここでブロックする。
    /// DAP サーバーが next/continue/stepIn を受信すると `vm_block` フラグが false になり再開する。
    pub fn wait_if_stopped(&self) {
        let (lock, cvar) = &*self.vm_block;
        let mut blocked = lock.lock().unwrap_or_else(|e| e.into_inner());
        while *blocked {
            blocked = cvar.wait(blocked).unwrap_or_else(|e| e.into_inner());
        }
    }

    /// DAP サーバーから呼ばれる: VM スレッドのブロックを解除する。
    pub fn resume_vm(&self) {
        let (lock, cvar) = &*self.vm_block;
        let mut blocked = lock.lock().unwrap_or_else(|e| e.into_inner());
        *blocked = false;
        cvar.notify_one();
    }
}
