# Spec: v95.0.0 — SAP Advanced 1.0 宣言 ★クリーンアップ

## Background

v94.1〜v94.9 で実装した SAP Advanced Era（Sprint 5）の全機能が完成した。
本バージョンは宣言バージョン（★クリーンアップ）であり、以下を実施する:

1. `cargo clean` でビルドキャッシュを削除
2. `Cargo.toml` バージョンを `95.0.0` に更新
3. `CHANGELOG.md` / `MILESTONE.md` / `README.md` / `versions/current.md` を更新
4. `driver.rs` 内の `cargo_toml_version_is_94_0_0` テストを `95.0.0` に更新し、`v95000_tests` 4 件を追加
5. SAP Advanced Era 全 4 スプリントのロードマップを「完了」にマーク

**宣言文**:
> 「`ctx.sap.batch(req)` で複数 SAP エンティティをまとめて更新できる。
>  `QueryBuilder<T>` で型安全なクエリを組み立て、`fetch_all_pages` で全件自動取得できる。
>  `fav infer --sap-metadata` で SAP の型定義が自動生成される。
>  Lambda SnapStart でコールドスタートは 93% 削減される。
>  それが、Favnir SAP Advanced 1.0 である。」

## Goals

1. `Cargo.toml` バージョンを `95.0.0` に更新する
2. `driver.rs` の `cargo_toml_version_is_94_0_0` テストを `cargo_toml_version_is_95_0_0` に更新する
3. `driver.rs` に `v95000_tests` 4 件を追加する
4. `CHANGELOG.md` に v95.0.0 エントリを追加する
5. `MILESTONE.md` に v95.0.0「SAP Advanced 1.0」エントリを追加する
6. `README.md` に v95.0「SAP Advanced 1.0」セクションを追加する
7. `versions/current.md` を v95.0.0 に更新する
8. SAP Advanced Era ロードマップ 5 ファイルを「完了」にマークする
9. `cargo clean` を実施する

## Success Criteria

1. `cargo test` で 4,164 tests（+4）、0 failures
2. `Cargo.toml` に `version = "95.0.0"` が含まれる
3. `CHANGELOG.md` に `v95.0.0` が含まれる
4. `MILESTONE.md` に `SAP Advanced` が含まれる
5. `README.md` に `SAP Advanced` が含まれる
6. `cargo clippy --locked -- -D warnings` pass
7. `fav fmt --check` pass（compiler.fav / checker.fav）

## v95000_tests（4 件）

```rust
fn cargo_toml_version_is_95_0_0()  // Cargo.toml に "version = \"95.0.0\"" が含まれる
fn changelog_has_v95_0_0()         // CHANGELOG.md に "v95.0.0" が含まれる
fn milestone_has_sap_advanced()    // MILESTONE.md に "SAP Advanced" が含まれる
fn readme_mentions_sap_advanced()  // README.md に "SAP Advanced" が含まれる
```

## Files to Modify

| ファイル | 操作 | 内容 |
|---|---|---|
| `fav/Cargo.toml` | **更新** | `version = "94.0.0"` → `version = "95.0.0"` |
| `fav/src/driver.rs` | **更新** | `cargo_toml_version_is_94_0_0` テスト更新 + `v95000_tests` 4 件追加 |
| `CHANGELOG.md` | **更新** | v95.0.0 エントリ追加 |
| `MILESTONE.md` | **更新** | v95.0.0「SAP Advanced 1.0」エントリ追加 |
| `README.md` | **更新** | v95.0「SAP Advanced 1.0」セクション追加 |
| `versions/current.md` | **更新** | v95.0.0 に更新 |
| `versions/roadmap/roadmap-v90.1-v91.0.md` | **更新** | Status: 完了 に更新 |
| `versions/roadmap/roadmap-v91.1-v92.0.md` | **更新** | Status: 完了 に更新 |
| `versions/roadmap/roadmap-v92.1-v93.0.md` | **更新** | Status: 完了 に更新 |
| `versions/roadmap/roadmap-v93.1-v94.0.md` | **更新** | Status: 完了 に更新 |
| `versions/roadmap/roadmap-v94.1-v95.0.md` | **更新** | Status: 完了 に更新 |
