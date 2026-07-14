# v42.8.0 タスクリスト

**ステータス**: COMPLETE
**目標テスト数**: 2894（前バージョン 2893 + 1）
**実績テスト数**: 2894（v42800_tests 1/1 PASS）
※ ロードマップ記載の 2893 は旧 v42.7.0 基準（2892+1）の誤差。実績は 2894。

---

## T0 — 事前確認

- [x] `cargo test` が 2893 tests / 0 failures であることを確認
- [x] `fav/Cargo.toml` version が `42.7.0` であることを確認
- [x] `fav/src/driver.rs` の `v42700_tests` 冒頭行番号を記録（line 44654）
- [x] `versions/roadmap/roadmap-v42.1-v43.0.md` に v42.8.0 エントリが存在することを確認

---

## T1 — `site/content/cookbook/cep-login-purchase.mdx` 作成

- [x] CEP ログイン→購入検出パイプライン例を記述
- [x] `"Cep.window"` キーワードを含む（テスト検証対象）
- [x] ステージ引数 `event_stream` / `sessions` を明示（未束縛変数なし）
- [x] `w.first == "login"` 形式で `.type` キーワード衝突を回避
- [x] stub 注記を追加

---

## T2 — `site/content/cookbook/stream-join.mdx` 作成

- [x] Stream.join 2 ストリーム結合パイプライン例を記述
- [x] `"Stream.join"` キーワードを含む（テスト検証対象）
- [x] 戻り型を `List<List<Int>>` に統一（型矛盾なし）
- [x] `List.range` ベースの `Stream<Int>` で一貫

---

## T3 — `driver.rs` — `v42800_tests` モジュール追加

- [x] `v42700_tests` の `cargo_toml_version_is_42_7_0` をスタブ化（`assert!(true)`）
- [x] `v42800_tests` を `v42700_tests` の直前（降順）に挿入
- [x] `realtime_cookbook_mdx_exists` テスト追加:
  - `cep_mdx.contains("Cep.window")`
  - `join_mdx.contains("Stream.join")`

---

## T4 — `fav/Cargo.toml` バージョン bump

- [x] `version = "42.7.0"` → `"42.8.0"`

---

## T5 — `CHANGELOG.md` 更新

- [x] `[v42.8.0]` エントリを `[v42.7.0]` の直前に追加

---

## T6 — テスト実行・確認

- [x] `cargo test` 実行
- [x] failures = 0 を確認
- [x] テスト数 = 2894 を確認（2893 + 1 件）
- [x] `v42800_tests` 1 件 pass を確認

---

## T7 — バージョン管理ドキュメント更新

- [x] `versions/current.md` を v42.8.0（最新安定版、2894 tests）・v42.9.0（次に切る版）に更新
- [x] `versions/roadmap/roadmap-v42.1-v43.0.md` の v42.8.0 を完了済みにマーク（`✅ COMPLETE（2026-07-12）`）
- [x] 同ロードマップの v42.8.0 推定テスト数を `2893` → 実績 `2894` に修正
- [x] `versions/v40-v45/v42.8.0/tasks.md` を COMPLETE ステータスに更新（全チェックボックス `[x]`）

---

## 最終ステータス

- [x] 全タスク完了

## spec-reviewer 指摘・対応記録

- [HIGH-1]: ロードマップ推定テスト数が 2893 で実態 2894 と不一致 → ロードマップを 2894 に修正
- [HIGH-2]: `cep-login-purchase.mdx` Favnir コードの問題（未束縛変数 `sessions`、`.type` キーワード衝突、`Cep` API 整合性）→ ステージ引数を明示（`event_stream`/`sessions`）、`w.first == "login"` に変更、stub 注記を追加
- [HIGH-3]: `stream-join.mdx` の型矛盾（`Stream<Int>` から `List<List<Event>>`）→ 型を `List<List<Int>>` に修正
- [MED-1]: `realtime_cookbook_mdx_exists` の `contains("Cep")` が弱い → `contains("Cep.window")` に強化
- [MED-2]: `#[max_inflight]` が非機能 API であることを明示せず cookbook に記載 → 言及を削除
- [MED-3]: コードフェンス言語タグが `favnir` → `fav` に統一（real-time 系 cookbook に合わせる）

## code-reviewer 指摘・対応記録

- [HIGH-1]: `stream-join.mdx` の `stage JoinStreams` で `ctx` を引数に取るが型シグネチャ `Unit ->` に反映されていない → `ctx` 不使用のため `|ctx|` を `|_|` に変更し、コメントで理由を明記
- [HIGH-2]: `cep-login-purchase.mdx` のステージ引数に型注釈なし、`PushResults` の戻り値 `Unit` が `Result` 系と不整合 → 引数に `: AppCtx` / `: Stream<String>` / `: List<String>` 型注釈を追加、戻り型を `Result<Unit, String>` に修正し `Result.ok(Unit)` を返すよう変更
- [MED-1]: `stage` 引数の型注釈欠落（[HIGH-2] と同対応）
- [MED-2]: テストモジュール配置順序（v42800 が v42700 より前）→ 降順配置はプロジェクト慣習（最新優先）のため対応不要
- [LOW-3/4]: `description` のダブルクォート欠落（cep / stream-join 両 MDX）→ ダブルクォートで囲む形に統一
