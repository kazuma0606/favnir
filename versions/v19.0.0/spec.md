# v19.0.0 Spec — Type System Maturity マイルストーン宣言

## 概要

v18.x シリーズ（v18.1〜v18.8）で構築した型システムの集大成を宣言するマイルストーンリリース。
新しい言語機能の追加はなく、ドキュメント整備・CHANGELOG 更新・README 更新・バージョン番号の更新が主な作業。

**テーマ**: 「信頼できる言語」への転換宣言

---

## v18.x で達成した型システム機能

| バージョン | 機能 | 状態 |
|---|---|---|
| v18.1.0 | エフェクト自動推論（`!Effect` 宣言省略） | 完了 |
| v18.2.0 | 行多相（`fn f<R with { id: Int }>` レコード制約） | 完了 |
| v18.3.0 | Refinement Types（引数 `where` 制約） | 完了 |
| v18.4.0 | スキーマ型（`schema "file:..."` インポート） | 完了 |
| v18.5.0 | 線形型（`-o` arrow、Connection/Tx 安全性） | 完了 |
| v18.6.0 | 共変・反変アノテーション（`<+T, -U>`） | 完了 |
| v18.7.0 | Const Generics（`const N: Int where { N > 0 }`） | 完了 |
| v18.8.0 | 型駆動 API 生成（`#[api(...)]` → OpenAPI / GraphQL） | 完了 |

---

## v19.0.0 実装内容

### 1. バージョン番号更新

- `fav/Cargo.toml`: `18.8.0` → `19.0.0`

### 2. CHANGELOG.md 更新

v18.1.0〜v18.8.0 の全エントリを先頭に追加（v18.0.0 の上）:

```markdown
## [v18.8.0] — 2026-06-16 — 型駆動 API 生成
## [v18.7.0] — 2026-06-16 — Const Generics
## [v18.6.0] — 2026-06-16 — 共変・反変アノテーション
## [v18.5.0] — 2026-06-16 — 線形型
## [v18.4.0] — 2026-06-16 — スキーマ型
## [v18.3.0] — 2026-06-16 — Refinement Types
## [v18.2.0] — 2026-06-16 — 行多相
## [v18.1.0] — 2026-06-16 — エフェクト推論
```

v19.0.0 エントリも追加:

```markdown
## [v19.0.0] — 2026-06-16 — Type System Maturity マイルストーン宣言
```

### 3. README.md 更新

- 「現在のバージョン」を v19.0.0 に更新
- Type System Maturity 達成を記載
- v18.x 機能一覧（effect inference / row polymorphism / refinement types / schema types / linear types / variance / const generics / API generation）を追加
- バージョン履歴表に v18.1.0〜v19.0.0 エントリ追加

### 4. ドキュメント確認（既存）

以下のドキュメントは既に作成済みのため、新規作成不要:
- `site/content/docs/language/effect-inference.mdx` ✅
- `site/content/docs/language/row-polymorphism.mdx` ✅
- `site/content/docs/language/refinement-types.mdx` ✅
- `site/content/docs/language/schema-types.mdx` ✅
- `site/content/docs/language/linear-types.mdx` ✅
- `site/content/docs/language/variance.mdx` ✅
- `site/content/docs/language/const-generics.mdx` ✅
- `site/content/docs/api/generate.mdx` ✅
- `site/content/docs/api/serve.mdx` ✅

### 5. テスト（v190000_tests、5件）

```rust
fn version_is_19_0_0()                // Cargo.toml に "19.0.0" が含まれる
fn changelog_has_v18_entries()        // CHANGELOG に v18.x エントリが含まれる
fn readme_mentions_effect_inference() // README に "エフェクト推論" or "effect inference" が含まれる
fn readme_mentions_schema_types()     // README に "スキーマ型" or "schema" が含まれる
fn api_docs_exist()                   // site/content/docs/api/generate.mdx の内容が include_str! で参照可能
```

---

## 完了条件

| 確認項目 | 状態 |
|---|---|
| `Cargo.toml` に `"19.0.0"` が含まれる | [ ] |
| `CHANGELOG.md` に v18.1.0〜v18.8.0 の全エントリが含まれる | [ ] |
| `CHANGELOG.md` に v19.0.0 エントリが含まれる | [ ] |
| `README.md` に Type System Maturity の記載がある | [ ] |
| `README.md` にエフェクト推論の記載がある | [ ] |
| `README.md` にスキーマ型の記載がある | [ ] |
| `cargo test v190000` — 5/5 PASS | [ ] |
| `cargo test` — リグレッションなし | [ ] |
