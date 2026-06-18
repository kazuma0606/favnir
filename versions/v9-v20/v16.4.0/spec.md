# v16.4.0 Spec — 標準ライブラリ拡充

Date: 2026-06-14

---

## 概要

データ処理で最頻使用するパターンを stdlib で直接サポートする。
`List.group_by` / `DateTime.now` / `Math.round_to` など、毎回自前実装していた関数を提供する。

**Cargo 依存追加:** `chrono 0.4`（DateTime 内部実装用）

---

## List 拡充（17 関数）

### グループ化・集計

```fav
List.group_by(rows, |r| r.category)       -- Map<String, List<Row>>
List.count_where(rows, |r| r.active)       -- Int（条件付き件数）
List.sum_by(rows, |r| r.amount)            -- Float（合計）
List.max_by(rows, |r| r.score)             -- 最大値要素（そのレコード）
List.min_by(rows, |r| r.score)             -- 最小値要素（そのレコード）
```

### ソート

```fav
List.sort_by(rows, |r| r.amount)           -- 昇順ソート
List.sort_by_desc(rows, |r| r.created_at) -- 降順ソート
```

### 変換・結合

```fav
List.flatten(nested)                       -- List<List<T>> -> List<T>
List.flat_map(rows, |r| expand(r))         -- map + flatten
List.zip(as, bs)                           -- List<Pair<A, B>>
List.zip_with(as, bs, |a, b| merge(a, b)) -- List<C>
List.unzip(pairs)                          -- Pair<List<A>, List<B>>
```

### フィルタリング・スライス

```fav
List.distinct(rows)                        -- 重複除去（== による）
List.distinct_by(rows, |r| r.id)          -- キー指定重複除去
List.chunk(rows, 100)                      -- List<List<T>>（バッチ分割）
List.take(rows, 10)                        -- 先頭 N 件
List.drop(rows, 10)                        -- 先頭 N 件を除く
```

### 型シグネチャ

| 関数 | 入力 | 出力 |
|---|---|---|
| `group_by(List<T>, fn(T)->String)` | List, key fn | Map<String, List<T>> |
| `sort_by(List<T>, fn(T)->Int)` | List, key fn | List<T> |
| `sort_by_desc(List<T>, fn(T)->Int)` | List, key fn | List<T> |
| `flatten(List<List<T>>)` | nested list | List<T> |
| `flat_map(List<T>, fn(T)->List<U>)` | List, fn | List<U> |
| `zip(List<A>, List<B>)` | 2 lists | List<Pair<A,B>> |
| `zip_with(List<A>, List<B>, fn(A,B)->C)` | 2 lists, fn | List<C> |
| `distinct(List<T>)` | List | List<T> |
| `distinct_by(List<T>, fn(T)->String)` | List, key fn | List<T> |
| `chunk(List<T>, Int)` | List, size | List<List<T>> |
| `take(List<T>, Int)` | List, n | List<T> |
| `drop(List<T>, Int)` | List, n | List<T> |
| `count_where(List<T>, fn(T)->Bool)` | List, pred | Int |
| `sum_by(List<T>, fn(T)->Float)` | List, fn | Float |
| `max_by(List<T>, fn(T)->Float)` | List, fn | T |
| `min_by(List<T>, fn(T)->Float)` | List, fn | T |
| `unzip(List<Pair<A,B>>)` | pair list | Pair<List<A>,List<B>> |

**注意:** `group_by` のキーは String のみ（ジェネリックキーは v18.x 以降）。

---

## String 拡充（17 関数）

```fav
-- 分割
String.split(s, ",")                      -- List<String>
String.split_once(s, "=")                 -- Pair<String, String>（見つからなければ Pair(s, "")）
String.lines(s)                           -- List<String>（改行で分割）

-- トリム
String.trim(s)                            -- 両端空白除去
String.trim_start(s)                      -- 先頭空白除去
String.trim_end(s)                        -- 末尾空白除去

-- 置換
String.replace(s, "old", "new")           -- 全置換
String.replace_first(s, "old", "new")     -- 先頭のみ置換

-- 判定
String.starts_with(s, prefix)             -- Bool
String.ends_with(s, suffix)               -- Bool
String.is_empty(s)                        -- Bool

-- 変換
String.to_upper(s)                        -- String
String.to_lower(s)                        -- String
String.repeat(s, n)                       -- "abc" を n 回繰り返す
String.char_at(s, i)                      -- String（見つからなければ ""）
String.pad_left(s, 10, "0")              -- ゼロ埋め（左）
String.pad_right(s, 20, " ")             -- スペース埋め（右）

-- 数値フォーマット
String.format_int(n, 3, "0")             -- "007"（幅 3、パッドカラ '0'）
String.format_float(f, 2)                 -- "3.14"（小数点以下 2 桁）
```

**型シグネチャ:**

| 関数 | 戻り型 |
|---|---|
| `split(String, String)` | List<String> |
| `split_once(String, String)` | Pair<String, String> |
| `lines(String)` | List<String> |
| `trim(String)` | String |
| `trim_start(String)` | String |
| `trim_end(String)` | String |
| `replace(String, String, String)` | String |
| `replace_first(String, String, String)` | String |
| `starts_with(String, String)` | Bool |
| `ends_with(String, String)` | Bool |
| `is_empty(String)` | Bool |
| `to_upper(String)` | String |
| `to_lower(String)` | String |
| `repeat(String, Int)` | String |
| `char_at(String, Int)` | String |
| `pad_left(String, Int, String)` | String |
| `pad_right(String, Int, String)` | String |
| `format_int(Int, Int, String)` | String |
| `format_float(Float, Int)` | String |

---

## DateTime（新モジュール）

### 型

`DateTime` は VM 内部では Unix timestamp（Int 秒）として保持する。
Favnir 上では `DateTime` 型として見える（不透明型）。

### 関数

```fav
-- 現在時刻
DateTime.now()                             -- DateTime
DateTime.now_unix()                        -- Int（Unix timestamp 秒）

-- 変換
DateTime.parse("2026-06-14T12:00:00Z")    -- Result<DateTime, String>
DateTime.format(dt, "YYYY-MM-DD")          -- String
DateTime.format_iso(dt)                    -- "2026-06-14T12:00:00Z"

-- 演算
DateTime.add_days(dt, 7)                   -- DateTime
DateTime.add_hours(dt, 24)                 -- DateTime
DateTime.diff_days(from, to)               -- Int
DateTime.diff_seconds(from, to)            -- Int

-- 比較
DateTime.before(a, b)                      -- Bool
DateTime.after(a, b)                       -- Bool
DateTime.between(dt, from, to)             -- Bool
```

### VM 実装方針

- `DateTime.now()` → `chrono::Utc::now()` → `Value::Int(timestamp)`
- `DateTime.now_unix()` → 同上（Int を直接返す）
- `DateTime.parse(s)` → `chrono::DateTime::parse_from_rfc3339(s)` → `Value::Result(...)`
- `DateTime.format(dt, fmt)` → `chrono` の `format` 関数
- `DateTime.format_iso(dt)` → `chrono` の `to_rfc3339`
- 演算系: chrono の `+Duration::days(n)` 等
- 比較系: Int 比較（timestamp 値で直接比較）

---

## Math 拡充（10 関数）

```fav
Math.abs(n)                                -- Int または Float
Math.round(f)                              -- Int（四捨五入）
Math.ceil(f)                               -- Int（切り上げ）
Math.floor(f)                              -- Int（切り捨て）
Math.round_to(f, 2)                        -- Float（小数点以下 n 桁）
Math.min(a, b)                             -- Int または Float
Math.max(a, b)                             -- Int または Float
Math.clamp(v, lo, hi)                      -- Int または Float
Math.sqrt(f)                               -- Float
Math.pow(base, exp)                        -- Float
Math.log(f)                                -- Float（自然対数）
Math.log2(f)                               -- Float
Math.log10(f)                              -- Float
```

**注意:** `abs` / `min` / `max` / `clamp` は Int / Float どちらも受け付ける（オーバーロード扱い、VM ランタイムで型判定）。

---

## エラーコード

新エラーコードの追加は最小限にとどめる。既存エラーコードで対応可能。

---

## テスト（v164000_tests — 5 件）

| # | テスト名 | 確認内容 |
|---|---|---|
| 1 | `version_is_16_4_0` | `Cargo.toml` バージョンが 16.4.0 |
| 2 | `list_group_by_works` | `group_by` で Map が返り、グループが正しい |
| 3 | `list_chunk_works` | `chunk(rows, 3)` で正しくバッチ分割される |
| 4 | `string_split_works` | `split("a,b,c", ",")` → 3 要素リスト |
| 5 | `datetime_now_unix_works` | `now_unix()` が正の Int を返す |

---

## 完了条件

| 確認項目 | 状態 |
|---|---|
| `Cargo.toml version == "16.4.0"` | [ ] |
| `chrono 0.4` が Cargo 依存に追加されている | [ ] |
| `List.group_by` / `List.chunk` / `List.sort_by` が動作する | [ ] |
| `String.split` / `String.trim` / `String.replace` が動作する | [ ] |
| `DateTime.now` / `DateTime.now_unix` / `DateTime.format_iso` が動作する | [ ] |
| `Math.round_to` / `Math.clamp` が動作する | [ ] |
| `cargo test v164000` 全テストパス（5/5） | [ ] |
| `cargo test` 全件パス（リグレッションなし） | [ ] |
| `site/content/docs/stdlib/` ドキュメントが追加されている | [ ] |
