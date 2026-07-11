# v37.4.0 spec — `fan_out` / `fan_in` リスト演算子

## バージョン概要

| 項目 | 内容 |
|---|---|
| バージョン | v37.4.0 |
| テーマ | `List.fan_out` / `List.fan_in` VM ビルトイン追加 |
| 前提 | v37.3.0 COMPLETE — `List.join_on` 実装済み |
| 完了条件 | `v37400_tests` 全テスト pass・`cargo test` 0 failures（≥ 2719 件） |

## 背景と目的

ロードマップ v37.4.0 は「`fan_out` ブロックが動作する / Rust テスト 2 件」を定義している。
`fan_out` / `fan_in` キーワード構文（AST・parser 変更）は単一スプリントには過大なため、
v37.3.0 の `List.join_on` と同様に **VM ビルトイン（関数形式）** として実装する。

**今バージョンで行うこと（スコープ確定）:**
- `List.fan_out(list, n)` — リストを n 個のほぼ等サイズチャンクに分割 → `List<List<A>>`
- `List.fan_in(lists)` — `List<List<A>>` を 1 レベルフラット化 → `List<A>`
- `checker.rs` に両関数の戻り型定義を追加
- `fan_out ... | ...` キーワード構文は v37.5.0 以降に持ち越し

**ユースケース（データパイプライン）:**
```
1. fan_out: 大量レコードを n ワーカーに分割して並列処理
2. fan_in:  各ワーカーの結果を 1 つのリストにマージ
```

## 実装スコープ

### 1. `fav/src/backend/vm.rs` — `List.fan_out` / `List.fan_in` ビルトイン追加

`"List.join_on"` ケースの直後（メソッドスコープ内、`self.error` が使用可能な箇所）に追加する。

**`List.fan_out`:**

```rust
"List.fan_out" => {
    if args.len() != 2 {
        return Err(self.error(artifact, "List.fan_out requires 2 arguments: (List, Int)"));
    }
    let mut it = args.into_iter();
    let list = it.next().expect("list");
    let n    = it.next().expect("n");
    match (list, n) {
        (VMValue::List(fl), VMValue::Int(n)) => {
            if n <= 0 {
                return Err(self.error(artifact, "List.fan_out: n must be >= 1"));
            }
            let items: Vec<VMValue> = fl.into_iter().collect();
            let chunk_size = {
                let total = items.len();
                if total == 0 { 1 } else { (total + (n as usize) - 1) / (n as usize) }
            };
            let chunks: Vec<VMValue> = items
                .chunks(chunk_size)
                .map(|c| VMValue::List(FavList::new(c.to_vec())))
                .collect();
            Ok(VMValue::List(FavList::new(chunks)))
        }
        _ => Err(self.error(artifact, "List.fan_out requires (List, Int)")),
    }
}
```

**`List.fan_in`（fan_out の直後に追加）:**

```rust
"List.fan_in" => {
    if args.len() != 1 {
        return Err(self.error(artifact, "List.fan_in requires 1 argument: (List<List>)"));
    }
    let list = args.into_iter().next().expect("list");
    match list {
        VMValue::List(outer) => {
            let mut out = Vec::new();
            for item in outer.iter() {
                match item {
                    VMValue::List(inner) => {
                        for v in inner.iter() {
                            out.push(v.clone());
                        }
                    }
                    other => {
                        return Err(self.error(
                            artifact,
                            &format!(
                                "List.fan_in expects List<List>, got inner element {}",
                                vmvalue_type_name(&other)
                            ),
                        ));
                    }
                }
            }
            Ok(VMValue::List(FavList::new(out)))
        }
        _ => Err(self.error(artifact, "List.fan_in requires a List<List> argument")),
    }
}
```

**動作:**
- `fan_out([1,2,3,4], 2)` → `[[1,2],[3,4]]`（2 チャンク）
- `fan_in([[1,2],[3,4]])` → `[1,2,3,4]`（フラット化）
- `fan_in(fan_out(list, n))` は元のリストと同じ要素を返す

**注意:** `List.join_on` と同様に、`self.error` / `FavList::new` を使うためメソッドスコープ（`List.filter` / `List.join_on` と同じ箇所）に配置すること。静的関数スコープ（`List.join` 周辺）に追加すると `self` が未解決エラーになる。

### 2. `fav/src/middle/checker.rs` — 戻り型定義追加

`("List", "join_on")` ケース（行 5730〜5735）の直後に追加:

```rust
("List", "fan_out") => {
    // fan_out(list, n) -> List<List<A>> where A = elem type of list
    // n の型（arg_tys[1]）が Int であることの検証は v37.5 以降
    let elem = self.expect_list_arg(&arg_tys, 0, span);
    Some(Type::List(Box::new(Type::List(Box::new(elem)))))
}
("List", "fan_in") => {
    // fan_in(lists) -> List<A> — 内部型の推論は v37.5 以降
    // Type::Unknown は checker.rs の unify で任意の型と一致するため、
    // 戻り型の型伝播は保証されない（List.length 等の型に依存しない呼び出しは問題なし）
    Some(Type::List(Box::new(Type::Unknown)))
}
```

### 3. `fav/src/driver.rs` — `v37400_tests` モジュール追加

```rust
// ── v37400_tests (v37.4.0) — `fan_out` / `fan_in` リスト演算子 ─────────────────
#[cfg(test)]
mod v37400_tests {
    use super::{build_artifact, exec_artifact_main};
    use crate::frontend::parser::Parser;
    use crate::value::Value;

    fn run(src: &str) -> Value {
        let program = Parser::parse_str(src, "v37400_test.fav").expect("parse");
        let artifact = build_artifact(&program);
        exec_artifact_main(&artifact, None).expect("exec")
    }

    #[test]
    fn cargo_toml_version_is_37_4_0() {
        let cargo = include_str!("../Cargo.toml");
        assert!(cargo.contains("37.4.0"), "Cargo.toml must contain version 37.4.0");
    }
    #[test]
    fn changelog_has_v37_4_0() {
        let src = include_str!("../../CHANGELOG.md");
        assert!(src.contains("[v37.4.0]"), "CHANGELOG.md must contain [v37.4.0]");
    }
    #[test]
    fn list_fan_out_basic() {
        // fan_out([1,2,3,4], 2) → [[1,2],[3,4]] → length 2
        // Favnir list: List.singleton + List.push（[x,y,z] リテラルなし）
        let result = run(r#"
public fn main() -> Int {
    bind items  <- List.push(List.push(List.push(List.singleton(1), 2), 3), 4)
    bind chunks <- List.fan_out(items, 2)
    List.length(chunks)
}
"#);
        assert_eq!(result, Value::Int(2));
    }
    #[test]
    fn list_fan_in_basic() {
        // fan_in(fan_out([1,2,3,4], 2)) → [1,2,3,4] → length 4
        let result = run(r#"
public fn main() -> Int {
    bind items  <- List.push(List.push(List.push(List.singleton(1), 2), 3), 4)
    bind chunks <- List.fan_out(items, 2)
    bind merged <- List.fan_in(chunks)
    List.length(merged)
}
"#);
        assert_eq!(result, Value::Int(4));
    }
}
```

**重要:**
- `exec_artifact_main` は `public fn main()` を必要とする（`fn main()` では見つからない）
- リストは `List.singleton(x)` + `List.push(list, x)` で構築する

## 注意事項

### vm.rs の配置場所

`List.fan_out` / `List.fan_in` は `self.error` を使用するため **メソッドスコープ** に配置する。

- **正しい場所（メソッドスコープ）:** `"List.join_on"` の直後（約行 3324 付近）
  → `self.error` / `self.call_value` が使用可能
- **誤った場所（静的関数スコープ）:** `"List.join"` の直後（約行 11084 付近）
  → `self` が未解決エラーになる（v37.3.0 実装時に同じ問題が発生）

`vmvalue_type_name` は自由関数（`self.` 不要）。`self.error` はメソッド。どちらもメソッドスコープ内では使用可能。

### fan_out の chunk_size 計算

- `chunk_size = ceil(total / n)` = `(total + n - 1) / n`
- `total < n` の場合、チャンク数は n より少なくなる（許容動作）
- `total == 0` の場合: `chunk_size = 1` だが `[].chunks(1)` は空イテレータ → **空リスト（0 チャンク）を返す**
  - 「空リストを 1 チャンクとして返す」という動作は Rust の `chunks()` の仕様上実現しない
  - `fan_out([], n)` の正しい戻り値は `[]`（空の `List<List<A>>`）

### v37300_tests のスタブ化対象

`cargo_toml_version_is_37_3_0` のみスタブ化する。
`changelog_has_v37_3_0` は CHANGELOG に `[v37.3.0]` エントリが残るため変更不要。

### スコープ外（v37.5 以降）

- `fan_out ... | ...` / `fan_in` キーワード構文（AST/parser 変更）
- `("List", "fan_in")` の内部型推論（現状 `Type::Unknown`）
- `fan_out` の並列実行（現状は逐次チャンク分割のみ）

## ロードマップとの整合

ロードマップ v37.4.0 当初: 「`fan_out` ブロックが動作する / Rust テスト 2 件」

**実際のスコープ（ロードマップを T8 で更新して記録）:**
- `fan_out` の実行ロジック: `List.fan_out` 関数として実装 ✓
- `fan_in` の実行ロジック: `List.fan_in` 関数として実装 ✓
- 型チェック: checker.rs に両関数の戻り型定義 ✓
- キーワード構文 `fan_out ... | ...`: v37.5.0 以降に持ち越し
- Rust テスト 4 件（ロードマップ指定 2 件 + meta 2 件）

## 完了条件

| # | 条件 | 検証方法 |
|---|---|---|
| 1 | `Cargo.toml` バージョンが `37.4.0` | `cargo_toml_version_is_37_4_0` テスト |
| 2 | `CHANGELOG.md` に `[v37.4.0]` が含まれる | `changelog_has_v37_4_0` テスト |
| 3 | `List.fan_out([1,2,3,4], 2)` が 2 チャンクを返す | `list_fan_out_basic` テスト |
| 4 | `List.fan_in(fan_out([1,2,3,4], 2))` が length 4 を返す | `list_fan_in_basic` テスト |
| 5 | `cargo test` 全通過（failures=0 かつテスト数 ≥ 2719） | `cargo test` 実行結果（v37.3.0 実績 2715 + v37400_tests 4 件 = 2719） |
