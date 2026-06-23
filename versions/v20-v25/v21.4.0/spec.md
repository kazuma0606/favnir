# v21.4.0 Spec — `fav lint` 強化（W010〜W019）

## 概要

`fav lint` に 10 の新しい静的解析ルール（W010〜W019）を追加し、
より実践的なコード品質チェックを提供する。

**テーマ**: Developer Tooling Complete シリーズ第4弾 — 「lint で設計の問題を早期検出」

---

## 動機

v9.3.0 で W001〜W005 を実装した。その後 v13.1.0〜v17.7.0 で W006〜W009 が追加された。
本バージョンでは未実装だった 10 ルールを追加し、実践的なコードレビュー相当の静的解析を実現する。

---

## ルールコード方針

ロードマップは W006〜W015 と記載しているが、W006〜W009 は既存実装で使用済み。
本バージョンでは **W010〜W019** を使用する（番号の重複を回避）。

### ロードマップとの対応表

| ロードマップ | 本バージョン | 変更点 |
|-------------|-------------|--------|
| W006（100行超 stage） | **W010** | 閾値を「100 行超」→「30 stmt 超」に変更（Span に end_line がないため stmt 数で代替） |
| W007（effectless IO） | **W011** | 変更なし |
| W008（unused type） | **W012** | 変更なし |
| W009（map+filter）   | **W013** | 変更なし |
| W010（Result.ok）    | **W014** | 変更なし |
| W011（rebind）       | **W015** | 変更なし |
| W012（wildcard only）| **W016** | 変更なし |
| W013（deep nesting） | **W017** | 変更なし |
| W014（magic number） | **W018** | 変更なし |
| W015（concat chain） | **W019** | 変更なし |

---

## 新規ルール一覧

| コード | ルール名 | 内容 |
|--------|----------|------|
| W010 | `stage_too_large` | TrfDef の body が 30 ステートメント超（分割を推奨） |
| W011 | `effectless_io_call` | エフェクト宣言なし TrfDef が ambient 名前空間を呼び出している |
| W012 | `unused_type` | TypeDef が他のどこからも参照されていない |
| W013 | `map_filter_chain` | `List.map(...)` の直後に `List.filter(...)` → `List.filter_map` を推奨 |
| W014 | `redundant_result_ok` | `bind x <- Result.ok(expr)` — `Result.ok` ラップが不要 |
| W015 | `rebind_in_block` | 同じ名前が同一ブロック内で 2 回以上 `bind` されている |
| W016 | `wildcard_only_match` | `match` が `_ =>` のみ — 具体的なパターンを推奨 |
| W017 | `deep_nesting` | `match` / `if` のネストが 4 レベル超 — 関数抽出を推奨 |
| W018 | `magic_number` | 整数・浮動小数点リテラルが 100 超（型注釈を除く） |
| W019 | `string_concat_chain` | `String.concat` を 2 回以上連鎖 → f-string を推奨 |

---

## 実装仕様

### W010 — `stage_too_large`

```
対象: TrfDef
条件: body.stmts.len() > 30
メッセージ: stage `<name>` has <N> statements (>30); consider splitting into smaller stages
```

Note: Span に end_line がないため、行数ではなく stmt 数で代替する。

### W011 — `effectless_io_call`

```
対象: TrfDef
条件: effects が空（Pure）かつ body 内に ambient 名前空間の FieldAccess 呼び出しがある
メッセージ: stage `<name>` calls `<NS>.<fn>` but declares no effects; add `!Io` or use ctx
ambient 名前空間（新規定数 W011_AMBIENT として lint.rs に追加）:
  IO, Postgres, AWS, Snowflake, Http, Grpc, Llm, Queue, Cache, Slack, Email
注意: lint.rs に AMBIENT_NAMESPACES という既存定数は存在しない。W011_AMBIENT を新規追加する。
```

### W012 — `unused_type`

```
対象: Item::TypeDef（Item::AliasDecl は visibility フィールドなしのため対象外）
条件: TypeDef.name がプログラム内の TypeExpr::Named に一度も登場しない
      かつ TypeDef.visibility.is_none()（pub は外部参照の可能性あり）
メッセージ: type `<name>` is defined but never used
collect_used_type_names は TypeExpr::Named の型引数を再帰収集（List<Ghost> の Ghost も検出）
```

### W013 — `map_filter_chain`

```
対象: Expr::Pipeline
条件: 連続する 2 ステップが Apply(FieldAccess(Ident("List"), "map"), ...) と
      Apply(FieldAccess(Ident("List"), "filter"), ...) である
メッセージ: `List.map(...) |> List.filter(...)` can be simplified to `List.filter_map(...)`
```

### W014 — `redundant_result_ok`

```
対象: Stmt::Bind(b)
条件: b.pattern が Pattern::Bind(name, _)（Wildcard ではない）かつ
      b.expr が Apply(FieldAccess(Ident("Result"), "ok"), [inner]) の形式
メッセージ: `bind <x> <- Result.ok(...)` — Result.ok is redundant; bind directly: `bind <x> <- <inner>`
注意: BindStmt には .name フィールドがなく .pattern: Pattern を使う
      Pattern::Wildcard(_) は除外（意図的な破棄）
```

### W015 — `rebind_in_block`

```
対象: Block
条件: block.stmts 内で Stmt::Bind(b) の b.pattern が Pattern::Bind(name, _) で同名が 2 回以上
      Pattern::Wildcard(_) はスキップ（`bind _ <- ...` は意図的）
メッセージ: binding `<name>` is rebound in the same block (first bind at line <L>)
注意: BindStmt.pattern: Pattern から名前を取り出す。.name フィールドは存在しない
```

### W016 — `wildcard_only_match`

```
対象: Expr::Match
条件: arms が 1 つのみ かつ arm.pattern が Pattern::Wildcard
メッセージ: match has only a wildcard arm `_ =>`; consider using a specific pattern or removing the match
```

### W017 — `deep_nesting`

```
対象: Expr（再帰的）
条件: Match または If ノードのネスト深さが 5 以上（depth > 4 = 5+ で発火）
      count_nesting_depth(expr) > 4
      ネスト 4 段は発火しない（> 4 = 5以上）
メッセージ: nesting depth <N> exceeds 4; consider extracting inner logic to a separate function
テスト: 5重 match → W017 を確認 + 4重 match → W017 が出ないことを確認（ネガティブ）
```

### W018 — `magic_number`

```
対象: Expr::Lit(Lit::Int(n)) / Expr::Lit(Lit::Float(f))
条件: n.unsigned_abs() > 100 または f.abs() > 100.0（101 以上で発火）
      ただし TypeExpr 内のリテラル（const generics = TypeExpr::ConstInt）は除外
メッセージ: magic number `<n>`; consider extracting to a named constant
境界値: 100 ちょうどは > 100 の条件に引っかからないため発火しない（101 以上が対象）
```

### W019 — `string_concat_chain`

```
対象: Expr
条件: Apply(FieldAccess(Ident("String"), "concat"), [Apply(FieldAccess(Ident("String"), "concat"), ...), ...])
      つまり String.concat の引数が別の String.concat 呼び出し
メッセージ: chained `String.concat` calls; consider using an f-string instead
```

---

## 新規 AST ノードの追加なし

W010〜W019 はすべて既存の AST ノード（TrfDef, TypeDef, Expr, Stmt 等）を走査するのみ。
新しい Expr/Stmt/TypeExpr/Pattern variant を追加しないため、
`fmt.rs` / `emit_python.rs` / `lineage.rs` / `lint.rs` の exhaustive match 更新は不要。

## 既存 lint_program との統合

`lint_program(program: &Program) -> Vec<LintError>` に W010〜W019 チェックを追加する。
各ルールは独立した `check_w010_*` 関数として実装し、`lint_program` の末尾で呼び出す。

`cmd_lint`（driver.rs）は既存のまま変更なし。
`cmd_explain_hint` に W010〜W019 の説明文を追加する。

---

## 成果物一覧

| 成果物 | 役割 |
|--------|------|
| `fav/src/lint.rs` | W010〜W019 ルール実装（`lint_program` に統合） |
| `fav/src/driver.rs` | `cmd_explain_hint` に W010〜W019 追加、v214000_tests 追加 |
| `fav/Cargo.toml` | version `21.3.0` → `21.4.0` |
| `CHANGELOG.md` | v21.4.0 エントリ追加 |
| `site/content/docs/tools/lint.mdx` | 更新（W010〜W019 ルール追加） |

---

## テスト（v214000_tests、12件）

| テスト名 | 内容 |
|----------|------|
| `version_is_21_4_0` | Cargo.toml に `"21.4.0"` が含まれる |
| `lint_w010_stage_too_large` | 31 stmt の stage → W010 |
| `lint_w011_effectless_io_call` | エフェクトなし stage が IO.println → W011 |
| `lint_w012_unused_type` | 参照されない TypeDef → W012 |
| `lint_w013_map_filter_chain` | List.map |> List.filter パイプライン → W013 |
| `lint_w014_redundant_result_ok` | bind x <- Result.ok(expr) → W014 |
| `lint_w015_rebind_in_block` | 同名 bind が同一ブロックに 2 回 → W015 |
| `lint_w016_wildcard_only_match` | match が `_ =>` のみ → W016 |
| `lint_w017_deep_nesting` | ネスト 5 段 → W017 |
| `lint_w017_no_w017_at_4_levels` | ネスト 4 段 → W017 が出ない（ネガティブ） |
| `lint_w018_magic_number` | リテラル 9999 → W018 |
| `lint_w019_string_concat_chain` | String.concat(String.concat(...)) → W019 |

---

## 完了条件

- [ ] `fav lint src/` で W010〜W019 が表示される
- [ ] 各ルールが独立して動作する（他ルールを誤検知しない）
- [ ] `cargo test v214000` — 12/12 PASS
- [ ] `cargo test` — リグレッションなし（exit 0）
- [ ] `CHANGELOG.md` に v21.4.0 エントリが追加されている
- [ ] `fav/Cargo.toml` version が `21.4.0`
- [ ] `site/content/docs/tools/lint.mdx` に W010〜W019 が記載されている
