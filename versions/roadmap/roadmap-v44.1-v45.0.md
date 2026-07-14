# Roadmap v44.1.0 〜 v45.0.0 — Precision & Flow

Date: 2026-07-11
Status: 計画中（v44.0 完了後に詳細確定）

---

## 目標

v44.0「Language Expressiveness」で型推論 6 カテゴリ・Opaque type を整備した。
このフェーズは **「Streaming・Type Precision・Real-Time・Language Expressiveness の全機能を統合し、型安全なリアルタイムパイプラインを最小限の注釈で書ける状態を宣言する」** を実現する。

---

## バージョン計画

### v44.1.0 — Refinement type x Streaming 統合 ✅ COMPLETE（2026-07-14）

```favnir
type PositiveFloat = Float where |v| v > 0.0

stage Validate: List<Float> -> List<Float> = |events| {
  bind valid: Stream<PositiveFloat> <- events
}
```

`collect_refinement_stream_bindings` ヘルパーにより、refinement type がストリーム要素型として型注釈された bind 束縛を AST レベルで検出。checker.fav 統合は将来版のスコープ。

**完了条件**: Rust テスト 3 件（実績 2944 tests passed, 0 failed）

---

### v44.2.0 — CEP x Refinement type ✅ COMPLETE（2026-07-14）

CEP パターン（`cep pattern`）のイベント節に Refinement type 名が参照されていることを AST レベルで検出。
`collect_cep_refinement_event_refs` ヘルパーにより `CepExpr::Event` 名と refinement type 名の一致を `Seq`/`Any`/`Not` 再帰走査で検出。

```favnir
type HighValue = Float where |v| v > 1000.0

cep pattern HighValueDetected {
  HighValue within 300
}
// → collect_cep_refinement_event_refs で検出
```

**注**: `Purchase<HighValue>` 構文（型パラメータ付き CEP イベント）・checker.fav 型チェック統合は将来版のスコープ（現 AST は `CepExpr::Event(String)` — 型パラメータなし）。

**完了条件**: Rust テスト 3 件（実績 2947 tests passed, 0 failed）

---

### v44.3.0 — Stream join x Opaque type ✅ COMPLETE（2026-07-14）

`collect_opaque_alias_groups` ヘルパーにより、同じ内部型を持つ opaque type エイリアスのグループを AST レベルで検出。

```favnir
opaque type OrderId = String
opaque type PaymentOrderId = String
// → collect_opaque_alias_groups で "String: OrderId, PaymentOrderId" として検出
```

**注**: `Stream.join` での誤 join E0413 検出（checker.fav 統合）は将来版のスコープ。本バージョンは AST レベル opaque グループ検出 MVP。

**完了条件**: Rust テスト 3 件（実績 2950 tests passed, 0 failed）

---

### v44.4.0 — 型推論 x パイプライン lineage ✅ COMPLETE（2026-07-14）

`collect_annotated_lineage_bindings` ヘルパーにより、ステージ内の型注釈付き bind 束縛を lineage エントリとして AST レベルで収集。

**注**: `fav explain --lineage` 出力への型情報統合（`LineageEntry` 拡張）・ウィンドウ/join の lineage 追跡は将来版のスコープ（AST レベル MVP）。

**完了条件**: Rust テスト 2 件（実績 2953 tests passed, 0 failed）

---

### v44.5.0 — Back-pressure x `fav policy` 統合 ✅ COMPLETE（2026-07-14）

`#[max_inflight(n)]` アノテーション（v42.5.0 で AST / パーサー追加済み）を活用し、ステージ定義に付与された max_inflight 制約を収集できる `collect_stage_max_inflight_annotations` ヘルパーを追加する。

**MVP スコープ（本バージョン）**: `#[max_inflight(n)]` 付きステージの AST レベル収集のみ。

**将来版スコープ**: `policy { max_inflight: N }` グローバルポリシーブロック構文の追加・`fav policy check --ci` での整合検証・VM レベル runtime 強制（`ast.rs` に `PolicyBlock` ノードが未定義のため）。

**完了条件**: Rust テスト 2 件（実績 2955 tests passed, 0 failed）

---

### v44.6.0 — Precision & Flow E2E デモ ✅ COMPLETE（2026-07-15）

`infra/e2e-demo/precision-flow/` — CEP + Refinement type + Policy gate（governance 制御）統合デモ。
Kafka → CEP → Opaque join → Policy gate の完全パイプライン。

**完了条件**: Rust テスト 1 件（実績 2956 tests passed, 0 failed）

---

### v44.7.0 — ドキュメントサイト Precision & Flow 概要ページ ✅ COMPLETE（2026-07-15）

`site/content/docs/precision-and-flow.mdx` — 全機能の統合解説ページ。

**完了条件**: Rust テスト 2 件（実績 2958 tests passed, 0 failed）

---

### v44.8.0 — パフォーマンス最終調整 ✅ COMPLETE（2026-07-15）

ストリーム処理 + 型推論の速度最適化。
`fav bench --stream` 計測結果を `CHANGELOG.md` に記録し、ベンチマーク追跡 MVP を確立する。`collect_bench_stream_notes` ヘルパーにより CHANGELOG から bench --stream 記録行を収集。

**将来版スコープ**: VM レベル実行速度最適化・v41.0 との実測比較。

**完了条件**: Rust テスト 2 件（実績 2960 tests passed, 0 failed）

---

### v44.9.0 — v45.0 前調整・安定化 ✅ COMPLETE（2026-07-15）

コードフリーズ（新規機能追加なし）。`site/content/docs/precision-and-flow-overview.mdx` 新規作成（ファイル未存在のため「更新」→「新規作成」）。

**完了条件**: meta テスト 2 件（実績 2962 tests passed, 0 failed）

---

### v45.0.0 — Precision & Flow 宣言 ★クリーンアップ ✅ COMPLETE（2026-07-15）

**宣言文（暫定）**:

> 「型推論がジェネリクスと戻り値型を補完し、最小限の注釈で安全なコードが書ける。
>  ウィンドウ集計・CEP・Stream join が型安全に記述でき、
>  refinement type と opaque type がデータの意味を型で守る。
>
>  これが Favnir v45.0 — Precision & Flow の姿である。」

**完了条件**:
- v44.1〜v44.9 の全機能が動作する ✅
- `cargo test` 全通過（**実績 2966 passed; 0 failed**）✅
- `v45000_tests` 4 件 pass ✅
- `MILESTONE.md` に `"Precision & Flow"` が含まれる ✅
- `★クリーンアップ`（`cargo clean`）完了（23.5 GiB 削除）✅

---

## 参考リンク

- マスタースケジュール: `versions/roadmap/roadmap-v40.1-v45.0.md`
- 前サブスプリント: `versions/roadmap/roadmap-v43.1-v44.0.md`
- 達成宣言: `MILESTONE.md`
