# v17.0.0 Spec — Language Ergonomics マイルストーン宣言

Date: 2026-06-14

---

## 概要

v16.x シリーズ（v16.1.0〜v16.8.0）の集大成として「書きたくなる言語」への転換を宣言する。
コード変更はなく、ドキュメント・バージョン番号・CHANGELOG・README の整備が主な作業。

---

## 実装内容

### 1. Cargo.toml バージョン更新

`fav/Cargo.toml` の `version` を `"17.0.0"` に更新。

---

### 2. CHANGELOG.md 更新

`CHANGELOG.md` に v16.1.0〜v16.8.0 の全エントリを追加。

現在のエントリ:
- `[v16.0.0]` — Production Multi-Cloud マイルストーン宣言（既存）

追加するエントリ（v16.1.0〜v16.8.0）:

| バージョン | 内容 |
|---|---|
| v16.1.0 | エラーメッセージ品質向上（rustc スタイル、typo ヒント、行番号表示）|
| v16.2.0 | f-string 文字列補間（`f"Hello, {name}!"`）|
| v16.3.0 | レコード更新構文（`{ ...base, field: val }`）|
| v16.4.0 | 標準ライブラリ拡充（List / String / DateTime / Math）|
| v16.5.0 | 型エイリアス（`alias Email = String`）|
| v16.6.0 | Namespace Alias（`use String as S`）|
| v16.7.0 | fav test 成熟（`assert_eq` / `test_group` / `assert_snapshot`）|
| v16.8.0 | tap / inspect パイプライン演算子（`\|> tap(fn)` / `--no-tap`）|

---

### 3. README.md 更新

- 「現在の状態」セクション: v16.0.0 → v17.0.0
- Language Ergonomics 達成を記載:
  - f-string 補間
  - record spread / update
  - stdlib 拡充（DateTime / List.group_by 等）
  - alias / namespace alias
  - assert_eq / test_group / snapshot
  - tap / inspect パイプライン演算子
- バージョン履歴表に v16.1.0〜v17.0.0 エントリ追加

---

### 4. サイトドキュメント更新・確認

#### 更新対象（既存ファイルの最終確認・必要に応じて補足）

- `site/content/docs/language/string-interpolation.mdx` — f-string の最終形を確認・必要なら補足
- `site/content/docs/language/record-update.mdx` — record spread の最終形を確認・必要なら補足
- `site/content/docs/language/testing.mdx` — v16.7.0 内容が反映済みか確認（反映済みのはず）
- `site/content/docs/language/modules.mdx` — v16.6.0 内容が反映済みか確認（反映済みのはず）

#### stdlib ドキュメント（既存ファイルを v16.4.0 追加分で更新）

- `site/content/docs/stdlib/list.mdx` — v16.4.0 追加関数を記載
  （sort_by / sort_by_desc / distinct / distinct_by / count_where / sum_by / max_by / min_by / unzip）
- `site/content/docs/stdlib/string.mdx` — v16.4.0 追加関数を記載
  （split_once / replace_first / format_int / format_float）
- `site/content/docs/stdlib/datetime.mdx` — v16.4.0 追加を確認・更新
  （now / parse / format / add_days / add_hours / diff_days / year / month / day / weekday / timestamp / from_timestamp / format_relative）
- `site/content/docs/stdlib/math.mdx` — v16.4.0 追加を確認・更新
  （round_to / log / log2 / log10）

---

### 5. テスト（v170000_tests、5件）

| テスト名 | 内容 |
|---|---|
| `version_is_17_0_0` | `Cargo.toml` に `"17.0.0"` が含まれる |
| `changelog_has_v16_entries` | `CHANGELOG.md` に `v16.1` 〜 `v16.8` のエントリが含まれる |
| `readme_mentions_fstring` | `README.md` に f-string (`f"`) への言及がある |
| `readme_mentions_record_spread` | `README.md` に record spread (`...`) への言及がある |
| `stdlib_datetime_doc_exists` | `site/content/docs/stdlib/datetime.mdx` が存在し、`DateTime.now` が記載されている |

---

## 完了条件

| 確認項目 | 状態 |
|---|---|
| `Cargo.toml version == "17.0.0"` | [ ] |
| CHANGELOG.md に v16.1.0〜v16.8.0 の全エントリが存在する | [ ] |
| README.md が v17.0.0 を「現在の状態」として記載している | [ ] |
| README.md に f-string・record spread への言及がある | [ ] |
| stdlib ドキュメント（list / string / datetime / math）が v16.4.0 内容を含む | [ ] |
| `cargo test v170000` 全テストパス（5/5） | [ ] |
| `cargo test` 全件パス（リグレッションなし） | [ ] |
