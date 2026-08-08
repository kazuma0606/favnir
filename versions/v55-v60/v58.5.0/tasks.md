# v58.5.0 Tasks — Policy-as-Code（`fav policy`）

Date: 2026-07-28
Status: COMPLETE（2026-07-28）— 3291 tests passed, 0 failed

---

## T0: 事前確認

- [x] `cargo test` でベースラインが 3289 tests passed, 0 failed であることを確認
- [x] `fav/Cargo.toml` のバージョンが `"58.4.0"` であることを確認
- [x] `versions/roadmap/roadmap-v58.1-v59.0.md` の v58.5.0 セクションを確認（スコープ変更ブロック記載済みを確認）
- [x] `fav/src/policy.rs` の既存 `cmd_policy_check` を確認（変更しない）
- [x] `fav/src/main.rs` の `Some("policy")` アームを確認（拡張点を把握）
- [x] `fav/src/error_catalog.rs` の最後のエントリ（E0424）を確認
- [x] `grep -c '"58.4.0"' fav/src/driver.rs` で ローリングアサーション件数が 5 であることを確認

---

## T1: Cargo.toml バージョン更新

- [x] `fav/Cargo.toml`: `version = "58.4.0"` → `"58.5.0"`

---

## T2: error_catalog.rs に E0425 予約コメント + E0426 追加

- [x] E0424 エントリの直後に予約コメント `// E0425: reserved（将来の policy 拡張用）` を追加
- [x] その直後に E0426 エントリを追加（全フィールドを spec.md から転記）
  - code: "E0426"
  - title: "policy violation"
  - description: "A pipeline violates a declared policy rule."
  - example: (spec.md 参照)
  - fix: (spec.md 参照)

---

## T3: driver.rs に関数追加

- [x] `cmd_policy_check_file(pipeline_file: &str, policy_dir: &str) -> i32` を追加
- [x] `cmd_policy_list(policy_dir: &str) -> i32` を追加

---

## T4: driver.rs テストモジュール追加

- [x] `v58500_tests` モジュールを v58400_tests の直前に挿入
  - [x] `policy_check_violation` テスト
  - [x] `policy_check_passes` テスト

---

## T5: driver.rs ローリングチェック更新

- [x] T0 で確認したローリングアサーション件数（5 件）を `replace_all` で `"58.5.0"` に一括更新

---

## T6: main.rs 拡張

- [x] use imports に `cmd_policy_check_file`, `cmd_policy_list` を追加
- [x] `Some("policy")` の `check` アームに `--policy-dir` フラグ対応を追加
- [x] `Some("policy")` に `list` サブコマンドアームを追加

---

## T7: テスト実行・確認

- [x] `cargo test -j 8 -- --test-threads=8` を実行
- [x] `policy_check_violation` pass を確認
- [x] `policy_check_passes` pass を確認
- [x] 総テスト数 **3291** tests passed, 0 failed を確認

---

## T8: 事後処理

- [x] `CHANGELOG.md` に v58.5.0 エントリを追加
- [x] `versions/current.md` を v58.5.0 / 3291 tests に更新
- [x] `versions/roadmap/roadmap-v58.1-v59.0.md` の v58.5.0 実績欄を更新
- [x] このファイル（tasks.md）を COMPLETE ステータスに更新

---

## コードレビュー記録

- [BUG] `args.get(3)` によるパイプラインファイル取得の誤り → positional 引数検索に修正（main.rs）
- [BUG] rolling check failure メッセージ 5 件が `"58.2.0"` のまま → `"58.5.0"` に修正（driver.rs）
- [STYLE] `cmd_policy_list` にテストなし・未使用 import 警告 → `policy_list_returns_zero` テスト追加（driver.rs）

最終テスト数: 3292 tests passed, 0 failed（code-review 対応で +1）

---

Status: COMPLETE（2026-07-28）— 3292 tests passed, 0 failed
