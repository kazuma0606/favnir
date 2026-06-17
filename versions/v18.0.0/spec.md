# v18.0.0 — Language Power マイルストーン宣言 仕様

## 概要

v17.1.0〜v17.8.0 で実装した「Language Power」機能群を総括し、
マイルストーンとして宣言する。
主な実装内容はバージョン更新・CHANGELOG・README・サイトドキュメントの整備。
新機能コードの追加はなし。

---

## 1. 宣言内容

「Language Power」達成とは、以下がすべて完成していることを意味する：

| 機能 | 対応バージョン | 状態 |
|---|---|---|
| 境界付きジェネリクス `fn f<T with Ord>(...)` | v17.1.0 | 完了 |
| or-pattern / list-pattern / guard | v17.2.0 | 完了 |
| コレクション内包表記 `[x * 2 \| x <- list]` | v17.3.0 | 完了 |
| `bind` バインディング統一（`let` 除去） | v17.4.0 | 完了 |
| REPL 品質向上（`:doc` / `:load` / タブ補完） | v17.5.0 | 完了 |
| `fav bench` マイクロベンチマーク | v17.6.0 | 完了 |
| `forall` プロパティベーステスト | v17.7.0 | 完了 |
| パッケージシステム（`fav add` / `fav publish`） | v17.8.0 | 完了 |

---

## 2. バージョン更新

- `fav/Cargo.toml`: `17.8.0` → `18.0.0`

---

## 3. CHANGELOG.md 更新

`CHANGELOG.md` に v17.1.0〜v17.8.0 の全エントリを追加。
各エントリの形式：

```markdown
## v17.x.0 — タイトル（YYYY-MM-DD）

- 追加機能の箇条書き
```

---

## 4. README.md 更新

以下の箇所を更新：

### 4.1 「現在の状態」セクション

```markdown
**現在のバージョン: v18.0.0 — Language Power**
```

### 4.2 Language Power 達成の記載

- 境界付きジェネリクス（bounded generics）
- パターンマッチ拡張（or-pattern / list-pattern）
- コレクション内包表記
- パッケージシステム（`fav add` / `fav publish`）

### 4.3 バージョン履歴表

v17.1.0〜v18.0.0 のエントリを追加。

---

## 5. サイトドキュメント新規作成

### 5.1 `site/content/docs/language/patterns.mdx`

- or-pattern（`"a" | "b" => ...`）
- list-pattern（`[head, ..tail]`）
- guard 条件（`if guard`）
- 実用例（データパイプラインのステータス分岐など）

### 5.2 `site/content/docs/language/comprehensions.mdx`

- 基本 map（`[x * 2 | x <- nums]`）
- フィルタ付き（`[x | x <- nums, x > 0]`）
- 複数ソース直積（`[Pair(a,b) | a <- as, b <- bs]`）
- Result 内包（`[? f(x) | x <- xs]`）
- Before/After 比較

### 5.3 `site/content/docs/language/bind.mdx`

- `bind x <- expr` の使い方（Result / 非 Result 両対応）
- `let` キーワードが存在しない理由
- モナディック bind との違い

### 5.4 `site/content/docs/packages/publishing.mdx`

- `fav publish` の使い方
- `fav publish --dry-run`
- `fav login` の認証フロー
- `fav.toml` の `[rune]` セクション（name / version / description 必須）

---

## 6. テスト（v180000_tests）

| テスト名 | 内容 |
|---|---|
| `version_is_18_0_0` | Cargo.toml に "18.0.0" が含まれる |
| `changelog_has_v17_entries` | CHANGELOG.md に "v17." が含まれる |
| `readme_mentions_bounded_generics` | README.md に "bounded generics" または "Bounded Generics" が含まれる |
| `readme_mentions_package_system` | README.md に "fav add" または "package system" が含まれる |
| `docs_generics_exists` | `site/content/docs/language/generics.mdx` が存在する |

---

## 7. 完了条件

- [ ] `fav/Cargo.toml` バージョンが `18.0.0`
- [ ] `CHANGELOG.md` に v17.1.0〜v17.8.0 エントリが存在する
- [ ] `README.md` に `v18.0.0` と `bounded generics` / `fav add` の記載がある
- [ ] `site/content/docs/language/patterns.mdx` が存在する
- [ ] `site/content/docs/language/comprehensions.mdx` が存在する
- [ ] `site/content/docs/language/bind.mdx` が存在する
- [ ] `site/content/docs/packages/publishing.mdx` が存在する
- [ ] `cargo test v180000` — 5/5 PASS
- [ ] `cargo test` — リグレッションなし
