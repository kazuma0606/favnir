# v42.9.0 タスクリスト

**ステータス**: COMPLETE
**目標テスト数**: 2896（前バージョン 2894 + 2）
**実績テスト数**: 2896（v42900_tests 2/2 PASS）
※ ロードマップ記載の 2895 は旧 v42.8.0 基準（2893+2）の誤差。実績は 2896。

---

## T0 — 事前確認

- [x] `cargo test` が 2894 tests / 0 failures であることを確認
- [x] `fav/Cargo.toml` version が `42.8.0` であることを確認
- [x] `fav/src/driver.rs` の `v42800_tests` 冒頭行番号を記録（line 44654）
- [x] `versions/roadmap/roadmap-v42.1-v43.0.md` に v42.9.0 エントリが存在することを確認

---

## T1 — `site/content/docs/real-time-power.mdx` 作成

- [x] v42.x リアルタイム機能（CEP / Stream join / back-pressure / WebSocket / fav monitor）を一覧
- [x] `"Stream.join"` / `"Cep.window"` / `"WebSocket"` キーワードを含む（テスト検証対象）
- [x] Stream join セクションに join キー型安全チェックの v43.x 延期注記を含む
- [x] 各機能に最小コード例と stub 注記を含む
- [x] コードフリーズ：新規言語機能・VM 機能の追加なし

---

## T2 — `driver.rs` — `v42900_tests` モジュール追加

- [x] `v42800_tests` の直前（降順）に `v42900_tests` を挿入
- [x] `cargo_toml_version_is_42_9_0` テスト追加（NOTE コメント付き：次バージョンでスタブ化すること）
- [x] `real_time_power_docs_exists` テスト追加:
  - `mdx.contains("Stream.join")`
  - `mdx.contains("Cep.window")`
  - `mdx.contains("WebSocket")`

---

## T3 — `fav/Cargo.toml` バージョン bump

- [x] `version = "42.8.0"` → `"42.9.0"`

---

## T4 — `CHANGELOG.md` 更新

- [x] `[v42.9.0]` エントリを `[v42.8.0]` の直前に追加
- [x] コードフリーズ・`real-time-power.mdx` 追加の旨を記載

---

## T5 — テスト実行・確認

- [x] `cargo test` 実行
- [x] failures = 0 を確認
- [x] テスト数 = 2896 を確認（2894 + 2 件）
- [x] `v42900_tests` 2 件 pass を確認

---

## T6 — バージョン管理ドキュメント更新

- [x] `versions/current.md` を v42.9.0（最新安定版、2896 tests）・v43.0.0（次に切る版）に更新
- [x] `versions/roadmap/roadmap-v42.1-v43.0.md` の v42.9.0 を完了済みにマーク（`✅ COMPLETE（2026-07-12）`）
- [x] 同ロードマップの v42.9.0 推定テスト数を `2895` → 実績 `2896` に修正（完了済み）
- [x] `versions/v40-v45/v42.9.0/tasks.md` を COMPLETE ステータスに更新（全チェックボックス `[x]`）

---

## 最終ステータス

- [x] 全タスク完了

## spec-reviewer 指摘・対応記録

- [HIGH-2]: `real_time_power_docs_exists` のアサーションが `"Cep"` と弱い → `"Cep.window"` に置き換え、`"WebSocket"` アサーションを追加（spec.md / plan.md / tasks.md 全修正）
- [MED-1]: plan.md の MDX コードブロック内でフェンスネスト問題（``` 内の ``` ）→ plan.md T1 のコードブロックを適切に区切り直し
- [MED-2]: Stream join セクションに join キー型安全チェックが v43.x 延期である旨の制限注記が欠落 → plan.md MDX テンプレートと tasks.md T1 に注記を追加
- [LOW-1]: spec.md 影響範囲テーブルに `tasks.md` エントリが欠落 → 追加
- [LOW-2]: tasks.md T1 にコードフリーズ宣言（新規機能追加なし）の明示チェックが欠落 → 追加

## code-reviewer 指摘・対応記録

- [MED]: `real-time-power.mdx` の `cep pattern` 宣言構文に stub 注記がなく、パーサーに実装済みか不明瞭 → 「AST 実装済み。イベント型検証は v44.x 予定」注記を追加
- [LOW-1]: `real_time_power_docs_exists` の `"WebSocket"` アサーションが弱い（見出しのみで通過） → `"WebSocket.send"` に強化（driver.rs 修正、テスト 2/2 PASS 確認）
- [LOW-2]: `cargo_toml_version_is_42_9_0` の `contains("42.9.0")` は将来バージョン（142.9.0 等）で偽陽性になるリスクがある → 現時点のバージョン体系（42.x 台）では問題なし。スタブ化コメントも付与済みのため対応不要と判断
- [LOW-3]: CHANGELOG の v42.8.0 エントリに `cargo_toml_version_is_42_8_0` 記載なし → v42.8.0 側の不一致であり今回スコープ外。次バージョン作業時に修正することを検討
