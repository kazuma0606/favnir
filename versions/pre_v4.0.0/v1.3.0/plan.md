# Favnir v1.3.0 実装計画

作成日: 2026-05-07

> スコープを守ることが最優先。各フェーズの Done definition を超えない。
>
> **前提**: v1.2.0 完了（394 テスト通過）
>
> **設計ドキュメント**: `dev/post-v1/roadmap/fav-abstract-flw.md`、`dev/post-v1/roadmap/fav-abstraction-system.md`

---

## 実装順序

```
Phase 0 (version bump)
  → Phase 1 (AST + Lexer + Parser)            ← 全フェーズの前提
  → Phase 2 (Checker: 型検査 + effect 推論)   ← Phase 3 の前提
  → Phase 3 (IR 生成 + VM 実行確認)
  → Phase 4 (fav check 部分束縛警告)          ← Phase 2 完了後
  → Phase 5 (fav explain 統合)                ← Phase 2 完了後、Phase 3 と並行可
  → Phase 6 (テスト・ドキュメント)
```

---

## Phase 0: バージョン更新

### Cargo.toml

```toml
version = "1.3.0"
```

### main.rs

```rust
const HELP: &str = "fav - Favnir language toolchain v1.3.0\n...";
```

---

## Phase 1: AST + Lexer + Parser

### 1-1: ast.rs の変更

#### 新規型の追加

```rust
// ast.rs に追加

/// `abstract trf Name: Input -> Output !Effects`
pub struct AbstractTrfDef {
    pub visibility: Option<Visibility>,
    pub name:       String,
    pub input:      TypeExpr,
    pub output:     TypeExpr,
    pub effects:    Vec<String>,
    pub span:       Span,
}

/// `abstract flw Name<T> { slot: Input -> Output !Effects; ... }`
pub struct AbstractFlwDef {
    pub visibility:  Option<Visibility>,
    pub name:        String,
    pub type_params: Vec<String>,
    pub slots:       Vec<FlwSlot>,
    pub span:        Span,
}

/// スロット宣言: `slot_name: Input -> Output !Effects`
pub struct FlwSlot {
    pub name:    String,
    pub input:   TypeExpr,
    pub output:  TypeExpr,
    pub effects: Vec<String>,
    pub span:    Span,
}

/// `flw Name = Template<T> { slot <- Impl; ... }`
pub struct FlwBindingDef {
    pub visibility: Option<Visibility>,
    pub name:       String,
    pub template:   String,
    pub type_args:  Vec<TypeExpr>,
    pub bindings:   Vec<(String, String)>, // (slot_name, impl_name)
    pub span:       Span,
}
```

#### `Item` enum への追加

```rust
pub enum Item {
    // ... 既存バリアント ...
    AbstractTrfDef(AbstractTrfDef),
    AbstractFlwDef(AbstractFlwDef),
    FlwBindingDef(FlwBindingDef),
}
```

### 1-2: lexer.rs の変更

```rust
// Token enum に追加
Abstract,

// キーワードマップに追加
"abstract" => TokenKind::Abstract,
```

`abstract` は単独では意味を持たず、次のトークン（`trf` / `flw`）とセットで解釈する。

### 1-3: parser.rs — `abstract trf` パース

```rust
fn parse_abstract_trf_def(&mut self, vis: Option<Visibility>) -> Result<AbstractTrfDef, ParseError> {
    let start = self.peek_span().clone();
    self.expect(&TokenKind::Trf)?;
    let (name, _) = self.expect_ident()?;
    self.expect(&TokenKind::Colon)?;
    let input = self.parse_type_expr()?;
    self.expect(&TokenKind::Arrow)?;
    let output = self.parse_type_expr()?;
    let effects = self.parse_effects()?; // 既存の !Effect パース
    Ok(AbstractTrfDef { visibility: vis, name, input, output, effects, span: self.span_from(&start) })
}
```

### 1-4: parser.rs — `abstract flw` パース

```rust
fn parse_abstract_flw_def(&mut self, vis: Option<Visibility>) -> Result<AbstractFlwDef, ParseError> {
    let start = self.peek_span().clone();
    self.expect(&TokenKind::Flw)?;
    let (name, _) = self.expect_ident()?;
    let type_params = self.parse_type_params_opt()?; // `<T, U>` or empty
    self.expect(&TokenKind::LBrace)?;
    let mut slots = vec![];
    while self.peek() != &TokenKind::RBrace {
        slots.push(self.parse_flw_slot()?);
    }
    self.expect(&TokenKind::RBrace)?;
    Ok(AbstractFlwDef { visibility: vis, name, type_params, slots, span: self.span_from(&start) })
}

fn parse_flw_slot(&mut self) -> Result<FlwSlot, ParseError> {
    let start = self.peek_span().clone();
    let (name, _) = self.expect_ident()?;
    self.expect(&TokenKind::Colon)?;
    let input = self.parse_type_expr()?;
    self.expect(&TokenKind::Arrow)?;
    let output = self.parse_type_expr()?;
    let effects = self.parse_effects()?;
    Ok(FlwSlot { name, input, output, effects, span: self.span_from(&start) })
}
```

### 1-5: parser.rs — `flw` 束縛パース

既存の `parse_flw_def` の先頭で形式を識別する:

```rust
fn parse_flw_def_or_binding(&mut self, vis: Option<Visibility>) -> Result<Item, ParseError> {
    let start = self.peek_span().clone();
    self.expect(&TokenKind::Flw)?;
    let (name, _) = self.expect_ident()?;

    if self.peek() == &TokenKind::Colon {
        // 既存: flw Name: A -> B = |x| { ... }
        self.parse_flw_def_rest(vis, name, start)
            .map(Item::FlwDef)
    } else if self.peek() == &TokenKind::Eq {
        // 新規: flw Name = Template<T> { slot <- Impl; ... }
        self.expect(&TokenKind::Eq)?;
        self.parse_flw_binding_rest(vis, name, start)
            .map(Item::FlwBindingDef)
    } else {
        Err(ParseError::new("expected `:` or `=` after flw name", self.peek_span()))
    }
}

fn parse_flw_binding_rest(
    &mut self,
    vis: Option<Visibility>,
    name: String,
    start: Span,
) -> Result<FlwBindingDef, ParseError> {
    let (template, _) = self.expect_ident()?;
    let type_args = self.parse_type_args_opt()?; // `<UserRow>` or empty
    self.expect(&TokenKind::LBrace)?;
    let mut bindings = vec![];
    while self.peek() != &TokenKind::RBrace {
        let (slot_name, _) = self.expect_ident()?;
        self.expect(&TokenKind::LArrow)?; // `<-`
        let (impl_name, _) = self.expect_ident()?;
        bindings.push((slot_name, impl_name));
    }
    self.expect(&TokenKind::RBrace)?;
    Ok(FlwBindingDef { visibility: vis, name, template, type_args, bindings, span: self.span_from(&start) })
}
```

### 1-6: `parse_item` の拡張

```rust
TokenKind::Abstract => {
    self.advance(); // consume "abstract"
    match self.peek() {
        TokenKind::Trf => Ok(Item::AbstractTrfDef(self.parse_abstract_trf_def(vis)?)),
        TokenKind::Flw => Ok(Item::AbstractFlwDef(self.parse_abstract_flw_def(vis)?)),
        _ => Err(ParseError::new(
            "expected `trf` or `flw` after `abstract`",
            self.peek_span().clone(),
        )),
    }
}
TokenKind::Flw => Ok(self.parse_flw_def_or_binding(vis)?), // 既存 flw を分岐に変更
```

---

## Phase 2: 型検査統合

### 2-1: チェッカー状態の拡張（`checker.rs`）

```rust
// Checker 構造体に追加
abstract_trf_registry: HashMap<String, AbstractTrfDef>,
abstract_flw_registry: HashMap<String, AbstractFlwDef>,
```

### 2-2: 第1パス（first-pass）での登録

既存の `first_pass` に `AbstractTrfDef` / `AbstractFlwDef` の登録を追加:

```rust
Item::AbstractTrfDef(d) => {
    self.abstract_trf_registry.insert(d.name.clone(), d.clone());
    // 型環境への登録（引数型として使えるように）
    self.env.define(d.name.clone(), Type::AbstractTrf {
        input:   Box::new(self.resolve_type_expr(&d.input)),
        output:  Box::new(self.resolve_type_expr(&d.output)),
        effects: d.effects.clone(),
    });
}
Item::AbstractFlwDef(d) => {
    self.abstract_flw_registry.insert(d.name.clone(), d.clone());
    self.env.define(d.name.clone(), Type::AbstractFlwTemplate(d.name.clone()));
}
```

### 2-3: `check_flw_binding_def` の実装

```rust
fn check_flw_binding_def(&mut self, def: &FlwBindingDef) {
    // 1. テンプレート存在確認
    let template = match self.abstract_flw_registry.get(&def.template) {
        Some(t) => t.clone(),
        None => {
            self.type_error("E002", format!("undefined abstract flw `{}`", def.template), &def.span);
            return;
        }
    };

    // 2. 型引数の代入（TypeSubst で type_params -> type_args を作成）
    let subst = self.build_type_subst(&template.type_params, &def.type_args, &def.span);

    // 3. スロット名確認 + 型照合
    let mut bound_slots: HashMap<String, Vec<String>> = HashMap::new(); // slot -> effects
    for (slot_name, impl_name) in &def.bindings {
        // 3a. スロット名がテンプレートに存在するか
        let slot = match template.slots.iter().find(|s| &s.name == slot_name) {
            Some(s) => s,
            None => {
                self.type_error("E049", format!("unknown slot `{}`", slot_name), &def.span);
                continue;
            }
        };

        // 3b. 実装の型を解決
        let impl_ty = self.env.lookup(impl_name).unwrap_or(Type::Error);

        // 3c. スロット期待型を具体化（型パラメータ代入）
        let expected_input  = self.apply_subst(&slot.input, &subst);
        let expected_output = self.apply_subst(&slot.output, &subst);
        let expected_effects = &slot.effects;

        // 3d. 型照合
        if !self.slot_type_matches(&impl_ty, &expected_input, &expected_output, expected_effects) {
            self.type_error("E048", format!(
                "slot `{}` expects `({} -> {})`, got `{}`",
                slot_name,
                expected_input.display(),
                expected_output.display(),
                impl_ty.display(),
            ), &def.span);
        }

        bound_slots.insert(slot_name.clone(), slot.effects.clone());
    }

    // 4. 未束縛スロット検出
    let unbound: Vec<String> = template.slots.iter()
        .filter(|s| !bound_slots.contains_key(&s.name))
        .map(|s| s.name.clone())
        .collect();

    // 5. 結果の型を環境に登録
    if unbound.is_empty() {
        // 完全束縛: 通常の flw として登録
        let effects: Vec<String> = bound_slots.values().flatten().cloned().collect::<HashSet<_>>()
            .into_iter().collect();
        let flw_ty = Type::Flw { /* ... resolved input/output/effects */ };
        self.env.define(def.name.clone(), flw_ty);
        // テンプレート由来情報を記録（fav explain 用）
        self.flw_binding_info.insert(def.name.clone(), FlwBindingInfo {
            template: def.template.clone(),
            bindings: def.bindings.clone(),
        });
    } else {
        // 部分束縛: PartialFlw 型として登録
        let partial_ty = Type::PartialFlw {
            template: def.template.clone(),
            type_args: def.type_args.iter().map(|t| self.resolve_type_expr(t)).collect(),
            unbound_slots: unbound,
        };
        self.env.define(def.name.clone(), partial_ty);
    }
}
```

### 2-4: `abstract trf` 直接呼び出しの検査

`check_expr` の Call 処理で、呼び出し先の型が `Type::AbstractTrf` の場合 E051:

```rust
Type::AbstractTrf { .. } => {
    self.type_error("E051",
        format!("`{}` is an abstract trf and has no implementation", func_name),
        &expr.span(),
    );
    Type::Error
}
```

### 2-5: Type enum の拡張（`checker.rs` または `ast.rs`）

```rust
pub enum Type {
    // ... 既存 ...
    AbstractTrf {
        input:   Box<Type>,
        output:  Box<Type>,
        effects: Vec<String>,
    },
    AbstractFlwTemplate(String),
    PartialFlw {
        template:      String,
        type_args:     Vec<Type>,
        unbound_slots: Vec<String>,
    },
}
```

---

## Phase 3: IR + VM 実行

### 3-1: compiler.rs での IR 生成

`abstract trf` / `abstract flw` テンプレートは IR を生成しない:

```rust
Item::AbstractTrfDef(_) => { /* IR なし */ }
Item::AbstractFlwDef(_) => { /* IR なし */ }
```

完全束縛 `FlwBindingDef` は具体 `flw` と同等の IR を生成する。
各スロットを順番に直列合成した `IRFnDef` を生成:

```rust
fn compile_flw_binding(&mut self, def: &FlwBindingDef, checker_info: &FlwBindingInfo) {
    // スロット順は abstract flw のスロット宣言順に従う
    let template = &self.abstract_flw_registry[&def.template];
    let ordered_bindings: Vec<_> = template.slots.iter()
        .map(|s| {
            let impl_name = def.bindings.iter()
                .find(|(slot, _)| slot == &s.name)
                .map(|(_, impl_name)| impl_name.clone())
                .unwrap();
            impl_name
        })
        .collect();

    // IR: 入力 → slot[0] → slot[1] → ... → slot[n] → 出力
    // 各スロットが trf の場合は直接呼び出し、list 要素に map が必要な場合は List.map を使用
    let body = self.build_pipeline_body(&ordered_bindings, template);
    self.program.fn_defs.push(IRFnDef {
        name:      def.name.clone(),
        params:    vec![("input".into(), resolved_input_ty)],
        return_ty: resolved_output_ty,
        effects:   inferred_effects,
        body,
        is_public: true,
    });
}
```

### 3-2: PartialFlw の実行阻止

`driver.rs` の `cmd_run` / `cmd_build` で、`main` 関数の戻り値や使用している flw が
`PartialFlw` 型であれば E050 を出す:

```rust
// driver.rs
fn check_no_partial_flw(checker_result: &CheckerResult) -> Result<(), String> {
    for (name, ty) in &checker_result.env {
        if let Type::PartialFlw { unbound_slots, .. } = ty {
            return Err(format!(
                "E050: `{}` has unbound slots: {}",
                name,
                unbound_slots.join(", ")
            ));
        }
    }
    Ok(())
}
```

---

## Phase 4: `fav check` 部分束縛警告

### 4-1: checker.rs の警告追加

`PartialFlw` が変数に束縛されたままトップレベルに残っている場合、
`fav check` コマンド時に警告として報告する（エラーではなく warning）:

```rust
// Checker に warnings フィールドが既存なら活用
fn check_partial_flw_warnings(&self) {
    for (name, ty) in &self.env.globals() {
        if let Type::PartialFlw { template, unbound_slots, .. } = ty {
            self.warn(format!(
                "`{}` (from `{}`) has unbound slots: {}",
                name, template, unbound_slots.join(", ")
            ));
        }
    }
}
```

---

## Phase 5: `fav explain` 統合

### 5-1: driver.rs の cmd_explain 拡張

`abstract flw` テンプレートの表示:

```rust
fn format_abstract_flw(def: &AbstractFlwDef) -> String {
    let slots = def.slots.iter()
        .map(|s| format!("  {:8}: {} -> {}{}", s.name, s.input, s.output, format_effects(&s.effects)))
        .collect::<Vec<_>>()
        .join("\n");
    format!("ABSTRACT FLW {}<{}>\n{}", def.name, def.type_params.join(", "), slots)
}
```

具体束縛の表示:

```rust
fn format_flw_binding(name: &str, info: &FlwBindingInfo, template: &AbstractFlwDef) -> String {
    let slots = template.slots.iter()
        .map(|s| {
            let impl_name = info.bindings.iter()
                .find(|(slot, _)| slot == &s.name)
                .map(|(_, impl_name)| impl_name.as_str())
                .unwrap_or("(unbound)");
            format!("  {:8}: {} -> {}  <- {}", s.name, s.input, s.output, impl_name)
        })
        .collect::<Vec<_>>()
        .join("\n");
    format!("FLW {} ({})\n{}\n\n  resolved: ...\n  effects : {}", name, info.template, slots, ...)
}
```

---

## Phase 6: テスト・ドキュメント

### テスト追加場所

- `frontend/parser.rs` の `#[cfg(test)]`（パーサーテスト）
- `middle/checker.rs` の `#[cfg(test)]`（型検査テスト）
- `src/integration/abstract_flw_tests.rs`（実行テスト）

### example ファイル

```fav
// examples/abstract_flw_basic.fav

type UserRow = { name: String  age: Int }
type OrderRow = { id: Int  amount: Int }

abstract flw ETL<In, Out> {
    extract:   String     -> List<In>!
    transform: In         -> Out!
    save:      List<Out>  -> Int       !Io
}

trf ParseUsers: String -> List<UserRow>! = |csv| {
    bind rows <- String.split(csv, "\n")
    // ... パース処理 ...
    rows |> List.map(|line| { ... })
}

trf UppercaseName: UserRow -> UserRow! = |u| {
    Result.ok(UserRow { name: String.to_upper(u.name)  age: u.age })
}

trf PrintUsers: List<UserRow> -> Int !Io = |users| {
    bind _ <- IO.println(Int.show.show(List.length(users)))
    Result.ok(List.length(users))
}

flw UserPipeline = ETL<UserRow, UserRow> {
    extract   <- ParseUsers
    transform <- UppercaseName
    save      <- PrintUsers
}

public fn main() -> Unit !Io {
    bind result <- UserPipeline("Alice,30\nBob,25")
    match result {
        Ok(n)  -> IO.println_int(n)
        Err(e) -> IO.println(e)
    }
}
```

```fav
// examples/abstract_flw_inject.fav
// 関数引数による動的注入パターン

abstract flw StoragePipeline<Row> {
    validate: Row -> Row!
    save:     Row -> Int  !Io
}

trf ValidateUser: UserRow -> UserRow! = |u| {
    if u.age > 0 { Result.ok(u) } else { Result.err("invalid age") }
}

trf SaveUserStdout: UserRow -> Int !Io = |u| {
    bind _ <- IO.println(u.name)
    Result.ok(1)
}

trf SaveUserMock: UserRow -> Int !Io = |_| {
    Result.ok(0)
}

fn make_pipeline(save_impl: UserRow -> Int !Io) -> flw UserRow -> Int !Io {
    bind p <- StoragePipeline<UserRow> {
        validate <- ValidateUser
        save     <- save_impl
    }
    p
}

public fn main() -> Unit !Io {
    bind prod_pipe <- make_pipeline(SaveUserStdout)
    bind test_pipe <- make_pipeline(SaveUserMock)
    bind alice <- UserRow { name: "Alice"  age: 30 }
    bind _ <- prod_pipe(alice)
    bind _ <- test_pipe(alice);
    IO.println("done")
}
```

### langspec.md 更新

`versions/v1.3.0/langspec.md` を新規作成（v1.2.0 langspec を起点に追加）:
- `abstract trf` 構文・意味・ユースケース
- `abstract flw` テンプレート構文
- スロット束縛（完全束縛・部分束縛）のルール
- effect の合成ルール
- `PartialFlw` の制約
- E048–E051 エラーコード
- `fav explain` の abstract flw 出力形式

---

## 先送り一覧

| 制約 | バージョン |
|---|---|
| スロット間の型連続性の型検査（`parse` 出力 = `validate` 入力） | v2.0.0 セルフホスト時 |
| `abstract flw` のネスト（flw のスロットが別 abstract flw） | v1.5.0 以降 |
| `fav graph` での abstract flw ノード描画 | v1.4.0（explain JSON 後） |
| `PartialFlw` を受け取る関数の型引数 (`PartialFlw<T, { s }>`) | v2.0.0 |
| `abstract flw` の継承（他の abstract flw を拡張） | v2.0.0 以降 |
| `abstract trf` のジェネリック型パラメータ | v1.4.0 以降 |
