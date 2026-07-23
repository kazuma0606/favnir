# Roadmap v47.1.0 〜 v48.0.0 — Standard Library 2.0

Date: 2026-07-15
Status: 計画中（v47.0 完了後に開始）

---

## 前提

- 直前完了: v47.0.0「Developer Experience」（v47.0 宣言後、tests ≥ 3011）
- マスターロードマップ: `roadmap-v45.1-v50.0.md`
- 本文書はマスターの v48.0 スプリント部分の詳細版

---

## 目標

`List` / `String` / `Float` / `Option` / `Result` / `Map` の主要操作を追加し、
**外部ライブラリなしに実務的なデータ変換が書ける標準ライブラリを整備する**。

---

## バージョン計画

### v47.1.0 — `List.zip` / `List.zip_with` / `List.chunk`

```favnir
bind pairs   <- List.zip(names, scores)          // [(String, Int)]
bind batched <- data |> List.chunk(100)          // List<List<Row>>
bind totals  <- List.zip_with(|a, b| a + b, xs, ys)
```

`List.zip` / `List.zip_with` / `List.chunk` を VM primitive として追加。
`vm.rs` に対応する `Op::ListZip` / `Op::ListZipWith` / `Op::ListChunk` を実装。

**完了条件**: Rust テスト 2 件（実績推定 3013 tests passed, 0 failed）
- `list_zip_pairs`
- `list_chunk_batches`

---

### v47.2.0 — `List.flat_map` / `List.group_by` / `List.dedupe`

```favnir
bind expanded  <- orders |> List.flat_map(|o| o.items)
bind by_region <- orders |> List.group_by(|o| o.region)  // Map<String, List<Order>>
bind unique    <- tags   |> List.dedupe
```

`List.flat_map` / `List.group_by` / `List.dedupe` を VM primitive として追加。
`group_by` の戻り型は `Map<K, List<V>>`（checker.rs 型チェック対応）。

**完了条件**: Rust テスト 3 件（実績推定 3021 tests passed, 0 failed）
- `list_flat_map`
- `list_group_by`
- `list_dedupe`

---

### v47.3.0 — `List.scan` / `List.take_while` / `List.drop_while`

```favnir
bind running_total <- prices |> List.scan(0, |acc, p| acc + p)
bind valid         <- rows   |> List.take_while(|r| r.status == "ok")
bind rest          <- rows   |> List.drop_while(|r| r.status == "ok")
```

`List.scan(list, init, f)` は累積値のリストを返す（初期値を含む）。
`take_while` / `drop_while` は述語が最初に偽になるまで要素を取得・スキップ。

**完了条件**: Rust テスト 3 件（実績推定 3024 tests passed, 0 failed）
- `list_scan_cumulative`
- `list_take_while`
- `list_drop_while`

---

### v47.4.0 — `String` 拡充

```favnir
bind padded  <- "42"      |> String.pad_left(6, "0")   // "000042"
bind trimmed <- "  hello  " |> String.trim_start
bind rep     <- "ab"      |> String.repeat(3)           // "ababab"
```

`String.trim_start` / `String.trim_end` / `String.repeat(n)` /
`String.pad_left(n, ch)` / `String.pad_right(n, ch)` を追加。
`vm.rs` に対応する 5 primitive を実装。

**完了条件**: Rust テスト 3 件（実績推定 3027 tests passed, 0 failed）
- `string_pad_left`
- `string_trim_start`
- `string_repeat`

---

### v47.5.0 — `Float` / `Int` 拡充

```favnir
bind rounded <- 3.14159 |> Float.round(2)      // 3.14
bind clamped <- score   |> Float.clamp(0.0, 100.0)
bind digits  <- 255     |> Int.to_hex           // "ff"
```

`Float.round(n)` / `Float.clamp(min, max)` /
`Int.to_hex` / `Int.abs` / `Float.abs` を追加。
checker.rs で各関数の型シグネチャを登録。

**完了条件**: Rust テスト 3 件（実績推定 3030 tests passed, 0 failed）
- `float_round`
- `float_clamp`
- `int_to_hex`

---

### v47.6.0 — `Option` 拡充

```favnir
bind doubled <- maybe_int  |> Option.map(|n| n * 2)
bind value   <- maybe_str  |> Option.unwrap_or("default")
bind chained <- maybe_user |> Option.and_then(|u| lookup(u.id))
```

`Option.map` / `Option.unwrap_or` / `Option.and_then` /
`Option.is_some` / `Option.is_none` を VM primitive として追加。
checker.rs で `Option<T>` のジェネリック型推論を確認。

**完了条件**: Rust テスト 3 件（実績推定 3033 tests passed, 0 failed）
- `option_map`
- `option_unwrap_or`
- `option_and_then`

---

### v47.7.0 — `Result` 拡充

```favnir
bind doubled <- result_int |> Result.map(|n| n * 2)
bind handled <- result_val |> Result.map_err(|e| "wrapped: " ++ e)
bind chained <- parse_int(s) |> Result.and_then(|n| validate(n))
```

`Result.map` / `Result.map_err` / `Result.and_then` /
`Result.is_ok` / `Result.is_err` を VM primitive として追加。
`Result<T, E>` の型変換が checker.rs で正しく追えることを確認。

**完了条件**: Rust テスト 3 件（実績推定 3036 tests passed, 0 failed）
- `result_map`
- `result_map_err`
- `result_and_then`

---

### v47.8.0 — `Map` 拡充

```favnir
bind merged   <- Map.merge(defaults, overrides)
bind filtered <- config |> Map.filter_values(|v| v != "")
bind mapped   <- scores |> Map.map_values(|v| v * 2)
```

`Map.merge` / `Map.filter_values` / `Map.map_values` /
`Map.keys` / `Map.values` を VM primitive として追加。
`Map.merge` の重複キー処理（右辺優先）を明確化。

**完了条件**: Rust テスト 3 件（実績推定 3039 tests passed, 0 failed）
- `map_merge`
- `map_filter_values`
- `map_map_values`

---

### v47.9.0 — stdlib ドキュメント + v48.0 前調整

`site/content/docs/stdlib/` 以下に各型の MDX ドキュメントを追加・更新。
List / String / Float / Option / Result / Map の全追加関数を網羅。
cookbook サンプル更新。v48.0 前コードフリーズ。

**完了条件**: Rust テスト 2 件（実績推定 3041 tests passed, 0 failed）
- `stdlib_v2_doc_exists`
- `stdlib_v2_overview_exists`

---

### v48.0.0 — Standard Library 2.0 宣言 ★クリーンアップ

**宣言文**:

> 「List・String・Float・Option・Result・Map の主要操作が揃い、
>  外部ライブラリなしに実務的なデータ変換が書ける。
>
>  これが Favnir v48.0 — Standard Library 2.0 の姿である。」

**完了条件**:
- v47.1〜v47.9 の全機能が動作する
- `cargo test` 全通過（failures=0 かつテスト数 ≥ **3045**）
- `v48000_tests` 4 件 pass:
  - `cargo_toml_version_is_48_0_0`
  - `changelog_has_v48_0_0`
  - `milestone_has_stdlib_v2` — MILESTONE.md に `"Standard Library 2.0"` が含まれる
  - `readme_mentions_stdlib_v2`
- `MILESTONE.md` に `"Standard Library 2.0"` が含まれる
- `★クリーンアップ`（`cargo clean`）完了

---

## 参考リンク

- マスターロードマップ: `versions/roadmap/roadmap-v45.1-v50.0.md`
- 前サブスプリント（アクティブ）: `versions/roadmap/roadmap-v46.1-v47.0.md`
- 次サブスプリント（v48.0 完了後に開始）: `versions/roadmap/roadmap-v48.1-v49.0.md`
- 達成宣言: `MILESTONE.md`
