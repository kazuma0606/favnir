# Roadmap v37.1.0 〜 v38.0.0 — Multi-Source ETL Power

Date: 2026-07-06
Status: 骨格確定（v35.0 完了時点）、詳細は v37.0 完了後に確定

---

## 目標

v37.0「Data Quality First」で「データ品質を型で保証できる」を実現した。
このフェーズは **「複数ソースを型安全につなげる」** を実現する。

**前版との関係**:
- v32.1: 境界付きジェネリクス `T with Ord/Eq/Hash` — 基本実装済み ✓
- v32.2: 行多相 `R with { id: Int }` — 基本実装済み ✓
- v37.x: 上記の実用強化（新制約 / ネスト型 / Generic Rune / Spread 演算子）+ `join` / CDC / lineage 追加

---

## バージョン計画

### v37.1.0 — 境界付きジェネリクス実用強化 ✅

v32.1 実装（`T with Ord/Eq/Hash`）に `Serialize` / `Deserialize` 制約と Generic Rune 対応を追加。

**完了条件**: 新制約が型チェックを通る / Rust テスト 4 件（実装: 4 件、2707 tests pass）

---

### v37.2.0 — 行多相実用強化 ✅

v32.2 実装にネスト行型 `R with { address: { city: String, .. }, .. }` と
レコード Spread `{ ...r, extra: True }` を追加。

**完了条件**: 複数フィールド行制約が call-site 型チェックを通る / ネスト行型がパースを通る
（RecordSpread は v16.3.0 実装済み・ネスト行型完全型チェックは v37.3 へ）/ Rust テスト 4 件（実装: 4 件、2711 tests pass）

---

### v37.3.0 — `join` ステージ演算子 ✅

**実装スコープ（縮小）**: `join ... on ...` キーワード構文は v37.4.0 以降に持ち越し。
`List.join_on(left, right, pred)` VM ビルトイン（left semi-join）として実装。

**完了条件（達成済み）**:
- `List.join_on(left, right, pred)` VM ビルトイン追加（left semi-join）
- `checker.rs` に `("List", "join_on")` 戻り型定義追加
- Rust テスト 3 件（`cargo_toml_version_is_37_3_0` / `changelog_has_v37_3_0` / `list_join_on_basic`）
- 2715 tests passed, 0 failed

---

### v37.4.0 — `fan_out` / `fan_in` ✅

**実装スコープ（縮小）**: `fan_out ... | ...` キーワード構文は v37.5.0 以降に持ち越し。
`List.fan_out(list, n)` / `List.fan_in(lists)` VM ビルトインとして実装。

**完了条件（達成済み）**:
- `List.fan_out(list, n)` VM ビルトイン追加（リストを n チャンクに分割）
- `List.fan_in(lists)` VM ビルトイン追加（List<List> を 1 レベルフラット化）
- `checker.rs` に両関数の戻り型定義追加
- Rust テスト 4 件（meta 2 件 + 機能 2 件）
- 2719 tests passed, 0 failed

---

### v37.5.0 — CDC Rune ✅

- `runes/cdc/cdc.fav` — Debezium JSON 形式の CDC イベント処理
- MySQL / Postgres 対応

**完了条件（達成済み）**:
- `CDC.extract_op` / `CDC.op_name` / `CDC.is_insert` / `CDC.is_update` / `CDC.is_delete` 実装
- `CDC.filter_inserts` / `CDC.filter_deletes` 実装
- Rust テスト 4 件（meta 2 件 + 機能 2 件）
- 2723 tests passed, 0 failed

---

### v37.6.0 — `fav lineage --graph` ✅

`fav lineage --graph --format dot/svg` でリネージグラフを出力する。

**完了条件（達成済み）**:
- `render_lineage_dot` — Graphviz DOT 形式のリネージグラフ出力
- `render_lineage_svg` — 外部ツール不要のインライン SVG 出力
- `fav explain --lineage --format dot/svg` でアクセス可能
- Rust テスト 4 件（meta 2 件 + 機能 2 件）
- 2727 tests passed, 0 failed

---

### v37.7.0 — `fav new --template multi-source` ✅

マルチソース ETL プロジェクトテンプレート追加。

**完了条件（達成済み）**:
- `create_multi_source_etl_project` — Postgres + CSV 結合 ETL テンプレート生成（6 ファイル）
- `TEMPLATE_GALLERY` に `"multi-source"` エントリ追加（6 エントリ）
- `cmd_new_list` に `"data-contract"` / `"multi-source"` 行追加
- Rust テスト 3 件（meta 2 件 + 機能 1 件）
- 2730 tests passed, 0 failed

---

### v37.8.0 — Multi-Source cookbook 5 本 ✅

- `site/content/cookbook/join-two-tables.mdx`
- `site/content/cookbook/cdc-postgres-to-warehouse.mdx`
- `site/content/cookbook/fan-out-by-region.mdx`
- `site/content/cookbook/generic-etl-function.mdx`
- `site/content/cookbook/lineage-visualization.mdx`

**完了条件（達成済み）**:
- 5 ファイルが存在し各キーワードを含む
- Rust テスト 3 件（meta 2 件 + 機能 1 件）
- 2733 tests passed, 0 failed

---

### v37.9.0 — v38.0 前調整・安定化 ✅

- `render_lineage_text` にサマリー行追加（`Total: N stage(s), M pipeline(s)`）
- `site/content/docs/multi-source-etl.mdx` — v37.x 系 Multi-Source ETL 機能一覧ドキュメント

**完了条件（達成済み）**:
- 2 機能実装
- Rust テスト 4 件（meta 2 件 + 機能 2 件）
- 2737 tests passed, 0 failed

---

### v38.0.0 — Multi-Source ETL Power マイルストーン宣言 ★クリーンアップ ✅

**完了条件（達成済み）**:
- v37.1〜v37.9 の全機能が動作する / テスト数 ≥ 2737 ✓（実績 2741）
- GitHub Issues の P1/P2 ラベル付きオープンバグが 0 件（OSS 公開前のため対象外）
- `★クリーンアップ` 完了（cargo clean 26.4 GiB 削除）
- Rust テスト 4 件（meta 2 件 + マイルストーン確認 2 件）
- 2741 tests passed, 0 failed

---

## 参考リンク

- マスタースケジュール: `versions/roadmap/roadmap-v35.1-v40.0.md`
- 前サブスプリント: `versions/roadmap/roadmap-v36.1-v37.0.md`
- 次サブスプリント: `versions/roadmap/roadmap-v38.1-v39.0.md`
