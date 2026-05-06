# Favnir v1.1.0 仕様書 — `interface` システム

作成日: 2026-05-05

> **テーマ**: 型の抽象化を `interface` キーワードで再設計し、自動合成・明示的な値渡しを確立する
>
> **設計ドキュメント**: `dev/post-v1/roadmap/fav-abstraction-system.md`、`dev/post-v1/roadmap/fav-algebraic-structures.md`

---

## 1. スコープ概要

| Phase | テーマ | Done definition |
|---|---|---|
| 0 | バージョン更新 | `v1.1.0` がビルドされ、HELP テキストに反映される |
| 1 | `interface` + 手書き `impl` — パーサー拡張 | `interface Show { ... }` と `impl Show for Int { ... }` がパースできる |
| 2 | 型検査統合 | interface の method 型検査・値渡し検査が動く |
| 3 | 自動合成 + `with` 糖衣構文 | `impl Show for UserRow`（本体なし）と `type T with Show { ... }` が動く |
| 4 | 標準 interface 移行（Eq / Ord / Show） | 旧 `cap` ベースの Eq/Ord/Show が `interface` として再定義される |
| 5 | `Gen` + `Field` interface 定義 | `Stat.one<T>` の基盤となる `Gen` と代数構造の `Field` が stdlib に追加される |
| 6 | `cap` 非推奨警告 | `cap` キーワードに W010 警告が出るが、コンパイルは通る |
| 7 | テスト・ドキュメント | 321 + 新規テストが全通過、langspec.md 更新 |

---

## 2. Phase 0 — バージョン更新

### 変更内容

- `Cargo.toml`: `version = "1.1.0"`
- `main.rs`: HELP テキスト `v1.1.0`
- `versions/v1.1.0/spec.md`: 本ファイル

---

## 3. Phase 1 — `interface` + 手書き `impl` — パーサー拡張

### 新規構文

#### `interface` 宣言

```fav
interface Show {
    show: Self -> String
}

interface Eq {
    eq: Self -> Self -> Bool
}

interface Ord : Eq {       -- Eq を前提条件とする（スーパーインターフェース）
    compare: Self -> Self -> Int
}
```

- `interface Name { method: Type }` — メソッドシグネチャのみ記述
- `interface Name : SuperInterface { ... }` — スーパーインターフェース継承
- `Self` は実装型を指す特殊型キーワード

#### `impl` 手書き実装（本体あり）

```fav
impl Show for Int {
    show = |x| Int.to_string(x)
}

impl Eq for UserRow {
    eq = |a b| a.id == b.id
}

impl Ord for Int : Eq {    -- Ord は Eq を必要とするので両方 impl が必要
    compare = |a b|
        if a < b { -1 }
        else if a > b { 1 }
        else { 0 }
}
```

### パーサー変更点（`frontend/parser.rs`）

- `interface` を新規キーワードとして `lexer.rs` に追加
- `parse_interface_decl` — `interface Name { method: Type... }` をパース
  - スーパーインターフェース: `: SuperName` オプション
  - メソッドシグネチャ: `name: Type -> Type` の繰り返し
- `parse_impl_decl` — `impl Name for Type { ... }` をパース
  - 本体あり（手書き）と本体なし（自動合成、Phase 3）を区別
- AST に `InterfaceDecl` / `ImplDecl` ノードを追加（`ast.rs`）

### AST ノード（`ast.rs`）

```rust
pub struct InterfaceDecl {
    pub name: String,
    pub super_interface: Option<String>,
    pub methods: Vec<InterfaceMethod>,  // (name, type_sig)
    pub span: Span,
}

pub struct ImplDecl {
    pub interface_name: String,
    pub type_name: String,
    pub methods: Vec<(String, Expr)>,   // 本体あり
    pub is_auto: bool,                  // 本体なし = auto-synthesis
    pub span: Span,
}
```

---

## 4. Phase 2 — 型検査統合

### InterfaceRegistry

`middle/checker.rs` に `InterfaceRegistry` を追加。既存の `IMPL_REGISTRY`（cap 用）とは別に管理し、v2.0.0 で統合する。

```rust
struct InterfaceRegistry {
    // interface 名 → (スーパーinterface, メソッド定義)
    interfaces: HashMap<String, InterfaceDef>,
    // (interface 名, 型名) → 実装メソッド群
    impls: HashMap<(String, String), ImplDef>,
}
```

### 型検査ルール

1. **メソッド型一致**: `impl` 内の各メソッドが interface 定義の型シグネチャに一致するか検査
2. **スーパーインターフェース充足**: `interface Ord : Eq` なら、`impl Ord for T` が存在するとき `impl Eq for T` も存在しなければエラー
3. **明示的な値渡し**: interface を型パラメータに使う関数は、実装値を引数として受け取る

#### 明示的な値渡しの例

```fav
-- interface 値を明示的に受け取る（暗黙解決なし）
fn sort<T>(items: List<T>, ord: Ord<T>) -> List<T> { ... }

-- 呼び出し側で実装を明示
bind sorted <- sort(users, User.ord)
--                         ^^^^^^^^ Ord<User> の実装値
```

- `User.ord` は `impl Ord for User` から生成される実装値
- コンパイラは `Ord<User>` が実装されているか検査するが、暗黙に解決はしない
- 未実装の場合は E043 を出す

### 新規エラーコード

| コード | 内容 | 例 |
|---|---|---|
| E041 | 未定義の interface を `impl` しようとした | `impl UnknownIface for Int { ... }` |
| E042 | `impl` のメソッドが interface シグネチャと型不一致 | `show` が `Self -> Int` なのに `Self -> String` を期待 |
| E043 | 値渡し時に要求 interface が未実装 | `sort(users, User.ord)` で `impl Ord for User` がない |

---

## 5. Phase 3 — 自動合成 + `with` 糖衣構文

### `impl`（本体なし）— 自動合成

```fav
-- UserRow の全フィールドが Show / Eq / Json / Csv を持つ場合のみ有効
impl Show, Eq, Json, Csv for UserRow
```

**合成ルール**:
- フィールドの型が全て対象 interface を実装していれば、コンパイラが実装を生成する
- フィールドが interface を満たさない場合は E044 を出す
- `show` の合成: `"{field1: {f1.show()}, field2: {f2.show()}, ...}"` 形式の文字列
- `eq` の合成: 全フィールドの `eq` の AND
- 再帰的な合成（ネストした型）も対応

### `with` 糖衣構文

```fav
type UserRow with Show, Eq, Json, Csv {
    name:  String
    email: String
    age:   Int
}
```

`with` は型宣言と同時に `impl Show, Eq, Json, Csv for UserRow` を宣言するシンタックスシュガー。
上記は以下と完全に等価：

```fav
type UserRow {
    name:  String
    email: String
    age:   Int
}

impl Show, Eq, Json, Csv for UserRow
```

### 新規エラーコード

| コード | 内容 | 例 |
|---|---|---|
| E044 | 自動合成時にフィールドが interface を未実装 | `impl Show for T` で `T.field` に `Show` がない |

---

## 6. Phase 4 — 標準 interface 移行（Eq / Ord / Show）

### 変更内容

既存の `cap Eq / Ord / Show`（v0.4.0 で導入）を `interface` として再定義し、
内部実装を `InterfaceRegistry` へ移行する。

```fav
-- stdlib として内部定義される標準 interface
interface Show {
    show: Self -> String
}

interface Eq {
    eq: Self -> Self -> Bool
}

interface Ord : Eq {
    compare: Self -> Self -> Int
}
```

### 組み込み型の impl（内部登録）

| 型 | 自動登録される impl |
|---|---|
| `Int` | `Show`, `Eq`, `Ord` |
| `Float` | `Show`, `Eq`, `Ord` |
| `Bool` | `Show`, `Eq` |
| `String` | `Show`, `Eq`, `Ord` |
| `List<T>` | `Show`（T が Show のとき）、`Eq`（T が Eq のとき） |
| `Option<T>` | `Show`（T が Show のとき）、`Eq`（T が Eq のとき） |
| `Result<T,E>` | `Show`（T, E が Show のとき） |

### 後方互換

- 旧 `cap Eq / Ord / Show` を用いた既存コードは引き続き動作する
- `IMPL_REGISTRY`（cap 用）の参照は内部で `InterfaceRegistry` にブリッジされる
- 旧 `cap` 構文へは W010 警告（Phase 6 で追加）

---

## 7. Phase 5 — `Gen` + `Field` interface 定義

### `Gen` interface

型駆動のデータ生成（`Stat.one<T>`）の基盤。v1.5.0 の `Stat` ルーンで使用する。

```fav
interface Gen {
    gen: Int? -> Self    -- seed を受け取り値を生成（None = ランダムシード）
}
```

**組み込み型の `impl Gen`（内部登録）**:

| 型 | 生成内容 |
|---|---|
| `Int` | 範囲 `[-1000, 1000]` の乱数 |
| `Float` | 範囲 `[0.0, 1.0]` の乱数 |
| `Bool` | `true` / `false` を確率 0.5 で |
| `String` | 長さ `[0, 16]` の英数字列 |

**自動合成の条件**:
`impl Gen for UserRow`（本体なし）は「全フィールドが `Gen` を実装している」場合のみ有効。
合成された `gen` はフィールドごとに `gen(seed)` を呼ぶ（seed は派生させる）。

### 代数構造 interface (`Field` 系列)

`fav-algebraic-structures.md` に基づく代数構造の interface 群。

```fav
interface Semigroup {
    combine: Self -> Self -> Self     -- 結合法則を持つ二項演算
}

interface Monoid : Semigroup {
    empty: Self                       -- 単位元
}

interface Group : Monoid {
    inverse: Self -> Self             -- 逆元
}

interface Ring : Monoid {
    multiply: Self -> Self -> Self    -- 分配法則を持つ乗算
}

interface Field : Ring {
    divide: Self -> Self -> Self!     -- 除算（ゼロ除算は T! で伝播）
}
```

**組み込み型の `impl`（内部登録）**:

| 型 | impl |
|---|---|
| `Float` | `Semigroup`, `Monoid`, `Group`, `Ring`, `Field` |
| `Int` | `Semigroup`, `Monoid`, `Group`, `Ring`（`Field` は除く: 整数除算は切り捨てのため） |

**演算子オーバーロードの対応**:

| interface | 演算子 |
|---|---|
| `Semigroup` | `+` |
| `Group` | 単項 `-` |
| `Ring` | `*` |
| `Field` | `/` |

---

## 8. Phase 6 — `cap` 非推奨警告

### W010: `cap` キーワードの使用

```
W010: `cap` is deprecated. Use `interface` instead.
  --> src/main.fav:3:1
  |
3 | cap Show { show: Self -> String }
  | ^^^ deprecated keyword
  |
  = help: replace `cap` with `interface`
```

**動作**:
- `cap` で書かれたコードは **引き続きコンパイル・実行できる**
- `fav check` 実行時に W010 が表示される（エラーではなく警告）
- `fav check --no-warn` で W010 を抑制できる

**対象**:
- `cap Name { ... }` 宣言
- `impl Name` / `impl Name for Type`（旧 cap スタイル）
- `where T: Cap` 構文（v0.4.0 の遺物）

---

## 9. Phase 7 — テスト・ドキュメント

### テスト要件

#### 既存テストの全通過

- v1.0.0 の 321 テストが全て通ること
- `cap` ベースのテスト（v0.4.0 の generics.fav / cap_sort.fav / cap_user.fav）が W010 付きで通ること

#### 新規テスト（`middle/checker.rs` または `integration/` に追加）

| テスト名 | 検証内容 |
|---|---|
| `interface_show_int` | `impl Show for Int { show = \|x\| ... }` が型検査を通る |
| `interface_manual_impl_type_mismatch` | method 型不一致で E042 が出る |
| `interface_super_missing` | `impl Ord for T` で `impl Eq for T` がなければ E043 が出る |
| `interface_auto_synthesis_ok` | `impl Show for UserRow`（本体なし）が合成される |
| `interface_auto_synthesis_fail` | フィールドが Show 未実装なら E044 が出る |
| `type_with_sugar` | `type T with Show, Eq { ... }` が等価 `impl` と同動作 |
| `interface_explicit_passing` | `fn sort<T>(items, ord: Ord<T>)` で未実装時に E043 |
| `gen_interface_builtin` | `impl Gen for Int` が内部登録されている |
| `gen_interface_auto_synthesis` | `impl Gen for UserRow`（全フィールド Gen あり）が動く |
| `field_interface_float` | `impl Field for Float` が内部登録されている |
| `cap_deprecated_warning` | `cap` キーワードに W010 が出る |
| `cap_still_compiles` | W010 が出ても実行結果は変わらない |

#### example ファイル

- `examples/interface_basic.fav` — `interface` / `impl` の基本使用例
- `examples/interface_auto.fav` — `with` 糖衣構文と自動合成の例
- `examples/algebraic.fav` — `Field` / `Ring` を使った加重平均の例

### ドキュメント更新

- `versions/v1.0.0/langspec.md` の「5. cap システム」節を「5. interface システム」に書き換え（旧 cap の説明を補足として残す）
- `README.md` に v1.1.0 セクションを追加
- `HELP` テキスト: `v1.1.0` を反映

---

## 10. 実装の注意点

### `cap` との共存

v1.1.0 では `cap` と `interface` が両方動く。内部実装:

```
IMPL_REGISTRY (旧 cap 用)     InterfaceRegistry (新)
      │                               │
      └──────── ブリッジ ────────────▶│
                                       │
                              checker.rs が統合参照
```

v2.0.0 で `IMPL_REGISTRY` を削除し、`InterfaceRegistry` に一本化する。

### `Self` 型の扱い

- `interface` のメソッドシグネチャ内の `Self` は「実装対象の型」を指す特殊キーワード
- `impl Show for Int` では `Self = Int` に展開される
- `List<Self>` のような使い方も許容する

### 自動合成の再帰

`impl Show for Pair<A, B>`（`A` と `B` が共に Show を持つ場合）のような再帰的な合成も v1.1.0 でサポートする。
合成失敗の場合は E044 でどのフィールドが原因かを示す。

### 明示的な値渡しの型表現

`Ord<T>` のように interface を型として使う場合、型検査器では `Type::Interface(name, type_args)` として扱う。
既存の `Type::Var` / `Type::Cap` 系と共存させ、v2.0.0 で統合する。

---

## 11. 完了条件（Done Definition）

- [ ] `interface Show { show: Self -> String }` と `impl Show for Int { show = |x| ... }` が動く
- [ ] `impl Show, Eq for UserRow`（本体なし）が全フィールドから自動合成される
- [ ] `type UserRow with Show, Eq { ... }` が上記のシンタックスシュガーとして機能する
- [ ] `fn sort<T>(items: List<T>, ord: Ord<T>) -> List<T>` の呼び出しで未実装時に E043 が出る
- [ ] `interface Gen { gen: Int? -> Self }` が定義され、`Int/Float/Bool/String` の impl が内部登録される
- [ ] `interface Field : Ring { divide: ... }` が定義され、`Float` の impl が内部登録される
- [ ] `cap` で書かれた既存コードに W010 警告が出るが、動作は継続する
- [ ] v1.0.0 の 321 テストが全て通る
- [ ] 新規 interface テストが全て通る
