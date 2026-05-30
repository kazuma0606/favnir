# Favnir v8.0.0 仕様書

Date: 2026-05-28
Theme: checker.fav Let 多相（型スキーム generalization / instantiation）

---

## 概要

v8.0.0 では `checker.fav` に **Let 多相（Parametric Polymorphism）** を実装する。
v7.9.0 で構築した `unify_deep` / `infer_hm` / `InfState` を基盤として、以下を追加する。

1. **`is_type_var_extended`** — 多文字型変数（`T0`, `T1` 等）のサポート
2. **型スキーム文字列形式** — `"forall|A,B|List<A>;Fn|List<B>"` による直列化
3. **`collect_type_vars_from_te`** — TypeExpr から型変数を収集
4. **`fn_to_scheme_str`** — FnDef からスキーム文字列を構築
5. **`collect_fn_schemes`** — プログラム内全関数のスキームを事前収集（pre-pass）
6. **`instantiate_fn_scheme`** — スキームを実引数型で具体化
7. **`infer_call_hm`** — ユーザー定義ジェネリクス関数の呼び出し推論
8. **`infer_hm` 更新** — `ECall` を `infer_call_hm` で処理

v8.0.0 完了後、以下が可能になる：

```favnir
fn first_elem(xs: List<A>) -> Option<A> {
    List.first(xs)
}

// infer_hm が first_elem([1,2,3]) の戻り型を "Option<Int>" と推論できる
```

---

## 型スキーム文字列形式

checker.fav の環境 `List<KVPair>` は `String → String` の連想リストである。
TypeScheme を環境に格納するため、以下の形式で文字列に直列化する。

```
"forall|<vars>|<params>|<ret>"
```

| セクション | 区切り | 内容 |
|------------|--------|------|
| `"forall"` | 固定キーワード | スキームであることの識別子 |
| `<vars>` | `,` で結合 | 全称量化された型変数名（例: `"A,B"`） |
| `<params>` | `;` で結合 | 各パラメータの型文字列（例: `"List<A>;Fn"`） |
| `<ret>` | — | 戻り型文字列（例: `"Option<A>"`） |

**注意**: `;` をパラメータ区切りに使う理由は、型文字列が `Map<K,V>` のようにカンマを含む場合があるため。

### 例

| 関数シグネチャ | スキーム文字列 |
|---------------|--------------|
| `fn id(x: A) -> A` | `"forall\|A\|A\|A"` |
| `fn first(xs: List<A>) -> Option<A>` | `"forall\|A\|List<A>\|Option<A>"` |
| `fn map(xs: List<A>, f: Fn) -> List<B>` | `"forall\|A,B\|List<A>;Fn\|List<B>"` |
| `fn length(xs: List<Int>) -> Int` | `"Int"`（型変数なし → monotype、そのまま格納） |

---

## 新規関数

### `is_type_var_extended(s: String) -> Bool`

v7.x の `is_type_var`（1 文字大文字のみ）を拡張し、2 文字の型変数（`T0`〜`Z9`）も受け入れる。

```
is_type_var_extended("A")  → true
is_type_var_extended("T0") → true
is_type_var_extended("T1") → true
is_type_var_extended("Int") → false
is_type_var_extended("AB") → false  // 2文字だが2文字目が大文字
```

判定条件:
- 1 文字で `A`〜`Z` なら true（既存 `is_type_var` と同等）
- 2 文字で 1 文字目が `A`〜`Z`、2 文字目が `0`〜`9` なら true
- それ以外は false

### `collect_type_vars_from_te(te: TypeExpr) -> List<String>`

TypeExpr を再帰的に走査し、型変数名のリストを返す。重複を含む可能性あり。

```
collect_type_vars_from_te(TeSimple("A"))          → ["A"]
collect_type_vars_from_te(TeList(TeSimple("A")))  → ["A"]
collect_type_vars_from_te(TeSimple("Int"))         → []
collect_type_vars_from_te(TeResult(TeSimple("A"), TeSimple("B"))) → ["A", "B"]
```

### `list_dedup(lst: List<String>) -> List<String>`

リストから重複を除去し、最初の出現順を維持する。

### `collect_type_vars_from_params(params: List<Param>) -> List<String>`

`List<Param>` の各 `p.ty` から型変数を収集し、重複除去した一覧を返す。

### スキーム文字列ヘルパー群

```favnir
fn is_fn_scheme_str(s: String) -> Bool       // String.starts_with(s, "forall|")
fn fn_scheme_vars_str(s: String) -> String   // 2番目の "|" セクション ("A,B")
fn fn_scheme_params_str(s: String) -> String // 3番目の "|" セクション ("List<A>;Fn")
fn fn_scheme_ret(s: String) -> String        // 4番目の "|" セクション ("Option<A>")
fn make_fn_scheme_str(vars_csv: String, params_semi: String, ret: String) -> String
```

`make_fn_scheme_str` は vars_csv が空文字なら `ret` をそのまま返す（monotype）。

### `fn_to_scheme_str(fd: FnDef) -> String`

FnDef からスキーム文字列を構築する。

1. `collect_type_vars_from_params(fd.params)` + `collect_type_vars_from_te(fd.ret)` → vars
2. vars が空 → `type_expr_to_str(fd.ret)` をそのまま返す（monotype）
3. vars が非空 → 各 param の型文字列を `;` で結合 + `make_fn_scheme_str`

### `collect_fn_schemes(items: List<Item>, env: List<KVPair>) -> List<KVPair>`

プログラムの全 `IFn` アイテムをスキャンし、各関数のスキーム文字列を env に追加して返す（pre-pass）。

```favnir
fn collect_fn_schemes(items: List<Item>, env: List<KVPair>) -> List<KVPair> {
    match List.first(items) {
        None => env
        Some(item) => match item {
            IFn(fd) => collect_fn_schemes(List.drop(items, 1),
                           env_insert(env, fd.name, fn_to_scheme_str(fd)))
            _ => collect_fn_schemes(List.drop(items, 1), env)
        }
    }
}
```

`check` 関数を更新してこの pre-pass を呼ぶ。

### `build_scheme_subst_inner(param_tys: List<String>, arg_tys: List<String>, subst: List<KVPair>) -> Result<List<KVPair>, String>`

パラメータ型リストと引数型リストをペアにして `unify_deep` を繰り返し適用し、置換環境を構築する。

```
param_tys = ["List<A>"]
arg_tys   = ["List<Int>"]
→ unify_deep("List<A>", "List<Int>", []) → A→Int
→ 最終 subst = [A→Int]
```

### `apply_scheme_subst(ret_ty: String, vars: List<String>, subst: List<KVPair>) -> String`

`vars` の各型変数について `subst_lookup` で実際の型を調べ、`String.replace` で `ret_ty` を書き換える。

```
ret_ty = "Option<A>", vars = ["A"], subst = [A→Int]
→ String.replace("Option<A>", "A", "Int") → "Option<Int>"
```

### `instantiate_fn_scheme(scheme: String, arg_tys: List<String>, state: InfState) -> Result<InfResult, String>`

スキーム文字列 + 実引数型リスト + 現在の InfState から、戻り型を決定する。

1. vars_str、params_str、ret_ty を抽出
2. `String.split(params_str, ";")` でパラメータ型リストを復元
3. `build_scheme_subst_inner(param_tys, arg_tys, state.subst)` で置換を構築
4. `apply_scheme_subst(ret_ty, vars, subst2)` で戻り型を具体化
5. `inf_result_of(instantiated_ret, updated_state)` を返す

### `infer_call_user(fname: String, args: Expr, env: List<KVPair>, state: InfState) -> Result<InfResult, String>`

名前空間なし（ユーザー定義関数）の呼び出し推論。

1. `env_lookup(env, fname)` でスキーム文字列を取得
2. スキームなら `instantiate_fn_scheme`
3. monotype なら `inf_result_of(ty, state)`
4. 未登録なら `inf_result_of("Unknown", state)`

### `infer_call_hm(ns: String, fname: String, args: Expr, env: List<KVPair>, state: InfState) -> Result<InfResult, String>`

`infer_call` の HM 版。`ns == ""` のとき `infer_call_user` を使い、それ以外は既存 `infer_call` にフォールバック。

### `infer_hm` 更新

`ECall` の `_` フォールバックから専用ケースに変更。

```favnir
ECall(ns, fname, args) => infer_call_hm(ns, fname, args, env, state)
```

### `check` 関数更新

```favnir
public fn check(prog: Program) -> Result<String, String> {
    bind init_env    <- env_empty()
    bind scheme_env  <- collect_fn_schemes(prog.items, init_env)
    check_items(prog.items, scheme_env)
}
```

---

## 既知の制約（v8.0.0 スコープ外）

- **fav check パイプライン差し替え**: `fav check foo.fav` はまだ Rust checker.rs を使用。checker.fav を実際の `fav check` に組み込むのは v8.1.0
- **相互再帰関数間のスキーム共有**: `collect_fn_schemes` は先頭から順に処理するため、前方参照にあたる関数は Unknown になる可能性あり（実用上はほぼ問題なし）
- **ネスト 2 段以上の型変数置換**: `"Map<K, List<V>>"` の `V` 置換は `String.replace` で動作するが、より複雑なネストは v8.1.0 で対応
- **型推論によるパラメータ型の自動補完**: パラメータ型が明示されていない場合（ラムダ等）の推論は v8.1.0

---

## テスト計画

### Favnir 内テスト（10 件）

| ID | テスト名 | 検証内容 |
|----|----------|----------|
| E-1 | `is_type_var_ext single` | `is_type_var_extended("A") == true` |
| E-2 | `is_type_var_ext two-char` | `is_type_var_extended("T0") == true` |
| E-3 | `is_type_var_ext not` | `is_type_var_extended("Int") == false && is_type_var_extended("AB") == false` |
| E-4 | `collect_te vars simple` | `collect_type_vars_from_te(TeSimple("A")) == ["A"]` |
| E-5 | `collect_te vars none` | `collect_type_vars_from_te(TeSimple("Int")) == []` |
| E-6 | `is_fn_scheme_str true` | `is_fn_scheme_str("forall\|A\|List<A>\|Option<A>") == true` |
| E-7 | `is_fn_scheme_str false` | `is_fn_scheme_str("Option<Int>") == false` |
| E-8 | `fn_scheme_ret` | `fn_scheme_ret("forall\|A\|List<A>\|Option<A>") == "Option<A>"` |
| E-9 | `instantiate simple` | `instantiate_fn_scheme("forall\|A\|A\|A", ["Int"], state0)` → `"Int"` |
| E-10 | `instantiate nested` | `instantiate_fn_scheme("forall\|A\|List<A>\|Option<A>", ["List<String>"], state0)` → `"Option<String>"` |

### driver.rs 統合テスト（3 件）

| ID | テスト名 | 検証内容 |
|----|----------|----------|
| F-1 | `checker_fav_scheme_str` | `is_fn_scheme_str("forall\|A\|List<A>\|Option<A>")` → `true` |
| F-2 | `checker_fav_instantiate_scheme` | `instantiate_fn_scheme("forall\|A\|List<A>\|Option<A>", ["List<Int>"], state0)` → `"Option<Int>"` |
| F-3 | `checker_fav_infer_hm_generic_call` | user-defined `fn first_elem(xs: List<A>) -> Option<A>` を infer_hm 経由で呼び出すと `"Option<Int>"` |

---

## 完了条件

- `fav check fav/self/checker.fav` — no errors
- `is_type_var_extended("T0") == true`
- `instantiate_fn_scheme("forall|A|List<A>|Option<A>", ["List<Int>"], state)` → `"Option<Int>"`
- ユーザー定義ジェネリクス関数への `ECall` を `infer_hm` が正しく推論できる
- 統合テスト 13 件追加（checker.fav 内 10 + driver.rs 3）
- 既存テスト 1100 件が全件通る（1113+ passing）
