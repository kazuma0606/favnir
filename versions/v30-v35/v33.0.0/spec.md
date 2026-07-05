# v33.0.0 仕様書 — Language Power マイルストーン宣言

## 概要

v32.1〜v32.9 の全コンポーネントが確認・記録されたことを受け、
**Language Power** マイルストーンを正式宣言する。

---

## 背景

ロードマップ v32.1〜v33.0 より:

> **Language Power の定義（本プロジェクト固有）**
> 「Favnir の型システムを使って、DB スキーマから型を自動生成し、
>  汎用的なレコード変換関数を型安全に書き、
>  コンパイル時に前提条件を保証できること」

---

## 達成コンポーネント

| コンポーネント | 完了バージョン | 内容 |
|---|---|---|
| 境界付きジェネリクス（T with Ord） | v32.1.0 | `fn f<T with Ord>(a: T, b: T) -> T` が型チェックを通る |
| 行多相（Row Polymorphism） | v32.2.0 | `fn f<R with { id: Int }>(row: R)` が型安全に動作 |
| where 制約（関数引数）| v32.3.0 | `fn f(x: Int where { x > 0 })` のコンパイル時チェック（E0331） |
| スキーマ型 | v32.4.0 | `type User = schema "postgres:users"` パース・型チェック |
| 線形型（Linear Types）| v32.5.0 | E0332（二重使用禁止）/ E0333（未使用変数）|
| 分散アノテーション | v32.6.0 | `<+T>` 共変 / `<-T>` 反変・E0334 |
| 定数ジェネリクス | v32.7.0 | `<const N: Int where { N > 0 }>` / E0335 |
| 型駆動 API 生成 | v32.8.0 | `#[api(method, path)]` / OpenAPI JSON / ルートテーブル |
| エフェクト推論 | v32.9.0 | `infer_effects_fn` / `EffectSet` / 推移的推論 |

---

## スコープ

### IN SCOPE

- `fav/Cargo.toml` — version `32.9.0` → `33.0.0`
- `fav/src/driver.rs` — `cargo_toml_version_is_32_9_0` をスタブ化
- `fav/src/driver.rs` — `v330000_tests`（4 件）追加（`use super::*` **なし**、`include_str!` のみ）
- `MILESTONE.md` — v33.0.0「Language Power」セクションを先頭に追加
- `README.md` — v33.0 マイルストーン宣言の一行を v32.0 行の直後に追加
- `CHANGELOG.md` — `[v33.0.0]` セクション追加
- `benchmarks/v33.0.0.json` 新規作成
- `versions/current.md` — v33.0.0 に更新
- **`cargo clean` + `fav/tmp/hello.fav` 復元 + `cargo build` + `cargo test`**（マイルストーン版の必須クリーンアップ）

### OUT OF SCOPE

- site/ MDX 更新（次フェーズで実施）
- v33.1〜のロードマップ作成（別途作業）

> **cargo clean 注意事項**:
> `cargo clean` を実行すると `fav/tmp/hello.fav` が削除される。
> `bootstrap_c2_artifact_roundtrip` テストはこのファイルに依存するため、
> `cargo clean` 直後に必ず復元すること。
>
> 復元内容:
> ```favnir
> fn add(a: Int, b: Int) -> Int {
>     a + b
> }
>
> fn main() -> Bool {
>     add(1, 2) == 3
> }
> ```

---

## テスト設計（v330000_tests — 4 件）

| # | テスト名 | 確認内容 |
|---|---------|----------|
| 1 | `cargo_toml_version_is_33_0_0` | `Cargo.toml` に `"33.0.0"` が含まれること |
| 2 | `milestone_language_power_declared` | `MILESTONE.md` に `"Language Power"` が含まれること |
| 3 | `readme_mentions_v33_0` | `README.md` に `"v33.0"` が含まれること |
| 4 | `benchmark_v33_0_0_exists` | `benchmarks/v33.0.0.json` に `"33.0.0"` が含まれること |

> `v330000_tests` は `use super::*` **なし**（`include_str!` のみ使用 — v32.0.0 / v31.0.0 と同じパターン）。
>
> テスト名は v180000_tests（`changelog_has_v17_entries` / `readme_mentions_bounded_generics` / `readme_mentions_package_system` / `docs_generics_exists`）と異なる。

---

## MILESTONE.md 追記内容（先頭に追加）

```markdown
## v33.0.0 — Language Power（2026-07-03）

> 「Favnir の型システムを使って、DB スキーマから型を自動生成し、
>  汎用的なレコード変換関数を型安全に書き、
>  コンパイル時に前提条件を保証できること」
> = Language Power の完成を象徴する定義

v33.0.0 をもって、Favnir の **Language Power** を正式に宣言する。

境界付きジェネリクス（`T with Ord`）と行多相（`R with { id: Int }`）により汎用的なレコード変換関数が
型安全に書けるようになった。`where { b != 0 }` で関数引数の前提条件をコンパイル時に保証し、
`type User = schema "postgres:users"` でスキーマから型を自動生成できる。
線形型（E0332/E0333）・分散アノテーション（E0334）・定数ジェネリクス（E0335）が加わり、
型システムが実用的なデータパイプライン設計に耐える水準に達した。

### 達成コンポーネント（v32.1〜v32.9）

| コンポーネント | バージョン | 内容 |
|---|---|---|
| 境界付きジェネリクス | v32.1 | `T with Ord` / E0325 制約チェック |
| 行多相 | v32.2 | `R with { id: Int }` / E0337 フィールド不足 |
| where 制約 | v32.3 | `fn f(x: Int where { x > 0 })` / E0331 |
| スキーマ型 | v32.4 | `schema "postgres:users"` パース |
| 線形型 | v32.5 | E0332（二重使用）/ E0333（未使用）|
| 分散アノテーション | v32.6 | `<+T>` / `<-T>` / E0334 |
| 定数ジェネリクス | v32.7 | `<const N: Int where { N > 0 }>` / E0335 |
| 型駆動 API 生成 | v32.8 | `#[api]` / OpenAPI JSON / ルートテーブル |
| エフェクト推論 | v32.9 | `infer_effects_fn` / 推移的推論 |

**宣言日**: 2026-07-03
**宣言バージョン**: v33.0.0
```

---

## README.md 追記内容

v32.0 の説明文（「エラーメッセージが rustc スタイルに刷新され...」で終わる行）の直後、
空行の前に追加（`grep -n "Language Polish" README.md` で挿入行を特定してから実施）:

```markdown
**v33.0（2026-07-03）で、[Language Power](./MILESTONE.md) マイルストーンを宣言しました。**
境界付きジェネリクス（`T with Ord`）・行多相（`R with { id: Int }`）・`where` 制約・スキーマ型・線形型・分散アノテーション・定数ジェネリクス・型駆動 API 生成・エフェクト推論が揃い、型で設計するデータパイプラインが現実になりました。
```

---

## 完了条件

- `Cargo.toml` version = `"33.0.0"`
- `MILESTONE.md` に `"Language Power"` セクションが存在すること
- `README.md` に `"v33.0"` の記述があること
- `cargo test --bin fav v330000` — 4/4 PASS
- `cargo test`（`cargo clean` 後）— 全件 PASS（0 failures）
- `CHANGELOG.md` に `[v33.0.0]` セクション
- `benchmarks/v33.0.0.json` 存在かつ `tests_passed` が実測値
- `benchmarks/v33.0.0.json` の `milestone` フィールドが `"Language Power"` であること
- `versions/current.md` を v33.0.0 に更新
- `tasks.md` が COMPLETE
