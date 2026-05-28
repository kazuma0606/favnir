# Favnir v7.4.0 Spec

Date: 2026-05-27
Theme: stdlib 高レベル層（Favnir 化）+ email Rune

---

## 概要

v7.4.0 はロードマップで「Favnir 化するもの」として挙げていた stdlib 操作を整理し、
実際に未実装のものを `runes/stdlib/` に追加する。

### 現状確認（vm.rs 実装済み済み）

ロードマップ策定時点では「Favnir 化対象」としていたが、**すでに vm.rs に実装済み**のもの:
- List: `zip`, `chunk`, `flat_map`, `flatten`, `sort`（comparator 版）, `partition`, `count`, `enumerate`
- String: `split`, `pad_left`, `pad_right`, `replace`, `trim`, `words`, `lines`
- Map: `merge`, `from_list`, `map_values`, `filter_values`

### 本バージョンで追加するもの

#### Phase A — Favnir 層 stdlib（`runes/stdlib/`）

vm.rs の thin primitive を組み合わせて書けるもの:

| 関数 | 説明 | 実装方法 |
|------|------|---------|
| `List.group_by` | キー関数でリストを Map にグループ化 | `List.fold_left` + `Map.set` |
| `List.zip_with` | 2 リストを関数で結合 | `List.zip`（pair）+ `List.map` |
| `List.sort_by` | キー抽出関数でソート | `List.sort`（comparator）を利用 |
| `List.intersperse` | リスト要素の間にセパレータを挿入 | `List.fold_left` |
| `List.tail` | 先頭要素を除いたリスト | `List.drop(xs, 1)` の別名 |
| `List.head` | 先頭要素（= `List.first` の別名） | `List.first` を呼ぶだけ |
| `Map.empty` | 空の Map を返す | vm.rs に primitive 追加 |

> **注意**: `List.zip_with` は `List.zip` が `{ first: A, second: B }` レコードを返すことを利用する。
> `List.sort_by` は `List.sort`（comparator が Int を返す）上で実装できる。

#### Phase B — email Rune（`runes/email/`）

バックエンドは AWS SES（既存 SigV4 インフラを再利用）。

**VM primitive（新規）**:
```
Email.send_raw(
    from:    String,
    to:      String,
    subject: String,
    body:    String
) -> Result<Unit, String>  !Email
```

SES の `SendEmail` API（HTTPS POST）を呼ぶ thin wrapper。

**Favnir 層（`runes/email/email.fav`）**:
```favnir
// シンプル送信
public fn send(from, to, subject, body) -> Result<Unit, String> !Email

// 複数宛先
public fn send_multi(from, to_list, subject, body) -> Result<Int, String> !Email

// テンプレート（純粋）
public fn build_html_body(title, content) -> String
```

`!Email` エフェクトを `BUILTIN_EFFECTS` と compiler.rs に追加。

---

## 設計上の注意点

### List.group_by の型

Favnir にはタプルがないため、戻り値は `Map<String, List<A>>` とする。
キー関数のシグネチャ: `A -> String`（キーは必ず String）。

### List.sort_by の実装

`List.sort` はすでに comparator（`(A, A) -> Int`）版が vm.rs にある。
`sort_by(xs, key_fn)` は `sort(xs, |a, b| String.compare(key_fn(a), key_fn(b)))` で実装できる。
ただし `String.compare` が未実装なら vm.rs に追加が必要。

### Map.empty の必要性

`group_by` の fold 初期値として `Map.empty()` が必要。
`List.fold_left` の初期値に `Map.from_list(List.empty())` で代用できるが、
`Map.empty()` として明示的に提供する方が可読性が高い。

---

## 完了条件

- `runes/stdlib/list.fav`、`runes/stdlib/string.fav`（追加分）、`runes/stdlib/map.fav` が `fav check` を通る
- `runes/email/email.fav` が `fav check` を通る
- 各 Rune に 3 件以上の統合テスト
- `!Email` エフェクトが checker で追跡される
- 既存テスト 1070 件が全件通る（目標: 1095+ tests）
- サイトドキュメント追加（stdlib/list.mdx、stdlib/string.mdx、runes/email.mdx）
