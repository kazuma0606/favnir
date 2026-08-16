# v76.5.0 タスクリスト — `fav lineage graph` 可視化

Date: 2026-08-15
Status: COMPLETE

---

## T0: 事前確認

- [x] `fav/Cargo.toml` のバージョンが `76.4.0` であることを確認
- [x] `cargo test` が全 pass（3722 tests）であることを確認（v76.5.0 テスト追加前の状態）
- [x] `fav/tmp/hello.fav` が存在することを確認（bootstrap テスト要件）

---

## T1: driver.rs — 型・関数追加

- [x] `fav/src/driver.rs` の末尾に `// --- v76.5.0: fav lineage graph 可視化 ---` コメントを追加する
- [x] `LineageNodeType` enum を追加する（Source / Transform / Sink、PartialEq 付き）
- [x] `LineageNode` 構造体を追加する（id: String, node_type: LineageNodeType, label: String）
- [x] `LineageEdge` 構造体を追加する（from: String, to: String）
- [x] `LineageGraph` 構造体を追加する（nodes: Vec<LineageNode>, edges: Vec<LineageEdge>）
- [x] `format_lineage_dot(graph: &LineageGraph) -> String` を追加する
  - `digraph lineage {` で始まり `}` で終わる
  - 各エッジを `    "<from>" -> "<to>"` 形式で出力（インデント 4 スペース）
  - 空グラフは `digraph lineage {\n}` を返す
- [x] `cargo test` で既存 3722 tests が pass することを確認する

---

## T2: CHANGELOG.md 更新（テスト追加より先）

- [x] `CHANGELOG.md` の先頭に v76.5.0 エントリを追加する
- [x] Added セクション（enum 1 件・struct 3 件・関数 1 件）と Tests セクション（2 件）を含める

---

## T3: driver.rs — テストモジュール追加

- [x] `fav/src/driver.rs` に `v765000_tests` モジュールを追加する（`use super::*` 必須）
- [x] `lineage_graph_built` テストを実装する
  - ノード 3 件・エッジ 2 件のグラフを構築
  - nodes.len() == 3、edges.len() == 2
  - ノード型（Source / Transform / Sink）を検証
- [x] `lineage_dot_format` テストを実装する
  - `format_lineage_dot` の出力が `digraph lineage {` で始まり `}` で終わる
  - エッジ行が `"from" -> "to"` 形式で含まれる
  - 空グラフで DOT 出力が正しい形式である
- [x] `cargo test v765000` で 2 件が pass することを確認する

---

## T4: Cargo.toml バージョン更新

- [x] `fav/Cargo.toml` の `version` を `"76.4.0"` → `"76.5.0"` に変更する
- [x] `driver.rs` 内の `76.4.0` バージョン文字列アサーションを `76.5.0` に一括更新

---

## T5: versions/current.md 更新

- [x] 「進行中バージョン」を v76.5.0 に更新する
- [x] 「次に切る版」を v76.6.0 に更新する

---

## T6: 最終確認

- [x] `cargo test` が全 pass であることを確認する（3724 tests）
- [x] `cargo test v765000` で 2 件が pass することを確認する
- [x] `fav/Cargo.toml` のバージョンが `76.5.0` であることを確認する
- [x] `CHANGELOG.md` の先頭が `[v76.5.0]` であることを確認する

---

## 完了チェックリスト

- [x] 全タスク（T0〜T6）が完了している
- [x] `lineage_graph_built` が pass
- [x] `lineage_dot_format` が pass
- [x] テスト総数: 3724（+2）
- [x] site/ MDX 追加: 本バージョンでは対象外（型基盤のみ）
- [x] `changelog_has_v76_5_0` テストの追加: 対象外（x.0.0 宣言バージョンのみに追加する慣例）。ただし CHANGELOG.md への v76.5.0 エントリ追加自体は T2 で必須
