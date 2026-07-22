# Tasks: v54.4.0 — fav dq-report データ品質レポートコマンド

Status: COMPLETE
Date: 2026-07-22

---

## T0 — 事前確認

- [x] `cargo test` 3191 passed, 0 failed を確認（ベース確認）
- [x] `cargo clippy -- -D warnings` クリーンであることを確認
- [x] `driver.rs` に `v54400_tests` が**存在しない**ことを確認
- [x] `driver.rs` に `v54300_tests` が存在することを確認（挿入位置の確認）
- [x] `driver.rs` に `cmd_dq_report_collect` が未存在であることを確認
- [x] `main.rs` に `dq-report` コマンドが未存在であることを確認
- [x] `Cargo.toml` の現在バージョンが `54.3.0` であることを確認

---

## T1 — `driver.rs` — `cmd_dq_report_collect` 追加

- [x] `v54300_tests` の直前に追加:
  - [x] doc コメント（単一行が schema/SLA 両方に寄与しうる旨を明記）
  - [x] `op == "schema_check" || op == "schema_error"` → `total_rows` +1 / スキーマ別カウント更新
  - [x] `op == "schema_error"` → `error_rows` +1
  - [x] `latency_ms > 200.0` → SLA 違反リストに追加
  - [x] `schema` を `entry()` に **move**（`.clone()` 不使用）
  - [x] スキーマ名はキーでソートして出力
  - [x] `total_rows == 0` のとき `error_pct = "0.00%"`
  - [x] Markdown 出力形式: `# Data Quality Report` ヘッダー・schema 統計・SLA 違反
- [x] `cargo build` → コンパイルエラーなし確認

---

## T2 — `main.rs` — `fav dq-report` コマンド追加

- [x] `Some("watch")` の直前に `Some("dq-report")` アームを追加:
  - [x] `--audit-log <path>` 必須引数の解析（省略時 `eprintln!` + `process::exit(1)`）
  - [x] `std::fs::read_to_string` でファイル読み込み（失敗時エラーメッセージ + exit 1）
  - [x] `driver::cmd_dq_report_collect` を呼び `println!` で出力
- [x] `cargo build` → コンパイルエラーなし確認

---

## T3 — `driver.rs` — `v54400_tests` 追加

- [x] `v54300_tests` の直前に `v54400_tests` を追加（2 テスト）:
  - [x] `SAMPLE_AUDIT_LOG` 定数（6 行 JSONL: schema_check×4 / schema_error×1 / write×1 / latency_ms:250×1）
  - [x] `cmd_dq_report_generates`:
    - [x] レポートが非空であること
    - [x] `"Data Quality Report"` ヘッダーを含む
  - [x] `cmd_dq_report_has_schema_stats`:
    - [x] `"Schema validation"` を含む
    - [x] `"rows checked"` を含む
    - [x] `"OrderRow"` を含む
    - [x] `"SLA violations"` を含む
    - [x] `!contains("SLA violations:  none")` — SLA 違反あり確認（精密マッチ）

---

## T4 — `fav/Cargo.toml` 更新 + テスト実行

- [x] `version = "54.3.0"` → `version = "54.4.0"` に変更
- [x] `cargo test -j 8 -- --test-threads=8` 実行 → 3193 passed, 0 failed を確認
- [x] `cargo clippy -- -D warnings` クリーンを確認

---

## T5 — 後処理

- [x] `CHANGELOG.md`: v54.4.0 エントリ追加（v54.3.0 の直上）
- [x] `versions/current.md` を v54.4.0（3193 tests）に更新
- [x] `roadmap-v54.1-v55.0.md` の v54.4.0 実績欄を更新（COMPLETE・3193 tests・2026-07-22）

---

## T6 — コードレビュー対応

- [x] [MED] `!report.contains("none")` が部分マッチで脆弱 → `!report.contains("SLA violations:  none")` に修正
- [x] [MED] `schema.clone()` が不要 → `schema` を `entry()` に直接 move
- [x] [LOW] doc コメントに「1行が schema 統計と SLA 統計の両方に寄与しうる」旨を追記
- [x] [LOW] `(u64, u64)` 構造体化は将来バージョンに委ねる（現状コメント補完済みのため現状維持）

---

## T7 — tasks.md 完了

- [x] tasks.md を COMPLETE に更新（T0〜T7 全 `[x]`）
