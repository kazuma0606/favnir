# Spec: v48.0.0 — Standard Library 2.0 宣言 ★クリーンアップ

## 概要

v47.1〜v47.9 で実装した標準ライブラリ拡充（List / String / Float / Option / Result / Map）を総括し、
「Standard Library 2.0」マイルストーンを宣言する。

---

## 宣言文

> 「List・String・Float・Option・Result・Map の主要操作が揃い、
>  外部ライブラリなしに実務的なデータ変換が書ける。
>
>  これが Favnir v48.0 — Standard Library 2.0 の姿である。」

---

## 実装スコープ

新機能追加なし（コードフリーズ）。以下のドキュメント・宣言作業のみ:

| 作業 | 内容 |
|---|---|
| `MILESTONE.md` 更新 | v48.0.0 — Standard Library 2.0 エントリを先頭に追加 |
| `README.md` 更新 | `"Standard Library 2.0"` への言及を追加 |
| `Cargo.toml` version bump | `47.9.0` → `48.0.0` |
| `CHANGELOG.md` エントリ追加 | v48.0.0 マイルストーン宣言エントリ |
| `v48000_tests` 追加 | 4 件の宣言確認テスト |
| `cargo clean` ★クリーンアップ | ビルド生成物のクリア |

---

## テスト（+4）

| テスト名 | 内容 |
|---|---|
| `cargo_toml_version_is_48_0_0` | `Cargo.toml` に `version = "48.0.0"` が含まれる |
| `changelog_has_v48_0_0` | `CHANGELOG.md` に `[v48.0.0]` が含まれる |
| `milestone_has_stdlib_v2` | `MILESTONE.md` に `"Standard Library 2.0"` が含まれる |
| `readme_mentions_stdlib_v2` | `README.md` に `"Standard Library 2.0"` が含まれる |

テスト数: 3041 → **3045**（+4）

---

## 達成コンポーネント一覧（v47.1〜v47.9）

| コンポーネント | バージョン | 内容 |
|---|---|---|
| `List.zip` / `List.chunk` | v47.1.0 | 2 リストのペア化・n 要素分割 |
| `List.flat_map` / `List.group_by` / `List.dedupe` | v47.2.0 | flatten+map・キーグループ化・重複除去 |
| `List.scan` / `List.take_while` / `List.drop_while` | v47.3.0 | 累積値リスト・先頭条件フィルタ |
| `String.pad_left` / `String.trim_start` / `String.repeat` | v47.4.0 | パディング・先頭トリム・繰り返し |
| `Float.round` / `Float.clamp` / `Float.abs` / `Int.to_hex` / `Int.abs` | v47.5.0 | 浮動小数点・整数拡張 |
| `Option.map` / `Option.unwrap_or` / `Option.and_then` / `Option.is_some` / `Option.is_none` | v47.6.0 | Option コンビネータ |
| `Result.map` / `Result.map_err` / `Result.and_then` / `Result.is_ok` / `Result.is_err` | v47.7.0 | Result コンビネータ |
| `Map.merge` / `Map.filter_values` / `Map.map_values` / `Map.keys` / `Map.values` | v47.8.0 | Map 拡充 |
| stdlib ドキュメント（`float.mdx` / `v2.mdx` / 各 MDX 更新） | v47.9.0 | Standard Library 2.0 全関数索引 |

---

## 完了条件

- `cargo test` ≥ 3045 passed, 0 failed（3041 + 4 件）
- `cargo clippy -- -D warnings` クリーン
- `fav/Cargo.toml` version → `"48.0.0"`
- `CHANGELOG.md` に v48.0.0 エントリ追加
- `MILESTONE.md` に v48.0.0 Standard Library 2.0 エントリ追加（`"Standard Library 2.0"` 含む）
- `README.md` に `"Standard Library 2.0"` 追加
- `versions/current.md` を v48.0.0 に更新、進行中バージョンを `v48.1.0` に更新
- `★クリーンアップ`（`cargo clean`）完了
- `tasks.md` を COMPLETE に更新（T0〜T4 全 `[x]`）
