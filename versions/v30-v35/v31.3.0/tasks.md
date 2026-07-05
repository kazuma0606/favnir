# v31.3.0 — タスクリスト

**ステータス**: COMPLETE

---

## 前提確認（T0）

- [x] `fav/Cargo.toml` の version が `31.2.0` であること
- [x] `cargo test 2>&1 | grep "test result"` を実行して通過件数（2430 passed 想定）を実測確認すること
- [x] `driver.rs` に `mod v313000_tests` が存在しないこと
- [x] v31.2.0 が COMPLETE であること
- [x] `get_explain_text("E0002")` が `None` を返すこと（追加対象）
- [x] `get_explain_text("E0003")` が `None` を返すこと（追加対象）
- [x] `get_explain_text("E0005")` が `None` を返すこと（追加対象）
- [x] `get_explain_text("E0021")` が `None` を返すこと（追加対象）

---

## 実装タスク

- [x] **T1** `fav/Cargo.toml` — version を `31.2.0` → `31.3.0` に更新
- [x] **T2** `fav/src/driver.rs` — `cargo_toml_version_is_31_2_0` をスタブ化
- [x] **T3** `fav/src/driver.rs` — `get_explain_text()` に E0002/E0003/E0004/E0005/E0006 を追加
- [x] **T4** `fav/src/driver.rs` — `get_explain_text()` に E0010/E0011/E0019/E0020/E0021 を追加
- [x] **T5** `fav/src/driver.rs` — `cmd_explain_code()` の unknown 時にコード一覧を表示するよう改善
- [x] **T6** `fav/src/driver.rs` — `v313000_tests`（3 件）を追加（`use super::*` あり）
- [x] **T7** `CHANGELOG.md` — `[v31.3.0]` セクションを先頭に追記
- [x] **T8** `benchmarks/v31.3.0.json` — 新規作成
- [x] **T9** `versions/current.md` — 「最新安定版」欄を v31.3.0 に更新

---

## テスト確認

- [x] **T10** `cargo test --bin fav v313000 2>&1 | tail -8` — 3/3 PASS
- [x] **T11** `cargo test 2>&1 | grep "test result"` — 全件 PASS（2433 passed、0 failures）

---

## 完了処理

- [x] **T12** `benchmarks/v31.3.0.json` の `tests_passed` を実測値で更新（2433 — 暫定値と一致）
- [x] **T13** このファイル（tasks.md）を COMPLETE に更新（全チェックボックス `[x]`）

---

## 完了条件チェックリスト（spec.md 対応）

- [x] `Cargo.toml` version = `"31.3.0"`
- [x] `get_explain_text()` が E0002〜E0021 の全コードで `Some(...)` を返す
- [x] `fav explain E0001` 〜 `fav explain E0021` が説明テキストを出力する
- [x] `fav explain unknown` が E0001〜E0021 の既知コード一覧を表示して exit(1) する（W コードは含まない）
- [x] `cargo test v313000` — 3/3 PASS
- [x] `cargo test` — 全件 PASS（0 failures）
- [x] `CHANGELOG.md` に `[v31.3.0]` セクション
- [x] `benchmarks/v31.3.0.json` 存在
- [x] `benchmarks/v31.3.0.json` の `tests_passed` が実測値で更新されていること（2433）
- [x] `versions/current.md` を v31.3.0 に更新
- [x] tasks.md が COMPLETE

---

## コードレビューチェックリスト

- [x] `v313000_tests` に `use super::*` があること
- [x] `cargo_toml_version_is_31_2_0` が空スタブになっていること（コメント付き）
- [x] `get_explain_text("E0002")` / `"E0003"` / `"E0004"` / `"E0005"` / `"E0006"` が `Some(...)` を返すこと（E0003 は checker.fav 検出、E0016 は Rust checker 検出と explain テキストで区別）
- [x] `get_explain_text("E0010")` / `"E0011"` / `"E0019"` / `"E0020"` / `"E0021"` が `Some(...)` を返すこと
- [x] 既存の `get_explain_text()` アーム（E0001/E0007/E0008/E0009/E0012-E0018）が変更されていないこと
- [x] `cmd_explain_code()` の unknown 時に E0001〜E0021 のコード一覧が `eprintln!` で出力されること
- [x] `cmd_explain_code()` の unknown 時も `process::exit(1)` で終了すること
- [x] site/ MDX が変更されていないこと（OUT OF SCOPE）
