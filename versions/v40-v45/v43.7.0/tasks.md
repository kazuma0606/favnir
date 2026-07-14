# v43.7.0 タスク — 構造体リテラル推論（Structural inference）

## ステータス: COMPLETE（2026-07-13）— 2922 tests

---

## T0 — 事前確認

- [x] `cargo test` 2920 / 0 確認
- [x] `Cargo.toml` version = `43.6.0` 確認
- [x] `v43700_tests` が `fav/src/driver.rs` に存在しないことを確認
- [x] `checker.fav` line 1957 に `ERecordLit({ _0: tname, _1: fields }) => Result.ok(tname)` が存在することを確認

---

## T1 — driver.rs — v43700_tests 追加

- [x] `v43600_tests` モジュールの直前に `v43700_tests` を挿入
- [x] `cargo_toml_version_is_43_7_0` テスト追加（`Cargo.toml` に `"43.7.0"` を含む）
- [x] `structural_record_literal_type_checks` テスト追加
  - `type Point = { x: Int  y: Int }; fn make_point() -> Point { Point { x: 1  y: 2 } }; fn shift(p: Point) -> Point { Point { x: p.x  y: p.y } }` → `Ok`

---

## T2 — Cargo.toml + v43600_tests スタブ化

- [x] `fav/Cargo.toml` version を `43.6.0` → `43.7.0` に更新
- [x] `v43600_tests::cargo_toml_version_is_43_6_0` の assert を削除してスタブ化

---

## T3 — CHANGELOG.md

- [x] v43.7.0 エントリ追加
  - Added: `v43700_tests` 2 件
  - Changed: `cargo_toml_version_is_43_6_0` スタブ化
  - Notes: checker.fav 変更なし（既存の ERecordLit 機構で動作）

---

## T4 — テスト実行・確認

- [x] `cargo test -j 8 -- --test-threads=8` 実行
- [x] 2922 passed; 0 failed 確認
- [x] `v43700_tests` 2 件 pass 確認

---

## T5 — バージョン管理ドキュメント更新

- [x] `versions/current.md` → v43.7.0 最新安定版（2922 tests）、次版 v43.8.0
- [x] `versions/roadmap/roadmap-v43.1-v44.0.md` → v43.7.0 を `✅ COMPLETE（2026-07-13）`、推定 2922 → 実績 2922 に修正
- [x] `versions/v40-v45/v43.7.0/tasks.md` → COMPLETE、全チェックボックス `[x]`

---

## T6 — サイトドキュメント注記（該当なし）

v43.7.0 はバリデーションリリースのため新規 MDX ファイルは作成しない。
匿名レコードリテラルおよびリスト・タプル文脈推論は v43.8.0 以降で MDX 追加予定。

---

## 既知制限の記録

- **匿名レコードリテラル**（`{ name: "Alice", age: 30 }` 型名なし、`tname = ""`）の文脈推論は非対応 → v43.8.0 双方向型推論（Bidirectional / top-down）のスコープ
- **フィールド型検証**: `ERecordLit` の `fields` はフィールド名/型の検証なし（将来課題）
- ロードマップ例 `process({ name: "Alice", age: 30 })` は匿名レコードのため v43.7.0 非対応（v43.8.0 以降）
