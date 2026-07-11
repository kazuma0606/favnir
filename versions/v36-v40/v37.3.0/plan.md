# v37.3.0 実装計画 — `join` ステージ演算子（関数形式）

## 変更ファイル一覧

| ファイル | 変更種別 | 内容 |
|---|---|---|
| `fav/src/backend/vm.rs` | 変更 | `"List.join_on"` ビルトイン追加（`"List.join"` の直後） |
| `fav/src/middle/checker.rs` | 変更 | `("List", "join_on")` 戻り型定義追加（`("List", "filter")` の直後） |
| `fav/src/driver.rs` | 変更 | `v37200_tests` スタブ化 / `v37300_tests` 追加 |
| `fav/Cargo.toml` | 更新 | `version = "37.2.0"` → `"37.3.0"` |
| `CHANGELOG.md` | 追記 | `[v37.3.0]` エントリ追加 |
| `versions/roadmap/roadmap-v37.1-v38.0.md` | 更新 | v37.3.0 の完了条件をスコープ縮小後の内容に更新 + ✅ マーク |
| `versions/current.md` | 更新 | 最新安定版 v37.3.0、次バージョン v37.4.0 |

## 実装順序

### Step 1: CHANGELOG.md に [v37.3.0] エントリ追加

`## [v37.2.0]` の `---` セパレータ直後に挿入（日付は実装当日）。

```markdown
## [v37.3.0] - 2026-07-09

### Added
- `List.join_on(left, right, pred)` VM ビルトイン追加（left semi-join / hash join）
- `checker.rs` に `("List", "join_on")` 戻り型定義追加
- `v37300_tests` 3 テスト追加

---
```

### Step 1.5: T0 — vm.rs の 2 引数 call_value 使用確認（必須）

`List.join_on` は `call_value(artifact, pred, vec![l, r])` と **2 引数**で呼ぶ。
`List.filter` は 1 引数のため参照にならない。

事前に vm.rs 内で 2 要素 vec を渡している既存ケースを grep 確認:

```bash
grep -n "call_value.*vec!\[" fav/src/backend/vm.rs | head -20
```

`List.sort_by` / `List.zip_with` 等で 2 引数パターンが存在することを確認してから実装に進む。
確認できない場合は `List.sort_by` 付近のコードを Read して引数渡し方法を特定する。

### Step 2: `vm.rs` — `"List.join_on"` ビルトイン追加

**実装箇所:** `"List.join"` ケース終端（`}` 行 11084）の直後

実装パターン: `List.filter`（行 3253-3283）の `call_value` + ループ構造と同じ。
ラベル付き `'outer: for l in` で semi-join を実装。

spec.md §1 のコードブロックに従い追加。

### Step 3: `checker.rs` — `("List", "join_on")` 戻り型定義追加

**実装箇所:** `("List", "filter")` ケース（行 5726-5729）の直後

```rust
("List", "join_on") => {
    let elem = self.expect_list_arg(&arg_tys, 0, span);
    Some(Type::List(Box::new(elem)))
}
```

1 行ずつ Read で確認してから Edit。

### Step 4: driver.rs — `v37200_tests::cargo_toml_version_is_37_2_0` スタブ化

ライブアサーション → `// Stubbed: version bumped to 37.3.0` に変更。

### Step 5: driver.rs — `v37300_tests` モジュール追加

`v37200_tests` の閉じ `}` の行番号を Read で特定してから Edit を実行。

追加内容は spec.md §3 のコードブロックに従う。
- `use super::{build_artifact, exec_artifact_main}` / `use crate::frontend::parser::Parser` / `use crate::value::Value`
- ローカル `run()` ヘルパー（v37100_tests と同じパターン）

### Step 6: Cargo.toml バージョン更新

Step 1〜5 完了・`cargo build` でコンパイルエラーがないことを確認後に `37.2.0` → `37.3.0` に更新。
**理由:** `cargo_toml_version_is_37_3_0` テストが `include_str!("../Cargo.toml")` を参照するため、
バージョン更新はテスト追加後・`cargo test` 実行前に必要。

## 依存関係

- `vm.rs` の `"List.join_on"` は `self.call_value` が必要 → `"List.filter"` と同じ self メソッドを使用（追加 import 不要）
- `checker.rs` の `expect_list_arg` は既存メソッド → 追加 import 不要
- `FavList::new` は vm.rs の `"List.filter"` で既に使用済み → 追加 import 不要
- `v37300_tests` は `run()` を使うため `use super::{build_artifact, exec_artifact_main}` + `use crate::value::Value` が必要

## リスク

| リスク | 対処 |
|---|---|
| `FavList` が vm.rs でインポートされていない | grep で `use.*FavList` を確認（List.filter で使用済みのためほぼ不要） |
| `call_value` の引数として 2 引数クロージャが VM で正しく動かない | Step 1.5 の grep で既存使用例を確認済みであれば問題ない。テスト失敗時は vm.rs の sort_by 等を参照 |
| `'outer` ラベル付き continue が Rust で構文エラーになる | `cargo build` エラーを確認して修正 |
