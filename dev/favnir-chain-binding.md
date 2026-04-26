# Favnir Chain Binding Draft

更新日: 2026-04-26

## 結論

`bind` と `chain` の組み合わせは、Favnir にかなり合う。

整理すると:

- `bind` = 値や関数値の初回導入
- `chain` = 既存の束縛に対して段階的に処理を積む

ただし、`chain` は再代入ではない。  
内部的には fresh binding へ展開される sugar として扱う。

## 基本イメージ

```fav
bind user <- row
chain user <- parse_user
chain user <- normalize_user
chain user <- enrich_user
```

これは概念的には次のように展開される。

```fav
bind user_1 <- row
bind user_2 <- parse_user(user_1)
bind user_3 <- normalize_user(user_2)
bind user_4 <- enrich_user(user_3)
```

つまり、同じ論理名に対して「状態を上書きしているように見える」が、  
実際には新しい束縛を段階的に作っている。

## なぜ良いか

### 1. 再代入なしで段階的な処理を書ける

Favnir は immutable を重視するので、通常の再代入は避けたい。  
`chain` はその思想を壊さずに、段階的な変換を書ける。

### 2. `flw` より小さいローカル合成になる

- `trf` = 名前付き処理片
- `flw` = 再利用可能な処理列
- `chain` = ローカルな mini-flow

という役割分担ができる。

### 3. 読み味が良い

```fav
bind user <- row
chain user <- parse_user
chain user <- validate_user
chain user <- enrich_user
```

はかなり読みやすい。

## effect の扱い

ここが最重要。

### 基本原則

**effect は消えない。合成される。**

つまり:

- 元の束縛が effect を持っていれば、`chain` 後もその effect は残る
- 後続の処理が effect を持っていれば、それも加算される

## failure と effect

Favnir には少なくとも 2 種類の「文脈」がある。

### 1. failure

- `T!`

### 2. effect

- `!Db`
- `!Io`
- `!Emit<E>`

`chain` はこの両方を伝播させるべき。

## 例

```fav
bind user <- row
chain user <- parse_user
chain user <- normalize_user
chain id <- save_user
```

仮に:

- `parse_user : Row -> User!`
- `normalize_user : User -> User`
- `save_user : User -> UserId !Db`

なら、この chain 全体は:

- failure 可能
- `Db` effect を持つ

## 直感的なルール

- `bind` は値を導入する
- `chain` はその値に処理を積む
- 処理が failure を返せば、chain も failure を持つ
- 処理が effect を持てば、chain もその effect を持つ
- 途中で発生した failure / effect は消えない

## `chain` は何に使えるか

推奨方針:

- pure value
- fallible value
- effectful 文脈

の全部に使えるようにする。

これにより、`chain` は単なる値変換ではなく、**文脈付き合成**の sugar になる。

## 関数型的な見方

`chain` の意味は、文脈によって少し見え方が変わる。

- pure なら function composition
- fallible なら `and_then`
- effectful なら effect accumulation
- 将来的に async を含めるなら、非同期 bind 的に扱える

つまり、表面にはモナドを出さなくても、内部意味論としてはかなりモナド的。

## `chain` と `trf` / `flw`

`chain` は `trf` や `flw` を置き換えるものではない。

- `trf` = 名前付き処理片
- `flw` = 公開可能な処理列
- `chain` = ローカル文脈での段階的処理

この分離を守るのが重要。

## 将来の async との関係

将来的には `chain` と `await` の関係を決める必要がある。

ただし初期段階では、`chain` 自体に非同期魔法を持たせすぎない方がよい。

まずは:

- pure
- `T!`
- effect

までで整理し、後で async との統合を検討する。

## 初期仕様の提案

最初に採るなら次のルールが自然。

1. `bind` は初回導入
2. `chain` は直前の同名束縛に処理を積む
3. `chain` は内部的には fresh binding に展開される
4. failure は伝播する
5. effect は蓄積する
6. 元の effect は消えない

## 短い結論

`bind / chain` は、Favnir における

- immutable
- pipeline
- effect
- fallible computation

をローカル文脈で気持ちよく書くためのかなり有望な案。

特に重要なのは:

> `chain` は値ではなく、文脈ごと処理を積む

という見方。
