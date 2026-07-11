# v37.4.0 実装計画 — `fan_out` / `fan_in` リスト演算子

## 変更ファイル一覧

| ファイル | 変更種別 | 内容 |
|---|---|---|
| `fav/src/backend/vm.rs` | 変更 | `"List.fan_out"` / `"List.fan_in"` ビルトイン追加（`"List.join_on"` の直後） |
| `fav/src/middle/checker.rs` | 変更 | `("List", "fan_out")` / `("List", "fan_in")` 戻り型定義追加（`("List", "join_on")` の直後） |
| `fav/src/driver.rs` | 変更 | `v37300_tests` スタブ化 / `v37400_tests` 追加 |
| `fav/Cargo.toml` | 更新 | `version = "37.3.0"` → `"37.4.0"` |
| `CHANGELOG.md` | 追記 | `[v37.4.0]` エントリ追加 |
| `versions/roadmap/roadmap-v37.1-v38.0.md` | 更新 | v37.4.0 の完了条件をスコープ縮小後の内容に更新 + ✅ マーク |
| `versions/current.md` | 更新 | 最新安定版 v37.4.0、次バージョン v37.5.0 |

## 実装順序

### Step 1: CHANGELOG.md に [v37.4.0] エントリ追加

`## [v37.3.0]` の `---` セパレータ直後に挿入（日付は実装当日）。

```markdown
## [v37.4.0] - 2026-07-09

### Added
- `List.fan_out(list, n)` VM ビルトイン追加（リストを n チャンクに分割）
- `List.fan_in(lists)` VM ビルトイン追加（List<List> を 1 レベルフラット化）
- `checker.rs` に `("List", "fan_out")` / `("List", "fan_in")` 戻り型定義追加
- `v37400_tests` 4 テスト追加

---
```

### Step 0（T0）: vm.rs メソッドスコープの配置場所確認（必須・Step 1 実施前）

v37.3.0 で判明した問題: `List.join` 付近（静的関数スコープ）に追加すると `self` が未解決エラーになる。
`List.fan_out` / `List.fan_in` は `self.error` を使うため **メソッドスコープ** に配置する。

```bash
grep -n "List.join_on" fav/src/backend/vm.rs
```

`"List.join_on"` ケースの行番号を確認し、その閉じ `}` の直後に `"List.fan_out"` を追加する。

### Step 2: `vm.rs` — `"List.fan_out"` / `"List.fan_in"` ビルトイン追加

**実装箇所:** `"List.join_on"` ケース終端（`}` 行）の直後

spec.md §1 のコードブロックに従い追加。
- `List.fan_out`: 2 引数ガード → `into_iter().collect()` → `chunks(chunk_size)` → `FavList::new`
- `List.fan_in`: 1 引数ガード → 外側ループ → 内側 `VMValue::List` マッチ → `out.push(v)`

### Step 3: `checker.rs` — 戻り型定義追加

**実装箇所:** `("List", "join_on")` ケース（行 5730〜5735）の直後

```rust
("List", "fan_out") => {
    let elem = self.expect_list_arg(&arg_tys, 0, span);
    Some(Type::List(Box::new(Type::List(Box::new(elem)))))
}
("List", "fan_in") => {
    Some(Type::List(Box::new(Type::Unknown)))
}
```

1 行ずつ Read で確認してから Edit。

### Step 4: driver.rs — `v37300_tests::cargo_toml_version_is_37_3_0` スタブ化

ライブアサーション → `// Stubbed: version bumped to 37.4.0 — assertion intentionally removed` に変更。

**注意:** `changelog_has_v37_3_0` はスタブ化しない（CHANGELOG に `[v37.3.0]` エントリが残るため）。

### Step 5: driver.rs — `v37400_tests` モジュール追加

`v37300_tests` の閉じ `}` の行番号を Read で特定してから Edit を実行。

追加内容は spec.md §3 のコードブロックに従う:
- `use super::{build_artifact, exec_artifact_main}` / `use crate::frontend::parser::Parser` / `use crate::value::Value`
- ローカル `run()` ヘルパー（v37300_tests と同じパターン）
- 4 テスト: `cargo_toml_version_is_37_4_0` / `changelog_has_v37_4_0` / `list_fan_out_basic` / `list_fan_in_basic`

### Step 6: Cargo.toml バージョン更新

Step 1〜5 完了・`cargo build` でコンパイルエラーがないことを確認後に `37.3.0` → `37.4.0` に更新。
**理由:** `cargo_toml_version_is_37_4_0` テストが `include_str!("../Cargo.toml")` を参照するため、
バージョン更新はテスト追加後・`cargo test` 実行前に必要。

## 依存関係

- `vm.rs` の `"List.fan_out"` は `self.error` / `FavList::new` / `vmvalue_type_name` が必要 → メソッドスコープ内（`"List.join_on"` の直後）
- `vm.rs` の `"List.fan_in"` も同様 → `"List.fan_out"` の直後
- `checker.rs` の `Type::Unknown` は既存型（行 5720 等で確認済み）→ 追加 import 不要
- `checker.rs` の `expect_list_arg` は既存メソッド → 追加 import 不要
- `v37400_tests` は `run()` を使うため `use super::{build_artifact, exec_artifact_main}` + `use crate::value::Value` が必要

## リスク

| リスク | 対処 |
|---|---|
| vm.rs に `"List.fan_out"` を静的関数スコープに誤配置 | grep で `"List.join_on"` の行番号を確認し、同スコープ内に配置する |
| `chunks()` の返り値型の mismatch | `items: Vec<VMValue>` を `Vec` で確保してから `chunks()` を呼ぶ |
| `Type::Unknown` が将来変更される | 現状 checker.rs 既存コードで多数使用中 → 問題なし |
| `list_fan_in_basic` で fan_out の結果型が List<List<Int>> になるか | `fan_out` の戻り型が checker で `List<List<Int>>` と推論され、`fan_in` 引数として通ることを `cargo test` で確認 |
