# Spec: v98.3.0 — SAP Analytics Cloud データプッシュ API（`SacDataset` 型）

## Background

v98.2.0 で `BwQuery<T>` / `BwResult<T>` を `analytics.fav` に追加した。
本バージョンでは SAC（SAP Analytics Cloud）の Data Import Service API へデータをプッシュするための
`SacDataset` 型と `sac_push_mock` ヘルパーを `runes/sap-odata/sac.fav`（新規）に実装する。

**スコープ注記**:
- `ctx.sap.sac_push()` の実 API 実装は本バージョンのスコープ外（型定義とモックのみ）
- `Effect::SapAnalytics` の Rust 側追加は v98.4.0 で対応する
  （v98.4.0 の roadmap 注記「Rust 側（v98.3.0 で追加）」はスプリント調整により v98.4.0 に延期）
- v98.2.0 の code-reviewer 指摘で `sap_odata.fav` に `use sap_odata.analytics` を追加済みのため、
  本バージョンでも同様に `use sap_odata.sac` を追加する

## Goals

1. `runes/sap-odata/sac.fav` を新規作成し、以下を定義する:
   - `SacDataset` レコード型（`model_id: String` / `rows: List<String>`）
   - `sac_push_mock(dataset: SacDataset) -> String` — テスト用モックヘルパー（送信シミュレート）
2. `runes/sap-odata/sap_odata.fav` に `use sap_odata.sac` と `SacDataset` / `sac_push_mock` の re-export を追加する
3. `fav/src/driver.rs` に `mod v98300_tests`（2 テスト）を追加する
4. `CHANGELOG.md` に v98.3.0 エントリを追加する
5. `versions/current.md` を v98.3.0 に更新する

## 型定義・API 例

```favnir
-- runes/sap-odata/sac.fav
-- SAP Analytics Cloud データプッシュ型（v98.3.0）

-- SAC Data Import Service 向けデータセット
-- rows は CSV 形式の行データ（ヘッダー行 + データ行）
public type SacDataset = {
    model_id: String,
    rows:     List<String>
}

-- テスト用モックヘルパー（実際の HTTP 通信は行わない）
public fn sac_push_mock(dataset: SacDataset) -> String {
    String.concat(["pushed:", dataset.model_id])
}
```

使用例（pipeline から SAC へプッシュ）:

```favnir
-- 注: ctx.sap.sac_push() の実装は v98.4.0 以降。本バージョンでは型定義のみ。
-- テスト用には sac_push_mock を使用する:
bind result <- sac_push_mock(SacDataset {
    model_id: "SAP__FI_GL_IM_GLACCOUNTS",
    rows:     csv_rows
})
```

## テスト（2 件）

```rust
// sac.fav が存在することを確認
fn sac_fav_exists()

// sac.fav に SacDataset が含まれることを確認
fn sac_fav_has_sac_dataset()
```

## Success Criteria

- `runes/sap-odata/sac.fav` が存在し `SacDataset` / `sac_push_mock` が含まれる
- `runes/sap-odata/sap_odata.fav` に `use sap_odata.sac` が追加されている
- `mod v98300_tests` の全テスト（2 件）が pass する
- `cargo test` で 4,241 tests, 0 failures
- `cargo clippy --locked -- -D warnings` が pass する
- `./target/debug/fav fmt --check self/compiler.fav` が pass する
- `./target/debug/fav fmt --check self/checker.fav` が pass する

## Error Codes

新規エラーコードなし。

## Files to Modify

| ファイル | 種別 | 内容 |
|---|---|---|
| `runes/sap-odata/sac.fav` | 新規 | `SacDataset` 型 + `sac_push_mock` ヘルパー |
| `runes/sap-odata/sap_odata.fav` | 追記 | `use sap_odata.sac` + `SacDataset` / `sac_push_mock` re-export |
| `fav/src/driver.rs` | 追記 | `mod v98300_tests`（2 テスト） |
| `CHANGELOG.md` | 追記 | v98.3.0 エントリ |
| `versions/current.md` | 更新 | 最新安定版を v98.3.0 に変更 |
