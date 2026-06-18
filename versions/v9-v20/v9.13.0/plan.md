# Favnir v9.13.0 実装計画 — `par` 並列 stage 実行

作成日: 2026-06-03

---

## 全体構成

変更が必要なファイルと依存関係：

```
Phase A: Rust AST + パーサー
  fav/src/ast.rs          ← FlwStep enum + FlwDef.steps 変更
  fav/src/frontend/parser.rs  ← par [...] 構文パース

Phase B: Rust VM + 周辺
  fav/src/backend/vm.rs   ← IO.par_execute_raw 実装
  fav/src/middle/compiler.rs  ← FlwStep への対応
  fav/src/middle/ast_lower_checker.rs ← lower_flw_def 更新
  fav/src/checker.rs      ← par エフェクト・型チェック（Rust pipeline）

Phase C: compiler.fav — SeqStep + codegen
  fav/self/compiler.fav   ← SeqStep 型・parser 拡張・build_pipe_call 更新

Phase D: checker.fav — par 型チェック
  fav/self/checker.fav    ← check_par_step・E0016・E0017

Phase E: fav explain + LSP
  fav/src/middle/lineage.rs   ← par ノードをリネージに追加

Phase F: テスト + バージョン更新
  fav/src/driver.rs       ← v9130_tests モジュール
  fav/Cargo.toml          ← version = "9.13.0"
  fav/self/cli.fav        ← version string 更新
```

---

## Phase A — Rust AST + パーサー

### A-1: FlwStep enum の追加（ast.rs）

```rust
/// A single step in a FlwDef pipeline.
#[derive(Debug, Clone)]
pub enum FlwStep {
    /// Single stage: `StageName`
    Stage(String),
    /// Parallel group: `par [A, B, ...]`
    Par(Vec<String>),
}
```

### A-2: FlwDef.steps の型変更（ast.rs）

```rust
pub struct FlwDef {
    pub name: String,
    pub steps: Vec<FlwStep>,  // was Vec<String>
    pub span: Span,
}
```

`AbstractFlwDef.slots` と `FlwBindingDef` は変更不要。

### A-3: パーサー更新（parser.rs）

現在の `parse_flw_pipeline` は `|>`-separated なステージ名リストをパース。
`par [A, B]` を一つの `FlwStep::Par(vec!["A", "B"])` として読む分岐を追加。

```rust
// par [ ident (, ident)* ]
if self.peek() == &TokenKind::Par {
    self.advance(); // consume `par`
    self.expect(&TokenKind::LBracket)?;
    let first = self.expect_ident()?.0;
    let mut names = vec![first];
    while self.peek() == &TokenKind::Comma {
        self.advance();
        names.push(self.expect_ident()?.0);
    }
    self.expect(&TokenKind::RBracket)?;
    steps.push(FlwStep::Par(names));
} else {
    let (name, _) = self.expect_ident()?;
    steps.push(FlwStep::Stage(name));
}
```

`TokenKind::Par` を Rust lexer に追加（キーワード `"par"`）。

### A-4: FlwDef を参照しているすべての Rust コードを更新

`compiler.rs`・`lineage.rs`・`checker.rs`・`ast_lower_checker.rs` で
`steps.iter()` / `def.steps` を使っている箇所を `FlwStep` に対応させる。

---

## Phase B — Rust VM + 周辺

### B-1: IO.par_execute_raw の実装（vm.rs）

```rust
// IO.par_execute_raw(names: List<String>, input: Value) -> Tuple<List<Value>>
// → 各 stage を std::thread::spawn で並列実行し、結果を VMValue::List で返す
"par_execute_raw" => {
    let input = args[1].clone();
    let names = extract_string_list(&args[0]);
    let artifact = Arc::clone(&artifact_arc);
    let db_url = db_url.clone();

    let handles: Vec<_> = names.into_iter().map(|fn_name| {
        let artifact = Arc::clone(&artifact);
        let input = input.clone();
        let db_url = db_url.clone();
        std::thread::spawn(move || {
            let fn_idx = artifact.fn_idx_by_name(&fn_name)?;
            VM::run(&artifact, fn_idx, vec![input], db_url.as_deref())
        })
    }).collect();

    let results: Vec<VMValue> = handles.into_iter()
        .map(|h| h.join().unwrap_or(Err(...)).unwrap_or(VMValue::Unit))
        .collect();

    VMValue::List(Arc::new(results))
}
```

返り値は `VMValue::List` のタプル相当。
`Merge` stage 側でタプル `(a, b)` をパターンマッチする。

### B-2: compiler.rs — FlwStep::Par の IR 生成

`par [A, B]` を `IO.par_execute_raw(["A", "B"], input)` の呼び出し IR に変換。

### B-3: ast_lower_checker.rs — lower_flw_def 更新

`FlwStep::Stage(s)` → `Variant("SStage", s)`
`FlwStep::Par(names)` → `Variant("SPar", List<String>)`
として checker.fav 側の `List<SeqStep>` に変換。

### B-4: checker.rs — Rust pipeline での par 型チェック（最小限）

Rust pipeline での型チェック（`checker.rs`）でも par を素通りせず
E0016/E0017 を検出できる最小実装を追加。

---

## Phase C — compiler.fav

### C-1: SeqStep 型定義の追加

```favnir
type SeqStep =
    | SStage(String)
    | SPar(List<String>)
```

### C-2: SeqDef.stages の型変更

```favnir
type SeqDef = {
    is_public:   Bool
    is_abstract: Bool
    name:        String
    stages:      List<SeqStep>   // was List<String>
    doc:         String
}
```

### C-3: パーサー更新（parse_seq_pipeline_acc）

現在：
```favnir
fn parse_seq_pipeline_acc(toks: List<Token>, acc: List<String>) -> ...
```

変更後：
```favnir
fn parse_seq_pipeline_acc(toks: List<Token>, acc: List<SeqStep>) -> ...
```

`TkPar` トークン + `[A, B]` リストを `SPar(names)` として読む分岐を追加。
`TkIdent` は `SStage(name)` として読む。

### C-4: build_pipe_call の更新

```favnir
fn build_pipe_call(stages: List<SeqStep>, input_expr: Expr) -> Expr {
    match List.first(stages) {
        None => input_expr
        Some(step) => {
            bind rest <- List.drop(stages, 1)
            match step {
                SStage(name) =>
                    build_pipe_call(rest, ECall(name, [input_expr]))
                SPar(names) => {
                    // IO.par_execute_raw(names_list_expr, input_expr) を生成
                    bind name_exprs <- List.map(names, |n| ELit(LStr(n)))
                    bind names_expr <- EList(name_exprs)
                    bind par_call   <- ECall("IO.par_execute_raw", [names_expr, input_expr])
                    build_pipe_call(rest, par_call)
                }
            }
        }
    }
}
```

### C-5: Token / keyword_token への TkPar 追加

```favnir
| TkPar
```

```favnir
else if s == "par" { Option.some(TkPar) }
```

### C-6: compile_seq_def・pretty_seq_def・doc_items_acc 更新

SeqStep に対応。SStage は既存と同様、SPar は pretty 表示用に文字列生成。

---

## Phase D — checker.fav

### D-1: SeqStep 型定義（checker.fav にも追加）

```favnir
type SeqStep =
    | SStage(String)
    | SPar(List<String>)
```

### D-2: SeqDef.stages の型変更（checker.fav）

compiler.fav と同様。

### D-3: check_par_step 関数の追加

```favnir
fn check_par_step(names: List<String>, input_ty: String, env: Env)
    -> Result<String, String>
// - names が空なら E0017
// - 各 name が env に存在するか確認（なければ E0017）
// - 各 name の入力型が input_ty と一致するか確認（なければ E0016）
// - 成功時は "Tuple" + 各 stage の出力型名をつないだ文字列を返す
```

### D-4: check_seq_pipeline の更新

現在の `check_seq_pipeline(stages: List<String>, env)` を
`check_seq_pipeline(stages: List<SeqStep>, env)` に変更。
`SStage` は既存ロジック、`SPar` は `check_par_step` を呼ぶ。

### D-5: check_item の ISeq 処理で SeqStep を参照

`collect_variant_constructors` で `SStage`・`SPar` を variant として登録。

---

## Phase E — fav explain

### E-1: lineage.rs — ParStep ノードの追加

`FlwStep::Par` を辿るとき、各 stage のエフェクトを union して lineage に追加。

### E-2: lineage.fav（cli.fav 側）— par 表示

```
par[
  FetchOrders  !Db
  FetchPrices  !AWS
]
```

---

## Phase F — テスト + バージョン更新

### テスト（v9130_tests）

| テスト名 | 内容 |
|---|---|
| par_executes_in_parallel | `par [A, B] \|> Merge` が並列実行され結果が正しい |
| par_effects_are_unioned | `!Db` + `!AWS` の union がチェックで追跡される |
| par_input_type_mismatch_e0016 | 入力型不一致 → E0016 |
| par_unknown_stage_e0017 | 未定義 stage 参照 → E0017 |
| par_lineage_shows_parallel | `fav explain` が par を並列表示する |
| par_compiles_with_favnir_pipeline | compiler.fav で par を含む seq がコンパイルできる |

---

## 実装順序と依存関係

```
A-1 → A-2 → A-3 → A-4 (Rust コンパイル確認)
           ↓
B-1 → B-2 → B-3 → B-4 (VM + checker.rs)
           ↓
C-1 → C-2 → C-3 → C-4 → C-5 → C-6 (compiler.fav)
           ↓
D-1 → D-2 → D-3 → D-4 → D-5 (checker.fav)
           ↓
E-1 → E-2 (lineage)
           ↓
F-1 (tests + version bump)
```

Rust 部分（Phase A・B）を先に完了させてから Favnir 部分（Phase C・D）を実装する。
`cargo build` が通ることを各フェーズ末で確認する。

---

## 注意点

### FlwDef の後方互換性

既存の `steps: Vec<String>` を参照しているすべての Rust コードを
`FlwStep` に対応させる必要がある（compiler.rs・checker.rs・lineage.rs・
`ast_lower_checker.rs`・ドライバテスト等）。
検索で見落としがないよう `grep -r "\.steps"` で確認する。

### DB 接続のスレッド安全性

`!Db` を持つ stage を並列実行する場合、各スレッドで独立した DB 接続を確立する。
グローバルな DB 接続（thread_local）を共有しない設計にする。

### bootstrap 維持

compiler.fav の `SeqDef.stages: List<SeqStep>` 変更は
bootstrap テストで `bytecode_A == bytecode_B` を確認するため
すべての変更を一貫してコミットする。

### checker.fav の self-check

`checker.fav` に `SeqStep` 型を追加した後、
`cargo test checker_fav_wire_self_check` で self-check が通ることを確認する。
