# Favnir v9.12.0 Spec

Date: 2026-06-02
Theme: ユーザー定義インターフェース self-hosted 対応 + LSP 定義ジャンプ改善

---

## 概要

2 つの独立したテーマを同バージョンで完成させる。

1. **interface/impl self-hosted 対応**
   `interface` / `impl ... for` 構文を compiler.fav・checker.fav（self-hosted pipeline）で完全に処理できるようにする。
   現状これらのキーワードは self-hosted lexer に登録されておらず、`fav run`（Favnir pipeline）で parse error になる。

2. **LSP 定義ジャンプ改善**（v9.11.0 延期分）
   Rune 関数へのジャンプ（`http.get(` の `get` → `runes/http/http.fav`）と、
   `seq` 内 stage 名クリックでの定義ジャンプを追加する。

---

## 現状確認

### interface/impl

| 層 | 状態 |
|---|---|
| Rust パーサー（`ast.rs`） | `InterfaceDecl` / `InterfaceImplDecl` 完全実装済み |
| Rust 型チェッカー（`checker.rs`） | interface 定義・impl チェック実装済み |
| Rust コンパイラ（`compiler.rs`） | impl ブロック内 fn のコード生成実装済み |
| **compiler.fav（self-hosted lexer）** | `interface` / `impl` をキーワードとして未認識 → **parse error** |
| **checker.fav（self-hosted checker）** | `Item` に `IInterface` / `IImpl` variant なし → **素通り** |

### LSP 定義ジャンプ

| 機能 | 状態 |
|---|---|
| ユーザー定義 fn / stage / type | `doc.def_at` 経由で動作済み |
| Rune 関数ジャンプ | 未実装（`def_at` に Rune 関数は含まれない） |
| `seq` 内 stage 名ジャンプ | `def_at` の対象になっているか要調査 |

---

## テーマ A: interface/impl self-hosted 対応

### A-1 compiler.fav — lexer/parser 拡張

**lexer 追加（keyword_token 関数）**:
```favnir
else if s == "interface" { Option.some(TkInterface) }
else if s == "impl"      { Option.some(TkImpl) }
```
- `TkInterface` / `TkImpl` トークン型を追加
- `Token` 型の variant として追加

**parser 追加**:
- `IInterface` — `interface <Name> { <method>: <type>  ... }` をパース
- `IImpl` — `impl <Iface> for <Type> { fn <method>(...) -> <ret> { ... }  ... }` をパース
  - impl 内の fn は通常の FnDef と同形式
- `parse_top_item` に `TkInterface` / `TkImpl` の分岐を追加

**Item 型拡張**:
```favnir
type InterfaceMethodDecl = {
    name: String
    ty: String   // 型シグネチャ文字列（型チェックには使わない、コード生成に使用）
}

type InterfaceDef = {
    name: String
    methods: List<InterfaceMethodDecl>
}

type ImplMethodDef = {
    fn_def: FnDef
}

type ImplDeclDef = {
    interface_names: List<String>
    type_name: String
    methods: List<FnDef>
}

type Item =
  | IFn(FnDef)
  | IType(TypeDef)
  | ITest(TestDef)
  | IWrapper(WrapperDef)
  | IInterface(InterfaceDef)   // 追加
  | IImpl(ImplDeclDef)         // 追加
```

**コード生成**:
- `IInterface` → コード生成なし（型定義のみ、バイトコードに影響しない）
- `IImpl` → impl ブロック内の各 FnDef を通常の `fn <TypeName>_<method>(...)` として出力
  - 既存の `compile_fn_def` をそのまま流用

### A-2 checker.fav — interface/impl チェック

**Item 型**: compiler.fav と同様に `IInterface` / `IImpl` variant を追加
（checker.fav は独自の lexer/parser を内蔵しているため、同じ変更が必要）

**型チェックロジック追加**:

`collect_interface_schemes(prog, env)`:
- `IInterface` を走査し、インターフェース名をスコープに登録
- `env_insert(env, "<IfaceName>", "interface")` で存在マーカーを登録

`collect_impl_decls(prog) -> List<{ iface: String, type_: String }>`:
- `IImpl` を走査し `(interface_name, type_name)` ペアのリストを返す

`check_type_with_impl(td, impl_list, iface_env)`:
- `td.with_impls` を走査
- 組み込み（`Eq` / `Show` / `Serialize` / `Deserialize`）はスキップ
- ユーザー定義インターフェース名の場合:
  - `iface_env` に存在しない → **E0015 — UndefinedInterface**
  - `impl_list` に `(iface, type_name)` ペアがない → **E0014 — MissingImpl**

`check_impl_decl(id, env)`:
- impl ブロック内の各 FnDef を `check_fn_def` で型チェック

**新エラーコード**:
- **E0014 — MissingImpl**: `with` で指定したユーザー定義インターフェースの `impl` ブロックがない
  ```
  E0014: Validatable is not implemented for Order
  hint: add `impl Validatable for Order { ... }` block
  ```
- **E0015 — UndefinedInterface**: `with` または `impl` で参照したインターフェース名が未定義
  ```
  E0015: undefined interface Validatable
  ```

---

## テーマ B: LSP 定義ジャンプ改善

### B-1 Rune 関数ジャンプ

**前提**: LSP サーバーがワークスペースルートを知る必要がある。

`LspServer` 構造体に `workspace_root: Option<String>` を追加。
`initialize` ハンドラで `rootUri` を取り出して保存:
```rust
self.workspace_root = params.get("rootUri").and_then(|v| v.as_str()).map(|s| s.to_string());
```

`definition.rs` に `handle_rune_definition(src, offset, workspace_root)` を追加:
1. カーソル位置の前後テキストを解析して `<ns>.<fn>` パターンを検出
2. `ns` が `KNOWN_RUNES`（completion.rs の定数）に含まれるか確認
3. `<workspace_root>/rune_modules/<ns>/<ns>.fav` のファイルパスを構築
4. ファイルを読み込み、`fn <fn>` / `public fn <fn>` の行を探す
5. Location を返す

`mod.rs` の `textDocument/definition` ハンドラ拡張:
```rust
let result = extract_hover_target(...)
    .and_then(|(uri, pos)| {
        handle_definition(&self.store, &uri, pos)
            .or_else(|| {
                self.workspace_root.as_deref()
                    .and_then(|root| handle_rune_definition(&doc.source, pos, root))
            })
    })
```

### B-2 seq 内 stage 名ジャンプ（調査 + 対応）

既存の `def_at` ロジック（`checker.rs` が `seq` の `|>` チェーン内の stage 名参照を記録しているか）を確認。
- 記録済みなら追加作業不要
- 未記録なら `checker.rs` の `check_seq_def` で stage 名の usage span → def span を追加

---

## 完了条件

| 条件 | 確認 |
|---|---|
| `interface` / `impl ... for` を含む `.fav` が `fav run` で動作する | |
| `with UserDefinedIface` の impl 漏れを E0014 で検出できる | |
| 未定義インターフェース参照を E0015 で検出できる | |
| `cargo test checker_fav_wire_self_check` 通過 | |
| `cargo test bootstrap` 維持 | |
| Rune 関数（`http.get` 等）の定義ジャンプが `rune_modules/<ns>/<ns>.fav` へ飛ぶ | |
| `seq` 内の stage 名ジャンプが動作する | |
| `cargo test v9120` — 5 件以上通過 | |

---

## スコープ外（将来版へ延期）

- `interface` の型制約付きジェネリクス（`fn f<T with Show>(v: T)` の型チェック）
- `impl` ブロック内メソッドの完全な型シグネチャ一致チェック（メソッド名の存在確認のみ）
- Rune ジャンプのキャッシュ（毎回ファイル読み込み）
- LSP のインクリメンタル解析
