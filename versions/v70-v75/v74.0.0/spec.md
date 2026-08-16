# v74.0.0 仕様書 — Production Proven 宣言 ★クリーンアップ

Date: 2026-08-13

---

## Background

v73.1〜v73.9 で実装した以下の機能群が本番運用レベルに達したことを宣言するマイルストーンバージョン。

| バージョン | 主な機能 |
|---|---|
| v73.1.0 | データコントラクト（スキーマ境界の型安全保証） |
| v73.2.0 | 品質スコア（データ劣化の警告） |
| v73.3.0 | PII 検出・マスキング（型での個人情報保護） |
| v73.4.0 | 監査ログ + OpenLineage（法的要件を満たすリネージ追跡） |
| v73.5.0 | SLA 監視 + アラート統合 |
| v73.6.0 | Rune 品質パス（linalg / stats VM primitive） |
| v73.7.0 | ドッグフーディング Sprint（Favnir が Favnir を運用） |
| v73.8.0 | GitHub Actions 公式 Action（CI に溶け込む） |
| v73.9.0 | 安定化・コードフリーズ |

---

## 宣言文

> 「データコントラクトがスキーマ境界を守り、品質スコアが劣化を警告する。
>  PII が型で保護され、監査ログが法的要件を満たす。
>  Favnir が Favnir 自身を運用し、GitHub Action が CI に溶け込む。
>
>  これが Favnir v74.0 — Production Proven の姿である。」

---

## Goals

1. `cargo clean` でビルドキャッシュをクリーンアップ
2. `Cargo.toml` バージョンを `74.0.0` に更新
3. `CHANGELOG.md` に v74.0.0 エントリを追加
4. `MILESTONE.md` に「Production Proven」マイルストーンを追記
5. `README.md` に v74.0 達成を追記
6. `v74000_tests` モジュール（4 件）を `driver.rs` に追加
7. `versions/current.md` を更新（進行中: v74.0.0 / 次: v74.1.0 — 宣言完了後は v74.1.0 スプリントへ移行）

---

## Success Criteria

1. `cargo_toml_version_is_74_0_0` — Cargo.toml に `version = "74.0.0"` が存在する
2. `changelog_has_v74_0_0` — CHANGELOG.md に `[v74.0.0]` エントリが存在する
3. `milestone_has_production_proven` — MILESTONE.md に「Production Proven」が存在する
4. `readme_mentions_production_proven` — README.md に「Production Proven」が存在する
5. `cargo test` で 3669 tests pass（0 failures）

---

## 実装上の注意

- `v74000_tests` モジュールは宣言ファイル（CHANGELOG.md / MILESTONE.md / README.md / Cargo.toml）の存在確認のみを行うため、`use super::*` は不要（外部シンボル未参照 — 宣言バージョンの共通パターン）

---

## Error Codes

新規エラーコードなし（宣言・クリーンアップのみ）

---

## Files to Modify

| ファイル | 変更内容 |
|---|---|
| `fav/Cargo.toml` | `version = "74.0.0"` に更新 |
| `fav/src/driver.rs` | `v74000_tests` モジュール 4 件追加 + バージョン文字列を `74.0.0` に更新 |
| `CHANGELOG.md` | v74.0.0 宣言エントリを先頭に追加 |
| `MILESTONE.md` | 「Production Proven」を追記 |
| `README.md` | v74.0 達成を追記 |
| `versions/current.md` | 進行中バージョン → v74.0.0、次 → v74.1.0 |
