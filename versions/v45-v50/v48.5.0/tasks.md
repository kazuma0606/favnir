# Tasks: v48.5.0 — import エイリアス完全化 + 旧構文 deprecation

Status: COMPLETE
Date: 2026-07-18

---

## T0 — 事前確認

- [x] `cargo test` 3053 passed, 0 failed を確認（ベース確認）
- [x] `lint.rs` に `check_w035_legacy_import_rune` が存在しないことを確認
- [x] `run_lint` に W035 登録がないことを確認
- [x] `ImportKind::Legacy` が `ast.rs` に存在することを確認（v48.1.0 追加済み）

## T1 — `lint.rs` W035 追加

- [x] `run_lint` の W034 コメント直後に `check_w035_legacy_import_rune` 呼び出しを追加
- [x] ファイル末尾に `check_w035_legacy_import_rune` 関数を追加
  - [x] `Item::ImportDecl { kind: ImportKind::Legacy, .. }` を検出
  - [x] W035 `LintError` を push（`path` を含むメッセージ）

## T2 — `driver.rs` テスト追加

- [x] `v485000_tests` モジュールを `v484000_tests` の直前に追加（2テスト）
  - [x] `import_alias_resolves`: `import postgres as db` → `kind=Package, alias=Some("db")` を確認
  - [x] `legacy_import_rune_w035`: `import rune "kafka"` で `run_lint` を呼ぶと W035 が 1 件、`message` に `"kafka"` を含む

## T3 — バージョン更新・完了

- [x] `fav/Cargo.toml` version → `"48.5.0"`
- [x] `CHANGELOG.md` に v48.5.0 エントリ追加
- [x] `cargo test` 3055 passed, 0 failed（3053 + 2 件）
- [x] `cargo clippy -- -D warnings` クリーン
- [x] `versions/current.md` を v48.5.0（3055 tests）に更新、進行中バージョンを `v48.6.0` に更新
- [x] `versions/roadmap/roadmap-v48.1-v49.0.md` の v48.5.0 テスト数を実績値 3055 に更新（`roadmap-v45.1-v50.0.md` への反映は v49.0.0 時・変更不要）
- [x] tasks.md を COMPLETE に更新（T0〜T3 全 `[x]`）

---

> **注記**: W035 は警告のみ（`import rune "kafka"` 構文の削除は v49.0.0 以降のスコープ外）
> **注記**: W034 は checker.rs 発行（別チャネル）—混同しないこと
