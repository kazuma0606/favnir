# Tasks: v53.4.0 — E2E 統合デモ Phase 1（Kafka → par transform → Snowflake）

Status: COMPLETE
Date: 2026-07-22

---

## T0 — 事前確認

- [x] `cargo test` 3169 passed, 0 failed を確認（ベース確認）
- [x] `cargo clippy -- -D warnings` クリーンであることを確認
- [x] `driver.rs` に `v53400_tests` が**存在しない**ことを確認:
  - [x] `rg -n "v53400_tests" fav/src/driver.rs` → 0 件
- [x] `driver.rs` に `v53300_tests` が存在することを確認（挿入位置の確認）:
  - [x] `rg -n "v53300_tests" fav/src/driver.rs` → 行番号を特定（47602）
- [x] `examples/v55-demo/` が**存在しない**ことを確認:
  - [x] `ls examples/v55-demo/ 2>/dev/null` → NOT FOUND
- [x] `Cargo.toml` の現在バージョンが `53.3.0` であることを確認

---

## T1 — `examples/v55-demo/` 作成

- [x] `examples/v55-demo/` ディレクトリ作成（`stages/` サブディレクトリを含む）
- [x] `examples/v55-demo/fav.toml` 作成:
  - [x] `name = "v55-demo"`, `version = "0.1.0"`
  - [x] `[runes]` に `kafka = "2.1.0"` / `snowflake = "1.0.0"` を含む
- [x] `examples/v55-demo/pipeline.fav` 作成:
  - [x] `import kafka` / `import snowflake` / `import "./stages/enrich" as enrich` / `import "./stages/validate" as validate` を含む
  - [x] `type RawOrder`, `type Order`, `type EnrichedOrder` 定義を含む
  - [x] `pipeline OrderIngestion {` ブロックを含む（Consume / Process / Store stage）
  - [x] Process stage に `par [enrich.run(order), validate.run(order)] |> Merge.ordered` を含む
- [x] `examples/v55-demo/stages/enrich.fav` 作成:
  - [x] `fn run(order: Order) -> Result<EnrichedOrder, String>` を定義
  - [x] `region: "us-east-1"` でフィールドを追加して返す
- [x] `examples/v55-demo/stages/validate.fav` 作成:
  - [x] `fn run(order: Order) -> Result<Order, String>` を定義
  - [x] `status != "pending"` / `amount < 0.0` のガードを含む
- [x] `cargo build` → コンパイルエラーなし確認

---

## T2 — `driver.rs` — `v53400_tests` 追加

- [x] `rg -n "v53300_tests" fav/src/driver.rs` で挿入位置（行番号）を確認
- [x] `v53300_tests` モジュールの直前に `v53400_tests` を追加:
  - [x] `e2e_integration_demo_structure` テスト:
    - [x] `env!("CARGO_MANIFEST_DIR").join("../examples/v55-demo")` でパスを構築
    - [x] `base.exists()` を assert
    - [x] `fav.toml` / `pipeline.fav` / `stages/enrich.fav` / `stages/validate.fav` の存在を assert
  - [x] `e2e_integration_demo_uses_par` テスト:
    - [x] `include_str!("../../examples/v55-demo/pipeline.fav")` で内容を読み込む
    - [x] `"par ["` を含むことを assert
    - [x] `"Merge.ordered"` を含むことを assert
    - [x] `"import kafka"` を含むことを assert
    - [x] `"import snowflake"` を含むことを assert
- [x] `cargo build` → コンパイルエラーなし確認

---

## T3 — `fav/Cargo.toml` 更新 + テスト実行

- [x] `version = "53.3.0"` → `version = "53.4.0"` に変更
- [x] v53300_tests にバージョンピンテストは存在しないため空化対象なし（確認済み）
- [x] `cargo test -j 8 -- --test-threads=8` 実行 → 3171 passed, 0 failed を確認
- [x] `cargo clippy -- -D warnings` クリーンを確認

---

## T4 — 後処理

- [x] `CHANGELOG.md` に v53.4.0 エントリ追加（直前の v53.3.0 エントリと同形式であることを確認）
- [x] `versions/current.md` を v53.4.0（3171 tests）に更新
- [x] `roadmap-v53.1-v54.0.md` の v53.4.0 実績欄を更新（未実施 → COMPLETE、テスト数 3171）
- [x] tasks.md を COMPLETE に更新（T0〜T4 全 `[x]`）
