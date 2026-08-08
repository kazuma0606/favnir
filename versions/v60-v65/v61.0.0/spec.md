# v61.0.0 Spec — Developer Experience 2.0 宣言 ★クリーンアップ

Date: 2026-07-31
Status: COMPLETE

---

## 概要

v60.1〜v60.9 で実装した全 DX 機能の統合確認と、**Developer Experience 2.0** マイルストーンの宣言。
新規言語機能の追加は行わず、**宣言・バージョン更新・クリーンアップ**のみを実施する。

---

## 宣言文

> 「エラーはソース位置を指し、修正候補は即座に現れる。
>  エディタは意図を理解し、フォーマッタはコメントを守る。
>  REPL でパイプラインを対話的に探索でき、ドキュメントは自動生成される。
>
>  Favnir のエラーメッセージはデータエンジニアの道標になった。
>
>  これが Favnir v61.0 — Developer Experience 2.0 の姿である。」

---

## 実装スコープ

| ファイル | 変更種別 | 内容 |
|---|---|---|
| `fav/Cargo.toml` | 更新 | version `"60.0.0"` → `"61.0.0"` |
| `MILESTONE.md` | 追加 | Developer Experience 2.0 宣言エントリ（v60.1〜v60.9 達成内容一覧付き） |
| `CHANGELOG.md` | 追加 | v61.0.0 エントリ（v60.1〜v60.9 全機能を網羅） |
| `README.md` | 追加 | v61.0.0 / Developer Experience 2.0 言及 |
| `fav/src/driver.rs` | 追加 | `v61000_tests` モジュール（4 件） |
| `fav/src/driver.rs` | 更新 | 旧バージョン tests の version assertion を `"61.0.0"` に更新（9 件） |

★クリーンアップ（`cargo clean`）はテスト全通過後に実施。

---

## テスト仕様（`v61000_tests` 4 件）

### `cargo_toml_version_is_61_0_0`
- `Cargo.toml` が `version = "61.0.0"` を含むことを確認

### `changelog_has_v61_0_0`
- `CHANGELOG.md` が `"v61.0.0"` を含むことを確認

### `milestone_has_dx2`
- `MILESTONE.md` が `"Developer Experience 2.0"` を含むことを確認

### `readme_mentions_dx2`
- `README.md` が `"Developer Experience 2.0"` を含むことを確認

---

## ベーステスト数の注意点

ロードマップ記載「ベース 3348 + 4 = 3352」は v60.8.0 XSS テスト追加前の想定値。
実際の v60.9.0 テスト数: **3349**（ロードマップ記載 3348 + XSS テスト +1）

実際のテスト数目標: **3349 + 4 = 3353** tests passed, 0 failed

---

## 完了条件

- v60.1〜v60.9 で追加された全 driver.rs テストが pass（テスト数推移表参照）
- `v61000_tests` 4 件全 pass
- 総テスト数: **3353** tests passed, 0 failed
- `MILESTONE.md` に `"Developer Experience 2.0"` 宣言エントリ追加
- `cargo clean` 完了

## テスト数推移（参照用）

| バージョン | テスト数 | 備考 |
|---|---|---|
| v60.0.0（ベース） | 3330 | Enterprise 1.0 宣言後 |
| v60.1.0〜v60.9.0 | +18 (+2×9) | DX 機能 9 スプリント |
| v60.9.0 実績 | 3349 | ロードマップ記載 3348 + XSS テスト +1 |
| v61.0.0 宣言 | **3353** | ベース 3349 + `v61000_tests` 4 件 |
