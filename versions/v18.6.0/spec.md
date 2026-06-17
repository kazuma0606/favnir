# v18.6.0 Spec — 共変・反変アノテーション（Variance）

Date: 2026-06-16

## 概要

ジェネリクスの型安全性を完成させる。`List<Cat>` が `List<Animal>` として扱えるか、
という問題を型レベルで正しく解決する。

`interface` の型パラメータに `+T`（共変）/ `-T`（反変）アノテーションを付与することで、
サブタイピング関係を制御できるようにする。

---

## 設計

### 構文

```favnir
// 共変 (+T): 出力のみ（Producer パターン）
// Source<Cat> は Source<Animal> として渡せる
interface Source<+T> {
  fn next() -> Option<T>
}

// 反変 (-T): 入力のみ（Consumer パターン）
// Sink<Animal> は Sink<Cat> として渡せる
interface Sink<-T> {
  fn write(val: T) -> Result<Unit, String>
}

// 不変（デフォルト、アノテーションなし）: 入出力両方
// List<Cat> は List<Animal> として渡せない
interface Transform<T> {
  fn apply(val: T) -> T
}
```

### 分散ルール（リスコフ置換原則）

| アノテーション | 名称 | サブタイピング | 使用可能な位置 |
|---|---|---|---|
| `+T` | 共変 (Covariant) | `I<Sub> <: I<Super>` | 出力（戻り値）のみ |
| `-T` | 反変 (Contravariant) | `I<Super> <: I<Sub>` | 入力（引数）のみ |
| `T` | 不変（デフォルト） | `I<A>` と `I<B>` は別型 | 入出力両方 |

### 使用例

```favnir
// 共変の活用
fn read_animals(source: Source<Animal>) -> List<Animal> { ... }

bind cat_source: Source<Cat> = ...
read_animals(cat_source)  // OK: Source<Cat> <: Source<Animal>（共変）

// 反変の活用
fn write_cats(sink: Sink<Cat>) -> Result<Unit, String> { ... }

bind animal_sink: Sink<Animal> = ...
write_cats(animal_sink)  // OK: Sink<Animal> <: Sink<Cat>（反変）

// 不変
interface Transform<T> {
  fn apply(val: T) -> T
}
bind cat_transform: Transform<Cat> = ...
// Transform<Animal> に渡すと型エラー（不変）
```

### エラー E0334 — 分散違反

共変型パラメータ（`+T`）が入力位置（引数）に現れた場合、
または反変型パラメータ（`-T`）が出力位置（戻り値）に現れた場合にエラーを出す。

```favnir
// E0334: +T なのに引数位置で使っている
interface BadSource<+T> {
  fn write(val: T) -> Result<Unit, String>  // E0334: covariant type +T used in input position
}

// E0334: -T なのに戻り値位置で使っている
interface BadSink<-T> {
  fn next() -> Option<T>  // E0334: contravariant type -T used in output position
}
```

---

## 実装方針

### AST 変更

**`GenericParam` に `variance` フィールドを追加する:**

```rust
// ast.rs
pub enum Variance {
    Covariant,     // +T
    Contravariant, // -T
    Invariant,     // T（デフォルト）
}

pub struct GenericParam {
    pub name: String,
    pub bounds: Vec<TypeConstraint>,
    pub variance: Variance,   // v18.6.0 追加（デフォルト Invariant）
}
```

**`InterfaceDecl` に `type_params` フィールドを追加する:**

```rust
// ast.rs — 現状
pub struct InterfaceDecl {
    pub visibility: Option<Visibility>,
    pub name: String,
    pub super_interface: Option<String>,
    pub methods: Vec<InterfaceMethod>,
    pub span: Span,
}

// v18.6.0 追加後
pub struct InterfaceDecl {
    pub visibility: Option<Visibility>,
    pub name: String,
    pub super_interface: Option<String>,
    pub type_params: Vec<GenericParam>,  // 新規: `interface X<+T, -U>` 等
    pub methods: Vec<InterfaceMethod>,
    pub span: Span,
}
```

### Lexer 変更

`+` / `-` はすでに `Plus` / `Minus` トークンとして存在する。
型パラメータの先頭に現れる `+` / `-` はパーサーが文脈で判断する（新トークン不要）。

### Parser 変更

`parse_generic_params` を拡張して `+T` / `-T` を認識:

```
'<' → peek '+' / '-' → variance = Covariant / Contravariant
     → advance → parse identifier → GenericParam { name, variance }
```

`parse_interface_decl` で `interface Name<...>` の型パラメータを解析。

### Checker 変更

**分散チェック（`check_interface_variance`）:**

各 `InterfaceMethod` の型シグネチャを解析し:
- 共変 `+T` の型パラメータが引数位置に出現 → E0334
- 反変 `-T` の型パラメータが戻り値位置に出現 → E0334

**サブタイピング（`is_subtype_with_variance`）:**

`interface X<+T>` に対して `X<A>` と `X<B>` を比較する際:
- `+T`: `A <: B` なら `X<A> <: X<B>`
- `-T`: `B <: A` なら `X<A> <: X<B>`（逆転）
- `T`: `A == B` のみ `X<A> == X<B>`

Favnir における「サブタイプ」の判定は `is_compatible` メソッドを拡張して対応。

---

## テスト（v186000_tests、5件）

| テスト名 | 内容 |
|---|---|
| `version_is_18_6_0` | Cargo.toml に "18.6.0" が含まれる |
| `variance_covariant_parses` | `interface X<+T> { fn next() -> Option<T> }` が `GenericParam { variance: Covariant }` としてパースされる |
| `variance_contravariant_parses` | `interface X<-T> { fn write(val: T) -> Unit }` が `GenericParam { variance: Contravariant }` としてパースされる |
| `variance_subtype_covariant` | 共変 interface では `I<Sub>` が `I<Super>` として型チェックを通る |
| `variance_violation_error` | 共変 `+T` が入力位置に使われると E0334 が出る |

---

## 完了条件（PASS=5）

1. `+T`（共変）/ `-T`（反変）アノテーションが `GenericParam` としてパースされる
2. `InterfaceDecl` が `type_params: Vec<GenericParam>` を持つ
3. 共変インターフェースで `I<Sub>` を `I<Super>` 引数として渡せる（型チェックが通る）
4. 反変インターフェースでサブタイピングが逆転する
5. 分散違反（`+T` を入力位置で使う）が E0334 になる

---

## スコープ外（v19.x 以降）

- 分散の自動推論（Scala の `@covariant` 相当）
- 型エイリアス・struct への分散アノテーション
- 存在型（`exists T. F<T>`）との組み合わせ
- ユーザー定義の半順序関係（カスタムサブタイピング）
