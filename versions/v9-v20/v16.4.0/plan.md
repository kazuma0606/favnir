# v16.4.0 Plan — 標準ライブラリ拡充

Date: 2026-06-14

---

## 実装フェーズ一覧

| Phase | 内容 | 主要ファイル |
|---|---|---|
| A | Cargo バージョン更新 + chrono 依存追加 | `Cargo.toml` |
| B | VM: List 拡充プリミティブ追加 | `vm.rs` |
| C | VM: String 拡充プリミティブ追加 | `vm.rs` |
| D | VM: DateTime プリミティブ追加 | `vm.rs` |
| E | VM: Math 拡充プリミティブ追加 | `vm.rs` |
| F | compiler.rs: builtin_ret_ty に新関数を追加 | `compiler.rs` |
| G | checker.rs: builtin_ret_ty に新関数を追加 | `checker.rs` |
| H | テスト追加（v164000_tests — 5 件） | `driver.rs` |
| I | サイトドキュメント追加 | `site/content/docs/stdlib/` |

---

## Phase A — Cargo バージョン更新 + chrono 依存追加

`fav/Cargo.toml` の変更:

```toml
[package]
version = "16.4.0"

[dependencies]
chrono = { version = "0.4", features = ["serde"] }
```

**確認:** `cargo build` でコンパイルエラーなし。

---

## Phase B — VM: List 拡充プリミティブ追加（vm.rs）

`call_builtin` の `"List"` ブランチに以下を追加:

| メソッド名 | 引数 | 実装方針 |
|---|---|---|
| `"group_by"` | `(list, fn)` | `HashMap<String, Vec<Value>>` に振り分け → `Value::Map` |
| `"sort_by"` | `(list, fn)` | `fn(elem) -> Int/Float` でキー取得、安定ソート |
| `"sort_by_desc"` | `(list, fn)` | `sort_by` の逆順 |
| `"flatten"` | `(nested_list)` | リストのリストを 1 段フラット化 |
| `"flat_map"` | `(list, fn)` | `map` → `flatten` |
| `"zip"` | `(list_a, list_b)` | 短い方の長さで止める、`Pair` を使う |
| `"zip_with"` | `(list_a, list_b, fn)` | zip + map(fn) |
| `"distinct"` | `(list)` | `Value` の `PartialEq` で重複除去（順序保持） |
| `"distinct_by"` | `(list, fn)` | キー（String）で重複除去 |
| `"chunk"` | `(list, size)` | `windows`/`chunks` で分割 |
| `"take"` | `(list, n)` | `&list[..n.min(len)]` |
| `"drop"` | `(list, n)` | `&list[n.min(len)..]` |
| `"count_where"` | `(list, pred)` | `filter` → `len` → `Value::Int` |
| `"sum_by"` | `(list, fn)` | `fn(elem) -> Float` を合計 |
| `"max_by"` | `(list, fn)` | `fn(elem) -> Float` が最大の要素を返す |
| `"min_by"` | `(list, fn)` | `fn(elem) -> Float` が最小の要素を返す |
| `"unzip"` | `(pair_list)` | `Pair` のリスト → `Pair(List<A>, List<B>)` |

**`Pair` の表現:** 既存の `Value::Variant("Pair", box Value::List([a, b]))` を使用（既存 `zip` の実装に合わせる）。
**`Map` の表現:** `Value::Map(HashMap<String, Value>)` を使用（既存 Map 実装に合わせる）。

**確認:** `cargo build` でコンパイルエラーなし。

---

## Phase C — VM: String 拡充プリミティブ追加（vm.rs）

`call_builtin` の `"String"` ブランチに以下を追加:

| メソッド名 | 引数 | 実装方針 |
|---|---|---|
| `"split"` | `(s, sep)` | `s.split(sep).collect::<Vec<_>>()` → `Value::List` |
| `"split_once"` | `(s, sep)` | `s.split_once(sep)` → `Value::Variant("Pair", ...)` |
| `"lines"` | `(s)` | `s.lines().collect()` → `Value::List` |
| `"trim"` | `(s)` | `s.trim().to_string()` |
| `"trim_start"` | `(s)` | `s.trim_start()` |
| `"trim_end"` | `(s)` | `s.trim_end()` |
| `"replace"` | `(s, old, new)` | `s.replace(old, new)` |
| `"replace_first"` | `(s, old, new)` | `s.replacen(old, new, 1)` |
| `"starts_with"` | `(s, prefix)` | `s.starts_with(prefix)` → `Value::Bool` |
| `"ends_with"` | `(s, suffix)` | `s.ends_with(suffix)` → `Value::Bool` |
| `"is_empty"` | `(s)` | `s.is_empty()` → `Value::Bool` |
| `"to_upper"` | `(s)` | `s.to_uppercase()` |
| `"to_lower"` | `(s)` | `s.to_lowercase()` |
| `"repeat"` | `(s, n)` | `s.repeat(n as usize)` |
| `"char_at"` | `(s, i)` | `s.chars().nth(i)` → `Value::Str` or `Value::Str("")` |
| `"pad_left"` | `(s, width, pad)` | `format!("{:>width$}", s, ...)` または手動ループ |
| `"pad_right"` | `(s, width, pad)` | `format!("{:<width$}", s, ...)` または手動ループ |
| `"format_int"` | `(n, width, pad)` | `format!("{:0>width$}", n, ...)` |
| `"format_float"` | `(f, digits)` | `format!("{:.digits$}", f)` |

**注意:** `split_once` は見つからない場合 `Pair(s, "")` を返す（Option 非使用）。

**確認:** `cargo build` でコンパイルエラーなし。

---

## Phase D — VM: DateTime プリミティブ追加（vm.rs）

`call_builtin` に `"DateTime"` ブランチを新規追加:

```rust
"DateTime" => match method {
    "now"          => { /* chrono::Utc::now().timestamp() */ }
    "now_unix"     => { /* chrono::Utc::now().timestamp() */ }
    "parse"        => { /* parse_from_rfc3339 → Result */ }
    "format"       => { /* chrono format */ }
    "format_iso"   => { /* to_rfc3339 */ }
    "add_days"     => { /* dt + Duration::days(n) */ }
    "add_hours"    => { /* dt + Duration::hours(n) */ }
    "diff_days"    => { /* (to - from).num_days() */ }
    "diff_seconds" => { /* (to - from).num_seconds() */ }
    "before"       => { /* a < b */ }
    "after"        => { /* a > b */ }
    "between"      => { /* from <= dt && dt <= to */ }
    _ => Err(...)
}
```

**DateTime の VM 内部表現:** `Value::Int(unix_timestamp_seconds)` として保持する。
パース・フォーマット時のみ `chrono::DateTime<Utc>` に変換。

**vm.rs の先頭に追加:**
```rust
use chrono::{DateTime, Duration, TimeZone, Utc};
```

**確認:** `cargo build` でコンパイルエラーなし。

---

## Phase E — VM: Math 拡充プリミティブ追加（vm.rs）

`call_builtin` の `"Math"` ブランチに以下を追加（既存の `sqrt` / `pow` 等がある場合は重複しないよう確認）:

| メソッド名 | 実装 |
|---|---|
| `"abs"` | Int→Int / Float→Float 分岐 |
| `"round"` | `f.round() as i64` |
| `"ceil"` | `f.ceil() as i64` |
| `"floor"` | `f.floor() as i64` |
| `"round_to"` | `let factor = 10f64.powi(digits); (f * factor).round() / factor` |
| `"min"` | Int/Float 分岐で比較 |
| `"max"` | Int/Float 分岐で比較 |
| `"clamp"` | `v.max(lo).min(hi)` |
| `"sqrt"` | `f.sqrt()` |
| `"pow"` | `base.powf(exp)` |
| `"log"` | `f.ln()` |
| `"log2"` | `f.log2()` |
| `"log10"` | `f.log10()` |

**注意:** vm.rs に既存の Math 関数があれば重複追加しない。`cargo build` 後に確認。

**確認:** `cargo build` でコンパイルエラーなし。

---

## Phase F — compiler.rs: builtin_ret_ty 更新

`fav/src/middle/compiler.rs` の `builtin_ret_ty` に以下のエントリを追加:

```
List.group_by       -> Map<String, List<T>>  (Type::Unknown で可)
List.sort_by        -> List<T>               (Type::Unknown で可)
List.sort_by_desc   -> List<T>               (Type::Unknown で可)
List.flatten        -> List<T>               (Type::Unknown で可)
List.flat_map       -> List<T>               (Type::Unknown で可)
List.zip            -> List<Pair<A,B>>       (Type::Unknown で可)
List.zip_with       -> List<C>               (Type::Unknown で可)
List.distinct       -> List<T>               (Type::Unknown で可)
List.distinct_by    -> List<T>               (Type::Unknown で可)
List.chunk          -> List<List<T>>         (Type::Unknown で可)
List.take           -> List<T>               (Type::Unknown で可)
List.drop           -> List<T>               (Type::Unknown で可)
List.count_where    -> Int
List.sum_by         -> Float
List.max_by         -> T                     (Type::Unknown で可)
List.min_by         -> T                     (Type::Unknown で可)
List.unzip          -> Pair<List<A>,List<B>> (Type::Unknown で可)

String.split        -> List<String>
String.split_once   -> Pair<String, String>  (Type::Unknown で可)
String.lines        -> List<String>
String.trim         -> String
String.trim_start   -> String
String.trim_end     -> String
String.replace      -> String
String.replace_first-> String
String.starts_with  -> Bool
String.ends_with    -> Bool
String.is_empty     -> Bool
String.to_upper     -> String
String.to_lower     -> String
String.repeat       -> String
String.char_at      -> String
String.pad_left     -> String
String.pad_right    -> String
String.format_int   -> String
String.format_float -> String

DateTime.now        -> Int  (DateTime = Int timestamp)
DateTime.now_unix   -> Int
DateTime.parse      -> Result<Int, String>
DateTime.format     -> String
DateTime.format_iso -> String
DateTime.add_days   -> Int
DateTime.add_hours  -> Int
DateTime.diff_days  -> Int
DateTime.diff_seconds -> Int
DateTime.before     -> Bool
DateTime.after      -> Bool
DateTime.between    -> Bool

Math.abs            -> Float  (Type::Unknown で可、Int/Float 両対応)
Math.round          -> Int
Math.ceil           -> Int
Math.floor          -> Int
Math.round_to       -> Float
Math.min            -> Float  (Type::Unknown で可)
Math.max            -> Float  (Type::Unknown で可)
Math.clamp          -> Float  (Type::Unknown で可)
Math.sqrt           -> Float
Math.pow            -> Float
Math.log            -> Float
Math.log2           -> Float
Math.log10          -> Float
```

**注意:** `builtin_ret_ty` が String → Type のマッチで実装されている場合、`("DateTime", method)` のブランチを追加する。

**確認:** `cargo build` でコンパイルエラーなし。

---

## Phase G — checker.rs: builtin_ret_ty 更新

`fav/src/middle/checker.rs` の `builtin_ret_ty`（または `ns_to_effect` / `datetime_fn` 等）に対応する型を追加。

実装パターン:
1. `checker.rs` の `builtin_ret_ty` / `check_apply` を確認
2. `"DateTime"` ブランチを追加（`"List"` / `"String"` / `"Math"` に倣う）
3. 各関数の戻り型を `"Int"` / `"String"` / `"Bool"` / `"Unknown"` で登録

**確認:** `cargo build` でコンパイルエラーなし。

---

## Phase H — テスト追加（driver.rs）

`fav/src/driver.rs` に `v164000_tests` モジュールを追加:

```rust
#[cfg(test)]
mod v164000_tests {
    // version_is_16_4_0
    // list_group_by_works    -- group_by でグループ数・要素確認
    // list_chunk_works       -- chunk(rows, 3) で正しく分割
    // string_split_works     -- split("a,b,c", ",") → 3 要素
    // datetime_now_unix_works -- now_unix() > 0
}
```

**テスト実行:** `cargo test v164000` → 5/5 PASS 確認。

---

## Phase I — サイトドキュメント

以下の新規ファイルを作成:

- `site/content/docs/stdlib/list.mdx` — List 全関数リファレンス
- `site/content/docs/stdlib/string.mdx` — String 全関数リファレンス
- `site/content/docs/stdlib/datetime.mdx` — DateTime モジュールリファレンス
- `site/content/docs/stdlib/math.mdx` — Math モジュールリファレンス

---

## 実装の注意点

1. **既存 Math/String 関数との重複確認**: vm.rs にすでに `Math.sqrt` / `String.length` 等が実装されている可能性がある。追加前に Grep で確認し、重複追加しない。
2. **`List.group_by` のキー型**: String のみ対応（ジェネリックキーは v18.x）。Int キーで呼ばれた場合は `to_string()` 変換するか、エラーとする（一貫性を取る）。
3. **`Value::Map` の存在確認**: 既存実装に `Value::Map` があるかを確認。なければ追加が必要。
4. **chrono の use**: vm.rs 先頭に `use chrono::{Duration, Utc};` を追加。`chrono::offset::TimeZone` トレイトも必要になる場合がある。
5. **`cargo test` 全件確認**: 追加後に full test run を実施してリグレッションなしを確認。
