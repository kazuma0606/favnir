# Spec: v97.0.0 — SAP Multi-system 1.0 宣言

## Background

v96.1.0〜v96.9.0 で SAP Multi-system スプリントの全機能を実装した:
- `SapEnvironment` 型 + `ctx.sap_env()` (v96.1.0)
- `fav.toml [sap.environments]` マルチ環境設定 (v96.2.0)
- SAP → Parquet / DuckDB エクスポート (v96.3.0)
- SAP → Snowflake リアルタイム同期 (v96.4.0)
- カスタム OData サービス対応 `--sap-service-name` (v96.5.0)
- `CleanCoreClient` (v96.6.0)
- Cross-system 型安全 JOIN `CrossSystem.join<A,B>` (v96.7.0)
- 接続プール / キャッシュ / リトライ `RetryPolicy` (v96.8.0)
- 安定化・コードフリーズ (v96.9.0)

v97.0.0 はこれらを統合し「SAP Multi-system 1.0」として正式宣言する。

## Goals

1. `fav/Cargo.toml` のバージョンを `97.0.0` に更新する
2. `mod v97000_tests` を追加し 4 件の宣言テストを通過させる（計 4,213 tests）
3. `MILESTONE.md` に v97.0.0 エントリを追加する
4. `README.md` に `## v97.0 — SAP Multi-system 1.0` セクションを追加する
5. `CHANGELOG.md` に v97.0.0 エントリを追加する
6. `versions/current.md` を v97.0.0 に更新する
7. `cargo clean` を実施し、クリーンビルドでテストを再確認する

## 宣言文

> 「Favnir が、SAP の境界を越えた。
>
>  `ctx.sap_env("PRD")` で本番に向き、
>  SAP のデータが Snowflake に流れ、
>  カスタムサービスの型も `fav infer` が生み出す。
>
>  それが、Favnir SAP Multi-system 1.0 である。」

## 追加テスト（`mod v97000_tests`）

```rust
#[test]
fn cargo_toml_version_is_97_0_0() { ... }  // Cargo.toml に "97.0.0" が含まれる

#[test]
fn changelog_has_v97_0_0() { ... }          // CHANGELOG.md に "[v97.0.0]" が含まれる

#[test]
fn milestone_has_sap_multi_system() { ... } // MILESTONE.md に "SAP Multi-system" が含まれる

#[test]
fn readme_mentions_sap_multi_system() { ... } // README.md に "SAP Multi-system" が含まれる
```

## Success Criteria

- `cargo test` で 4,213 tests, 0 failures
- `cargo clean` 後の再テストでも同数
- `cargo clippy --locked -- -D warnings` pass
- `./target/debug/fav fmt --check self/compiler.fav` pass
- `./target/debug/fav fmt --check self/checker.fav` pass

## Files to Modify

| ファイル | 変更内容 |
|---|---|
| `fav/Cargo.toml` | `version = "97.0.0"` |
| `fav/src/driver.rs` | `mod v97000_tests` 追加（4 テスト） |
| `CHANGELOG.md` | `[v97.0.0]` エントリ追加（先頭） |
| `MILESTONE.md` | v97.0.0 エントリ追加 |
| `README.md` | `## v97.0 — SAP Multi-system 1.0` セクション追加 |
| `versions/current.md` | 最新安定版を v97.0.0 に更新（テスト数 4,213） |
