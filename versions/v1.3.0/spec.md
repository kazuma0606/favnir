# Favnir v1.3.0 仕様書 — `abstract trf` / `abstract flw`

作成日: 2026-05-07

> **テーマ**: パイプライン構造そのものを抽象化し、型安全な依存注入を実現する
>
> **設計ドキュメント**: `dev/post-v1/roadmap/fav-abstract-flw.md`、`dev/post-v1/roadmap/fav-abstraction-system.md`
>
> **注**: v1.3.0 では `abstract trf` / `abstract flw` のキーワードを使用する。v2.0.0 で `abstract stage` / `abstract seq` にリネーム予定。

---

## 1. スコープ概要

| Phase | テーマ | Done definition |
|---|---|---|
| 0 | バージョン更新 | `v1.3.0` がビルドされ、HELP テキストに反映される |
| 1 | AST + Lexer + Parser | `abstract trf` / `abstract flw` / `flw X = T { }` が正しくパースされる |
| 2 | 型検査統合 | スロット型検査・effect 推論・PartialFlw 型が動く |
| 3 | IR + VM 実行 | 完全束縛 `flw` が実際に実行できる |
| 4 | `fav check` 部分束縛警告 | 未束縛スロットを `fav check` で報告する |
| 5 | `fav explain` 統合 | テンプレート名・具体バインディング・解決済み型が表示される |
| 6 | テスト・ドキュメント | 新規テストが全通過、langspec.md 更新 |

---

## 2. Phase 0 — バージョン更新

- `Cargo.toml`: `version = "1.3.0"`
- `main.rs`: HELP テキスト `v1.3.0`

---

## 3. Phase 1 — AST + Lexer + Parser

### 3-1. 新規構文

#### `abstract trf` — 実装なし変換宣言

```fav
// シグネチャだけ宣言。実装は外部から注入する。
abstract trf FetchUser:    UserId -> User?      !Db
abstract trf ValidateUser: User   -> User!
abstract trf SaveUser:     User   -> UserId     !Db
```

- `abstract trf` は型シグネチャのみを宣言する（本体なし）
- `abstract trf` の値は `abstract flw` のスロットとして渡せる
- 関数引数の型として `FetchUser` を書くと、その引数は "FetchUser シグネチャを持つ任意の trf" を受け入れる

#### `abstract flw` — スロット付きパイプラインテンプレート

```fav
abstract flw DataPipeline<Row> {
    parse:    String    -> List<Row>!
    validate: Row       -> Row!
    save:     List<Row> -> Int      !Db
}
```

- スロット宣言は `slot_name: Input -> Output !Effects` の形式
- 型パラメータ `<Row>` は全スロット間で共有される
- `abstract flw` 自体はコードを生成しない（テンプレート定義のみ）

#### `flw X = Template<T> { ... }` — スロット束縛（完全束縛）

```fav
flw UserImport = DataPipeline<UserRow> {
    parse    <- ParseUserCsv
    validate <- ValidateUser
    save     <- SaveUsers
}
```

- テンプレートの全スロットを `slot <- Impl` で束縛する
- 型パラメータ `<UserRow>` を指定してスロットの期待型を確定する
- 束縛完了後、`UserImport` は通常の `flw` として実行できる
- `UserImport` の effect は全スロットの effect の合成（上記の例: `!Db`）

#### `flw X = Template<T> { ... }` — 部分束縛

```fav
// 一部のスロットだけを埋める（部分束縛）
flw PartialImport = DataPipeline<UserRow> {
    parse <- ParseUserCsv
}
// 型: PartialFlw<DataPipeline<UserRow>, { validate save }>
// validate, save が未束縛のため実行不可
```

#### 動的注入（関数引数から注入）

```fav
fn make_import(save: List<UserRow> -> Int !Db) -> flw String -> Int !Db {
    bind pipeline <- DataPipeline<UserRow> {
        parse    <- ParseUserCsv
        validate <- ValidateUser
        save     <- save        // 引数から注入
    }
    pipeline
}
```

### 3-2. 新規 AST ノード（`ast.rs`）

#### `AbstractTrfDef`

```rust
pub struct AbstractTrfDef {
    pub visibility: Option<Visibility>,
    pub name:       String,
    pub input:      TypeExpr,
    pub output:     TypeExpr,
    pub effects:    Vec<String>,
    pub span:       Span,
}
```

#### `AbstractFlwDef`

```rust
pub struct AbstractFlwDef {
    pub visibility:  Option<Visibility>,
    pub name:        String,
    pub type_params: Vec<String>,
    pub slots:       Vec<FlwSlot>,
    pub span:        Span,
}

pub struct FlwSlot {
    pub name:    String,
    pub input:   TypeExpr,
    pub output:  TypeExpr,
    pub effects: Vec<String>,
    pub span:    Span,
}
```

#### `FlwBindingDef`

```rust
pub struct FlwBindingDef {
    pub visibility: Option<Visibility>,
    pub name:       String,
    pub template:   String,
    pub type_args:  Vec<TypeExpr>,
    pub bindings:   Vec<(String, String)>, // slot_name -> impl_name
    pub span:       Span,
}
```

`Item` enum に3バリアントを追加:
- `Item::AbstractTrfDef(AbstractTrfDef)`
- `Item::AbstractFlwDef(AbstractFlwDef)`
- `Item::FlwBindingDef(FlwBindingDef)`

### 3-3. Lexer 変更（`lexer.rs`）

```rust
// 既存の Token に追加
Abstract,    // "abstract" キーワード

// キーワードマップ
"abstract" => Token::Abstract,
```

`abstract trf` / `abstract flw` はトップレベルのパーサーが `Token::Abstract` を見た時に次のトークン（`Trf` / `Flw`）で分岐する。

### 3-4. Parser 変更（`parser.rs`）

#### `parse_item` の拡張

```rust
Token::Abstract => {
    self.advance(); // consume "abstract"
    match self.peek() {
        Token::Trf => Ok(Item::AbstractTrfDef(self.parse_abstract_trf_def(vis)?)),
        Token::Flw => Ok(Item::AbstractFlwDef(self.parse_abstract_flw_def(vis)?)),
        _ => Err(ParseError::new("expected `trf` or `flw` after `abstract`", span)),
    }
}
```

#### `parse_flw_def` の拡張

既存の `flw Name: A -> B = |x| { ... }` に加え、`flw Name = Template<T> { ... }` 形式を識別:

```
flw → peek_next:
  ident ":" → 既存 FlwDef
  ident "=" ident "<" | ident "{" → FlwBindingDef
```

#### `parse_abstract_flw_def`

```
"abstract" "flw" Name ("<" type_params ">")?
    "{" slot* "}"

slot: ident ":" TypeExpr "->" TypeExpr effects
```

#### `parse_flw_binding_def`

```
"flw" Name "=" TemplateIdent ("<" type_args ">")?
    "{" binding* "}"

binding: ident "<-" ident
```

---

## 4. Phase 2 — 型検査統合

### 4-1. `abstract trf` の型検査

- `abstract trf FetchUser: UserId -> User? !Db` を型環境に登録
- `FetchUser` を型として使うと `TrfRef("FetchUser")` または `Type::Trf(input, output, effects)` に解決
- 型引数なしで利用できる（v1.3.0 では abstract trf は非ジェネリック）

### 4-2. `abstract flw` のテンプレート登録

チェッカー内に `abstract_flw_registry: HashMap<String, AbstractFlwDef>` を追加:

```rust
// check_abstract_flw_def
fn check_abstract_flw_def(&mut self, def: &AbstractFlwDef) {
    // スロット名の重複チェック
    // スロットの型式を解決・保存
    self.abstract_flw_registry.insert(def.name.clone(), def.clone());
}
```

### 4-3. `flw` 束縛の型検査（コアロジック）

`check_flw_binding_def` で以下を検査:

1. **テンプレート存在確認**: `template` が `abstract_flw_registry` に存在するか
2. **型引数の代入**: `type_params` に `type_args` を代入してスロット型を具体化
3. **スロット名確認**: 束縛したスロット名がテンプレートに存在するか（E049）
4. **スロット型照合**: 束縛した実装 `Impl` の型がスロット期待型と一致するか（E048）
5. **未束縛スロット検出**: 束縛されていないスロットが残っているか
   - 残りあり → `PartialFlw<Template, {remaining}>` 型として記録
   - 残りなし → 具体 `flw` として完成。effect は全スロット effect の合成

### 4-4. 新規エラーコード

| コード | 内容 | 例 |
|---|---|---|
| E048 | `abstract flw` スロット型不一致 | `validate <- ValidateOrder` (OrderRow 期待, UserRow のスロット) |
| E049 | 未知スロット名（テンプレートに存在しない） | `extra <- SomeImpl` |
| E050 | 必須スロット未束縛のまま実行 | `PartialFlw` に `main` が依存している |
| E051 | `abstract trf` を直接実行しようとした | `FetchUser(id)` (abstract trf は実装なし) |

### 4-5. Effect 推論

完全束縛の `flw` の effect = 全スロットの effect の合成:

```rust
fn infer_flw_effects(slots: &[ResolvedSlot]) -> Vec<String> {
    let mut effects = vec![];
    for slot in slots {
        for e in &slot.effects {
            if !effects.contains(e) {
                effects.push(e.clone());
            }
        }
    }
    effects
}
```

テンプレートのスロット宣言に書かれた effect と、実際に束縛された実装の effect の両方を合算する。

### 4-6. `PartialFlw<Template, {remaining}>` 型の表現

```rust
// Type enum に追加
Type::PartialFlw {
    template:         String,
    type_args:        Vec<Type>,
    unbound_slots:    Vec<String>,
}
```

`PartialFlw` の値は `fav run` / `fav build` でエントリポイントとして使えない（E050）。
ただし変数に束縛して渡すことは可能（段階的に完成させるユースケース向け）。

---

## 5. Phase 3 — IR + VM 実行

### 5-1. 完全束縛 `flw` の IR 生成

`abstract trf` と `abstract flw` テンプレート自体はコードを生成しない（型情報のみ）。

完全束縛 `flw X = Template<T> { ... }` は以下のように IR を生成:

```
flw UserImport:
    fn UserImport(input: String) -> Int !Db {
        bind parsed   <- ParseUserCsv(input)   // parse スロット
        bind validated <- ValidateUser(parsed) // validate スロット (mapした形)
        bind result   <- SaveUsers(validated)  // save スロット
        result
    }
```

実際には `flw` のセマンティクス（stage の直列合成）に従い、各スロットの出力を次スロットの入力として繋げる。

### 5-2. スロット合成の IR パターン

スロット間の型接続:

```
abstract flw DataPipeline<Row> {
    parse:    String    -> List<Row>!    // A -> B!
    validate: Row       -> Row!          // B_elem -> C!
    save:     List<Row> -> Int      !Db  // C_list -> D
}
```

合成ルール:
- `parse` の出力 `List<Row>!` の中身 `List<Row>` が `save` の入力
- `validate` は `List<Row>` の各要素に `map` で適用される（`List<Row>! -> List<Row>!`）
- chain / collect を組み合わせて IR を生成

### 5-3. 部分束縛の実行エラー

`PartialFlw` を `fav run` で実行しようとした場合、コンパイル段階で E050 を出す:

```
Error E050: `PartialImport` has unbound slots: validate, save
  Hint: bind remaining slots before running
```

### 5-4. VM の動作確認

完全束縛された `flw` は通常の `flw` の実行パスで処理される。
VM 側に追加変更は不要（コンパイラが通常の `flw` IR を生成するため）。

---

## 6. Phase 4 — `fav check` 部分束縛警告

### 6-1. 部分束縛の警告

`fav check` で `PartialFlw` が final expression / 戻り値として使われている場合に警告:

```
Warning: `PartialImport` has unbound slots: validate, save
  hint: use `flw PartialImport = DataPipeline<UserRow> { validate <- ...; save <- ... }` to complete
```

### 6-2. トップレベル `FlwBindingDef` の完全性チェック

`check` コマンドはトップレベルの `flw X = Template { ... }` 宣言が全スロットを束縛しているか確認し、
未束縛スロットがあれば警告を出す（エラーにはしない—部分束縛は合法）。

---

## 7. Phase 5 — `fav explain` 統合

### 7-1. `abstract flw` テンプレートの表示

```
ABSTRACT FLW DataPipeline<Row>
  parse    : String -> List<Row>!
  validate : Row -> Row!
  save     : List<Row> -> Int !Db
```

### 7-2. 具体束縛の表示

```
FLW UserImport  (DataPipeline<UserRow>)
  parse    : String -> List<UserRow>!     <- ParseUserCsv
  validate : UserRow -> UserRow!          <- ValidateUser
  save     : List<UserRow> -> Int !Db    <- SaveUsers

  resolved : String -> Int !Db
  effects  : Db
```

### 7-3. `abstract trf` の表示

```
ABSTRACT TRF FetchUser   : UserId -> User? !Db
ABSTRACT TRF ValidateUser: User   -> User!
```

---

## 8. Phase 6 — テスト・ドキュメント

### テスト要件

#### パーサーテスト

| テスト名 | 検証内容 |
|---|---|
| `test_parse_abstract_trf` | `abstract trf Name: A -> B !Fx` がパースできる |
| `test_parse_abstract_flw_single_slot` | 1スロット `abstract flw` がパースできる |
| `test_parse_abstract_flw_multi_slot` | 複数スロット・型パラメータ付きがパースできる |
| `test_parse_flw_binding_full` | 全スロット束縛がパースできる |
| `test_parse_flw_binding_partial` | 部分束縛がパースできる |

#### 型検査テスト

| テスト名 | 検証内容 |
|---|---|
| `test_flw_binding_type_ok` | 型一致スロット束縛が型検査を通る |
| `test_flw_binding_e048` | スロット型不一致で E048 が出る |
| `test_flw_binding_e049` | 未知スロット名で E049 が出る |
| `test_flw_binding_effect_inference` | 完全束縛の effect が正しく推論される |
| `test_flw_partial_type` | 部分束縛が `PartialFlw` 型になる |
| `test_abstract_trf_direct_call_e051` | abstract trf の直接呼び出しで E051 が出る |

#### 実行テスト

| テスト名 | 検証内容 |
|---|---|
| `test_flw_binding_exec_ok` | 完全束縛 flw が正常実行される |
| `test_flw_binding_partial_e050` | `PartialFlw` の実行で E050 が出る |
| `test_flw_effect_combined` | 合成 effect が `!Db !Network` のように結合される |

#### example ファイル

- `examples/abstract_flw_basic.fav` — `abstract flw` + 完全束縛 + 実行
- `examples/abstract_flw_inject.fav` — 関数引数による動的注入パターン

### ドキュメント更新

- `versions/v1.3.0/langspec.md` を新規作成（v1.2.0 langspec を起点に abstract trf/flw 節を追加）
  - `abstract trf` / `abstract flw` 構文と例
  - スロット束縛のルール（型パラメータ代入・effect 合成）
  - `PartialFlw` の制約
  - E048–E051 エラーコード
- `README.md` に v1.3.0 セクションを追加

---

## 9. 実装上の注意点

### `flw` と `abstract flw` の構文的区別

既存の `flw` 構文:
```
flw Name: A -> B !Fx = |x| { ... }
```

新規束縛構文:
```
flw Name = Template<T> { slot <- Impl; ... }
```

パーサーは `flw Name :` と `flw Name =` の次のトークンで分岐する。
`Name` 後に `:` が来れば既存形式、`=` が来れば束縛形式。

### スロット合成の型安全性

`abstract flw DataPipeline<Row>` の `parse: String -> List<Row>!` と `save: List<Row> -> Int !Db` は
「parse の出力型 = save の入力型（`List<Row>`）」という連続性を暗黙に持つ。
v1.3.0 ではこの連続性の **型検査は行わない**（各スロットの独立した型照合のみ）。
スロット間の連続性の型検査は v2.0.0 のセルフホスト移植時に追加予定。

### 型パラメータの単態化

`DataPipeline<UserRow>` で `Row = UserRow` を代入すると:
- `parse` スロット: `String -> List<UserRow>!`
- `validate` スロット: `UserRow -> UserRow!`
- `save` スロット: `List<UserRow> -> Int !Db`

型パラメータの代入は `check_flw_binding_def` 内で `TypeSubst` を使って行う。

---

## 10. 完了条件（Done Definition）

- [ ] `abstract flw DataPipeline<Row> { parse: String -> List<Row>!; save: List<Row> -> Int !Db }` が定義できる
- [ ] `flw UserImport = DataPipeline<UserRow> { parse <- ParseCsv; save <- SaveUsers }` が型検査を通る
- [ ] スロット型不一致で E048 が出る
- [ ] 部分束縛中の `PartialFlw<...>` が `fav run`/`fav build` で E050 になる
- [ ] `fav explain` にテンプレート名・具体バインディング・解決済み型が表示される
- [ ] `abstract trf FetchUser: UserId -> User? !Db` が定義できる
- [ ] 完全束縛 `flw` が実際に実行できる
- [ ] v1.2.0 の全テストが引き続き通る
