# Spec: v98.1.0 — `KpiDefinition<T>` / `KpiSnapshot<T>` 型定義

## Background

v98.0.0 で SAP Workflow 1.0 を宣言した。
本バージョン（v98.1.0）から SAP Analytics Sprint（v98.1〜v99.0）を開始する。

**テスト数の注記**: ロードマップ（roadmap-v98.1-v99.0.md）は v98.0.0 完了時のベースラインを 4,230 と記載していたが、
code-reviewer 対応の累積により実際の v98.0.0 完了時テスト数は 4,235 である。
本スプリントの目標テスト数はすべて +5 で修正済み（ロードマップも更新済み）。

第 1 弾として、KPI を型で定義し計測結果をスナップショットとして保持する
コア型群（`KpiThreshold` / `KpiDefinition<T>` / `KpiStatus` / `KpiSnapshot<T>`）と
ヘルパー関数（`measure_kpi_status` / `make_kpi_snapshot`）を
`runes/sap-odata/analytics.fav`（新規）に実装する。

## Goals

1. `runes/sap-odata/analytics.fav` を新規作成し、以下の型と関数を定義する:
   - `KpiThreshold` レコード型
   - `KpiDefinition<T>` ジェネリックレコード型
   - `KpiStatus` バリアント型（`Ok` / `Warning(Float)` / `Critical(Float)`）
   - `KpiSnapshot<T>` ジェネリックレコード型
   - `measure_kpi_status(kpi: KpiDefinition<T>, value: Float) -> KpiStatus` ヘルパー関数
   - `make_kpi_snapshot(kpi: KpiDefinition<T>, value: Float, measured_at: String) -> KpiSnapshot<T>` ヘルパー関数
2. `fav/src/driver.rs` に `mod v98100_tests`（2 テスト）を追加する
3. `CHANGELOG.md` に v98.1.0 エントリを追加する
4. `versions/current.md` を v98.1.0 に更新する

## 型定義・API 例

```favnir
-- runes/sap-odata/analytics.fav

public type KpiThreshold = {
    warning:  Float,
    critical: Float
}

public type KpiDefinition<T> = {
    name:      String,
    unit:      String,
    threshold: KpiThreshold,
    extract:   fn(T) -> Float
}

public type KpiStatus =
    | Ok
    | Warning(Float)
    | Critical(Float)

public type KpiSnapshot<T> = {
    kpi:         KpiDefinition<T>,
    value:       Float,
    status:      KpiStatus,
    measured_at: String
}

-- KPI 値から KpiStatus を判定するヘルパー（T はジェネリック型パラメータ）
public fn measure_kpi_status<T>(kpi: KpiDefinition<T>, value: Float) -> KpiStatus {
    match value {
        v if v >= kpi.threshold.critical -> Critical(v)
        v if v >= kpi.threshold.warning  -> Warning(v)
        _                                -> Ok
    }
}

-- KpiSnapshot を生成するヘルパー（T はジェネリック型パラメータ）
public fn make_kpi_snapshot<T>(
    kpi:         KpiDefinition<T>,
    value:       Float,
    measured_at: String
) -> KpiSnapshot<T> {
    KpiSnapshot {
        kpi:         kpi,
        value:       value,
        status:      measure_kpi_status(kpi, value),
        measured_at: measured_at
    }
}
```

## テスト（2 件）

```rust
// analytics.fav が存在することを確認
fn analytics_fav_exists()

// analytics.fav に KpiDefinition が含まれることを確認
fn analytics_fav_has_kpi_definition()
```

## Success Criteria

- `runes/sap-odata/analytics.fav` が存在する
- `analytics.fav` に `KpiDefinition` / `KpiSnapshot` / `KpiStatus` / `KpiThreshold` が含まれる
- `mod v98100_tests` の全テスト（2 件）が pass する
- `cargo test` で 4,237 tests, 0 failures
- `cargo clippy --locked -- -D warnings` が pass する

## Error Codes

新規エラーコードなし。

## Files to Modify

| ファイル | 種別 | 内容 |
|---|---|---|
| `runes/sap-odata/analytics.fav` | 新規 | `KpiThreshold` / `KpiDefinition<T>` / `KpiStatus` / `KpiSnapshot<T>` 型定義 + ヘルパー関数 2 件 |
| `fav/src/driver.rs` | 追記 | `mod v98100_tests`（2 テスト） |
| `CHANGELOG.md` | 追記 | v98.1.0 エントリ |
| `versions/current.md` | 更新 | 最新安定版を v98.1.0 に変更 |
