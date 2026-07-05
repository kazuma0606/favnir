# v31.3.0 仕様書 — fav explain E0001 コマンド完成

## 概要

`fav explain E0001` 〜 `fav explain E0021` が詳細な説明・原因・修正例を出力できるようにする。
また `fav explain unknown`（未知のコード）がエラーコード一覧を表示するよう改善する。

---

## 背景

ロードマップ v31.3 より:

```bash
$ fav explain E0001
error[E0001]: undefined variable

説明:
  スコープ内に定義されていない変数を参照しようとしました。

よくある原因:
  1. 変数名の typo（例: `userId` を `user_id` と書いた）
  2. bind の前に変数を使用した
  3. 別の関数スコープで定義された変数を参照した

修正例:
  // NG
  fn process() -> String {
      user_id  // 未定義
  }

  // OK
  fn process(user_id: String) -> String {
      user_id
  }

参照: https://favnir.dev/errors/E0001
```

---

## 既存実装の確認事項

| 項目 | 状態 |
|---|---|
| `fav explain E0001` ルーティング (`main.rs:812`) | **実装済み** — `cmd_explain_code()` を呼ぶ |
| `cmd_explain_code()` (driver.rs:10273) | **実装済み** — `get_explain_text()` を呼ぶ |
| `get_explain_text()` E0001 | **実装済み** |
| `get_explain_text()` E0007/E0008/E0009/E0012-E0018 | **実装済み** |
| `get_explain_text()` E0002/E0003/E0004/E0005/E0006 | **未実装** — 追加対象 |
| `get_explain_text()` E0010/E0011/E0019/E0020/E0021 | **未実装** — 追加対象 |
| `fav explain unknown` 時の挙動 | **要改善** — 現状は `eprintln!` + `exit(1)` のみ |

---

## スコープ

### IN SCOPE

- `fav/Cargo.toml` — version `31.2.0` → `31.3.0`
- `fav/src/driver.rs` — `cargo_toml_version_is_31_2_0` をスタブ化
- `fav/src/driver.rs` — `get_explain_text()` に E0002〜E0006/E0010/E0011/E0019/E0020/E0021 のテキストを追加
- `fav/src/driver.rs` — `cmd_explain_code()` の unknown 時に `get_explain_text()` の既知コード一覧を表示
- `fav/src/driver.rs` — `v313000_tests`（3 件）追加（`use super::*` あり）
- `CHANGELOG.md` — `[v31.3.0]` セクション追加
- `benchmarks/v31.3.0.json` 新規作成
- `versions/current.md` — v31.3.0 に更新

### OUT OF SCOPE

- E0022〜E0320 への explain テキスト追加 — 順次追加予定
- `fav explain` 以外のエラー表示コマンドの変更 — 対象外
- site/ MDX 更新 — v32.0 マイルストーン宣言時に実施

---

## 実装詳細

### get_explain_text() 追加エントリ

E0001〜E0021 の未追加コード（E0002/E0003/E0004/E0005/E0006/E0010/E0011/E0019/E0020/E0021）に
既存エントリと同形式（コードタイトル + 説明 + 修正例 + 関連）のテキストを追加する。

| コード | タイトル | 内容 |
|---|---|---|
| E0002 | Condition type error | 条件式が Bool でない |
| E0003 | Effect not declared (self-hosted) | checker.fav（self-hosted checker）が検出するエフェクト未宣言（Rust checker が検出する同種エラーは E0016）|
| E0004 | Non-exhaustive pattern match | Option/Result の全ケースをカバーしていない |
| E0005 | Type annotation mismatch | 型注釈と推論型が不一致 |
| E0006 | Match arm type mismatch | match の各 arm の型が異なる |
| E0010 | Interface not fully implemented | interface の全メソッドを実装していない |
| E0011 | Undefined type | 未定義の型を参照している |
| E0019 | Circular interface inheritance | interface の継承が循環している |
| E0020 | Capability not available | ctx に必要な capability がない |
| E0021 | Wrong context type | 必要な ctx フィールドが存在しない |

### cmd_explain_code() の改善

unknown コード時に既知コード一覧（E0001〜E0021）を表示する:

```
error: unknown error code `E9999`
Available codes:
  E0001  Undefined variable
  E0002  Condition type error
  ...
  E0021  Wrong context type
  (use `fav explain <code>` to see details)
```

実装方針: `get_explain_text()` の既知コードをハードコードしたリストから `eprintln!` で出力し、`process::exit(1)` する。

> **注意**: 表示対象は E0001〜E0021 のみ（W コードは含まない）。

---

## テスト設計（v313000_tests — 3 件）

| # | テスト名 | 確認内容 |
|---|---------|----------|
| 1 | `cargo_toml_version_is_31_3_0` | `Cargo.toml` に `version = "31.3.0"` |
| 2 | `benchmark_v31_3_0_exists` | `benchmarks/v31.3.0.json` に `"31.3.0"` |
| 3 | `get_explain_text_e0002_through_e0021` | E0002/E0003/E0004/E0005/E0006/E0010/E0011/E0019/E0020/E0021 が `Some(...)` を返す |

> `v313000_tests` は `use super::*` あり（`get_explain_text` は `pub(crate)` のため `super::` で参照可能）。

---

## 完了条件

- `Cargo.toml` version = `"31.3.0"`
- `get_explain_text("E0002")` 〜 `get_explain_text("E0021")` が全コードで `Some(...)` を返す
- `fav explain E0001` 〜 `fav explain E0021` が説明テキストを出力する
- `fav explain unknown` が既知コード一覧を表示して exit(1) する
- `cargo test v313000` — 3/3 PASS
- `cargo test` — 全件 PASS（0 failures）
- `CHANGELOG.md` に `[v31.3.0]` セクション
- `benchmarks/v31.3.0.json` 存在
- `benchmarks/v31.3.0.json` の `tests_passed` が実測値（`cargo test` 後）で記録されていること
- `versions/current.md` を v31.3.0 に更新
- `tasks.md` が COMPLETE
