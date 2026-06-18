# Favnir v9.3.0 実装計画 — fav lint

作成日: 2026-06-01

---

## 前提

- ベース: v9.2.0 (1167 tests)
- Rust 変更: **なし**（`checker.fav` + `cli.fav` + 必要なら `vm.rs` に tiny builtin 追加のみ）
- 依存: `checker.fav` の `lint_program` / `lint_source` が中心

---

## Phase A: `checker.fav` — LintWarning 型 + lint ルール実装

### A-1: `LintWarning` レコード型を追加

```favnir
type LintWarning = {
    code:    String
    message: String
    name:    String
}
```

### A-2: W001 — EffectlessSink

```favnir
fn lint_stage_effectless_sink(sd: StageDef) -> List<LintWarning>
```

- `sd.ret_ty == TeSimple("Unit")` かつ `List.length(sd.effects) == 0` → W001

### A-3: W004 — TooManyArgs（stage）

```favnir
fn count_type_args(ty: TypeExpr) -> Int
fn lint_stage_too_many_args(sd: StageDef) -> List<LintWarning>
```

- `count_type_args(sd.param_ty) >= 4` → W004

### A-4: W002 — NoWriteInSeq

```favnir
fn has_write_effect(effects: List<String>) -> Bool
fn lookup_stage_effects(name: String, prog: Program) -> List<String>
fn lint_seq_no_write(sd: SeqDef, prog: Program) -> List<LintWarning>
```

- `seq` の `stages` の最終要素のエフェクトに `!Db` / `!AWS` / `!IO` がない → W002
- 最終 stage の `StageDef` を `prog` から探索して判定

### A-5: W005 — WildcardOnlyMatch

```favnir
fn is_wildcard_only_match(expr: Expr) -> Bool
fn lint_expr_wildcard_match(expr: Expr) -> List<LintWarning>
```

- `EMatch(parts)` の `parts._1`（arms）が `EArm` 1 本のみ + `parts._0 == PWild` → W005

### A-6: W003 — UnusedBinding

```favnir
fn collect_bound_names(expr: Expr) -> List<String>
fn collect_used_names(expr: Expr) -> List<String>
fn lint_unused_bindings(expr: Expr) -> List<LintWarning>
```

- `EBind(parts)` の束縛変数名が後続 `parts._2` の自由変数に含まれない → W003
- `bind _ <- ...` はスキップ（慣用的な無視パターン）

### A-7: `lint_item` / `lint_program` 統合

```favnir
fn lint_item(item: Item, prog: Program) -> List<LintWarning>
fn lint_program(prog: Program) -> List<LintWarning>
```

- 各 `Item` に対して A-2〜A-6 を適用
- `List.concat` で結果をフラット化

---

## Phase B: `checker.fav` — `lint_source` 公開 API

### B-1: `public fn lint_source(src: String) -> Result<String, String>`

`lex → parse_tokens → lint_program → format_warnings` のパイプライン。

```favnir
// 警告リストを改行区切り文字列にシリアライズ
fn format_warning(w: LintWarning) -> String
fn format_warnings(ws: List<LintWarning>) -> String

public fn lint_source(src: String) -> Result<String, String>
    // Ok(formatted_warnings) | Err(parse_error)
```

出力フォーマット（改行区切り）:
```
W001:stage DoNothing:stage DoNothing の戻り型が Unit ですがエフェクトがありません
W003:result:変数 result は定義されていますが使用されていません
```

または `""` （警告なし）。

### B-2: `fav check fav/self/checker.fav` — self-check 通過確認

---

## Phase C: Rust ブリッジ（最小）

### C-1: `Compiler.lint_source_raw` builtin を `vm.rs` に追加

```rust
"Compiler.lint_source_raw" => {
    let src = vm_string(args...)?;
    match crate::checker_fav_runner::lint_source_str(&src) {
        Ok(output) => Ok(ok_vm(VMValue::Str(output))),
        Err(msg)   => Ok(err_vm(VMValue::Str(msg))),
    }
}
```

### C-2: `lint_source_str` を `checker_fav_runner.rs` に追加

既存 `check_source_str` と同パターン（OnceLock キャッシュ済み artifact を再利用）。

```rust
pub fn lint_source_str(src: &str) -> Result<String, String> { ... }
```

---

## Phase D: `cli.fav` — `fav lint` コマンド

### D-1: `CmdLint(String, Bool)` を `CliCmd` に追加

```favnir
| CmdLint(String, Bool)   // (path, warn_as_error)
```

### D-2: `parse_lint_cmd` / `run_lint` 実装

```favnir
fn parse_lint_cmd(args: List<String>) -> CliCmd
fn run_lint(path: String, warn_as_error: Bool) -> Unit !IO
```

`run_lint` のロジック:
1. `IO.read_file_raw(path)` でソース読み込み
2. `Compiler.lint_source_raw(src)` で警告文字列取得
3. 警告が空 → `"ok: <path>"` を表示
4. 警告あり + `warn_as_error == false` → 警告を表示（終了コード 0）
5. 警告あり + `warn_as_error == true` → 警告を stderr に出力 + `IO.exit_raw(1)`

### D-3: `parse_named_cmd` に `"lint"` ブランチ追加

### D-4: `run_help` に lint コマンド説明を追加

---

## Phase E: 統合テスト

### E-1: `lint_w001_effectless_sink`

```rust
let src = "stage Noop: String -> Unit = |s| ()";
let output = lint_source_str(src).unwrap();
assert!(output.contains("W001"));
```

### E-2: `lint_w002_no_write_in_seq`

```rust
let src = "stage A: Int -> Int = |x| x\nseq S = A";
let output = lint_source_str(src).unwrap();
assert!(output.contains("W002"));
```

### E-3: `lint_w003_unused_binding`

```rust
let src = "fn f() -> Int {\n    bind x <- 42\n    99\n}";
let output = lint_source_str(src).unwrap();
assert!(output.contains("W003"));
```

### E-4: `lint_w004_too_many_args`

```rust
let src = "stage Big: (Int, Int, Int, Int, Int) -> Int = |x| 0";
let output = lint_source_str(src).unwrap();
assert!(output.contains("W004"));
```

### E-5: `lint_w005_wildcard_only`

```rust
let src = "fn f(x: Int) -> String {\n    match x {\n        _ => \"ok\"\n    }\n}";
let output = lint_source_str(src).unwrap();
assert!(output.contains("W005"));
```

### E-6: `lint_clean_source_no_warnings`

```rust
let src = "fn add(a: Int, b: Int) -> Int { a + b }";
let output = lint_source_str(src).unwrap();
assert_eq!(output.trim(), "");
```

---

## Phase F: self-check + Bootstrap 検証

- F-1: `fav check fav/self/checker.fav` — self-check 通過
- F-2: `cargo test bootstrap` — `bytecode_A == bytecode_B` 維持確認
- F-3: `cargo test` — 全件通過（1172 件以上）

---

## Phase G: バージョン更新・ドキュメント

- G-1: `Cargo.toml` version → `"9.3.0"`
- G-2: `versions/v9.3.0/tasks.md` 完了チェック
- G-3: `memory/MEMORY.md` に v9.3.0 完了を記録
- G-4: commit

---

## 実装上の注意

### W002 の stage 探索
`seq` の最終 stage 名を `prog.items` から `IStage` として探す。
`IStage` が見つからない場合（外部 stage 等）は警告しない（偽陽性を避ける）。

### W003 の実装難度
`EBind` の束縛変数が後続式（`parts._2`）に現れるかを再帰的に確認する。
`bind _ <- ...` は名前が `"_"` なのでスキップ条件に含める。

### W005 の判定
`EMatch(parts)` の `parts._1` が `EArm(arm_parts)` で `arm_parts._3 == EArmNil`（腕が1本）、
かつ `arm_parts._0 == PWild` の場合に W005。

### `lint_source` の出力形式
`check_raw` が `Ok(msg)` / `Err(msg)` を返すのと同様に、
`lint_source` は `Ok(warnings_str)` / `Err(parse_error)` を返す。
`cli.fav` はパース失敗時 `Err` を受け取ったらエラー表示して終了。

---

## テスト数見込み

| フェーズ | テスト数 |
|---|---|
| E-1〜E-6（lint unit tests） | 6 件 |
| 既存（v9.2.0） | 1167 件 |
| **合計** | **1173 件以上** |
