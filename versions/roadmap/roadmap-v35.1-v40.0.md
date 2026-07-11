# Favnir Master Roadmap — v35.1 〜 v40.0

Date: 2026-07-06
Status: 計画中（v35.0.0 完了時点）

---

## 背景と方針

v35.0.0「Production Ready」の宣言をもって、Favnir は以下を達成した:

```
v31.0 — Real-World Readiness  : 「実案件で .fav が動く」         ✓
v32.0 — Language Polish       : 「書いたときが気持ちいい」         ✓
v33.0 — Language Power        : 「型で設計できる」               ✓
v34.0 — Performance & Tooling : 「本番で速い」                  ✓
v35.0 — Production Ready      : 「実案件で Favnir を選べる」      ✓
```

v35.0 宣言時の「Lambda にデプロイして実データを処理できる」は、
`fav build --target native` 生成バイナリを **手動** でデプロイする形での達成である。
v35.1〜v36.0 の Deployment Story では、これを **`fav deploy` CLI による自動化** に引き上げる。

```
v36.0 — Deployment Story      : 「どこにでも自動デプロイできる」
v37.0 — Data Quality First    : 「データ品質を型で保証できる」
v38.0 — Multi-Source ETL Power: 「複数ソースを型安全につなげる」
v39.0 — Intelligence & Assist : 「AI がパイプラインを補助する」
v40.0 — Enterprise Governance : 「チームで安全に運用できる」
```

---

## バージョン命名規則

| 種別 | 意味 |
|---|---|
| **x.0.0** — マイルストーン宣言版 | 直前の x-1.1〜x-1.9 の成果を宣言 + **ビルドクリーンアップ実施（必須・例外なし）** |
| **x.1〜x.9** — 実装版 | 1 バージョン 1 テーマで実装（クリーンアップ不要） |

---

## クリーンアップ規約

> **ルール: ★クリーンアップは本スプリントの x.0.0 全件（v36.0 / v37.0 / v38.0 / v39.0 / v40.0 の 5 件すべて）で必ず実施する。例外はない。**

### クリーンアップ手順

以下を **CI ジョブ `cleanup`** として実行するか、ローカルで同等のコマンドを順に実行する。
作業ディレクトリは `favnir/fav`。

| ステップ | CI コマンド | 合否判定 |
|---|---|---|
| 1. ビルドクリーン | `cargo clean && cargo build --locked` | exit 0 |
| 2. テスト全通過 | `cargo test --locked` | 0 failures |
| 3. Clippy クリーン | `cargo clippy --locked -- -D warnings` | exit 0 |
| 4. fmt 確認 | `cargo run --bin fav -- fmt --check self/compiler.fav self/checker.fav` | exit 0 |
| 5. lint (compiler) | `cargo run --bin fav -- lint --deny-warnings --allow W017 --allow W018 --allow W019 self/compiler.fav` | exit 0 |
| 6. lint (checker) | `cargo run --bin fav -- lint --deny-warnings --allow W012 --allow W017 --allow W018 --allow W019 self/checker.fav` | exit 0 |
| 7. ベンチ記録 | `benchmarks/vX.0.0.json` にスナップショット JSON を作成 | ファイルが存在する |

Windows (PowerShell) では各コマンドを個別実行し、都度 `if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }` で失敗を確認すること。

`X.0.0` は実際のバージョン番号（36.0.0 / 37.0.0 / 38.0.0 / 39.0.0 / 40.0.0）で置き換える。

---

## スプリント概要

### v35.1〜v36.0 — Deployment Story

**テーマ**: 「`fav deploy` CLI で Lambda / Docker / k8s に自動デプロイできる」

詳細: `versions/roadmap/roadmap-v35.1-v36.0.md`

| バージョン | テーマ |
|---|---|
| v35.1 | `fav deploy --target lambda` パッケージング + デプロイ |
| v35.2 | `fav deploy --target docker` イメージ生成 |
| v35.3 | `fav ci init` — GitHub Actions ワークフロー自動生成 |
| v35.4 | `fav deploy --target k8s` Manifest 生成 |
| v35.5 | `deploy.fav` 宣言的デプロイ設定 |
| v35.6 | `fav deploy status` + `fav rollback` |
| v35.7 | `fav deploy --dry-run` + 差分確認 |
| v35.8 | デプロイ cookbook 3 本 + ドキュメント |
| v35.9 | v36.0 前調整・安定化 |
| **v36.0** ★ | **Deployment Story マイルストーン宣言 + クリーンアップ** |

---

### v36.1〜v37.0 — Data Quality First

**テーマ**: 「データ品質を型で保証できる」

詳細: `versions/roadmap/roadmap-v36.1-v37.0.md`

| バージョン | テーマ |
|---|---|
| v36.1 | `schema` リテラル定義構文（v32.4 基盤を拡張） |
| v36.2 | `expect` ブロック — 品質ルール宣言 |
| v36.3 | W025 `schema_mismatch` lint ルール |
| v36.4 | `fav validate` コマンド（CSV / Parquet 検証） |
| v36.5 | Data Contract 規約 + `fav contract check` |
| v36.6 | E0380〜E0384 スキーマ不整合エラーコード |
| v36.7 | Great Expectations 互換エクスポート |
| v36.8 | `fav schema diff` — スキーマ進化追跡 |
| v36.9 | v37.0 前調整・安定化 |
| **v37.0** ★ | **Data Quality First マイルストーン宣言 + クリーンアップ** |

---

### v37.1〜v38.0 — Multi-Source ETL Power

**テーマ**: 「複数ソースを型安全につなげる」

> v32.1 実装済みの境界付きジェネリクス・v32.2 実装済みの行多相を実用強化。`join` / CDC / lineage graph 追加。

詳細: `versions/roadmap/roadmap-v37.1-v38.0.md`

| バージョン | テーマ |
|---|---|
| v37.1 | 境界付きジェネリクス強化（Serialize / Deserialize + Generic Rune） |
| v37.2 | 行多相強化（ネスト行型・Spread 演算子） |
| v37.3 | `join` ステージ演算子（hash join） |
| v37.4 | `fan_out` / `fan_in` パターン |
| v37.5 | CDC Rune（Debezium JSON） |
| v37.6 | `fav lineage --graph`（DOT / SVG 出力） |
| v37.7 | `fav new --template multi-source` |
| v37.8 | Multi-Source cookbook 5 本 |
| v37.9 | v38.0 前調整・安定化 |
| **v38.0** ★ | **Multi-Source ETL Power マイルストーン宣言 + クリーンアップ** |

---

### v38.1〜v39.0 — Intelligence & Assistance

**テーマ**: 「AI がパイプラインを補助する」

> v9.6 実装済みの Llm Rune を基盤に `fav suggest` / `fav generate` / LSP AI 補完を追加。

詳細: `versions/roadmap/roadmap-v38.1-v39.0.md`

| バージョン | テーマ |
|---|---|
| v38.1 | `fav suggest` — エラーから修正案を LLM 生成 |
| v38.2 | `fav generate --from sql` — SQL → Favnir 変換 |
| v38.3 | `fav generate --from csv` 型推論強化（schema + expect 出力） |
| v38.4 | LSP AI 補完（rerank オプション） |
| v38.5 | `fav explain --verbose` LLM 拡張 |
| v38.6 | `fav new --template rag-pipeline` |
| v38.7 | Llm Rune 強化（streaming / function calling / embeddings） |
| v38.8 | AI 支援 cookbook 3 本 |
| v38.9 | v39.0 前調整・安定化 |
| **v39.0** ★ | **Intelligence & Assistance マイルストーン宣言 + クリーンアップ** |

---

### v39.1〜v40.0 — Enterprise Governance

**テーマ**: 「チームで安全に運用できる」

詳細: `versions/roadmap/roadmap-v39.1-v40.0.md`

| バージョン | テーマ |
|---|---|
| v39.1 | RBAC Rune（Auth.require_role / check_permission） |
| v39.2 | Audit Log Rune（パイプライン実行記録） |
| v39.3 | `fav policy` — 組織ポリシー宣言的定義 |
| v39.4 | Secret Rune 強化（Vault / AWS / GCP） |
| v39.5 | マルチテナント対応（ctx.tenant_id） |
| v39.6 | `fav audit` — 依存ライセンス・CVE 報告 |
| v39.7 | `fav policy check --ci` — CI ポリシーゲート |
| v39.8 | Enterprise cookbook 3 本 + ガバナンスドキュメント |
| v39.9 | v40.0 前調整・安定化 + 全スプリント振り返り |
| **v40.0** ★ | **Enterprise Governance マイルストーン宣言 + クリーンアップ** |

---

## v40.0 完了基準（暫定）

| コンポーネント | 完了基準 |
|---|---|
| デプロイ | `fav deploy --target lambda/docker/k8s` が動作する |
| データ品質 | `fav validate` + `expect` ブロックが型検査と統合されている |
| マルチソース | `join` ステージが動き、境界付きジェネリクス・行多相が実用強化済み |
| AI 支援 | `fav suggest` / `fav generate --from sql` が実用的に動く |
| ガバナンス | RBAC・Audit Log・Policy・Secret 強化・Multitenancy が揃っている |
| テスト | テスト数 5000+（`cargo test --locked` 0 failures） |
| 未解決バグ | GitHub Issues の `P1` / `P2` ラベル付きオープンバグが **0 件** |

---

## 参考リンク

- 前マスタースケジュール: `versions/roadmap/roadmap-v30.1-v35.0.md`
- 達成宣言: `MILESTONE.md`
- 現バージョン: `versions/current.md`
