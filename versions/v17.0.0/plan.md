# v17.0.0 Plan — Language Ergonomics マイルストーン宣言

Date: 2026-06-14

---

## Phase A — Cargo バージョン更新

`fav/Cargo.toml` の `version` を `"17.0.0"` に更新。
`cargo build` → コンパイルエラーなし確認。

---

## Phase B — CHANGELOG.md 更新

`CHANGELOG.md` に v16.1.0〜v16.8.0 のエントリを追加（v16.0.0 エントリの直後）。

各エントリ形式:
```
## [v16.x.0] — 2026-06-14 — <タイトル>

### 追加
- ...

### 変更
- ...
```

対象バージョン:
- v16.1.0: エラーメッセージ品質向上（rustc スタイル、Span、typo ヒント）
- v16.2.0: f-string 文字列補間（`f"Hello, {name}!"`、`f"""..."""` 三重クォート）
- v16.3.0: レコード更新構文（`{ ...base, field: val }`、MergeRecord opcode）
- v16.4.0: 標準ライブラリ拡充（List 9関数 / String 4関数 / DateTime 12関数 / Math 4関数）
- v16.5.0: 型エイリアス（`alias Email = String`、ジェネリクス `alias Result2<T> = Result<T, String>`）
- v16.6.0: Namespace Alias（`use String as S`、`use List as L`）
- v16.7.0: fav test 成熟（`assert_eq` / `test_group` / `assert_snapshot` / `--update-snapshots`）
- v16.8.0: tap / inspect パイプライン演算子（`|> tap(fn)` / `|> inspect` / `--no-tap`）

---

## Phase C — README.md 更新

1. 「現在の状態」記述を v16.0.0 → v17.0.0 に更新
2. Language Ergonomics 達成を「機能ハイライト」として追記:
   - f-string / record spread / stdlib 拡充
   - alias / namespace alias
   - assert_eq / test_group / snapshot
   - tap / inspect
3. バージョン履歴表に v16.1.0〜v17.0.0 エントリ追加

---

## Phase D — stdlib ドキュメント更新（site/content/docs/stdlib/）

### D-1: list.mdx 更新

v16.4.0 で追加した 9 関数を追記:
- `List.sort_by` / `List.sort_by_desc`
- `List.distinct` / `List.distinct_by`
- `List.count_where`
- `List.sum_by` / `List.max_by` / `List.min_by`
- `List.unzip`

### D-2: string.mdx 更新

v16.4.0 で追加した 4 関数を追記:
- `String.split_once`
- `String.replace_first`
- `String.format_int`
- `String.format_float`

### D-3: datetime.mdx 更新

v16.4.0 の DateTime モジュール全体を確認・更新:
- `DateTime.now` / `DateTime.parse` / `DateTime.format`
- `DateTime.add_days` / `DateTime.add_hours`
- `DateTime.diff_days`
- `DateTime.year` / `DateTime.month` / `DateTime.day` / `DateTime.weekday`
- `DateTime.timestamp` / `DateTime.from_timestamp`
- `DateTime.format_relative`

### D-4: math.mdx 更新

v16.4.0 で追加した 4 関数を追記:
- `Math.round_to`
- `Math.log` / `Math.log2` / `Math.log10`

---

## Phase E — language ドキュメント最終確認（site/content/docs/language/）

以下のファイルを読み、v16.x の内容が最新になっているか確認。
不足があれば補足追記する。

- `string-interpolation.mdx` — f-string の triple-quote、式埋め込み例
- `record-update.mdx` — record spread / update 構文
- `testing.mdx` — assert_eq / test_group / snapshot (v16.7.0 で更新済みのはず)
- `modules.mdx` — namespace alias (v16.6.0 で作成済みのはず)
- `pipeline.mdx` — tap / inspect (v16.8.0 で更新済みのはず)
- `type-alias.mdx` — alias キーワード (v16.5.0 で作成済みのはず)

---

## Phase F — テスト追加（v170000_tests）

`fav/src/driver.rs` に `v170000_tests` モジュール追加（5件）:

1. `version_is_17_0_0` — `Cargo.toml` に `"17.0.0"` が含まれる
2. `changelog_has_v16_entries` — CHANGELOG に v16.1〜v16.8 のエントリが含まれる
3. `readme_mentions_fstring` — README に `f"` への言及がある
4. `readme_mentions_record_spread` — README に `...` record spread への言及がある
5. `stdlib_datetime_doc_exists` — `datetime.mdx` が存在し `DateTime.now` が記載されている

`cargo test v170000` → 5/5 PASS 確認。

---

## Phase G — 最終確認 + コミット

- `cargo test v170000` → 5/5 PASS 最終確認
- `cargo test` → 全件 PASS（リグレッションなし）
- コミット: `feat: v17.0.0 — Language Ergonomics マイルストーン宣言`

---

## 依存関係

```
A（Cargo）→ F（テスト: version check）
B（CHANGELOG）→ F（テスト: changelog check）
C（README）→ F（テスト: readme checks）
D（stdlib docs）→ F（テスト: datetime doc check）
E（lang docs）— 独立（テストに直接依存しない）
F → G
```

Phase A〜E は順不同で並行実施可能。Phase F は A〜E の完了後。
