# v37.0.0 実装計画 — Data Quality First マイルストーン宣言

## 変更ファイル一覧

| ファイル | 変更種別 | 内容 |
|---|---|---|
| `MILESTONE.md` | 追記 | v37.0 Data Quality First 宣言セクションをファイル先頭に追加 |
| `README.md` | 変更 | v35.0 宣言の後に v36.0 / v37.0 マイルストーン宣言を追加 |
| `CHANGELOG.md` | 追記 | `[v37.0.0]` エントリ追加 |
| `fav/src/driver.rs` | 変更 | `v36900_tests` スタブ化 / `v37000_tests` 追加 |
| `fav/Cargo.toml` | 更新 | `version = "36.9.0"` → `"37.0.0"` |
| `versions/current.md` | 更新 | 最新安定版 v37.0.0、次バージョン v37.1.0 |
| `versions/roadmap/roadmap-v36.1-v37.0.md` | 更新 | v37.0.0 完了済みにマーク（✅） |

## 実装順序

### Step 1: CHANGELOG.md に [v37.0.0] エントリ追加

`## [v36.9.0]` の `---` セパレータ直後に挿入。

### Step 2: ★クリーンアップ — cargo clean

```bash
cargo clean
```

x.0.0 マイルストーン規約。Step 3 以降の変更前に実施する。

### Step 3: MILESTONE.md に v36.0 / v37.0 セクション追加

**注意**: 現在の MILESTONE.md 先頭は `# Favnir Milestones` → `## v35.0.0` の順になっており、
v36.0.0 の先頭セクションが存在しない。v37.0.0 を挿入する前に v36.0.0 セクションも追加し、
MILESTONE.md 先頭を `v37.0.0` → `v36.0.0` → `v35.0.0` の順にする。

`# Favnir Milestones` ヘッダの直後（`## v35.0.0` の直前）に以下を追加:

```markdown
## v37.0.0 — Data Quality First（YYYY-MM-DD）

> 「`schema` でテーブル/列の型と制約を宣言し、
>  `expect` でビジネスルールをパイプラインに埋め込み、
>  `fav validate` でデータを検証できる。
>  スキーマ不整合は W025 lint で静的に検出され、
>  違反は E0380〜E0384 として報告される。
>  `fav schema diff` で変更の後方互換性を即座に把握できる。
>
>  これが Favnir v37.0 — Data Quality First の姿である。」

v37.0.0 をもって、Favnir の **Data Quality First** を正式に宣言する。

### 達成コンポーネント（v36.1〜v36.9）

| コンポーネント | バージョン | 内容 |
|---|---|---|
| schema 定義構文 | v36.1 | `schema Orders { id: Int, ... }` インライン定義 |
| expect ブロック | v36.2 | `expect rows { not_empty, all(...) }` ビジネスルール宣言 |
| W025 lint | v36.3 | `schema_mismatch` — 静的フィールドアクセス検証 |
| fav validate | v36.4 | `fav validate --schema orders.fav data.csv` |
| Data Contract | v36.5 | `contracts/` 規約 + `fav contract check` |
| E0380〜E0384 | v36.6 | スキーマ不整合エラーカタログ |
| GE エクスポート | v36.7 | `--export ge` — Great Expectations 互換出力 |
| fav schema diff | v36.8 | フィールドレベル差分・後方互換性チェック |
| 安定化 | v36.9 | W025↔E0380 連携・validate サマリー・docs 統合 |

**宣言日**: YYYY-MM-DD
**宣言バージョン**: v37.0.0

---

## v36.0.0 — Deployment Story（2026-07-08）

> 「`fav deploy --target lambda` で Lambda に自動デプロイし、
>  `fav deploy --target docker` で Docker イメージを生成し、
>  `fav ci init` で GitHub Actions CI を自動設定できる。
>  `!Effect` 廃止（v35.4〜v35.8）により、すべての API が ctx: AppCtx ベースに統一された。
>
>  これが Favnir v36.0 — Deployment Story の姿である。」

v36.0.0 をもって、Favnir の **Deployment Story** を正式に宣言する。

**宣言日**: 2026-07-08
**宣言バージョン**: v36.0.0

---
```

### Step 4: README.md — v36.0 / v37.0 宣言追加

`**v35.0（2026-07-04）で、[Production Ready](./MILESTONE.md) マイルストーンを宣言しました。**` の後に追加:

```markdown
**v36.0（2026-07-08）で、[Deployment Story](./MILESTONE.md) マイルストーンを宣言しました。**
`fav deploy --target lambda/docker` / `fav ci init` / ctx 構文統一（`!Effect` 廃止）が揃い、Lambda 本番デプロイと GitHub Actions CI が自動化されました。
**v37.0（2026-07-09）で、[Data Quality First](./MILESTONE.md) マイルストーンを宣言しました。**
`schema` 型定義 / `expect` 品質ルール / `fav validate` / W025 lint / E0380〜E0384 / GE エクスポート / `fav schema diff` が揃い、型でデータ品質を保証できる状態になりました。
```

### Step 5: driver.rs — `v36900_tests::cargo_toml_version_is_36_9_0` スタブ化

ライブアサーション → `// Stubbed: version bumped to 37.0.0` に変更。

### Step 6: driver.rs — `v37000_tests` モジュール追加

`v36900_tests` の閉じ `}` の行番号を Read で特定してから Edit を実行する。

### Step 7: Cargo.toml バージョン更新

Step 3〜6 完了後に `36.9.0` → `37.0.0` に更新。

## 依存関係

- `cargo clean` は Step 1 完了後、Step 3 より前に実施（Step 2）
- `cargo clean` 後は `cargo build` でコンパイルエラーがないことを確認してから続行する
- `v37000_tests` は `use super::*` 不要（`include_str!` のみ使用）
- `MILESTONE.md` と `README.md` の更新は独立しており並列実施可能（Step 3/4）
- `include_str!` パス: `Cargo.toml` → `"../Cargo.toml"` / `MILESTONE.md` / `README.md` → `"../../MILESTONE.md"` / `"../../README.md"`

## リスク

| リスク | 対処 |
|---|---|
| `cargo clean` 後に `fav/tmp/hello.fav` が消失して `bootstrap_c2_artifact_roundtrip` が FAIL | T0 で `hello.fav` の存在と内容を確認し、clean 後に復元する |
| MILESTONE.md の挿入位置が誤り先頭セクションを壊す | Read で行番号を特定してから Edit を実行 |
| README.md の挿入位置が誤り既存の v35.0 宣言と重複 | `v35.0（2026-07-04）` 行の直後に挿入することを確認 |
