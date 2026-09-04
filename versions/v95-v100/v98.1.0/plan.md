# Plan: v98.1.0 — `KpiDefinition<T>` / `KpiSnapshot<T>` 型定義

## 実装順序

### Step 1: `runes/sap-odata/analytics.fav` を新規作成

以下の順序で定義する（依存関係順）:

1. `KpiThreshold` レコード型（`warning: Float` / `critical: Float`）
2. `KpiDefinition<T>` ジェネリックレコード型（`name`, `unit`, `threshold: KpiThreshold`, `extract: fn(T) -> Float`）
3. `KpiStatus` バリアント型（`Ok` / `Warning(Float)` / `Critical(Float)`）
4. `KpiSnapshot<T>` ジェネリックレコード型（`kpi`, `value`, `status: KpiStatus`, `measured_at: String`）
5. `measure_kpi_status(kpi: KpiDefinition<T>, value: Float) -> KpiStatus` — 値と閾値を比較して KpiStatus を返す
6. `make_kpi_snapshot(kpi: KpiDefinition<T>, value: Float, measured_at: String) -> KpiSnapshot<T>` — measure_kpi_status を呼んで KpiSnapshot を構築する

### Step 2: `fav/src/driver.rs` に `mod v98100_tests` を追加

`mod v98000_tests` の直後に `#[cfg(test)] mod v98100_tests { ... }` を追加する（2 テスト）:

- `analytics_fav_exists`: `std::fs::read_to_string("../runes/sap-odata/analytics.fav")` でファイル存在を確認
- `analytics_fav_has_kpi_definition`: `content.contains("KpiDefinition")` で型定義の存在を確認

### Step 3: `cargo test` で全 pass 確認

`cargo test 2>&1 | grep "test result"` を実行し、4,237 tests, 0 failures を確認する。

### Step 4: `CHANGELOG.md` に v98.1.0 エントリを追加

`[v98.0.0]` エントリの直前（先頭）に `[v98.1.0]` エントリを追加する。

**Note**: v98.1.0 の driver テストには `changelog_has_v98_1_0` が含まれないため、
CHANGELOG を Step 3（cargo test）の後に追加しても問題ない。ただし、
将来バージョンで整合性テストが追加された場合に備え、次の宣言版からは CHANGELOG 先行を遵守する。

### Step 5: `versions/current.md` 更新

- `最終更新:` ヘッダーを `v98.1.0` に更新する
- 最新安定版を `v98.1.0` に更新する（テスト数 4,237）

### Step 6: CI 事前確認

`cargo test`（Step 3）実行後、`target/debug/fav` バイナリが存在することを前提とする。

- `cargo clippy --locked -- -D warnings` が pass することを確認する
- `./target/debug/fav fmt --check self/compiler.fav` が pass することを確認する
- `./target/debug/fav fmt --check self/checker.fav` が pass することを確認する
