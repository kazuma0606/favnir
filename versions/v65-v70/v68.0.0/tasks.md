# v68.0.0 タスクリスト

Status: COMPLETE
Version: 68.0.0
Base tests: 3515
Target tests: 3519

---

## T0: 事前確認

- [x] `cargo test -j 8 -- --test-threads=8` でベース 3515 tests passed, 0 failed を確認
- [x] `fav/Cargo.toml` の version が `"67.0.0"` であることを確認（本バージョンで `"68.0.0"` に更新する）
- [x] `driver.rs` に `v67900_tests` が存在することを確認（`v68000_tests` の挿入位置）
- [x] `driver.rs` に `v68000_tests` が存在しないことを確認（新規追加）
- [x] `cargo test --bin fav v67900_tests` で 2 件 PASS することを確認
  - 前バージョンのテスト関数名: `dev_intelligence_all_stable`, `debug_viz_suggest_docs_complete`
- [x] `versions/current.md` の「進行中バージョン」が `v67.9.0` であることを確認
- [x] `versions/current.md` の「最新安定版」が `v67.0.0` であることを確認（`v66.0.0` のままだったため手動修正済み）

---

## T1: `fav/Cargo.toml` — バージョン更新

- [x] `version = "67.0.0"` → `version = "68.0.0"` に変更
- [x] `cargo build` でエラーなし

---

## T2: `MILESTONE.md` — v68.0.0 エントリを先頭に追加

- [x] `## v67.0.0（2026-08-06）— AI-Native Stage Layer` の直前に v68.0.0 エントリを挿入
  - [x] `"Developer Intelligence"` を含む（`milestone_has_dev_intelligence` テスト要件）
  - [x] 宣言文（`「ステップ実行デバッガが...」`）を含む
  - [x] v67.1〜v67.9 の達成内容を箇条書きで記載
  - [x] テスト数 `3519` を記載

---

## T3: `README.md` — v68.0.0 宣言を追加

- [x] `"Developer Intelligence"` を含む記述を追加（`readme_mentions_dev_intelligence` テスト要件）
- [x] v67.0.0「AI-Native Stage Layer」の記述の直前に配置

---

## T4: `CHANGELOG.md` — v68.0.0 エントリを先頭に追加

- [x] `## [v67.0.0]` の直前に `## [v68.0.0] — 2026-08-07 — Developer Intelligence 宣言 ★クリーンアップ` を挿入
  - [x] `"v68.0.0"` を含む（`changelog_has_v68_0_0` テスト要件）
  - [x] v67.1〜v67.9 の機能一覧を Added セクションに記載
  - [x] Cargo.toml バージョン変更を Changed セクションに記載
  - [x] `cargo clean` 実施を Note セクションに記載

---

## T5: `driver.rs` — `v68000_tests` 追加 + 旧バージョン文字列一括更新

- [x] `// -- v67900_tests (v67.9.0) --` の直前に `v68000_tests` を挿入（4件）
  - [x] `cargo_toml_version_is_68_0_0`: `include_str!("../Cargo.toml")` に `"version = \"68.0.0\""` を含む
  - [x] `changelog_has_v68_0_0`: `include_str!("../../CHANGELOG.md")` に `"v68.0.0"` を含む
  - [x] `milestone_has_dev_intelligence`: `include_str!("../../MILESTONE.md")` に `"Developer Intelligence"` を含む
  - [x] `readme_mentions_dev_intelligence`: `include_str!("../../README.md")` に `"Developer Intelligence"` または `"v68.0"` を含む
- [x] 旧バージョン全 `cargo_toml_version_is_XX_X_0` テストの `"67.0.0"` → `"68.0.0"` に一括更新（16件）
- [x] `cargo build` でエラーなし

---

## T6: `cargo clean` + `fav/tmp/hello.fav` 復元

- [x] `cargo clean` を実行（★クリーンアップ: 9.6GiB 削除）
- [x] `fav/tmp/hello.fav` が存在することを確認（内容正常）

---

## T7: ビルド・テスト

- [x] `cargo test --bin fav v68000_tests` で 4 件 PASS
  - [x] `cargo_toml_version_is_68_0_0` PASS
  - [x] `changelog_has_v68_0_0` PASS
  - [x] `milestone_has_dev_intelligence` PASS
  - [x] `readme_mentions_dev_intelligence` PASS
- [x] `cargo test -j 8 -- --test-threads=8` で 3519 tests passed, 0 failed を確認

---

## T8: ドキュメント・ステータス更新

- [x] `versions/roadmap/roadmap-v67.1-v68.0.md` の v68.0.0「状態」列を「未着手」→「完了」に変更
- [x] `versions/current.md` を更新:
  - 「最新安定版」を `v68.0.0 — Developer Intelligence 宣言 — 3519 tests` に変更
  - 「前バージョン」を `v67.0.0 — AI-Native Stage Layer 宣言 — 3497 tests` に変更（x.0.0 系列の直前マイルストーン宣言版を指す）
  - 「進行中バージョン」欄を次スプリント計画中に更新
- [x] 本 `tasks.md` を COMPLETE に更新（T0 を含む全チェックボックスを `[x]` に）

---

## 設計上の意図的省略

- v69.x 以降のスプリント計画: 別途策定

## 実装上の注意点（次バージョン実装者へ）

- `cargo clean` 後、driver.rs 内の全 `cargo_toml_version_is_XX_X_0` テストが旧バージョン文字列をアサートしているため、Cargo.toml 更新後に一括 `replace_all` で更新が必要（v68.0.0 では 16件を "67.0.0" → "68.0.0" に更新）
- 次の x.0.0 宣言（v69.0.0）では同様に全件 "68.0.0" → "69.0.0" に更新すること

## コードレビュー指摘と対応

| 深刻度 | 内容 | 対応 |
|--------|------|------|
| [MED] | `v67000_tests::cargo_toml_version_is_67_0_0` のアサーション失敗メッセージが `"67.0.0"` のまま（同パターンが 4件連鎖） | `replace_all` で `"should have version 67.0.0"` → `"should have version 68.0.0"` に 4件一括更新 |
