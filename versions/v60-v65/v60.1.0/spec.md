# v60.1.0 Spec — エラーメッセージ span 表示（ソース位置・アンダーライン）

Date: 2026-07-30
Status: 計画中

---

## 概要

`fav check` のテキスト出力に rustc スタイルのソース位置表示（`-->` / `|` / `^` アンダーライン形式）を確立し、
テストで保証する。

---

## 背景

**ロードマップとの関係:**
ロードマップ v60.1.0 セクションは「`error_catalog.rs` の `DiagEntry` に `span` フィールドを追加」「`checker.rs` / `parser.rs` を更新」「`main.rs` の `print_diag` を実装」を記載している。しかしこれらはすでに `driver.rs` L47-95 の `format_diagnostic` として完結している（`Span` 情報の付与・アンダーライン出力）。本バージョンはその公開とテスト保証のみを行う。

`driver.rs` の `format_diagnostic` 関数（L47-95、private）はすでに以下の形式でエラーを出力する：

```
error[E0001]: undefined variable: `user_id`
  --> pipeline.fav:12:15
   |
12 |   transform(user_id, name)
   |             ^^^^^^^
  = help: ...
```

ただし、この関数は private かつテストがない。v60.1.0 では：

1. `pub fn cmd_check_span_output(src: &str) -> String` ヘルパーを `driver.rs` に追加し、
   任意のソースコードに対して span 付きエラー出力文字列を返せるようにする
2. Rust テスト 2 件でこの出力形式を保証する

---

## 完了条件

- `cargo test` 全通過（3330 → **3332** tests passed, 0 failed）
- 以下の 2 テストが pass:
  - `v60100_tests::error_span_display_e0001`
  - `v60100_tests::error_span_underline_format`

---

## 変更ファイル

| ファイル | 変更内容 |
|---|---|
| `fav/src/driver.rs` | `cmd_check_span_output` 追加、`v60100_tests` モジュール追加 |

---

## テスト仕様

### `error_span_display_e0001`

- 未定義変数（E0001）を含むソースを `cmd_check_span_output` で実行
- 出力に `"-->"` が含まれることを assert

### `error_span_underline_format`

- 未定義変数を含むソースを `cmd_check_span_output` で実行
- 出力に `"^"` が含まれることを assert（アンダーライン形式の確認）

---

## ローリングチェック

v60.1.0 はサブバージョンのため、`fav/Cargo.toml` の `version` は `"60.0.0"` のまま変更しない。
よって rolling check（`version = "..."` assertion）の更新も不要。
ただし v60100_tests 追加後は `driver.rs` の合計テスト数が +2 増加する。

---

## 参照

- ロードマップ: `versions/roadmap/roadmap-v60.1-v61.0.md`（v60.1.0 セクション）
- 既存実装: `fav/src/driver.rs` L47-95 `format_diagnostic`
- 次バージョン: v60.2.0 — `fav check --fix` 自動修正
