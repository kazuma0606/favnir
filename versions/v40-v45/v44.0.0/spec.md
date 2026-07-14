# v44.0.0 Spec — Language Expressiveness 宣言 ★クリーンアップ

## 概要

v43.1〜v43.13 で構築した型推論 6 カテゴリ・opaque type・冗長注釈 lint の成果を **マイルストーンとして正式宣言** するリリース。

新規機能の追加は一切行わない（宣言・クリーンアップ専用版）。

---

## 宣言文

> 「戻り値型は省略でき、ジェネリクスは呼び出し側から推論される。
>  ラムダ引数はパイプライン上流の型から確定し、
>  `opaque type` で型の境界を守れる。
>
>  これが Favnir v44.0 — Language Expressiveness の姿である。」

---

## 成果物

### 1. `MILESTONE.md` 更新

`v44.0.0 — Language Expressiveness` セクションを追加。v43.1〜v43.13 の達成コンポーネント一覧を記載。

### 2. `README.md` 更新

`Language Expressiveness` / `v44.0` への言及を追加（マイルストーン一覧またはフィーチャーセクション）。

### 3. `CHANGELOG.md` 更新

v44.0.0 エントリ追加。

### 4. `Cargo.toml` バンプ

`43.13.0` → `44.0.0`

### 5. `★クリーンアップ`

`cargo clean` 実行（ビルドアーティファクト削除）。

---

## テスト

`v44000_tests` 4 件:

| テスト名 | 内容 |
|---|---|
| `cargo_toml_version_is_44_0_0` | `Cargo.toml` に `"44.0.0"` が含まれる |
| `changelog_has_v44_0_0` | `CHANGELOG.md` に `"[v44.0.0]"` が含まれる |
| `milestone_has_language_expressiveness` | `MILESTONE.md` に `"Language Expressiveness"` が含まれる |
| `readme_mentions_language_expressiveness` | `README.md` に `"Language Expressiveness"` または `"v44.0"` が含まれる |

---

## 完了条件

- `cargo test -j 8 -- --test-threads=8` で **2941 passed; 0 failed**（2937 + 4）
- `v44000_tests` 4 件 pass
- `MILESTONE.md` に `"Language Expressiveness"` が含まれる
- `cargo clean` 完了
