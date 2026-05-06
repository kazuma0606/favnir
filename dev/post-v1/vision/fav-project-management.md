# Favnir Project Management & Schema-driven Workflow

日付: 2026-05-01

## 概要

Favnir は、型定義を核とした **Schema-driven Development (SDD)** を推進する。
GraphQL が通信の繋ぎ込みを抽象化したように、Favnir はデータパイプラインの繋ぎ込みを抽象化し、開発者が「データの変換ロジック（stage）」のみに集中できる環境を提供する。

これを支えるのが、TOML ベースのテンプレート機能を持つプロジェクトマネージャーである。

---

## 1. スキーマ駆動 (Schema-driven) の流れ

1. **型定義 (Defining Schema)**:
   `src/schemas/` 配下で `type` と `invariant` を定義する。これがプロジェクトの「唯一の正解」となる。
2. **構造定義 (Defining Contract)**:
   `abstract seq` を用いて、処理の全体構造を定義する。
3. **ロジック実装 (Implementing Logic)**:
   プロジェクトマネージャーが生成した `stage` の雛形に、具体的な変換処理を記述する。
4. **自動結合 (Automatic Wiring)**:
   型情報に基づき、コンパイラまたはテンプレートエンジンが `seq` の結合部分を自動生成する。

---

## 2. プロジェクトマネージャーの機能

`fav` コマンドを拡張し、プロジェクトのライフサイクルを管理する。

### `fav new <name> [--template <id>]`
指定したテンプレート（ETL, API, Batch 等）に基づき、規約に則ったディレクトリ構造を生成する。
- `fav.toml`: プロジェクトのメタデータと使用テンプレートの定義。
- `src/schemas/`: 型定義の置き場。
- `src/transforms/`: 原子的な変換ロジックの置き場。
- `src/pipelines/`: `abstract seq` とその具体化（バインディング）の置き場。

### `fav generate` (or `fav build` 内での統合)
型定義（Schema）から、以下の周辺コードを自動生成する。
- DB マイグレーション用の SQL。
- 外部システム（Job 投入側）向けの型安全なクライアントコード。
- `abstract seq` を埋めるためのデフォルトのボイラープレート。

---

## 3. TOML によるテンプレート管理

`fav.toml` にテンプレート設定を記述することで、プロジェクトの「性格」を固定し、手続き型的な逸脱を抑制する。

```toml
[rune]
name    = "user_analytics"
version = "1.0.0"
template = "standard-etl"  # 規約セットの指定

[template.options]
db_backend = "postgres"
strict_mode = true         # 規約違反 (Lint) をエラーにする
```

---

## 4. なぜこれが「手続き型」を抑制するのか

- **居場所の限定**: 「どこに何を書くべきか」がテンプレートとディレクトリ構造で決まっている。
- **繋ぎの自動化**: 手続き型で「だらだらと」書かれがちな、データの受け渡し（繋ぎ込み）部分を `abstract seq` テンプレートが代行する。
- **契約の遵守**: スキーマ（型）が先行するため、実装者はその型に合わせることを強制される。

---

## 一言でいうと

> スキーマ（型）を定義すれば、パイプライン（構造）はテンプレートから導かれ、
> 開発者はトランスフォーム（ロジック）を書くだけで済む。
