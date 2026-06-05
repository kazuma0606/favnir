# Favnir v11.2.0 仕様書

Date: 2026-06-06
Theme: stage / seq → Python パイプライン変換

---

## 概要

v11.1.0 で構築した `emit_python.rs` を拡張し、
Fav のパイプライン構造（`stage` / `seq`）を Python 関数チェーンに変換する。

v11.2.0 のスコープ:
1. **`stage` → Python `def`** — 型シグネチャ付き関数として生成
2. **`seq` → Python 合成関数** — `def pipeline(x): return c(b(a(x)))` 形式
3. **`fn main()` → `__main__` ガード** — `if __name__ == "__main__": main()`
4. **`IO.argv()` → `sys.argv[1:]`** — CLI 引数受け渡し
5. **`par` ステップ** — `concurrent.futures.ThreadPoolExecutor` を使った並列実行に変換
6. **テスト** — `stage` / `seq` / `main` / `par` の Python 出力検証 8 件

---

## 構文変換仕様

### 1. `stage` → Python `def`

**入力 (Fav)**:
```fav
stage Validate: List<TxnRow> -> List<TxnRow> !IO = |rows| {
  bind valid <- List.filter(rows, |row| row.amount > 0.0)
  bind n <- List.length(valid)
  bind _ <- IO.println(String.concat("Valid: ", Int.to_string(n)))
  valid
}
```

**出力 (Python)**:
```python
# effects: IO
def validate(rows: List[TxnRow]) -> List[TxnRow]:
    valid = [_x for _x in rows if (lambda row: (row.amount > 0.0))(_x)]
    n = len(valid)
    print("Valid: " + str(n))
    return valid
```

変換ルール:
- `stage Name: A -> B !Eff = |param| { body }` → `def name(param: A) -> B:`
- ステージ名は **PascalCase → snake_case** に変換（`LoadAll` → `load_all`）
- シングルパラメータ closure の `|param|` が引数名になる
- パラメータが複数の場合は `|a, b|` → `def stage(a, b):`
- エフェクト宣言はコメントに落とす

**PascalCase → snake_case 変換ルール:**
```
LoadAll     → load_all
ValidateTxn → validate_txn
WriteOutput → write_output
```
実装: 大文字の前（先頭以外）に `_` を挿入して全小文字化。

---

### 2. `seq` → Python 合成関数

**入力 (Fav)**:
```fav
seq AnalyzePipeline = LoadAll |> Validate |> WriteOutput
```

**出力 (Python)**:
```python
def analyze_pipeline(x):
    return write_output(validate(load_all(x)))
```

変換ルール:
- `seq Name = A |> B |> C` → `def name(x): return c(b(a(x)))`
- ステージ名はすべて snake_case に変換
- 引数名は `x`（汎用）
- 2ステージの場合: `def name(x): return b(a(x))`
- 1ステージの場合: `def name(x): return a(x)`

---

### 3. `par` ステップ → 並列実行

**入力 (Fav)**:
```fav
seq ParallelPipeline = Load |> par [EnrichA, EnrichB] |> Merge
```

**出力 (Python)**:
```python
def parallel_pipeline(x):
    _step0 = load(x)
    from concurrent.futures import ThreadPoolExecutor
    with ThreadPoolExecutor() as _pool:
        _futures = [_pool.submit(enrich_a, _step0), _pool.submit(enrich_b, _step0)]
        _par_results = [_f.result() for _f in _futures]
    return merge(_par_results)
```

変換ルール:
- `par [A, B]` の前後のステージは通常通りチェーン
- `par` ステップは `ThreadPoolExecutor` で並列実行し、結果をリストで次ステージに渡す

---

### 4. `fn main()` → `__main__` ガード

**入力 (Fav)**:
```fav
fn main() -> Unit !IO !AWS {
  bind paths <- IO.argv()
  AnalyzePipeline(paths)
}
```

**出力 (Python)**:
```python
# effects: IO, Unknown
def main() -> None:
    paths = sys.argv[1:]
    return analyze_pipeline(paths)

if __name__ == "__main__":
    main()
```

変換ルール:
- `fn main()` を検出した場合、ファイル末尾に `if __name__ == "__main__": main()` を追加
- `IO.argv()` → `sys.argv[1:]`（v11.1.0 の IO プレースホルダーを上書き）

---

### 5. `IO.argv()` の変換

v11.1.0 では `IO.argv()` → `_io_argv()` のプレースホルダーだったが、
v11.2.0 で正式変換:

| Fav | Python |
|---|---|
| `IO.argv()` | `sys.argv[1:]` |
| `IO.argv_all()` | `sys.argv` |

`sys` は prelude で `import sys` 済みのため追加インポート不要。

---

### 6. 名前変換関数

```rust
/// PascalCase / camelCase → snake_case
fn to_snake(name: &str) -> String {
    let mut out = String::new();
    for (i, ch) in name.chars().enumerate() {
        if ch.is_uppercase() && i > 0 {
            out.push('_');
        }
        out.push(ch.to_lowercase().next().unwrap());
    }
    out
}
```

例:
| 入力 | 出力 |
|---|---|
| `LoadAll` | `load_all` |
| `ValidateTxn` | `validate_txn` |
| `WriteOutput` | `write_output` |
| `AnalyzePipeline` | `analyze_pipeline` |
| `IOHelper` | `i_o_helper` |（※ 略語は非対応、v11.2.0 スコープ外）

---

## テスト方針

`emit_python.rs` の `v11200_tests` モジュールに 8 件追加。
テスト件数: 7（v11.1.0）+ 8（v11.2.0）= **15 件**

| テスト名 | 検証内容 |
|---|---|
| `transpile_stage_basic` | `stage Foo: A -> B = \|x\| { x }` → `def foo(x: A) -> B:` |
| `transpile_stage_effects_comment` | `!IO` エフェクトがコメントになる |
| `transpile_stage_multiline_body` | `bind` / `IO.println` を含む stage の変換 |
| `transpile_seq_two_stages` | `seq P = A \|> B` → `def p(x): return b(a(x))` |
| `transpile_seq_three_stages` | `seq P = A \|> B \|> C` → 3段チェーン |
| `transpile_seq_snake_case` | `LoadAll` → `load_all` の名前変換 |
| `transpile_main_guard` | `fn main()` → `if __name__ == "__main__": main()` |
| `transpile_io_argv` | `IO.argv()` → `sys.argv[1:]` |

---

## テスト件数

| バージョン | 追加テスト | 累計（lib） |
|---|---|---|
| v11.1.0 | +8 | 683 |
| **v11.2.0** | **+8** | **691** |

---

## 完了条件

| 条件 | 状態 |
|---|---|
| `fav transpile --target python` で stage/seq が Python 関数に変換される | |
| `fn main()` を含む .fav ファイルで `if __name__ == "__main__":` が生成される | |
| `cargo test v11200` 8 件通過 | |
| `cargo test --lib` 全件通過（691 件） | |
