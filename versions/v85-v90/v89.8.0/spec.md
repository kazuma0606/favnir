# Spec: v89.8.0 — パフォーマンス確認

## Background

v89.7.0 で OSS 整備（CONTRIBUTING.md + Issue テンプレート）が完了した。
本バージョンでは SAP パイプラインのパフォーマンスを計測・記録し、
`benchmarks/sap-odata-v89.8.0.json` としてベースラインを残す。

既存の `benchmarks/v80.0.0.json` 等と同じ JSON 形式で記録する。

### 現行のベンチマーク基盤

- `benchmarks/` ディレクトリに `v*.json` 形式で蓄積
- `benchmarks/baseline.json`・`benchmarks/lambda_coldstart.sh` が存在
- SAP OData 固有の計測結果ファイルは本バージョンが初

## Goals

1. `benchmarks/sap-odata-v89.8.0.json` を作成する
   - 既存 JSON 形式（`version` / `milestone` / `date` / `tests_passed` / `duration_ms` / `notes`）に準拠
   - SAP 固有フィールドとして `lambda_cold_start_ms`・`pagination_1000_ms` を追加
2. `fav/src/driver.rs` に `mod v89800_tests` を追加する（2 件）

## JSON 仕様

```json
{
  "version": "89.8.0",
  "milestone": "SAP OData パフォーマンス計測",
  "date": "2026-08-25",
  "tests_passed": 4033,
  "tests_failed": 0,
  "duration_ms": 17000,
  "lambda_cold_start_ms": 1200,
  "pagination_1000_ms": 3500,
  "notes": "v89.8.0 SAP パイプライン パフォーマンスベースライン。Lambda cold start・ページネーション（1000 件）計測値を含む。"
}
```

## Success Criteria（Rust テストで担保）

- `sap_perf_benchmark_json_exists`:
  `benchmarks/sap-odata-v89.8.0.json` が存在する
- `sap_perf_benchmark_has_duration_ms`:
  `benchmarks/sap-odata-v89.8.0.json` に `"duration_ms"` を含む
- `cargo test` で 4,035 tests, 0 failures（4,033 + 2）

## Files to Create / Modify

| ファイル | 変更種別 |
|---|---|
| `benchmarks/sap-odata-v89.8.0.json` | 新規作成 |
| `fav/src/driver.rs` | `mod v89800_tests` 追加 |

**前提確認**:
- `benchmarks/v80.0.0.json` が参照パターンとして存在（`version` / `milestone` / `date` / `tests_passed` / `tests_failed` / `duration_ms` / `notes` の 7 フィールド）
- SAP 固有フィールド（`lambda_cold_start_ms` / `pagination_1000_ms`）は既存フォーマットの拡張として追加
- ファイル名は SAP 固有プレフィックスを付与（`sap-odata-v89.8.0.json`）するが、JSON フィールド構造は既存形式に準拠する（既存の `vXX.json` 命名規則とは意図的に異なる）
- `"tests_passed": 4033` は v89.7.0 完了時点の値（本バージョン実装前のベースライン）
- `"duration_ms"` は `cargo test --release` の実測値を記録する
- テストは `fav/` を cwd として実行されるため `"../benchmarks/"` は `benchmarks/` に解決される（既存テストと同じパターン）

**Note**: CHANGELOG / MILESTONE 更新は v90.0.0 宣言バージョンでまとめて実施する（本バージョンは対象外）
**Note**: Cargo.toml のバージョンは v90.0.0 宣言まで `89.0.0` のまま維持する。
