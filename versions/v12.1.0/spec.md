# Favnir v12.1.0 仕様書

作成日: 2026-06-07
テーマ: `bind` イミュータビリティ強制（E0018）

---

## 背景と目的

Favnir の `bind` は「変数束縛」を意図したキーワードだが、
現状は同一スコープで同じ変数名に何度でも `bind` できてしまう。

```favnir
// 現状：エラーにならない（問題）
stage Broken: Int -> Int !IO = |n| {
  bind x <- IO.println("a")
  bind x <- IO.println("b")  // 再束縛 — 今は通る
  x
}
```

関数型言語の「束縛（binding）」は本来「一度決まったら変わらない」を意味する。
再束縛を許すと：
- コードの追跡が困難になる（`x` がどの値を指すか不明）
- AI（Claude Code / Codex）が誤って同名変数を複数回束縛しても検出できない
- `bind` と `let` の区別が曖昧になる

v12.1.0 では `checker.fav` に「束縛済みセット」の管理を追加し、
同一スコープでの再 `bind` を **E0018** としてコンパイルエラーにする。

---

## エラー仕様：E0018

### トリガー条件

以下のいずれかが同一スコープ内で発生した場合:

1. `bind x <- ...` が同名で 2 回以上現れる
2. `chain x <- ...` が同名で 2 回以上現れる
3. `bind x` の後に `chain x`（または逆順）— キーワードが異なっても再束縛は禁止
4. fn/stage のパラメータ名と同名の `bind x <- ...` — パラメータも束縛の一種

### 例外

- `_`（アンダースコア）は何度でも `bind _ <- ...` できる（捨て変数の慣例）

### エラーメッセージ形式

```
E0018: variable 'x' is already bound in this scope
  --> pipeline.fav:12:3
   |
 8 | bind x <- compute1()
   |      - first bound here
...
12 | bind x <- compute2()
   |      ^ cannot rebind 'x'
   |
   = help: use a different name: `bind x2 <- compute2()`
   = help: or discard the value: `bind _ <- compute2()`
```

### help: メッセージ（AI 自己修正対応）

| 代替案 | help: 文 |
|---|---|
| 別名を使う | `help: use a different name: \`bind x2 <- compute2()\`` |
| 戻り値を捨てる | `help: or discard the value: \`bind _ <- compute2()\`` |

---

## スコープ規則

### スコープの単位

| 構文 | スコープ |
|---|---|
| `fn` / `stage` 本体 | 本体全体が 1 スコープ |
| `match` の各 arm | arm ごとに独立したスコープ |
| `|lambda|` 本体 | lambda 本体が独立したスコープ |

### 正常系（OK）

```favnir
// OK: match arm は別スコープ
fn process(result: Result<Int, String>) -> String = {
  match result {
    Ok(x)  => String.from_int(x)   // x はこの arm のみ
    Err(x) => x                     // x はこの arm のみ（別スコープ）
  }
}

// OK: _ は何度でも可
stage LoadMany: String -> Int !IO = |path| {
  bind _ <- IO.println("loading...")
  bind _ <- IO.println("done")
  0
}

// OK: lambda 内の bind は外部スコープと独立
fn run(items: List<String>) -> List<String> =
  List.map(items, |s| {
    bind x <- String.to_upper(s)  // 外部に x があっても別スコープ
    x
  })

// OK: 別名を使う
stage Good: Int -> Int = |n| {
  bind x  <- some_fn(n)
  bind x2 <- other_fn(n)
  x2
}
```

### 異常系（E0018）

```favnir
// NG-1: 同一スコープで bind x を 2 回
stage Bad1: Int -> Int = |n| {
  bind x <- some_fn(n)
  bind x <- other_fn(n)  // E0018: 'x' is already bound
  x
}

// NG-2: bind → chain のクロスキーワード再束縛
stage Bad2: Int -> Int !IO = |n| {
  bind x  <- some_fn(n)
  chain x <- IO.println("hello")  // E0018: 'x' is already bound
  x
}

// NG-3: chain → bind の逆順クロスキーワード
stage Bad3: Int -> Int !IO = |n| {
  chain x <- IO.println("hello")
  bind x  <- some_fn(n)  // E0018: 'x' is already bound
  x
}

// NG-4: fn パラメータ名と同名の bind
fn compute(x: Int) -> Int = {
  bind x <- double(x)  // E0018: 'x' is already bound (parameter)
  x
}

// NG-5: stage 引数名と同名の bind
stage Process: Int -> Int = |input| {
  bind input <- transform(input)  // E0018: 'input' is already bound (stage parameter)
  input
}

// NG-6: 3 回以上の再束縛（2 回目で E0018）
stage Bad6: Int -> Int = |n| {
  bind x <- fn1(n)
  bind x <- fn2(n)  // E0018
  bind x <- fn3(n)  // E0018（継続してチェック）
  x
}
```

---

## 実装方針

### checker.fav への変更

`infer_hm_let` 関数（`EBind` 処理）の呼び出し時に、
現在のスコープの「束縛済みセット」を引き回す。

#### 新規ヘルパー関数

```favnir
// 束縛済みセットに変数名が含まれるか確認
fn bound_set_contains(set: List<String>, name: String) -> Bool

// 束縛済みセットに変数名を追加
fn bound_set_add(set: List<String>, name: String) -> List<String>
```

#### E0018 発行ロジック（擬似コード）

```
check_fn_body(expr, env, bound_set):
  case EBind(vname, val_e, cont_e):
    if vname != "_" && bound_set_contains(bound_set, vname):
      return Err(fmt_e0018(vname))
    new_set = bound_set_add(bound_set, vname)
    infer and continue with new_set
```

### Rust checker.rs への変更

`--legacy` モードの Rust チェッカーにも E0018 を追加する（低優先度 — 主要実装は checker.fav）。

---

## テスト仕様

### Rust テスト（`driver.rs` に `v12100_tests` モジュール）

#### 正常系（エラーにならないこと）

| テスト名 | 内容 |
|---|---|
| `e0018_underscore_allowed` | `bind _ <- ...` の連続 → エラーなし |
| `e0018_match_arm_independent` | match arm 内の同名変数（別スコープ）→ エラーなし |
| `e0018_lambda_scope_independent` | lambda 内の `bind x` は外部の `x` と独立 → エラーなし |

#### 異常系（E0018 が出ること）

| テスト名 | 内容 |
|---|---|
| `e0018_bind_rebind_detected` | `bind x` → `bind x` → E0018 |
| `e0018_chain_rebind_detected` | `chain x` → `chain x` → E0018 |
| `e0018_bind_then_chain_cross` | `bind x` → `chain x` → E0018（クロスキーワード） |
| `e0018_chain_then_bind_cross` | `chain x` → `bind x` → E0018（逆順クロス） |
| `e0018_param_shadowing_fn` | fn パラメータ名と同名の `bind` → E0018 |
| `e0018_param_shadowing_stage` | stage `\|param\|` と同名の `bind` → E0018 |
| `e0018_triple_rebind` | 3 回の再束縛でも E0018 が出ること |

#### バージョン確認

| テスト名 | 内容 |
|---|---|
| `version_is_12_1_0` | `CARGO_PKG_VERSION == "12.1.0"` |

---

## 完了条件

| 確認項目 | 状態 |
|---|---|
| `checker.fav` に E0018 チェック追加 | |
| E0018 エラーメッセージに `help:` 付き | |
| `_` の再束縛は許可（例外処理） | |
| `chain x` の再束縛も E0018 検出 | |
| `cargo test v12100` 11 件通過（正常系 3 + 異常系 7 + バージョン 1） | |
| `cargo test --lib` 1290 件以上通過 | |
