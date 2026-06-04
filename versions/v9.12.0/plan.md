# Favnir v9.12.0 Plan

Date: 2026-06-02
Theme: ユーザー定義インターフェース self-hosted 対応 + LSP 定義ジャンプ改善

---

## Phase A: compiler.fav — lexer/parser/codegen 拡張

### A-1: Token 型に TkInterface / TkImpl を追加

`compiler.fav` の `Token` sum type に 2 variant を追加:
```favnir
type Token =
  | ...既存...
  | TkInterface
  | TkImpl
```

### A-2: keyword_token 関数に分岐を追加

`keyword_token(s: String) -> Option<Token>` の末尾 `else` の前に:
```favnir
else if s == "interface" { Option.some(TkInterface) }
else if s == "impl"      { Option.some(TkImpl) }
```

### A-3: Item 型に IInterface / IImpl を追加

型定義を新規追加してから Item に variant 追加:
```favnir
type InterfaceMethodDecl = {
    name: String
    ty:   String
}

type InterfaceDef = {
    name:    String
    methods: List<InterfaceMethodDecl>
}

type ImplDeclDef = {
    interface_names: List<String>
    type_name:       String
    methods:         List<FnDef>
}

type Item =
  | IFn(FnDef)
  | IType(TypeDef)
  | ITest(TestDef)
  | IWrapper(WrapperDef)
  | IInterface(InterfaceDef)   // 追加
  | IImpl(ImplDeclDef)         // 追加
```

### A-4: parse_interface_item を追加

```
interface <Name> {
    <method>: <type_sig>
    ...
}
```
をパースして `InterfaceDef` を返す。

実装方針:
- `TkInterface` を consume → 識別子（Name）を consume → `TkLBrace`
- ループ: `TkRBrace` まで `name: type_sig` 行を読む（`TkColon` で区切り）
- `List.map` で `InterfaceMethodDecl` のリストを構築

### A-5: parse_impl_item を追加

```
impl <Iface> for <Type> {
    fn <method>(...) -> <ret> { ... }
    ...
}
```
をパースして `ImplDeclDef` を返す。

実装方針:
- `TkImpl` を consume → インターフェース名 → `for` キーワード → 型名 → `TkLBrace`
- ループ: `TkRBrace` まで通常の `parse_fn_def` を呼ぶ
- `interface_names: List.of1(iface_name)`、`type_name`、`methods: fn_defs`

注: `for` はキーワードではなく識別子として扱う（既存 parser との互換性）。

### A-6: parse_top_item に TkInterface / TkImpl 分岐を追加

```favnir
else if peek() == TkInterface { IInterface(parse_interface_item()) }
else if peek() == TkImpl      { IImpl(parse_impl_item()) }
```

### A-7: compile_item に IInterface / IImpl 分岐を追加

```favnir
| IInterface(_) -> ()   // コード生成なし
| IImpl(id)     -> compile_impl_decl(id)
```

`compile_impl_decl(id: ImplDeclDef)`:
- `id.methods` の各 `FnDef` を `compile_fn_def` で処理
- 関数名は `<TypeName>_<method>` として出力（既存 Rust コンパイラの命名規則に合わせる）

---

## Phase B: checker.fav — interface/impl 型チェック

### B-1: Item 型拡張（compiler.fav と同内容）

checker.fav は独自 lexer/parser を内蔵するため A-1〜A-6 と同じ変更が必要:
- `Token` 型に `TkInterface` / `TkImpl` 追加
- `keyword_token` に分岐追加
- `InterfaceDef` / `ImplDeclDef` 型定義追加
- `Item` に `IInterface` / `IImpl` 追加
- parse_interface_item / parse_impl_item / parse_top_item 追加

### B-2: collect_interface_schemes を追加

```favnir
fn collect_interface_schemes(prog: List<Item>, env: Env) -> Env {
    List.fold(prog, env, |e, item|
        match item {
            | IInterface(id) -> env_insert(e, id.name, "interface")
            | _              -> e
        }
    )
}
```

### B-3: collect_impl_decls を追加

```favnir
type ImplEntry = { iface: String, type_: String }

fn collect_impl_decls(prog: List<Item>) -> List<ImplEntry> {
    List.flat_map(prog, |item|
        match item {
            | IImpl(id) ->
                List.map(id.interface_names, |iface|
                    ImplEntry { iface: iface, type_: id.type_name }
                )
            | _ -> List.empty()
        }
    )
}
```

### B-4: check_type_with_impl を追加

`TypeDef.with_impls` の各インターフェース名を検証:
- 組み込み（`Eq` / `Show` / `Serialize` / `Deserialize`）→ スキップ
- `iface_env` に存在しない → **E0015 — UndefinedInterface**
- `impl_list` に `(iface, type_name)` なし → **E0014 — MissingImpl**

### B-5: check_impl_decl を追加

```favnir
fn check_impl_decl(id: ImplDeclDef, env: Env) -> List<FavError> {
    List.flat_map(id.methods, |fn_def| check_fn_def(fn_def, env))
}
```

### B-6: check(prog) の更新

```
collect_fn_schemes
  → collect_variant_constructors
  → collect_interface_schemes   // 追加
  → collect_impl_decls         // 追加
  → check_items (IInterface/IImpl 分岐追加)
```

`check_items` に分岐:
```favnir
| IInterface(_) -> List.empty()   // 型定義のみ、チェック不要
| IImpl(id)     -> check_impl_decl(id, env)
```

**新エラーコード**:
- `E0014` — `MissingImpl: Validatable is not implemented for Order`
  - hint: `add \`impl Validatable for Order { ... }\` block`
- `E0015` — `UndefinedInterface: undefined interface Validatable`

---

## Phase C: LSP 定義ジャンプ改善

### C-1: LspServer に workspace_root を追加

`fav/src/lsp/mod.rs` の `LspServer` 構造体:
```rust
pub struct LspServer {
    store: DocumentStore,
    workspace_root: Option<String>,
}
```

`initialize` ハンドラで `rootUri` を取り出して保存。

### C-2: definition.rs に handle_rune_definition を追加

`fav/src/lsp/definition.rs` に新関数:
```rust
pub fn handle_rune_definition(
    src: &str,
    offset: usize,
    workspace_root: &str,
) -> Option<serde_json::Value>
```

アルゴリズム:
1. `offset` 前後のテキストから `<ns>.<fn>` パターンを検出（正規表現不使用、文字スキャン）
2. `ns` が `KNOWN_RUNES`（completion.rs）に含まれるか確認
3. `<workspace_root>/rune_modules/<ns>/<ns>.fav` を構築
4. ファイルを読み込み `fn <fn>` / `public fn <fn>` の行番号を探す
5. `{ uri, range: { start: { line, character: 0 }, end: { line, character: 0 } } }` を返す

### C-3: textDocument/definition ハンドラを拡張

既存の `handle_definition` が `None` を返した場合に `handle_rune_definition` を試みる:
```rust
handle_definition(&self.store, &uri, pos)
    .or_else(|| {
        self.workspace_root.as_deref()
            .and_then(|root| handle_rune_definition(&doc.source, pos, root))
    })
```

### C-4: seq 内 stage 名ジャンプ（調査 + 対応）

`checker.rs` の `check_seq_def` を調査:
- `def_at` に stage 名参照 span が記録されているか確認
- 記録済み → 追加作業不要（テストのみ）
- 未記録 → stage 名 usage span → def span のマッピングを追加

---

## Phase D: テスト + バージョン更新 + commit

### D-1: v9120 テストモジュールを追加（5 件以上）

`fav/src/lsp/mod.rs` に `v9120_tests` モジュール:
- `interface_keyword_in_compiler_fav_does_not_error` — interface キーワードが parse エラーにならない
- `impl_keyword_in_compiler_fav_does_not_error` — impl キーワードが parse エラーにならない
- `missing_impl_e0014_detected` — with UserDefinedIface + impl 欠落 → E0014
- `undefined_interface_e0015_detected` — 未定義 iface → E0015
- `rune_definition_jump_http_get` — http.get の定義ジャンプ → rune_modules/http/http.fav
- `rune_definition_returns_none_for_unknown` — 未知 ns → None

### D-2: cargo test v9120 — 5 件以上通過

### D-3: cargo test checker_fav_wire_self_check — 通過

### D-4: cargo test bootstrap — 通過

### D-5: cargo test — 全件通過

### D-6: fav/Cargo.toml version → "9.12.0"

### D-7: fav/self/cli.fav の run_version → "9.12.0"

### D-8: memory/MEMORY.md に v9.12.0 完了を記録

### D-9: commit

---

## 実装順序

```
A-1 → A-2 → A-3 → A-4 → A-5 → A-6 → A-7   (compiler.fav)
B-1 → B-2 → B-3 → B-4 → B-5 → B-6           (checker.fav)
C-1 → C-2 → C-3 → C-4                        (LSP)
D-1 → D-2 → D-3 → D-4 → D-5 → D-6 → D-7 → D-8 → D-9
```

A と B は compiler.fav / checker.fav をそれぞれ独立して編集できる。
C は Rust コードのみ（.fav ファイルに影響しない）。
