# v45.0.0 Spec — Precision & Flow 宣言 ★クリーンアップ

## 概要

v44.1〜v44.9 で構築した Refinement type × Streaming / CEP × Refinement type / Stream join × Opaque type / 型推論 × Lineage / Back-pressure × Policy / E2E デモ / ドキュメント / ベンチマーク追跡 / 安定化の成果を **マイルストーンとして正式宣言** するリリース。

新規機能の追加は一切行わない（宣言・クリーンアップ専用版）。

---

## 宣言文

> 「型推論がジェネリクスと戻り値型を補完し、最小限の注釈で安全なコードが書ける。
>  ウィンドウ集計・CEP・Stream join が型安全に記述でき、
>  refinement type と opaque type がデータの意味を型で守る。
>
>  これが Favnir v45.0 — Precision & Flow の姿である。」

---

## 成果物

### 1. `MILESTONE.md` 更新

`v45.0.0 — Precision & Flow` セクションを `# Favnir Milestones` タイトル行の直後（`## v44.0.0 — Language Expressiveness` の直前）に追加。
- 宣言文（`>` 引用ブロック）
- 達成コンポーネント一覧テーブル（v44.1〜v44.9 全 9 件）
- 宣言日: 2026-07-15
- `"Precision & Flow"` を含む

### 2. `README.md` 更新

`v44.0（2026-07-13）` 記述行の直後に `v45.0 — Precision & Flow` 言及を追加。
- `"Precision & Flow"` および `"v45.0"` の両方を含む

### 3. `CHANGELOG.md` 更新

v45.0.0 エントリ追加（`[v45.0.0]` を含む）。

### 4. `Cargo.toml` バンプ

`44.9.0` → `45.0.0`

### 5. `★クリーンアップ`

`cargo clean` 実行（ビルドアーティファクト削除）。

---

## テスト

`v45000_tests` 4 件:

| テスト名 | 内容 |
|---|---|
| `cargo_toml_version_is_45_0_0` | `Cargo.toml` に `"45.0.0"` が含まれる |
| `changelog_has_v45_0_0` | `CHANGELOG.md` に `"[v45.0.0]"` が含まれる |
| `milestone_has_precision_and_flow` | `MILESTONE.md` に `"Precision & Flow"` が含まれる |
| `readme_mentions_precision_and_flow` | `README.md` に `"Precision & Flow"` または `"v45.0"` が含まれる（テストは OR 条件。実装では両方を含む文字列を挿入するため実際は両条件を満たす） |

スタブ化: `v44900_tests::cargo_toml_version_is_44_9_0` の `assert!` 行を削除し `// Stubbed: version bumped to 45.0.0 in v45.0.0.` に置き換える。

---

## 完了条件

- `cargo test -j 8 -- --test-threads=8` で **2966 passed; 0 failed**（2962 + 4）
- `v45000_tests` 4 件 pass
- `MILESTONE.md` に `"Precision & Flow"` が含まれる
- `cargo clean` 完了

---

## 注意事項

- コードフリーズ: 新規 Rust 機能・AST 変更・新規ヘルパー関数は追加しない
- `v44900_tests` に `cargo_toml_version_is_44_9_0` が存在するためスタブ化必須
- `cargo clean` はテスト完了後に実施（テスト再実行なし）
- `MILESTONE.md` 挿入位置: `# Favnir Milestones` H1 タイトルの直後、`## v44.0.0` H2 の直前
