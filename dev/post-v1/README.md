# Post-v1 設計ドキュメント

Favnir v1.1.0 〜 v2.0.0 に向けた設計・思想・アイデアのドキュメント群。

---

## ディレクトリ構成

### `roadmap/` — ロードマップ直結の実装仕様

バージョンごとの実装仕様。`versions/roadmap-v2.md` と直接対応している。

| ファイル | 対応バージョン | 内容 |
|---|---|---|
| `fav-abstraction-system.md` | v1.1.0 | `interface` / `impl` / `with` 設計 |
| `fav-algebraic-structures.md` | v1.1.0 | `Field` / `Ring` 代数構造 |
| `fav-standard-states.md` | v1.2.0 | `PosInt` / `Email` など標準 State 型 |
| `fav-db-schema-integration.md` | v1.2.0 | DB スキーマへの invariant マッピング |
| `fav-abstract-flw.md` | v1.3.0 | `abstract stage` / `abstract seq` 設計 |
| `fav-explain-bundle.md` | v1.4.0 | `fav explain --format json` / `fav bundle` |
| `favnir-graph-explain.md` | v1.4.0 | Data Lineage グラフ出力 |
| `stat-rune-architecture.md` | v1.5.0 | `Stat.*` ルーン統合設計 |
| `favnir-ergonomics-random-sample-binding.md` | v1.5.0 | `bind` / `match` 局所束縛（§1–4 有効） |
| `validate-rune-architecture.md` | v1.6.0 | `validate.field` / `flow` / `db` 設計 |
| `validate-stat-integration.md` | v1.6.0 | validate + stat の連携パターン |
| `validate-stat-favnir-style-examples.md` | v1.6.0 | validate + stat のコード例 |
| `favnir-async.md` | v1.7.0 | `Task<T>` 非同期モデル設計 |
| `favnir-concurrency.md` | v1.7.0 | 非同期・並列・コルーチンの整理 |
| `fav-sss-architecture.md` | v2.0.0 | SSS アーキテクチャ（リネームの根拠） |
| `favnir-selfhost-plan.md` | v2.0.0 | セルフホスト戦略 |
| `fav-error-code-system.md` | v2.0.0 | エラーコード体系 E0100– |
| `favnir-post1-roadmap.md` | 全体 | Phase A–E の製品ロードマップ |

### `ideas/` — 将来検討・言語設計メモ

roadmap に未確定だが、将来採用候補の設計案。

| ファイル | 内容 |
|---|---|
| `fav-safe-cast.md` | 安全なキャスト設計 |
| `fav-null-safety-and-option-ergonomics.md` | Option の利便性・null 安全設計 |
| `fav-type-inference.md` | 型推論の方針 |
| `favnir-open-questions.md` | 未決定の設計問題一覧 |
| `favnir-next-candidates.md` | 次バージョン候補の機能リスト |
| `forge-syntax.md` | Forge との構文比較 |

### `vision/` — 思想・規約・プロジェクト方針

Favnir の設計哲学と長期的な方向性。実装の優先度は低い。

| ファイル | 内容 |
|---|---|
| `fav-manifesto.md` | Favnir の設計原則 |
| `fav-coc-vision.md` | Convention over Configuration ビジョン |
| `fav-project-management.md` | プロジェクト管理の方針 |
| `fav-directory-convention.md` | ディレクトリ構成規約 |

---

## 参照先

- ロードマップ全体: `versions/roadmap-v2.md`
- 各バージョン仕様: `versions/v1.x.0/spec.md`
