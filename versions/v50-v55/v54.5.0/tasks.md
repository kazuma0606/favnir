# Tasks: v54.5.0 — fav doctor 環境診断コマンド

Status: COMPLETE
Date: 2026-07-23

---

## T0 — 事前確認

- [x] `cargo test` 3193 passed, 0 failed を確認（ベース確認）
- [x] `cargo clippy -- -D warnings` クリーンであることを確認
- [x] `driver.rs` に `v54500_tests` が**存在しない**ことを確認
- [x] `driver.rs` に `v54400_tests` が存在することを確認（挿入位置の確認）
- [x] `driver.rs` に `DoctorCheck` が未存在であることを確認
- [x] `main.rs` に `doctor` コマンドが未存在であることを確認
- [x] `Cargo.toml` の現在バージョンが `54.4.0` であることを確認

---

## T1 — `driver.rs` — 型定義 + ロジック関数追加

- [x] `v54400_tests` の直前に追加:
  - [x] `DoctorCheck` struct（`status: DoctorStatus` / `label: String` / `detail: String`）
  - [x] `DoctorStatus` enum（`Ok` / `Warn` / `Fail`）
  - [x] `DoctorStatus::prefix()` — 固定幅 6 文字プレフィクス（コメントでパディング意図明記）
  - [x] `cmd_doctor_collect(checks: &[DoctorCheck]) -> String` — 純粋関数・環境非依存
  - [x] `cmd_doctor_run() -> Vec<DoctorCheck>` — 実環境チェック（fav version / Rust toolchain / fav.toml / .fav-cache）
  - [x] `cmd_doctor_collect` doc コメントに `cmd_doctor_run` との分離設計を記述
- [x] `cargo build` → コンパイルエラーなし確認

---

## T2 — `main.rs` — `fav doctor` コマンド追加

- [x] `Some("dq-report")` の直前に `Some("doctor")` アームを追加:
  - [x] `driver::cmd_doctor_run()` を呼び出してチェックリストを取得
  - [x] `driver::cmd_doctor_collect(&checks)` でレポート生成
  - [x] `println!("{report}")` で出力
- [x] `cargo build` → コンパイルエラーなし確認

---

## T3 — `driver.rs` — `v54500_tests` 追加

- [x] `v54400_tests` の直前に `v54500_tests` を追加（2 テスト）:
  - [x] `cmd_doctor_passes_clean_env`:
    - [x] `Ok` チェック 2 件を `cmd_doctor_collect` に渡す
    - [x] `"[OK]"` を含む
    - [x] `"fav version"` を含む
  - [x] `cmd_doctor_detects_missing_rune`:
    - [x] コメントで「`cmd_doctor_collect` の WARN フォーマット検証」旨を明記
    - [x] `Warn` チェックを含むリストを `cmd_doctor_collect` に渡す
    - [x] `"[WARN]"` を含む
    - [x] `"rune kafka"` を含む

---

## T4 — `fav/Cargo.toml` 更新 + テスト実行

- [x] `version = "54.4.0"` → `version = "54.5.0"` に変更
- [x] `cargo test -j 8 -- --test-threads=8` 実行 → 3195 passed, 0 failed を確認
- [x] `cargo clippy -- -D warnings` クリーンを確認

---

## T5 — 後処理

- [x] `CHANGELOG.md`: v54.5.0 エントリ追加（v54.4.0 の直上）
- [x] `versions/current.md` を v54.5.0（3195 tests）に更新
- [x] `roadmap-v54.1-v55.0.md` の v54.5.0 実績欄を更新（COMPLETE・3195 tests・2026-07-23）

---

## T6 — コードレビュー対応

- [x] [MED] `cmd_doctor_detects_missing_rune` テスト名と実装の乖離 → コメント追記で意図を明記
- [x] [LOW] `cmd_doctor_collect` doc コメントに `cmd_doctor_run` との分離設計を記述
- [x] [LOW] `prefix()` の trailing-space 設計にコメントでパディング目的を明記

---

## T7 — tasks.md 完了

- [x] tasks.md を COMPLETE に更新（T0〜T7 全 `[x]`）
