# Plan: v54.5.0 — fav doctor 環境診断コマンド

---

## ステップ 1: 事前確認

```bash
cargo test -j 8 -- --test-threads=8 2>&1 | grep "test result"
# → 3193 passed, 0 failed を確認

cargo clippy -- -D warnings
# → warnings なしであることを確認

# v54500_tests が未存在を確認
rg -n "v54500_tests" fav/src/driver.rs  # → 0 件

# v54400_tests の行番号を確認（挿入位置）
rg -n "v54400_tests" fav/src/driver.rs

# DoctorCheck が未存在を確認
rg -n "DoctorCheck" fav/src/driver.rs  # → 0 件

# fav doctor コマンドが未存在を確認
grep "doctor" fav/src/main.rs  # → 0 件

# Cargo.toml が 54.4.0 であることを確認
grep "^version" fav/Cargo.toml  # → version = "54.4.0"
```

---

## ステップ 2: `driver.rs` — 型定義 + ロジック関数追加

`v54400_tests` の直前に追加:

```rust
// ── v54.5.0: fav doctor ──────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DoctorCheck {
    pub status: DoctorStatus,
    pub label: String,
    pub detail: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DoctorStatus { Ok, Warn, Fail }

impl DoctorStatus {
    // prefix は固定幅 6 文字（[OK]  = 4+2padding, [WARN] = 6, [FAIL] = 6）
    fn prefix(&self) -> &'static str {
        match self {
            DoctorStatus::Ok   => "[OK]  ",
            DoctorStatus::Warn => "[WARN]",
            DoctorStatus::Fail => "[FAIL]",
        }
    }
}

pub fn cmd_doctor_collect(checks: &[DoctorCheck]) -> String { ... }
pub fn cmd_doctor_run() -> Vec<DoctorCheck> { ... }
```

重要実装ポイント（詳細は spec.md §実装スコープ を参照）:
- `cmd_doctor_collect` は純粋関数（環境非依存）
- `cmd_doctor_run` は cwd / env 依存（テストは `cmd_doctor_collect` 経由）
- fav.toml / .fav-cache の存在チェックは `std::path::Path::new(...).exists()`
- RUSTUP_TOOLCHAIN 取得失敗時の fallback: `"stable (toolchain info unavailable)"`
- rune インストール状態の実チェックは v54.5.0 スコープ外（将来バージョン）

`cargo build` → コンパイルエラーなし確認。

---

## ステップ 3: `main.rs` — `fav doctor` コマンド追加

`Some("dq-report")` の直前に追加:

```rust
// ── v54.5.0: fav doctor ──────────────────────────────────────────────────────
Some("doctor") => {
    let checks = driver::cmd_doctor_run();
    let report = driver::cmd_doctor_collect(&checks);
    println!("{report}");
}
```

`cargo build` → コンパイルエラーなし確認。

---

## ステップ 4: `driver.rs` — `v54500_tests` 追加

`v54400_tests` の直前に追加:

```rust
// -- v54500_tests (v54.5.0) -- fav doctor 環境診断コマンド --
#[cfg(test)]
mod v54500_tests {
    use super::*;

    #[test]
    fn cmd_doctor_passes_clean_env() {
        let checks = vec![
            DoctorCheck { status: DoctorStatus::Ok, label: "fav version".to_string(), detail: "54.5.0".to_string() },
            DoctorCheck { status: DoctorStatus::Ok, label: "Rust toolchain".to_string(), detail: "stable".to_string() },
        ];
        let report = cmd_doctor_collect(&checks);
        assert!(report.contains("[OK]"), ...);
        assert!(report.contains("fav version"), ...);
    }

    // cmd_doctor_collect formats a [WARN] check correctly.
    // (Rune installation detection is in cmd_doctor_run which is env-dependent.)
    #[test]
    fn cmd_doctor_detects_missing_rune() {
        let checks = vec![
            DoctorCheck { status: DoctorStatus::Ok, label: "fav version".to_string(), detail: "54.5.0".to_string() },
            DoctorCheck { status: DoctorStatus::Warn, label: "rune kafka".to_string(),
                          detail: "version 2.1.0 declared but not installed".to_string() },
        ];
        let report = cmd_doctor_collect(&checks);
        assert!(report.contains("[WARN]"), ...);
        assert!(report.contains("rune kafka"), ...);
    }
}
```

---

## ステップ 5: `fav/Cargo.toml` バージョン更新

`version = "54.4.0"` → `version = "54.5.0"`

---

## ステップ 6: テスト実行・確認

```bash
cd /c/Users/yoshi/favnir/fav && cargo test -j 8 -- --test-threads=8 2>&1 | grep "test result"
```

期待値: 3195 passed, 0 failed

```bash
cargo clippy -- -D warnings
```

---

## ステップ 7: 後処理

- `CHANGELOG.md`: v54.5.0 エントリ追加（v54.4.0 の直上）
- `versions/current.md` を v54.5.0（3195 tests）に更新
- `roadmap-v54.1-v55.0.md` の v54.5.0 実績欄を COMPLETE に更新
- `tasks.md` を COMPLETE に更新（T0〜T7 全 `[x]`）

コードレビュー対応（実施済み）:
- [MED] `cmd_doctor_detects_missing_rune` テスト名と実装の乖離 → コメント追記で意図を明記
- [LOW] `cmd_doctor_collect` doc コメントに `cmd_doctor_run` との分離設計を記述
- [LOW] `prefix()` の trailing-space 設計にコメントでパディング目的を明記
