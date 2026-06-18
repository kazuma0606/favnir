# Favnir v9.13.0 仕様書 — `par` 並列 stage 実行

作成日: 2026-06-03

---

## 概要

独立した複数の `stage` を並列実行する `par` 構文を追加する。
「複数ソースから並列取得 → マージ → 保存」というパターンをデータパイプライン専用言語として
言語ネイティブに表現できるようにする。

---

## 構文

### seq 内での par 使用

```favnir
stage FetchOrders: String -> List<Order> !Db  = |conn|   { ... }
stage FetchPrices: String -> List<Price> !AWS = |bucket| { ... }
stage Merge:       (List<Order>, List<Price>) -> Report   = |pair| { ... }
stage Save:        Report -> Unit !Db                     = |r|    { ... }

// par: 複数 stage を並列実行し、結果をタプルで次 stage に渡す
seq FullReport = par [FetchOrders, FetchPrices] |> Merge |> Save
```

`par` は `seq` のパイプライン定義の中でのみ使用できる。
トップレベルの式や `fn` のボディでは使えない。

### 一般形

```
seq <Name> = <step> (|> <step>)*

<step> ::= <stage_name>
         | par [ <stage_name> (, <stage_name>)* ]
```

---

## 型規則

### par の入力型・出力型

`par [A, B, ...]` において：

- 各 stage の入力型はすべて同一でなければならない（= 直前ステップの出力型）
- `par` の出力型はタプル：`(A_out, B_out, ...)`
- 次の stage の入力型は `(A_out, B_out, ...)` のタプルでなければならない

```
stage A: X -> Y1
stage B: X -> Y2
par [A, B] の出力型: (Y1, Y2)
次の stage: (Y1, Y2) -> Z
```

### エフェクト規則

`par [A, B]` のエフェクトは各 stage のエフェクトの **和集合**：

```
stage FetchOrders: String -> List<Order> !Db
stage FetchPrices: String -> List<Price> !AWS

par [FetchOrders, FetchPrices] のエフェクト: !Db !AWS

seq FullReport = par [FetchOrders, FetchPrices] |> Merge |> Save
// seq FullReport のエフェクト: !Db !AWS（Merge は純粋）
```

---

## エラー

### E0016 — ParInputTypeMismatch
`par [A, B]` の各 stage の入力型が一致しない場合。

```
E0016: par ステップ内の stage 入力型が一致しません
  FetchOrders: String -> List<Order>  (入力: String)
  FetchPrices: Int    -> List<Price>  (入力: Int)
```

### E0017 — ParStageMustBeKnown
`par [...]` 内に未定義の stage 名が含まれる場合（E0007 の par 版）。

```
E0017: par ステップ内の stage 'FetchUsers' が定義されていません
```

---

## VM 実行モデル

### 並列実行の仕組み

コンパイル後、`par [A, B]` は VM プリミティブ呼び出しに変換される：

```
IO.par_execute_raw(["FetchOrders", "FetchPrices"], input)
```

VM は：
1. `std::thread::spawn` で各 stage 用のスレッドを起動
2. 各スレッドで `VM::run(artifact, fn_idx_of(stage), vec![input.clone()])` を実行
3. すべてのスレッドの完了を待機（`thread.join()`）
4. 結果を `VMValue::Tuple([a_result, b_result])` として返す

### artifact のクローン

各スレッドは `Arc<FvcArtifact>` を共有参照で保持（clone は参照カウントのみ）。
VM ローカル状態（スタック・フレーム）は各スレッドで独立。

### DB 接続の扱い

`!Db` エフェクトを持つ stage が並列実行される場合、各スレッドは独立した DB 接続を使用する。
`VM::run` に `db_url` を渡す経路を使い、スレッドごとに新規接続を確立する。

---

## compiler.fav の変更

### SeqStep 型の追加

```favnir
type SeqStep =
    | SStage(String)
    | SPar(List<String>)
```

### SeqDef の変更

```favnir
// before
type SeqDef = {
    is_public:   Bool
    is_abstract: Bool
    name:        String
    stages:      List<String>  // ← 変更
    doc:         String
}

// after
type SeqDef = {
    is_public:   Bool
    is_abstract: Bool
    name:        String
    stages:      List<SeqStep>  // ← SeqStep に変更
    doc:         String
}
```

### build_pipe_call の変更

```favnir
fn build_pipe_call(stages: List<SeqStep>, input_expr: Expr) -> Expr {
    match List.first(stages) {
        None => input_expr
        Some(step) => {
            bind rest <- List.drop(stages, 1)
            match step {
                SStage(name) =>
                    build_pipe_call(rest, ECall(name, [input_expr]))
                SPar(names) =>
                    // IO.par_execute_raw(["A", "B"], input) を生成
                    bind name_list_expr <- build_string_list_expr(names)
                    bind par_call <- ECall("IO.par_execute_raw", [name_list_expr, input_expr])
                    build_pipe_call(rest, par_call)
            }
        }
    }
}
```

### par パーサーの追加（compiler.fav 内）

`parse_seq_pipeline_acc` を拡張し、`par [A, B]` を `SPar(["A", "B"])` として読む。

---

## checker.fav の変更

### par ステップの型チェック

```
check_par_step(names, input_ty, env) -> Result<(out_ty, effects), String>
  for each name in names:
    lookup stage type: name: input_ty -> out_ty_i !E_i
    verify all input_ty_i == input_ty (E0016 on mismatch)
    verify all names are defined (E0017 on missing)
  out_ty = Tuple([out_ty_1, out_ty_2, ...])
  effects = union(E_1, E_2, ...)
```

---

## fav explain の変更

`par` を含む seq の `--lineage` 出力：

```
seq FullReport
  par[
    FetchOrders  !Db
    FetchPrices  !AWS
  ]
  |> Merge
  |> Save  !Db

Effects: !Db !AWS
```

---

## 完了条件

| 条件 | 確認 |
|---|---|
| `par [A, B] \|> Merge` が並列実行される | |
| 各 stage のエフェクトが型チェッカーで追跡される | |
| E0016（入力型不一致）が検出できる | |
| E0017（未定義 stage 参照）が検出できる | |
| `fav explain --lineage` が並列構造を表示する | |
| `cargo test checker_fav_wire_self_check` 通過 | |
| `cargo test bootstrap` 維持 | |
| 統合テスト 6 件以上 | |

---

## スコープ外（将来版）

- `par` 内の stage が異なる入力型を持つケース（fan-out パターン）
- `par` のネスト（`par [par [...], B]`）
- `par` の動的 stage リスト（`par $stages`）
- キャンセル・タイムアウト制御
- エラー発生時の部分完了ハンドリング（最初のエラーで中断 vs 全完了待ち）
