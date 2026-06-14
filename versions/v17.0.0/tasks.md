# v17.0.0 Tasks — Language Ergonomics マイルストーン宣言

Date: 2026-06-14
Branch: master

---

## Phase A — Cargo バージョン更新

- [ ] A-1: `fav/Cargo.toml` の `version` を `"17.0.0"` に変更
- [ ] A-2: `cargo build` → コンパイルエラーなし確認

---

## Phase B — CHANGELOG.md 更新

- [ ] B-1: `CHANGELOG.md` に `[v16.1.0]` エントリ追加
  - エラーメッセージ品質向上（rustc スタイル Span 表示、typo ヒント、`E0xxx` エラーコード URL）
- [ ] B-2: `[v16.2.0]` エントリ追加
  - f-string 文字列補間（`f"Hello, {name}!"`、`f"""..."""` 三重クォート対応）
- [ ] B-3: `[v16.3.0]` エントリ追加
  - レコード更新構文（`{ ...base, field: val }`、`MergeRecord` VM opcode）
- [ ] B-4: `[v16.4.0]` エントリ追加
  - stdlib 拡充: `List` 9 関数 / `String` 4 関数 / `DateTime` 12 関数 / `Math` 4 関数
- [ ] B-5: `[v16.5.0]` エントリ追加
  - 型エイリアス（`alias Email = String`、ジェネリクスエイリアス対応）
- [ ] B-6: `[v16.6.0]` エントリ追加
  - Namespace Alias（`use String as S`、`use List as L`）
- [ ] B-7: `[v16.7.0]` エントリ追加
  - fav test 成熟（`assert_eq` / `assert_approx_eq` / `assert_contains` / `assert_length` /
    `assert_str_contains` / `assert_str_starts_with` / `assert_err_eq` / `assert_snapshot` /
    `test_group` / `--update-snapshots`）
- [ ] B-8: `[v16.8.0]` エントリ追加
  - tap / inspect パイプライン演算子（`|> tap(fn)` / `|> inspect` / `--no-tap` フラグ）

---

## Phase C — README.md 更新

- [ ] C-1: 「現在の状態」の記述を v16.0.0 → v17.0.0 に更新
  - Production Multi-Cloud（v16.0.0）→ Language Ergonomics（v17.0.0）
- [ ] C-2: 機能ハイライトに v16.x 機能を追記
  - f-string 補間（`f"Hello, {name}!"`）
  - record spread / update（`{ ...base, field: val }`）
  - stdlib 拡充（DateTime / List.group_by 等）
  - 型エイリアス・Namespace Alias
  - fav test 成熟（assert_eq / test_group / snapshot）
  - tap / inspect パイプライン演算子
- [ ] C-3: バージョン履歴表に v16.1.0〜v17.0.0 のエントリを追加

---

## Phase D — stdlib ドキュメント更新（site/content/docs/stdlib/）

- [ ] D-1: `site/content/docs/stdlib/list.mdx` に v16.4.0 追加関数を追記
  - `List.sort_by` / `List.sort_by_desc`
  - `List.distinct` / `List.distinct_by`
  - `List.count_where`
  - `List.sum_by` / `List.max_by` / `List.min_by`
  - `List.unzip`
- [ ] D-2: `site/content/docs/stdlib/string.mdx` に v16.4.0 追加関数を追記
  - `String.split_once` / `String.replace_first`
  - `String.format_int` / `String.format_float`
- [ ] D-3: `site/content/docs/stdlib/datetime.mdx` を v16.4.0 内容で更新
  - `DateTime.now` / `DateTime.parse` / `DateTime.format`
  - `DateTime.add_days` / `DateTime.add_hours` / `DateTime.diff_days`
  - `DateTime.year` / `DateTime.month` / `DateTime.day` / `DateTime.weekday`
  - `DateTime.timestamp` / `DateTime.from_timestamp` / `DateTime.format_relative`
- [ ] D-4: `site/content/docs/stdlib/math.mdx` に v16.4.0 追加関数を追記
  - `Math.round_to` / `Math.log` / `Math.log2` / `Math.log10`

---

## Phase E — language ドキュメント最終確認（site/content/docs/language/）

- [ ] E-1: `string-interpolation.mdx` を読んで確認（f-string triple-quote / 式埋め込み）。不足があれば補足追記
- [ ] E-2: `record-update.mdx` を読んで確認（record spread / update 構文例）。不足があれば補足追記
- [ ] E-3: `testing.mdx` を読んで確認（v16.7.0 内容が反映済みか）。不足があれば補足追記
- [ ] E-4: `modules.mdx` を読んで確認（namespace alias が反映済みか）。不足があれば補足追記
- [ ] E-5: `pipeline.mdx` を読んで確認（tap / inspect が反映済みか）。不足があれば補足追記
- [ ] E-6: `type-alias.mdx` を読んで確認（alias キーワードが反映済みか）。不足があれば補足追記

---

## Phase F — テスト追加（v170000_tests）

- [ ] F-1: `fav/src/driver.rs` に `v170000_tests` モジュール追加
- [ ] F-2: `version_is_17_0_0` — `Cargo.toml` に `"17.0.0"` が含まれる
- [ ] F-3: `changelog_has_v16_entries` — `CHANGELOG.md` に `v16.1` 〜 `v16.8` の全エントリが含まれる
- [ ] F-4: `readme_mentions_fstring` — `README.md` に `f"` への言及がある
- [ ] F-5: `readme_mentions_record_spread` — `README.md` に record spread への言及がある
- [ ] F-6: `stdlib_datetime_doc_exists` — `site/content/docs/stdlib/datetime.mdx` が存在し `DateTime.now` が記載されている
- [ ] F-7: `cargo test v170000` → 5/5 PASS 確認

---

## Phase G — 最終確認 + コミット

- [ ] G-1: `cargo test v170000` → 5/5 PASS 最終確認
- [ ] G-2: `cargo test` → 全件 PASS（リグレッションなし）確認（version check 旧版は除外）
- [ ] G-3: コミット

---

## 完了条件

| 確認項目 | 状態 |
|---|---|
| `Cargo.toml version == "17.0.0"` | [ ] |
| CHANGELOG.md に v16.1.0〜v16.8.0 の全エントリが存在する | [ ] |
| README.md が v17.0.0 を「現在の状態」として記載している | [ ] |
| README.md に f-string・record spread への言及がある | [ ] |
| `site/content/docs/stdlib/datetime.mdx` に `DateTime.now` が記載されている | [ ] |
| `cargo test v170000` 全テストパス（5/5） | [ ] |
| `cargo test` 全件パス（リグレッションなし） | [ ] |

---

## 技術メモ

- **コード変更なし**: v17.0.0 はドキュメント整備のみ。Rust ソースへの変更は Cargo.toml バージョンと driver.rs テストモジュール追加のみ。
- **`include_str!` のパス**: `driver.rs` 内テストから `Cargo.toml` は `"../Cargo.toml"`、`CHANGELOG.md` は `"../../CHANGELOG.md"`、`README.md` は `"../../README.md"`、`datetime.mdx` は `"../../site/content/docs/stdlib/datetime.mdx"` 相対パス。
- **既存 stdlib docs**: `site/content/docs/stdlib/` には datetime / list / math / string が既に存在する。新規作成ではなく更新作業。
- **既存 language docs**: string-interpolation / record-update / testing / modules / pipeline / type-alias はすでに存在する。最終確認・補足追記のみ。
- **version check 旧版テスト**: `cargo test` 全件実行時に v16.x の version_is_xxx テストが FAIL する（想定内）。v170000_tests の 5/5 と 1611+ pass が確認できれば OK。
