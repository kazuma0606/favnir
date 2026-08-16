# v72.8.0 タスクリスト — インタラクティブチュートリアル（`fav learn`）

Date: 2026-08-12
Status: 未着手

---

## T0: 事前確認

- [ ] `fav/Cargo.toml` のバージョンが `72.7.0` であることを確認
- [ ] `cargo test` が 3638 tests pass（0 failures）であることを確認
- [ ] `driver.rs` に `v727000_tests` モジュールが存在することを確認
- [ ] `driver.rs` に `v728000_tests` が未存在であることを確認
- [ ] `driver.rs` 内の `"72.7.0"` 文字列（バージョンアサーション）の件数を grep で確認しておく

---

## T1: `LearnChapter` 構造体 + `LEARN_CHAPTERS` 静的データ追加

- [ ] `#[derive(Debug)] pub struct LearnChapter` を追加した（`chapter: u32` / `title: &'static str` / `prompt: &'static str` / `hint: &'static str` / `expected_contains: &'static str`）
- [ ] `pub static LEARN_CHAPTERS: &[LearnChapter]` を追加した（5 エントリ）
  - Chapter 1: `expected_contains = "fn main"`（最初のパイプライン）
  - Chapter 2: `expected_contains = "schema"`（型システムの力）
  - Chapter 3: `expected_contains = "import rune"`（Rune を使ったデータ処理）
  - Chapter 4: `expected_contains = "Llm"`（AI パイプライン）
  - Chapter 5: `expected_contains = "par"`（分散実行）
- [ ] `cargo build` でエラーがないことを確認

---

## T2: `cmd_learn` 追加

- [ ] `pub fn cmd_learn()` を追加した
  - `"Favnir インタラクティブチュートリアル v1.0"` を表示する
  - 各章を順番に表示し stdin から 1 行読み込む
  - `expected_contains` を含む入力で `"✓ 正解！ 次へ進みます。"` を表示して次章へ
  - 含まない入力で `"ヒント: {hint}"` を表示して再入力を促す
  - 全 5 章クリア後に `"全章完了！ fav.dev/docs で次のステップへ。"` を表示する
- [ ] `cargo build` でエラーがないことを確認

---

## T3: `main.rs` — `fav learn` コマンド追加

- [ ] `cmd_learn` を driver.rs の import リストに追加した
- [ ] `Some("learn")` アームを追加した（`crate::driver::cmd_learn()` を呼ぶ）
- [ ] `cargo build` でエラーがないことを確認

---

## T4: `v728000_tests` モジュール追加

- [ ] `v727000_tests` モジュールの直後に `v728000_tests` モジュールを追加した
- [ ] `use super::LEARN_CHAPTERS` を追加した（`LearnChapter` はフィールドアクセスで型注釈不要なので不要）
- [ ] `learn_chapter1_exists` テストを実装した
  - `LEARN_CHAPTERS.len() >= 1` を assert
  - `LEARN_CHAPTERS[0].chapter == 1` を assert
  - `LEARN_CHAPTERS[0].title` が空でないことを assert
  - `LEARN_CHAPTERS[0].expected_contains` が `"fn"` または `"main"` を含むことを assert
- [ ] `learn_chapter5_exists` テストを実装した
  - `LEARN_CHAPTERS.len() >= 5` を assert
  - `LEARN_CHAPTERS[4].chapter == 5` を assert
  - `LEARN_CHAPTERS[4].title` が空でないことを assert
  - `LEARN_CHAPTERS[4].expected_contains` が `"par"` を含むことを assert
- [ ] `cargo test v728000` で 2 件 pass することを確認

---

## T5: `fav/Cargo.toml` バージョン更新 + `driver.rs` version アサーション更新

- [ ] `fav/Cargo.toml` の `version = "72.7.0"` → `version = "72.8.0"` に変更した
- [ ] `driver.rs` 内の `version = \"72.7.0\"` 文字列を `version = \"72.8.0\"` に replace_all した
- [ ] `driver.rs` 内のエラーメッセージ `"Cargo.toml version should be 72.7.0"` を `"72.8.0"` に replace_all した
- [ ] `driver.rs` 内のエラーメッセージ `"Cargo.toml should declare version 72.7.0"` を `"72.8.0"` に replace_all した
- [ ] 残存 72.7.0 はコメント・セクションヘッダーのみで意図的保持を確認

---

## T6: 部分テスト確認

- [ ] `cargo test v728000` で 2 件 pass することを確認

---

## T7: 全体テスト確認

- [ ] `cargo test` 全体で 3640 tests pass（0 failures）であることを確認

---

## T8: `CHANGELOG.md` 更新

- [ ] `## [v72.8.0]` エントリを先頭に追加した

---

## T9: `versions/current.md` 更新

- [ ] 「進行中バージョン」を `v72.8.0`（インタラクティブチュートリアル）に更新した
- [ ] 「次に切る版」を `v72.9.0` に更新した

---

## T10: 最終確認

- [ ] `cargo test v728000` で 2 件 pass することを確認
- [ ] `cargo test` 全体で 3640 tests pass（0 failures）であることを確認
- [ ] `fav/Cargo.toml` のバージョンが `72.8.0` であることを確認
- [ ] `LEARN_CHAPTERS` に 5 エントリが存在することを確認
- [ ] `cmd_learn` が `pub fn` で存在することを確認
- [ ] `main.rs` に `fav learn` → `cmd_learn()` の呼び出しが存在することを確認
- [ ] `versions/current.md` の「進行中バージョン」が `v72.8.0`、「次に切る版」が `v72.9.0` であることを確認

---

## スコープ外（明示的除外）

- 進捗保存（`~/.fav_learn_progress`）— v73.x 以降
- `:skip` / `:quit` / `:restart` コマンド — v73.x 以降
- サイト側ドキュメント更新（v73.x 以降）
- stdin/stdout インタラクションのユニットテスト（プロセス依存のため除外）
