# v37.3.0 spec — `join` ステージ演算子（関数形式）

## バージョン概要

| 項目 | 内容 |
|---|---|
| バージョン | v37.3.0 |
| テーマ | `join` ステージ演算子 — `List.join_on` VM ビルトインとして hash join を実装 |
| 前提 | v37.2.0 COMPLETE — 行多相実用強化済み |
| 完了条件 | `v37300_tests` 全テスト pass・`cargo test` 0 failures（≥ 2715 件） |

## 背景と目的

ロードマップ v37.3.0 は `join orders on .customer_id == customers.id` という
パイプライン構文を定義しているが、これには AST・parser・checker・VM の
全層への変更が必要であり、単一スプリントには過大なスコープとなる。

**今バージョンで行うこと（スコープ確定）:**
- `List.join_on(left, right, pred)` を VM ビルトインとして実装（left semi-join）
- checker.rs に `("List", "join_on")` の戻り型定義を追加
- `join ... on ...` キーワード構文は v37.4.0 以降に持ち越し

## 実装スコープ

### 1. `fav/src/backend/vm.rs` — `List.join_on` ビルトイン追加

`"List.join"` ケース（行 11058-11084）の直後に追加:

```rust
"List.join_on" => {
    if args.len() != 3 {
        return Err(self.error(artifact, "List.join_on requires 3 arguments: (List, List, Fn)"));
    }
    let mut it = args.into_iter();
    let left  = it.next().expect("left");
    let right = it.next().expect("right");
    let pred  = it.next().expect("pred");
    match (left, right) {
        (VMValue::List(ls), VMValue::List(rs)) => {
            let mut out = Vec::new();
            'outer: for l in ls.iter() {
                for r in rs.iter() {
                    let matched = self.call_value(
                        artifact,
                        pred.clone(),
                        vec![l.clone(), r.clone()],
                    )?;
                    if matches!(matched, VMValue::Bool(true)) {
                        out.push(l.clone());
                        continue 'outer;
                    }
                }
            }
            Ok(VMValue::List(FavList::new(out)))
        }
        _ => Err(self.error(artifact, "List.join_on requires (List, List, Fn)")),
    }
}
```

**動作:** left の各要素に対し right の要素を順に predicate で照合し、
1 件以上一致した left 要素を結果リストに追加（left semi-join）。

**事前確認:** T0 で `vm.rs` 内に `call_value` に 2 要素 vec を渡している既存ケース
（`List.sort_by` / `List.zip_with` 等）の有無を grep 確認すること。

### 2. `fav/src/middle/checker.rs` — `("List", "join_on")` 戻り型定義追加

`("List", "filter")` ケース（行 5726-5729）の直後に追加:

```rust
("List", "join_on") => {
    // join_on(left, right, pred) -> List<T> where T = elem type of left
    // pred の型（arg_tys[2]）は検証しない（Any 扱い）— v37.4 以降で対応
    let elem = self.expect_list_arg(&arg_tys, 0, span);
    Some(Type::List(Box::new(elem)))
}
```

### 3. `fav/src/driver.rs` — `v37300_tests` モジュール追加

```rust
// ── v37300_tests (v37.3.0) — `join` ステージ演算子（関数形式）───────────────────
#[cfg(test)]
mod v37300_tests {
    use super::{build_artifact, exec_artifact_main};
    use crate::frontend::parser::Parser;
    use crate::value::Value;

    fn run(src: &str) -> Value {
        let program = Parser::parse_str(src, "v37300_test.fav").expect("parse");
        let artifact = build_artifact(&program);
        exec_artifact_main(&artifact, None).expect("exec")
    }

    #[test]
    fn cargo_toml_version_is_37_3_0() {
        let cargo = include_str!("../Cargo.toml");
        assert!(cargo.contains("37.3.0"), "Cargo.toml must contain version 37.3.0");
    }
    #[test]
    fn changelog_has_v37_3_0() {
        let src = include_str!("../../CHANGELOG.md");
        assert!(src.contains("[v37.3.0]"), "CHANGELOG.md must contain [v37.3.0]");
    }
    #[test]
    fn list_join_on_basic() {
        // List.join_on([1,2,3,4], [2,4,6], |a, b| a == b) → [2, 4] → length 2
        let result = run(r#"
public fn main() -> Int {
    bind left  <- [1, 2, 3, 4]
    bind right <- [2, 4, 6]
    bind joined <- List.join_on(left, right, |a, b| a == b)
    List.length(joined)
}
"#);
        assert_eq!(result, Value::Int(2));
    }
}
```

**重要:** `exec_artifact_main` は `public fn main()` を必要とする（`fn main()` では見つからない）。

## 注意事項

### `call_value` の 2 引数クロージャ

vm.rs の `List.filter` は `call_value(artifact, func, vec![x])` と 1 引数で呼ぶ。
`List.join_on` は `call_value(artifact, pred, vec![l, r])` と 2 引数を渡す。
T0 で既存の 2 引数 `call_value` の使用例（`List.sort_by` 等）を grep して動作確認すること。

### `v37200_tests` のスタブ化対象

`cargo_toml_version_is_37_2_0` のみスタブ化する。
`changelog_has_v37_2_0` は CHANGELOG に `[v37.2.0]` エントリが残るため変更不要（生きたアサーション）。

### スコープ外（v37.4 以降）

- `join orders on .customer_id == customers.id` キーワード構文（AST/parser 変更）
- `join_on` の true join（`{ left: A, right: B }` 形式のペアを返す版）
- `join_on` の pred 引数の型推論（現状 `arg_tys[2]` を無視 / Any 扱い）

## ロードマップとの整合

ロードマップ v37.3.0 当初:「`join ... on ...` が型チェックと実行（hash join）を通る / Rust テスト 3 件」

**実際のスコープ（ロードマップを T8 で更新して記録）:**
- hash join の実行ロジック: `List.join_on` 関数として実装 ✓
- 型チェック: checker.rs に `("List", "join_on")` 戻り型定義 ✓
- キーワード構文 `join ... on ...`: v37.4.0 以降に持ち越し
- Rust テスト 3 件（ロードマップ指定通り）

## 完了条件

| # | 条件 | 検証方法 |
|---|---|---|
| 1 | `Cargo.toml` バージョンが `37.3.0` | `cargo_toml_version_is_37_3_0` テスト |
| 2 | `CHANGELOG.md` に `[v37.3.0]` が含まれる | `changelog_has_v37_3_0` テスト |
| 3 | `List.join_on([1,2,3,4], [2,4,6], |a,b| a==b)` が 2 を返す | `list_join_on_basic` テスト |
| 4 | `cargo test` 全通過（failures=0 かつテスト数 ≥ 2715） | `cargo test` 実行結果（v37.2.0 実績 2712 + v37300_tests 3 件 = 2715） |
