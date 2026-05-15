# Favnir v2.8.0 タスクリスト

作成日: 2026-05-13

---

## Phase 0 — バージョン更新

- [x] `Cargo.toml`: `version = "2.8.0"` に変更
- [x] `src/main.rs`: HELP テキストを `v2.8.0` に更新

---

## Phase 1 — `rand` クレートの追加

- [x] `Cargo.toml` に `rand = "0.8"` を追加

---

## Phase 2 — チェッカー拡張（`!Random` エフェクト）

### `src/middle/checker.rs`

- [x] `BUILTIN_EFFECTS` に `"Random"` を追加
  - `const BUILTIN_EFFECTS: &[&str] = &["Pure", "Io", "Db", ..., "Random"];`
- [x] グローバル名前空間リストに `"Random"` を追加
  - `for ns in &["Math", "List", ..., "Random"]`

---

## Phase 3 — VM 拡張（`Random.int` / `Random.float`）

### `src/backend/vm.rs`

- [x] `"Random.int"` ハンドラを追加
  - [x] 引数 `min: Int`, `max: Int` を取り出す
  - [x] `rand::thread_rng().gen_range(min..=max)` で乱数生成
  - [x] `VMValue::Int(n)` を返す
  - [x] 型エラー時は適切なメッセージを返す
- [x] `"Random.float"` ハンドラを追加
  - [x] `rand::thread_rng().gen::<f64>()` で乱数生成
  - [x] `VMValue::Float(f)` を返す

---

## Phase 4 — `runes/stat/stat.fav` の実装

- [x] `public type ProfileReport = { total: Int  min_v: Int  max_v: Int }` を定義
- [x] `public fn sample_int() -> Int !Random` を実装
  - [x] `Random.int(0, 100)` を呼ぶ
- [x] `public fn sample_float() -> Float !Random` を実装
  - [x] `Random.float()` を呼ぶ
- [x] `public fn sample_bool() -> Bool !Random` を実装
  - [x] `Random.int(0, 1) == 1` を返す
- [x] `public fn uniform(min: Int) -> Int -> Int !Random` を実装
  - [x] カリー化: `|max| { Random.int(min, max) }`
- [x] `public fn choice_str(xs: List<String>) -> String !Random` を実装
  - [x] 空リストガード: `List.length(xs) == 0` のとき `""` を返す
  - [x] `Random.int(0, List.length(xs) - 1)` でインデックスを生成
  - [x] `List.first(List.drop(xs, i))` で要素を取得
  - [x] `Option.unwrap_or(..., "")` でデフォルト処理
- [x] `public fn choice_int(xs: List<Int>) -> Int !Random` を実装
  - [x] 空リストガード: `List.length(xs) == 0` のとき `0` を返す
  - [x] `choice_str` と同様の実装
- [x] `public fn list_int(n: Int) -> List<Int> !Random` を実装
  - [x] `List.map(List.range(0, n), |_| Random.int(0, 100))`
- [x] `public fn list_float(n: Int) -> List<Float> !Random` を実装
  - [x] `List.map(List.range(0, n), |_| Random.float())`
- [x] `public fn profile_int(xs: List<Int>) -> ProfileReport` を実装
  - [x] `total: List.length(xs)` を設定
  - [x] `min_v` / `max_v`: `List.fold` + `Option` パターン OR スタブ（0）

---

## Phase 5 — `runes/stat/stat.test.fav` の作成

スタンドアロン形式（型・関数をインラインで定義）。

### uniform のテスト

- [x] `test "uniform min==max always returns that value"`: `uniform(7)(7)` が 7 を返す
- [x] `test "uniform(0)(0) returns 0"`: `uniform(0)(0)` が 0 を返す

### choice_str のテスト

- [x] `test "choice_str single element"`: `choice_str(["only"])` が `"only"` を返す

### choice_int のテスト

- [x] `test "choice_int single element"`: `choice_int([42])` が `42` を返す

### list_int のテスト

- [x] `test "list_int returns correct length"`: `list_int(5)` の長さが 5
- [x] `test "list_int length 0 returns empty list"`: `list_int(0)` の長さが 0

### list_float のテスト

- [x] `test "list_float returns correct length"`: `list_float(3)` の長さが 3

### profile_int のテスト

- [x] `test "profile_int total count"`: `profile_int(4要素リスト)` の `total` が 4
- [x] `test "profile_int empty list"`: `profile_int([])` の `total` が 0

---

## Phase 6 — examples/stat_demo の作成

- [x] `fav/examples/stat_demo/fav.toml` を作成
  - [x] `[rune] name = "stat_demo"`, `src = "src"`
  - [x] `[runes] path = "../../../runes"`
- [x] `fav/examples/stat_demo/src/main.fav` を作成
  - [x] `import rune "stat"` でインポート
  - [x] `stat.sample_int`, `stat.sample_float`, `stat.sample_bool` のデモ
  - [x] `stat.uniform(1)(10)` のデモ
  - [x] `stat.choice_str([...])` のデモ
  - [x] `stat.list_int(5)` のデモ
  - [x] `stat.profile_int(xs)` のデモ

---

## Phase 7 — Rust 統合テスト（src/driver.rs）

- [x] `stat_rune_random_int_min_equals_max`: `Random.int(7, 7)` が常に 7 を返す
- [x] `stat_rune_uniform_deterministic`: `stat.uniform(5)(5)` が 5 を返す
- [x] `stat_rune_choice_str_single`: `stat.choice_str(["only"])` が `"only"` を返す
- [x] `stat_rune_choice_int_single`: `stat.choice_int([42])` が `42` を返す
- [x] `stat_rune_list_int_length`: `stat.list_int(4)` の長さが 4
- [x] `stat_rune_list_float_length`: `stat.list_float(3)` の長さが 3
- [x] `stat_rune_profile_int_total`: `profile_int(3要素)` の `total` が 3
- [x] `stat_rune_sample_bool_returns_bool`: `stat.sample_bool()` が Bool を返す

---

## Phase 8 — ドキュメント・最終確認

### 最終テスト確認

- [x] `cargo build` で警告ゼロを確認
- [x] `cargo test` で全テスト通過を確認（v2.7.0 の 617 → 目標 626 程度）
- [x] `fav test stat/stat.test.fav`（`runes/` から）で全テスト通過

### ドキュメント作成

- [x] `versions/v2.8.0/langspec.md` を作成
  - [x] `Random.int` / `Random.float` の仕様説明
  - [x] `!Random` エフェクトの説明
  - [x] 各 stat 関数の API ドキュメント
  - [x] `import rune "stat"` の利用手順
  - [x] 互換性（既存テスト影響なし）

---

## 完了条件チェック

- [x] `rand = "0.8"` が `Cargo.toml` の `[dependencies]` に追加されている
- [x] `"Random"` が `BUILTIN_EFFECTS` に含まれている
- [x] `"Random"` がグローバル名前空間に登録されている
- [x] `Random.int(7, 7)` が `7` を返す
- [x] `Random.float()` が `[0.0, 1.0)` の範囲の Float を返す
- [x] `stat.fav` に Rust コードが一行もない
- [x] `sample_int()` が Int を返す
- [x] `sample_float()` が Float を返す
- [x] `sample_bool()` が Bool を返す
- [x] `uniform(5)(5)` が 5 を返す
- [x] `choice_str(["only"])` が `"only"` を返す
- [x] `choice_int([42])` が `42` を返す
- [x] `list_int(5)` が長さ 5 のリストを返す
- [x] `list_float(3)` が長さ 3 のリストを返す
- [x] `profile_int(xs).total` が `List.length(xs)` に等しい
- [x] `import rune "stat"` でユーザーコードから使える
- [x] `cargo test` 全テスト通過
- [x] `cargo build` 警告ゼロ
- [x] `Cargo.toml` バージョンが `"2.8.0"`
- [x] `versions/v2.8.0/langspec.md` 作成済み
