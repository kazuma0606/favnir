# Plan: v90.6.0 — `pipeline.fav` を `ctx.sap.*` で書き換え

## 依存関係

```
Step 1（現状確認）
    ↓
Step 2（pipeline.fav 書き換え）
    ↓
Step 3（driver.rs テスト追加）
    ↓
Step 4（cargo test）
    ↓
Step 5（CHANGELOG 更新）
    ↓
Step 6（CI 事前確認）
```

## Steps

### Step 1: 現状確認

- `infra/e2e-demo/sap-odata/pipeline.fav` の全体を読み込み、4 シナリオの構造を確認する
- `bind cfg <- sap_odata.sap_config_from_env()` の呼び出し件数を確認する（4 件）
- 現テスト数が 4052 であることを確認する

### Step 2: `pipeline.fav` を書き換え

各シナリオで以下を実施する:

1. `bind cfg <- sap_odata.sap_config_from_env()` の行を削除する
2. `SapClient` interface 対応関数（`business_partners` / `sales_orders` / `materials` / `journal_entries`）の呼び出しを `ctx.sap.METHOD(filter)` に書き換える
3. `sap_odata.business_partners(cfg, filter)` → `ctx.sap.business_partners(filter)` 等、`cfg` 引数を除去する
4. シナリオ 4 の `purchase_orders` は `SapClient` interface 外のため、`purchase_orders` の呼び出しを除去する。`outstanding_payables` は `ctx.sap.journal_entries` のみを使用する簡略版に変更する（`purchase_orders` 対応は v91.x.x 予定）

コメントスタイルは `--`（pipeline.fav の既存スタイルに合わせる）。

### Step 3: `driver.rs` に `mod v90600_tests` を追加

- `mod v90500_tests` の直後に `#[cfg(test)] mod v90600_tests { ... }` を追加する
- `pipeline_fav_uses_ctx_sap`: `pipeline.fav` に `ctx.sap.` が含まれることを確認
- `pipeline_fav_no_explicit_cfg`: `pipeline.fav` に `sap_config_from_env` が含まれないことを確認

### Step 4: `cargo test` で全 pass 確認

- `cargo test 2>&1 | grep "test result"` で 4054 tests, 0 failures を確認する

### Step 5: `CHANGELOG.md` に v90.6.0 エントリを追加

- `## [v90.5.0]` の前に v90.6.0 エントリを追加する
- `ctx.sap.*` / `sap_config_from_env 削除` / `4054` が含まれることを確認する

### Step 6: CI 事前確認

- `cargo clippy --locked -- -D warnings` が pass することを確認する
- `./target/debug/fav fmt --check self/compiler.fav` が pass することを確認する
- `./target/debug/fav fmt --check self/checker.fav` が pass することを確認する
