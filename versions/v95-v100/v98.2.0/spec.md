# Spec: v98.2.0 — `BwQuery<T>` / `BwResult<T>` + `ctx.sap.bw_query()`

## Background

v98.1.0 で `KpiDefinition<T>` / `KpiSnapshot<T>` を `runes/sap-odata/analytics.fav` に追加した。
本バージョンでは BW/4HANA の InfoProvider / Query に対応する型安全クエリインターフェースとして
`BwQuery<T>` / `BwResult<T>` 型と `bw_query_mock<T>` ヘルパーを `analytics.fav` に追記する。

## Goals

> **スコープ注記**: `ctx.sap.bw_query()` の実 API 実装は本バージョンのスコープ外。
> 型定義（`BwQuery<T>` / `BwResult<T>`）とテスト用モックヘルパー（`bw_query_mock<T>`）のみ追加する。
> 実 API は v98.x 以降の ctx 統合バージョンで対応する。

1. `runes/sap-odata/analytics.fav` に以下を追記する:
   - `BwQuery<T>` ジェネリックレコード型（`info_provider` / `characteristics` / `key_figures` / `filters`）
   - `BwResult<T>` ジェネリックレコード型（`rows: List<T>` / `total: Int`）
   - `bw_query_mock<T>(query: BwQuery<T>, rows: List<T>) -> BwResult<T>` — テスト用モックヘルパー
2. `fav/src/driver.rs` に `mod v98200_tests`（2 テスト）を追加する
3. `CHANGELOG.md` に v98.2.0 エントリを追加する
4. `versions/current.md` を v98.2.0 に更新する

## 型定義・API 例

```favnir
-- BW/4HANA クエリ定義（T はクエリ結果の行型）
public type BwQuery<T> = {
    info_provider:   String,
    characteristics: List<String>,
    key_figures:     List<String>,
    filters:         List<String>
}

-- BW クエリ結果
public type BwResult<T> = {
    rows:  List<T>,
    total: Int
}

-- テスト用モックヘルパー（実行せず rows をそのまま BwResult に包む）
public fn bw_query_mock<T>(query: BwQuery<T>, rows: List<T>) -> BwResult<T> {
    BwResult {
        rows:  rows,
        total: List.length(rows)
    }
}
```

使用例:

```favnir
-- BW クエリ実行（ctx.sap.bw_query は v98.x 以降で実装）
bind result <- ctx.sap.bw_query<SalesKpi>(BwQuery {
    info_provider:   "0SD_C03",
    characteristics: ["0CALMONTH", "0SOLD_TO"],
    key_figures:     ["0NET_VAL_S"],
    filters:         ["0CALMONTH = 202608"]
})
```

## テスト（2 件）

```rust
// analytics.fav に BwQuery が含まれることを確認
fn analytics_fav_has_bw_query()

// analytics.fav に BwResult が含まれることを確認
fn analytics_fav_has_bw_result()
```

## Success Criteria

- `runes/sap-odata/analytics.fav` に `BwQuery` / `BwResult` / `bw_query_mock` が含まれる
- `mod v98200_tests` の全テスト（2 件）が pass する
- `cargo test` で 4,239 tests, 0 failures
- `cargo clippy --locked -- -D warnings` が pass する
- `./target/debug/fav fmt --check self/compiler.fav` が pass する
- `./target/debug/fav fmt --check self/checker.fav` が pass する

## Error Codes

新規エラーコードなし。

## Files to Modify

| ファイル | 種別 | 内容 |
|---|---|---|
| `runes/sap-odata/analytics.fav` | 追記 | `BwQuery<T>` / `BwResult<T>` 型 + `bw_query_mock<T>` ヘルパー |
| `fav/src/driver.rs` | 追記 | `mod v98200_tests`（2 テスト） |
| `CHANGELOG.md` | 追記 | v98.2.0 エントリ |
| `versions/current.md` | 更新 | 最新安定版を v98.2.0 に変更 |
