# Tasks: v54.2.0 — fav run --watch 高度化（差分表示・サマリー）

Status: COMPLETE
Date: 2026-07-22

---

## T0 — 事前確認

- [x] `cargo test` 3187 passed, 0 failed を確認（ベース確認）
- [x] `cargo clippy -- -D warnings` クリーンであることを確認
- [x] `driver.rs` に `v54200_tests` が**存在しない**ことを確認:
  - [x] `rg -n "v54200_tests" fav/src/driver.rs` → 0 件
- [x] `driver.rs` に `v54100_tests` が存在することを確認（挿入位置の確認）:
  - [x] `rg -n "v54100_tests" fav/src/driver.rs` → 行番号を特定（47602）
- [x] `Cargo.toml` の現在バージョンが `54.1.0` であることを確認
- [x] `main.rs` に `--watch-diff` / `--watch-summary` が未存在であることを確認

---

## T1 — `driver.rs` — WatchEvent + フォーマット関数追加

- [x] `cmd_run` の直前に追加:
  - [x] `WatchEvent` 構造体（`#[derive(Debug, Clone, PartialEq, Eq)]`、4 pub フィールド）
  - [x] `format_watch_diff`: f64 差分計算・`d==0.0` → delta 省略・`:.1` フォーマット
  - [x] `format_watch_summary`: 空スライス → `"no changes"` メッセージ / それ以外 → フィールド一覧
- [x] `cargo build` → コンパイルエラーなし確認

---

## T2 — `main.rs` — `--watch-diff` / `--watch-summary` フラグ追加

- [x] 変数宣言: `let mut watch_diff = false; let mut watch_summary = false;`
- [x] `match` アーム追加（`"--resume"` の直後）:
  - [x] `"--watch-diff"`: `watch_diff = true; i += 1; file_idx = i;`
  - [x] `"--watch-summary"`: `watch_summary = true; i += 1; file_idx = i;`
- [x] ループ後の警告出力（サイレント無視を防ぐ）:
  - [x] `watch_diff` → `eprintln!("warning: --watch-diff is not yet fully implemented; ...")`
  - [x] `watch_summary` → `eprintln!("warning: --watch-summary is not yet fully implemented; ...")`

---

## T3 — `driver.rs` — `v54200_tests` 追加

- [x] `v54100_tests` の直前に `v54200_tests` を追加（2 テスト）:
  - [x] `run_watch_diff_numeric`:
    - [x] `[watch]` を含む
    - [x] `order.amount` を含む
    - [x] `0.0` と `99.0` を含む
    - [x] `Δ+99.0` を含む（精度 `:.1` で一致）
    - [x] `Parse` を含む
  - [x] `run_watch_summary_output`:
    - [x] `[watch-summary]` を含む
    - [x] `order.amount` / `order.status` を含む
    - [x] `Parse` / `Validate` を含む
    - [x] 空スライス → `no changes` を含む

---

## T4 — `fav/Cargo.toml` 更新 + テスト実行

- [x] `version = "54.1.0"` → `version = "54.2.0"` に変更
- [x] `cargo test -j 8 -- --test-threads=8` 実行 → 3189 passed, 0 failed を確認
- [x] `cargo clippy -- -D warnings` クリーンを確認

---

## T5 — 後処理

- [x] `CHANGELOG.md`: v54.2.0 エントリ追加（v54.1.0 の直上）
- [x] `versions/current.md` を v54.2.0（3189 tests）に更新
- [x] `roadmap-v54.1-v55.0.md` の v54.2.0 実績欄を更新（COMPLETE・3189 tests・2026-07-22）

---

## T6 — コードレビュー対応

- [x] [MED] `d==0.0` のとき `"Δ+0.0"` が出力される問題 → `if d == 0.0 { String::new() }` で delta 省略
- [x] [MED] `--watch-diff/--watch-summary` サイレント無視 → `eprintln!` 警告に変更
- [x] [LOW] f64 フォーマット `{}` → `:.1` で `Δ+99.0` 形式を保証
- [x] [LOW] `WatchEvent` に `PartialEq, Eq` を derive 追加
- [x] [LOW] テストアサーション `Δ+99` → `Δ+99.0` に精度向上

---

## T7 — tasks.md 完了

- [x] tasks.md を COMPLETE に更新（T0〜T7 全 `[x]`）
