# Favnir Data Source Integration & Schema Integrity

日付: 2026-05-02

## 概要

Favnir におけるデータソース（DB, CSV, API）は、単なる外部入力ではなく、**「型システムの境界（Edge）」** である。
外部データの曖昧さを排除し、ソースコードと物理スキーマの間の「不一致（Drift）」を未然に防ぐ仕組みを規約（CoC）として提供する。

---

## 1. データソース・ファイアウォール

データの入力地点で `invariant` によるバリデーションを強制し、汚れたデータがパイプラインの深部に入り込むのを防ぐ。

```fav
type UserRow {
    id:    Int
    email: Email      -- Email の invariant がチェックされる
    age:   PosInt     -- age >= 0 がチェックされる
}

-- 変換時に 1件でも違反があれば、その場でエラーとして報告される
bind users <- Csv.parse<UserRow>("data.csv")
```

---

## 2. スキーマからの型自動生成 (Schema Synthesis)

DB のメタデータや CSV ヘッダから、Favnir の `type` 定義を自動生成する。

*   **`fav state sync`**: DB のテーブル定義、カラム型、CHECK 制約を Favnir の `type` と `invariant` に変換する。
*   **Single Source of Truth**: スキーマが真実の起点となり、プログラム側の型は常にそれに追従する。

---

## 3. 実行前のサンプリング・チェック

大規模なジョブを実行する前に、実際のデータの一部を読み込んで型定義と照合する。

*   **`fav check --sample N`**: 
    - 実際にデータソースから N 件を読み込む。
    - 定義した型や Invariant との適合率を算出。
    - 「期待される NULL 率より高い」「型は合っているが値の範囲が異常」といった傾向のズレを報告する。

---

## 4. 契約によるクエリ (Contract-based Query)

SQL を文字列でハードコードせず、型（State）を介してデータを要求する。

```fav
-- DB から特定の型に合致するデータのみを取得
bind users <- Db.fetch<ActiveUser>(where: "status = 'active'")
```

### メリット
- **SQL インジェクションの排除**: パラメータは常に型安全にバインドされる。
- **カラム不一致の静的検知**: `ActiveUser` に定義されたフィールドが DB に存在しない場合、コンパイル時にエラーとなる。

---

## 5. SQL の隔離とマイグレーション

- **`migrations/` ディレクトリ**: SQL はこのディレクトリ内に隔離され、Favnir ソースコード内に生 SQL を直接書くことを抑制する。
- **Job 投入型アーキテクチャ**: Favnir は「クエリの結果（データ）」を扱うことに集中し、データの永続化やスキーマ変更は規約に基づいた安全な方法で行う。
