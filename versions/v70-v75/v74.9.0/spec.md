# v74.9.0 仕様書 — 安定化・コードフリーズ（Favnir 2.0 前最終調整）

Date: 2026-08-14

---

## Background

v70.1〜v74.8 で実装した全機能を通しで確認する最終安定化スプリント。
v74.x スプリント（9 バージョン）が CHANGELOG に揃っていること、
ショーケースデモ（`infra/e2e-demo/favnir2-showcase/`）が構造的に完走可能であることを
Rust テストで保証し、Favnir 2.0 宣言（v75.0.0）へのコードフリーズを完了させる。

新規機能の追加は行わない。テストによる安定性保証が本バージョンの唯一の目的。

---

## Goals

1. `v749000_tests` モジュール（2 件）を `driver.rs` に追加する
   - `favnir2_full_sprint_all_stable` — v74.1〜v74.8 の全エントリが CHANGELOG に存在することを確認
   - `favnir2_e2e_showcase_runs` — ショーケースの全ファイルが揃い主要要素を持つことを確認

---

## テスト仕様

### `favnir2_full_sprint_all_stable`

`include_str!("../../CHANGELOG.md")` を使用し、以下の全バージョンエントリが存在することを assert:
- `[v74.1.0]` / `[v74.2.0]` / `[v74.3.0]` / `[v74.4.0]`
- `[v74.5.0]` / `[v74.6.0]` / `[v74.7.0]` / `[v74.8.0]`

### `favnir2_e2e_showcase_runs`

ショーケースの主要ファイルを `include_str!` で確認:
- `pipeline.fav` に `Result.ok` / `import rune` / `ShowcaseContract` が含まれる
- `fav.toml` に `schedule` / `tenant` が含まれる
- `contract.fav` に `ShowcaseInputContract` が含まれる
- `quality.fav` / `rune.toml` は内容が最低限（存在確認は v74.8.0 の showcase_demo_structure_complete で実施済み）のため本テストでは対象外

**パス:** `include_str!("../../CHANGELOG.md")` はルートの `favnir/CHANGELOG.md`。
ショーケースは `include_str!("../../infra/e2e-demo/favnir2-showcase/...")` でアクセス。

---

## Success Criteria

1. `favnir2_full_sprint_all_stable` テストが pass する
2. `favnir2_e2e_showcase_runs` テストが pass する
3. `cargo test` で 3688 tests pass（0 failures）

---

## スコープ外（明示的除外）

- 新規機能・新規構造体・新規関数の追加（安定化スプリントのため不要）
- `fav run pipeline.fav` の実際の実行（後続フェーズで対応）
- CI 自動実行パイプラインの構築（後続フェーズで対応）
- GitHub Actions CI のグリーン確認（v75.0.0 宣言時に実施）
- `site/` MDX 追加（v75.0.0 または後続フェーズで対応）
- MILESTONE.md 更新（宣言バージョン v75.0.0 で実施）

---

## Error Codes

新規エラーコードなし

---

## Files to Modify / Create

| ファイル | 変更内容 |
|---|---|
| `fav/src/driver.rs` | `v749000_tests` 追加 |
| `fav/Cargo.toml` | `version = "74.9.0"` に更新 |
| `CHANGELOG.md` | v74.9.0 エントリを先頭に追加 |
| `versions/current.md` | 進行中バージョン・次に切る版を更新 |
