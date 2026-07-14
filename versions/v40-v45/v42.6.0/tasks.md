# v42.6.0 タスクリスト

**ステータス**: COMPLETE
**目標テスト数**: 2891（前バージョン 2889 + 2）
**実績テスト数**: 2891（v42600_tests 2/2 PASS）

---

## T0 — 事前確認

- [x] `cargo test` が 2889 tests / 0 failures であることを確認
- [x] `fav/Cargo.toml` version が `42.5.0` であることを確認
- [x] `fav/src/backend/vm.rs` の `Email.send_raw` ブロック行番号を記録（line 18737）
- [x] `fav/src/driver.rs` の `v42500_tests` 冒頭行番号を記録（line 44646）
- [x] `versions/roadmap/roadmap-v42.1-v43.0.md` に v42.6.0 エントリが存在することを確認

---

## T1 — `runes/websocket/rune.toml` 作成

- [x] `[rune]` セクションのみ（name / version / entry / description）

---

## T2 — `runes/websocket/websocket.fav` 作成

- [x] `send(ctx, url, message)` — `WebSocket.send_raw` 呼び出し
- [x] `broadcast(ctx, url, messages)` — `WebSocket.broadcast_raw` を直接呼び出し（`List.map` は使わない）

---

## T3 — `vm.rs` — WebSocket primitives 追加

- [x] `Email.send_raw` ブロックの直後に `WebSocket.send_raw` stub（3 args → `ok_vm(VMValue::Unit)`）
- [x] `WebSocket.broadcast_raw` stub（3 args → `ok_vm(VMValue::Int(count))`、messages リスト長を返す）
- [x] エラーは `Err("...".to_string())` 形式（`vm_call_builtin` は自由関数のため `self.error` 不使用）

---

## T4 — `site/content/docs/runes/websocket.mdx` 作成

- [x] `site/content/docs/runes/websocket.mdx` を他 Rune MDX に倣い作成
- [x] `send` / `broadcast` 関数シグネチャ・使用例・v44.x stub 注記を含む

---

## T5 — `driver.rs` — `v42600_tests` モジュール追加

- [x] `v42500_tests` の `cargo_toml_version_is_42_5_0` をスタブ化（`assert!(true)`）
- [x] `v42600_tests` を `v42500_tests` の直前（降順）に挿入
- [x] `cargo_toml_version_is_42_6_0` テスト追加
- [x] `websocket_rune_fav_exists` テスト追加（以下をすべて assert）:
  - `fav_src.contains("WebSocket.send_raw")`
  - `fav_src.contains("broadcast")`
  - `rune_toml.contains("websocket")`
  - `mdx.contains("WebSocket")`（`include_str!("../../site/content/docs/runes/websocket.mdx")`）

---

## T6 — `fav/Cargo.toml` バージョン bump

- [x] `version = "42.5.0"` → `"42.6.0"`

---

## T7 — `CHANGELOG.md` 更新

- [x] `[v42.6.0]` エントリを `[v42.5.0]` の直前に追加
- [x] `WebSocket Rune`・`websocket.mdx`・`v44.x stub` の旨を記載

---

## T8 — テスト実行・確認

- [x] `cargo test` 実行
- [x] failures = 0 を確認
- [x] テスト数 = 2891 を確認（2889 + 2 件）
- [x] `v42600_tests` 2 件 pass を確認

---

## T9 — バージョン管理ドキュメント更新

- [x] `versions/current.md` を v42.6.0（最新安定版、2891 tests）・v42.7.0（次に切る版）に更新
- [x] `versions/roadmap/roadmap-v42.1-v43.0.md` の v42.6.0 を完了済みにマーク（`✅ COMPLETE（2026-07-12）`）
- [x] 同ロードマップの v42.6.0 推定テスト数を `2890` → 実績 `2891` に修正
- [x] `versions/v40-v45/v42.6.0/tasks.md` を COMPLETE ステータスに更新（全チェックボックス `[x]`）

---

## 最終ステータス

- [x] 全タスク完了

## spec-reviewer 指摘・対応記録

- [HIGH-1]: ロードマップ推定テスト数が 2890 で実態（2891）と不一致 → tasks T9 でロードマップ修正・実績値上書きを明示
- [HIGH-2]: `site/content/docs/runes/websocket.mdx` が影響範囲から欠落 → spec §影響範囲・plan T4・tasks T4 に追加
- [MED-1]: `broadcast` の挙動テストなし → stub では常に messages.len() が返るため非スコープに明記；`websocket_rune_fav_exists` に `broadcast` 文字列チェックを追加
- [MED-2]: plan.md と tasks.md のタスク番号体系ずれ → 両ファイルを T0〜T9 で統一
- [MED-3]: `versions/current.md` 更新記述が plan.md に重複 → plan.md T9 に一本化
- [LOW-1]: `WebSocket.broadcast_raw` が dead code → `broadcast` 関数を `broadcast_raw` 直接呼び出しに変更（spec §2・plan T2・tasks T2 修正）
- [LOW-2]: `websocket_rune_fav_exists` が `broadcast` を未検証 → `fav_src.contains("broadcast")` アサーション追加
- [LOW-3]: `bind ok_count <- List.fold(...)` 型不整合リスク → `broadcast_raw` 直接呼び出しに変更し問題解消

## code-reviewer 指摘・対応記録

- [実装時修正]: `vm_call_builtin` は自由関数のため `self.error(artifact, ...)` / `VMValue::Result` が使えない → `Err("...".to_string())` + `ok_vm(VMValue::Unit)` / `ok_vm(VMValue::Int(count))` に修正してコンパイル解消
- [MED-1]: 引数数エラーの返却形式が `Err(String)`（VM 強制終了）になっており `err_vm` パターン非準拠 → `Ok(err_vm(VMValue::Str(...)))` に修正（Favnir レベルのエラー値として返すよう統一）
- [MED-2]: `broadcast_raw` の `expect()` 呼び出し前に SAFETY コメント欠落 → `// SAFETY: args.len() == 3 is checked above.` を追加
- [MED-3]: `rune.toml` に `effects` フィールド未記載 → stub 段階のため意図的省略。CHANGELOG・MDX にすでに明記済みのため対応不要
- [LOW-1]: `websocket.fav` にエフェクト注釈（`!WebSocket`）未付与 → stub 段階では省略。v44.x 実装時に追加予定（CHANGELOG 記載済み）
- [LOW-2]: 空リスト境界ケース（`Ok(0)`）が MDX に未記載 → stub 段階では許容。v44.x 実装時に仕様ドキュメント整備予定
