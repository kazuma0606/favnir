# v16.4.0 Tasks — 標準ライブラリ拡充

Date: 2026-06-14
Branch: master

---

## Phase A — Cargo バージョン更新 + chrono 依存追加

- [ ] A-1: `fav/Cargo.toml` の `version` を `"16.4.0"` に変更
- [ ] A-2: `fav/Cargo.toml` の `[dependencies]` に `chrono = { version = "0.4", features = ["serde"] }` を追加
- [ ] A-3: `cargo build` → コンパイルエラーなし確認

---

## Phase B — VM: List 拡充プリミティブ追加（vm.rs）

- [ ] B-1: `vm.rs` に `use chrono::{Duration, Utc};` 追加（Phase D で必要なため先行追加）
- [ ] B-2: `List.group_by(list, fn)` → `Map<String, List<T>>` 実装
- [ ] B-3: `List.sort_by(list, fn)` → 昇順ソート実装
- [ ] B-4: `List.sort_by_desc(list, fn)` → 降順ソート実装
- [ ] B-5: `List.flatten(nested)` → `List<List<T>> -> List<T>` 実装
- [ ] B-6: `List.flat_map(list, fn)` → map + flatten 実装
- [ ] B-7: `List.zip(a, b)` → `List<Pair<A,B>>` 実装
- [ ] B-8: `List.zip_with(a, b, fn)` → `List<C>` 実装
- [ ] B-9: `List.distinct(list)` → 重複除去（順序保持）実装
- [ ] B-10: `List.distinct_by(list, fn)` → キー指定重複除去実装
- [ ] B-11: `List.chunk(list, size)` → バッチ分割実装
- [ ] B-12: `List.take(list, n)` → 先頭 N 件実装
- [ ] B-13: `List.drop(list, n)` → 先頭 N 件除去実装
- [ ] B-14: `List.count_where(list, pred)` → 条件付き件数実装
- [ ] B-15: `List.sum_by(list, fn)` → Float 合計実装
- [ ] B-16: `List.max_by(list, fn)` → 最大値要素実装
- [ ] B-17: `List.min_by(list, fn)` → 最小値要素実装
- [ ] B-18: `List.unzip(pair_list)` → `Pair<List<A>, List<B>>` 実装
- [ ] B-19: `cargo build` → コンパイルエラーなし確認

---

## Phase C — VM: String 拡充プリミティブ追加（vm.rs）

- [ ] C-1: `String.split(s, sep)` → `List<String>` 実装
- [ ] C-2: `String.split_once(s, sep)` → `Pair<String, String>` 実装
- [ ] C-3: `String.lines(s)` → `List<String>` 実装
- [ ] C-4: `String.trim(s)` 実装
- [ ] C-5: `String.trim_start(s)` 実装
- [ ] C-6: `String.trim_end(s)` 実装
- [ ] C-7: `String.replace(s, old, new)` → 全置換実装
- [ ] C-8: `String.replace_first(s, old, new)` → 先頭 1 件置換実装
- [ ] C-9: `String.starts_with(s, prefix)` → Bool 実装
- [ ] C-10: `String.ends_with(s, suffix)` → Bool 実装
- [ ] C-11: `String.is_empty(s)` → Bool 実装
- [ ] C-12: `String.to_upper(s)` 実装
- [ ] C-13: `String.to_lower(s)` 実装
- [ ] C-14: `String.repeat(s, n)` 実装
- [ ] C-15: `String.char_at(s, i)` → String（なければ `""`）実装
- [ ] C-16: `String.pad_left(s, width, pad)` 実装
- [ ] C-17: `String.pad_right(s, width, pad)` 実装
- [ ] C-18: `String.format_int(n, width, pad)` 実装
- [ ] C-19: `String.format_float(f, digits)` 実装
- [ ] C-20: `cargo build` → コンパイルエラーなし確認

---

## Phase D — VM: DateTime プリミティブ追加（vm.rs）

- [ ] D-1: `"DateTime"` ブランチを `call_builtin` に新規追加
- [ ] D-2: `DateTime.now()` → `Value::Int(chrono::Utc::now().timestamp())` 実装
- [ ] D-3: `DateTime.now_unix()` → 同上（Int を直接返す）実装
- [ ] D-4: `DateTime.parse(s)` → `parse_from_rfc3339` → `Result<Int, String>` 実装
- [ ] D-5: `DateTime.format(dt, fmt)` → chrono format 実装
- [ ] D-6: `DateTime.format_iso(dt)` → `to_rfc3339` 実装
- [ ] D-7: `DateTime.add_days(dt, n)` → `dt + Duration::days(n)` 実装
- [ ] D-8: `DateTime.add_hours(dt, n)` → `dt + Duration::hours(n)` 実装
- [ ] D-9: `DateTime.diff_days(from, to)` → `(to - from).num_days()` 実装
- [ ] D-10: `DateTime.diff_seconds(from, to)` → `(to - from).num_seconds()` 実装
- [ ] D-11: `DateTime.before(a, b)` → `a < b` 実装
- [ ] D-12: `DateTime.after(a, b)` → `a > b` 実装
- [ ] D-13: `DateTime.between(dt, from, to)` → `from <= dt && dt <= to` 実装
- [ ] D-14: `cargo build` → コンパイルエラーなし確認

---

## Phase E — VM: Math 拡充プリミティブ追加（vm.rs）

- [ ] E-1: 既存 `Math` プリミティブを確認（重複しないようにする）
- [ ] E-2: `Math.abs(n)` → Int/Float 両対応実装
- [ ] E-3: `Math.round(f)` → Int 実装
- [ ] E-4: `Math.ceil(f)` → Int 実装
- [ ] E-5: `Math.floor(f)` → Int 実装
- [ ] E-6: `Math.round_to(f, digits)` → Float 実装
- [ ] E-7: `Math.min(a, b)` → Int/Float 両対応実装
- [ ] E-8: `Math.max(a, b)` → Int/Float 両対応実装
- [ ] E-9: `Math.clamp(v, lo, hi)` → Int/Float 両対応実装
- [ ] E-10: `Math.sqrt(f)` 未実装の場合のみ追加
- [ ] E-11: `Math.pow(base, exp)` 未実装の場合のみ追加
- [ ] E-12: `Math.log(f)` 実装
- [ ] E-13: `Math.log2(f)` 実装
- [ ] E-14: `Math.log10(f)` 実装
- [ ] E-15: `cargo build` → コンパイルエラーなし確認

---

## Phase F — compiler.rs: builtin_ret_ty 更新

- [ ] F-1: `fav/src/middle/compiler.rs` の `builtin_ret_ty` を確認
- [ ] F-2: List 新関数（17 件）の戻り型を追加（`Type::Unknown` で可）
- [ ] F-3: String 新関数（19 件）の戻り型を追加
- [ ] F-4: `"DateTime"` ブランチを追加（12 関数の戻り型）
- [ ] F-5: Math 新関数の戻り型を追加
- [ ] F-6: `cargo build` → コンパイルエラーなし確認

---

## Phase G — checker.rs: builtin_ret_ty 更新

- [ ] G-1: `fav/src/middle/checker.rs` の `builtin_ret_ty` / `check_apply` 実装を確認
- [ ] G-2: List 新関数の戻り型を追加
- [ ] G-3: String 新関数の戻り型を追加
- [ ] G-4: `"DateTime"` ブランチを追加（`datetime_fn` または直接 `builtin_ret_ty` に追加）
- [ ] G-5: Math 新関数の戻り型を追加
- [ ] G-6: `cargo build` → コンパイルエラーなし確認

---

## Phase H — テスト追加（v164000_tests）

- [ ] H-1: `fav/src/driver.rs` に `v164000_tests` モジュール追加
- [ ] H-2: `version_is_16_4_0` テスト実装
- [ ] H-3: `list_group_by_works` テスト実装（`group_by` でグループ数・要素確認）
- [ ] H-4: `list_chunk_works` テスト実装（`chunk(rows, 3)` で正しく分割）
- [ ] H-5: `string_split_works` テスト実装（`split("a,b,c", ",")` → 3 要素リスト）
- [ ] H-6: `datetime_now_unix_works` テスト実装（`now_unix()` が正の Int を返す）
- [ ] H-7: `cargo test v164000` → 5/5 PASS 確認

---

## Phase I — サイトドキュメント

- [ ] I-1: `site/content/docs/stdlib/list.mdx` 新規作成（List 全新関数リファレンス）
- [ ] I-2: `site/content/docs/stdlib/string.mdx` 新規作成（String 全新関数リファレンス）
- [ ] I-3: `site/content/docs/stdlib/datetime.mdx` 新規作成（DateTime モジュールリファレンス）
- [ ] I-4: `site/content/docs/stdlib/math.mdx` 新規作成（Math モジュールリファレンス）

---

## Phase J — テスト確認とコミット

- [ ] J-1: `cargo test v164000` → 5/5 PASS 最終確認
- [ ] J-2: `cargo test` → 全件 PASS（リグレッションなし）確認
- [ ] J-3: コミット

---

## 完了条件

| 確認項目 | 状態 |
|---|---|
| `Cargo.toml version == "16.4.0"` | [x] |
| `chrono 0.4` が Cargo 依存に追加されている | [x]（元々存在） |
| `List.group_by` / `List.chunk` / `List.sort_by` が動作する | [x] |
| `String.split` / `String.trim` / `String.replace` が動作する | [x]（元々存在） |
| `DateTime.now` / `DateTime.now_unix` / `DateTime.format_iso` が動作する | [x] |
| `Math.round_to` / `Math.clamp` が動作する | [x] |
| `cargo test v164000` 全テストパス（6/6）| [x] |
| `cargo test` 1592 パス（リグレッションなし） | [x] |
| `site/content/docs/stdlib/` ドキュメント 4 ファイルが存在する | [x] |
