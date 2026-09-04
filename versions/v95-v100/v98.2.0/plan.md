# Plan: v98.2.0 — `BwQuery<T>` / `BwResult<T>` + `ctx.sap.bw_query()`

## 実装順序

### Step 1: `runes/sap-odata/analytics.fav` に追記

既存の `make_kpi_snapshot` の後に以下を追記する（依存関係順）:

1. `BwQuery<T>` ジェネリックレコード型
   - `info_provider: String`
   - `characteristics: List<String>`
   - `key_figures: List<String>`
   - `filters: List<String>`
2. `BwResult<T>` ジェネリックレコード型
   - `rows: List<T>`
   - `total: Int`
3. `bw_query_mock<T>(query: BwQuery<T>, rows: List<T>) -> BwResult<T>` ヘルパー
   - `BwResult { rows: rows, total: List.length(rows) }` を返す
   - テスト用モック（`ctx.sap.bw_query` の実 API は v98.x 以降で対応）

コメントはすべて `--` スタイルを使用する（`//` は Favnir 規約違反）。

### Step 2: `fav/src/driver.rs` に `mod v98200_tests` を追加

`mod v98100_tests` の直後に `#[cfg(test)] mod v98200_tests { ... }` を追加する（2 テスト）:

- `analytics_fav_has_bw_query`: `content.contains("BwQuery")` で型定義の存在を確認
- `analytics_fav_has_bw_result`: `content.contains("BwResult")` で型定義の存在を確認

両テストとも `std::fs::read_to_string("../runes/sap-odata/analytics.fav")` を使用する。

### Step 3: `cargo test` で全 pass 確認

`cargo test -- --test-threads=1 2>&1 | grep "test result"` を実行し、4,239 tests, 0 failures を確認する。

### Step 4: `CHANGELOG.md` に v98.2.0 エントリを追加

`[v98.1.0]` エントリの直前（先頭）に `[v98.2.0]` エントリを追加する。

### Step 5: `versions/current.md` 更新

- `最終更新:` ヘッダーを `v98.2.0` に更新する
- 最新安定版を `v98.2.0` に更新する（テスト数 4,239）

### Step 6: CI 事前確認

`cargo test`（Step 3）実行後、`target/debug/fav` バイナリが存在することを前提とする。

- `cargo clippy --locked -- -D warnings` が pass することを確認する
- `./target/debug/fav fmt --check self/compiler.fav` が pass することを確認する
- `./target/debug/fav fmt --check self/checker.fav` が pass することを確認する
