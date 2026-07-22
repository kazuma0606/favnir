# Tasks: v53.5.0 — E2E 統合デモ Phase 2（assert_schema + audit-log + OTel）

Status: COMPLETE
Date: 2026-07-22

---

## T0 — 事前確認

- [x] `cargo test` 3171 passed, 0 failed を確認（ベース確認）
- [x] `cargo clippy -- -D warnings` クリーンであることを確認
- [x] `driver.rs` に `v53500_tests` が**存在しない**ことを確認:
  - [x] `rg -n "v53500_tests" fav/src/driver.rs` → 0 件
- [x] `driver.rs` に `v53400_tests` が存在することを確認（挿入位置の確認）:
  - [x] `rg -n "v53400_tests" fav/src/driver.rs` → 行番号を特定（47602）
- [x] `examples/v55-demo/run.sh` が**存在しない**ことを確認:
  - [x] `ls examples/v55-demo/run.sh 2>/dev/null` → NOT FOUND
- [x] `examples/v55-demo/pipeline.fav` に `assert_schema` が**含まれない**ことを確認:
  - [x] `rg "assert_schema" examples/v55-demo/pipeline.fav` → 0 件
- [x] `Cargo.toml` の現在バージョンが `53.4.0` であることを確認

---

## T1 — `examples/v55-demo/pipeline.fav` 更新

- [x] `type ValidOrder = { id: Int, amount: Float, status: String }` を型定義セクションに追加
- [x] `SchemaCheck` stage を `Consume` と `Process` の間に追加:
  - [x] ステージ型: `Order -> Result<ValidOrder>`
  - [x] `bind checked <- assert_schema<ValidOrder>(order)` を含む
  - [x] `Ok(checked)` を返す
  - [x] OTel / audit-log の説明コメントを含む
- [x] `Process` stage の入力型を `Order` → `ValidOrder` に更新、ラムダ引数名を `|valid_order|` に変更
- [x] 内容確認: `grep "assert_schema"` / `grep "ValidOrder"` → 各 1 件以上ヒット

---

## T2 — `examples/v55-demo/run.sh` 新規作成

- [x] `run.sh` を `examples/v55-demo/` に作成:
  - [x] shebang `#!/bin/bash` を含む
  - [x] `set -euo pipefail` を含む
  - [x] `fav run pipeline.fav --audit-log ./audit.log` を含む
- [x] `chmod +x examples/v55-demo/run.sh` で実行権限を付与
- [x] 内容確認: `grep "\-\-audit-log" examples/v55-demo/run.sh` → 1 件ヒット

---

## T3 — `driver.rs` — `v53500_tests` 追加

- [x] `rg -n "v53400_tests" fav/src/driver.rs` で挿入位置（行番号）を確認
- [x] `v53400_tests` モジュールの直前に `v53500_tests` を追加:
  - [x] `e2e_integration_demo_has_schema` テスト:
    - [x] `include_str!("../../examples/v55-demo/pipeline.fav")` で内容を読み込む
    - [x] `"assert_schema"` を含むことを assert
    - [x] `"ValidOrder"` を含むことを assert
  - [x] `e2e_integration_demo_has_audit_log` テスト:
    - [x] `include_str!("../../examples/v55-demo/run.sh")` で内容を読み込む
    - [x] `"fav run"` を含むことを assert
    - [x] `"--audit-log"` を含むことを assert
- [x] `cargo build` → コンパイルエラーなし確認

---

## T4 — `fav/Cargo.toml` 更新 + テスト実行

- [x] `version = "53.4.0"` → `version = "53.5.0"` に変更
- [x] v53400_tests にバージョンピンテストは存在しないため空化対象なし（確認済み）
- [x] `cargo test -j 8 -- --test-threads=8` 実行 → 3173 passed, 0 failed を確認
- [x] `cargo clippy -- -D warnings` クリーンを確認

---

## T5 — 後処理

- [x] `CHANGELOG.md` に v53.5.0 エントリ追加（直前の v53.4.0 エントリと同形式であることを確認）
- [x] `versions/current.md` を v53.5.0（3173 tests）に更新
- [x] `roadmap-v53.1-v54.0.md` の v53.5.0 実績欄を更新（未実施 → COMPLETE、テスト数 3173）
  - [x] 推定値 3167 → 実績 3173 の差異を注記
- [x] tasks.md を COMPLETE に更新（T0〜T5 全 `[x]`）
