# v31.1.0 仕様書 — エラーメッセージ v2（rustc スタイル）

## 概要

`fav check` のエラーメッセージに「どう直すか」のヒントを追加する。
`format_diagnostic()` はすでに rustc スタイルで実装済み（driver.rs:47）。
本バージョンでは `get_help_text()` の未設定コード（E0002〜E0006、E0010）にヒントを追加する。

---

## 背景

ロードマップ v31.1 より:

**現状**（E0001 例 — すでに rustc スタイルで表示される）:
```
error[E0001]: undefined variable: user_id
  --> src/stages.fav:12:5
   |
12 |   transform(user_id, name)
   |             ^
  = help: check the variable name for typos
  = help: introduce the variable with `bind x <- expr`
  = 参照: https://favnir.dev/errors/E0001
```

**現状**（E0002〜E0006、E0010 — hint なし）:
```
error[E0005]: type mismatch: expected Int, got String
  --> src/stages.fav:7:3
   |
7  |   "hello"
   |   ^^^^^^^
  = 参照: https://favnir.dev/errors/E0005
（= help: 行がない）
```

**目標**: E0001〜E0010 全コードに `= help:` 行を付与する。

---

## 既存実装の確認事項

| 項目 | 状態 |
|---|---|
| `format_diagnostic()` (driver.rs:47) | **実装済み** — rustc スタイル完全実装 |
| `get_help_text()` (driver.rs:150) | **一部実装** — E0001/E0007/E0008/E0009/E0013/E0014/E0015/E0018 のみ |
| `= 参照: https://favnir.dev/errors/EXXXX` 出力 | **実装済み** (driver.rs:93) |
| `= ヒント:` 行（`TypeError.hints` フィールド由来） | **実装済み** (driver.rs:88) |
| `error_catalog.rs` — `error_hint()` 関数 | 存在しない（`lookup()` / `list_all()` のみ） |

---

## スコープ

### IN SCOPE

- `fav/Cargo.toml` — version `31.0.0` → `31.1.0`
- `fav/src/driver.rs` — `cargo_toml_version_is_31_0_0` をスタブ化
- `fav/src/driver.rs` — `get_help_text()` に E0002〜E0006、E0010 の hint を追加
  - E0002: `"the condition must be a Bool expression"`
  - E0003: `"pattern match requires an enum type or literal"`
  - E0004: `"the right-hand side of bind must return Result<T>"`
  - E0005: `"check that the type annotation matches the inferred type"`
  - E0006: `"all match arms must return the same type"`
  - E0010: `"implement all required methods declared in the interface"`
- `fav/src/driver.rs` — `v311000_tests`（4 件）追加（`use super::*` あり）
- `CHANGELOG.md` — `[v31.1.0]` セクション追加
- `benchmarks/v31.1.0.json` 新規作成
- `versions/current.md` — v31.1.0 に更新

### OUT OF SCOPE

- E0011〜E0021 へのヒント追加 — v31.2.0 で実施
- typo 候補（Levenshtein）— v31.2.0 で実施
- E0011〜E0021 へのヒントテキスト追加（URL 出力自体は driver.rs:93 で実装済み）— v31.2.0 で実施
- `fav explain` コマンド — v31.3.0 で実施
- `format_diagnostic()` の実装変更 — 実装済みのため対象外
- site/ MDX 更新 — v32.0 マイルストーン宣言時に実施

---

## テスト設計（v311000_tests — 4 件）

| # | テスト名 | 確認内容 |
|---|---------|----------|
| 1 | `cargo_toml_version_is_31_1_0` | `Cargo.toml` に `version = "31.1.0"` |
| 2 | `benchmark_v31_1_0_exists` | `benchmarks/v31.1.0.json` に `"31.1.0"` |
| 3 | `get_help_text_e0002_is_set` | `get_help_text("E0002")` が空でないスライスを返す |
| 4 | `get_help_text_e0005_is_set` | `get_help_text("E0005")` が空でないスライスを返す |

> `v311000_tests` は `use super::*` あり（`get_help_text` はモジュール内関数のため）。

---

## 完了条件

- `Cargo.toml` version = `"31.1.0"`
- `get_help_text()` が E0001〜E0010 の全コードに非空スライスを返す
- `cargo test v311000` — 4/4 PASS
- `cargo test` — 全件 PASS（0 failures）
- `CHANGELOG.md` に `[v31.1.0]` セクション
- `benchmarks/v31.1.0.json` 存在
- `benchmarks/v31.1.0.json` の `tests_passed` が実測値（`cargo test` 後）で記録されていること
- `versions/current.md` を v31.1.0 に更新
- `tasks.md` が COMPLETE
