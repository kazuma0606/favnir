# Favnir v4.1.5 Language Specification

## Theme: 型制約システム — `schemas/*.yaml` + コンパイル時検査 + `T.validate` 自動生成

v4.1.5 では Favnir の型定義に**制約（constraint）**を付与し、
コンパイル時リテラル検査・実行時バリデーション・SQL DDL 生成を統合する。
「型安全なデータパイプライン専用言語」のピッチを型システムのレベルで体現する最初のリリース。

---

## 変更サマリー

| 分類 | 機能 | 由来 |
|------|------|------|
| プロジェクト設定 | `schemas/*.yaml` — 型ごとの制約定義ファイル | ロードマップ v4.1.5 |
| コンパイル時検査 | リテラル値の制約違反 → コンパイルエラー（E05xx） | ロードマップ v4.1.5 |
| 自動生成 | `T.validate : Map<String,String> -> Result<T, List<ValidationError>>` | ロードマップ v4.1.5 |
| CLI | `fav build --schema` — 型定義 + 制約 → SQL DDL 生成 | ロードマップ v4.1.5 |
| 組み込み型 | `ValidationError = { field: String, constraint: String, value: String }` | ロードマップ v4.1.5 |

---

## 1. `schemas/*.yaml` — 制約定義ファイル

### 配置場所

```
project-root/
  schemas/
    order.yaml
    user.yaml
    product.yaml
  src/
    main.fav
  fav.toml
```

`fav check` / `fav run` / `fav build` 起動時に `schemas/` ディレクトリを自動スキャンして読み込む。
`schemas/` が存在しない場合は何も起きない（エラーなし）。

### YAML 形式

```yaml
# schemas/order.yaml
Order:
  id:
    constraints: [primary_key, positive]
  email:
    constraints: [unique]
    max_length: 255
    pattern: "^[a-z0-9._%+-]+@[a-z0-9.-]+\\.[a-z]{2,}$"
  amount:
    constraints: [positive]
    min: 0.01
  note:
    nullable: true
```

トップレベルキーが**型名**（`Order`）。各フィールドに制約を記述する。

### 対応する Favnir 型定義

`schemas/order.yaml` の `Order` は同プロジェクトの `.fav` ファイルに存在する型と紐付く:

```favnir
type Order = {
  id: Int
  email: String
  amount: Float
  note: Option<String>
}
```

型定義が見つからない場合はコンパイル警告（W05xx）を出力し、無視する。

---

## 2. 制約一覧

| 制約キー | 型 | 意味 |
|----------|-----|------|
| `primary_key` | Int | 主キー（DDL 生成で PRIMARY KEY になる） |
| `positive` | Int / Float | 値 > 0 |
| `non_negative` | Int / Float | 値 >= 0 |
| `unique` | 任意 | DDL 生成で UNIQUE になる |
| `max_length: N` | String | 文字列長 <= N |
| `min_length: N` | String | 文字列長 >= N |
| `min: V` | Int / Float | 値 >= V |
| `max: V` | Int / Float | 値 <= V |
| `pattern: "..."` | String | 正規表現にマッチ（コンパイル時はリテラルのみ、実行時は全値） |
| `nullable: true` | 任意 | `Option<T>` と対応（DDL で NOT NULL を付けない） |

`constraints:` リストで複数指定可能:
```yaml
id:
  constraints: [primary_key, positive]
```

---

## 3. コンパイル時リテラル検査

`fav check` はレコードリテラルのフィールド値を制約と照合し、違反があればコンパイルエラーを報告する。

### 検査対象: 静的に決定できるリテラルのみ

```favnir
// E0510: id must be positive (got -1)
let o = Order { id: -1, email: "user@example.com", amount: 100.0, note: None }

// E0511: amount must be positive (got -5.0)
let o = Order { id: 1, email: "user@example.com", amount: -5.0, note: None }

// E0512: email does not match pattern (got "not-an-email")
let o = Order { id: 1, email: "not-an-email", amount: 100.0, note: None }

// E0513: email exceeds max_length 255
let o = Order { id: 1, email: String.repeat("a", 300), amount: 100.0, note: None }
// ↑ String.repeat は実行時なので検査しない。リテラル文字列のみ。
```

### 検査しない（実行時バリデーションに委ねる）

- 変数を介した値（`let x = -1; Order { id: x, ... }`）
- 関数呼び出しの戻り値（`Order { id: compute_id(), ... }`）
- `db.query` / `csv.read` 等の外部データ

---

## 4. `T.validate` — 自動生成バリデーション関数

`schemas/<T>.yaml` が存在する型 `T` に対して、`fav check` は `T.validate` 関数を
**コンパイラが自動生成する**（ソースコードには現れない）。

### シグネチャ

```favnir
// 自動生成（ソース不要）
T.validate : Map<String, String> -> Result<T, List<ValidationError>>
```

### `ValidationError` 組み込み型

```favnir
type ValidationError = {
  field:      String  // 違反フィールド名 ("amount")
  constraint: String  // 違反した制約名 ("positive")
  value:      String  // 実際の値（文字列化）("-5.0")
}
```

`ValidationError` はプロジェクト側で定義不要（checker.rs に事前登録）。

### 使い方

```favnir
// db.query<Order> は内部で Order.validate を自動呼び出し（v4.2.0 以降で統合）
// v4.1.5 では手動呼び出しで動作確認できる

bind raw <- Map.set(Map.set(Map.set((), "id", "5"), "email", "ok@example.com"), "amount", "100.0")
bind result <- Order.validate(raw)
match result {
    Ok(order) => IO.println("valid")
    Err(errs) => IO.println("invalid")
}
```

### 内部動作

`T.validate(raw: Map<String, String>)` は以下を順番に実行する:

1. 各フィールドを `Map.get(raw, field_name)` で取得
2. `Option.is_none` の場合: `nullable: true` でなければ `ValidationError { field, constraint: "required", value: "" }` を追加
3. 型変換（`Int.parse` / `Float.parse` 等）が失敗したら `ValidationError { ..., constraint: "type_mismatch", ... }` を追加
4. 各制約を検査してエラーを `List<ValidationError>` に蓄積
5. エラーリストが空なら `Result.ok(T { ... })`、非空なら `Result.err(errs)`

---

## 5. `fav build --schema` — SQL DDL 生成

### コマンド形式

```
fav build --schema src/types.fav [--out migrations/001_create.sql]
```

`--out` を省略した場合は stdout に出力する。

### 生成ルール

| Favnir 型 | SQL 型 |
|-----------|--------|
| `Int` | `INTEGER` |
| `Float` | `REAL` |
| `String` | `TEXT` |
| `Bool` | `INTEGER` (0/1) |
| `Option<T>` | 対応 SQL 型（NOT NULL なし） |

| 制約 | SQL |
|------|-----|
| `primary_key` | `PRIMARY KEY AUTOINCREMENT` |
| `unique` | `UNIQUE` |
| `max_length: N` | `VARCHAR(N)` |
| `positive` | `CHECK (col > 0)` |
| `non_negative` | `CHECK (col >= 0)` |
| `min: V` | `CHECK (col >= V)` |
| `max: V` | `CHECK (col <= V)` |
| `pattern: "..."` | `CHECK (col REGEXP '...')` |
| `nullable: false`（デフォルト） | `NOT NULL` |

### 生成例

```yaml
# schemas/order.yaml
Order:
  id:
    constraints: [primary_key, positive]
  email:
    constraints: [unique]
    max_length: 255
    pattern: "^[a-z0-9._%+-]+@[a-z0-9.-]+\\.[a-z]{2,}$"
  amount:
    constraints: [positive]
    min: 0.01
  note:
    nullable: true
```

```sql
-- 生成される DDL
CREATE TABLE orders (
    id      INTEGER PRIMARY KEY AUTOINCREMENT CHECK (id > 0),
    email   VARCHAR(255) UNIQUE NOT NULL
            CHECK (email REGEXP '^[a-z0-9._%+-]+@[a-z0-9.-]+\.[a-z]{2,}$'),
    amount  REAL NOT NULL CHECK (amount > 0) CHECK (amount >= 0.01),
    note    TEXT
);
```

テーブル名は型名を snake_case の複数形に変換（`Order` → `orders`、`UserProfile` → `user_profiles`）。
DDL は出発点として生成し、最終編集は人間が行う（上書きではなく手動で `migrations/` に配置）。

---

## 6. 他モジュールとの統合（将来）

| バージョン | 統合先 | 内容 |
|-----------|--------|------|
| v4.2.0 | DB Rune 2.0 | `db.query<T>` が自動で `T.validate` を呼ぶ |
| v4.3.0 | DuckDB Rune | `duckdb.query<T>` も同様 |
| v4.4.0 | Gen Rune 2.0 | `schemas/*.yaml` の制約を満たす検証データを生成 |
| v4.11.0 | AWS SDK | `aws.s3.read_csv<T>` が `T.validate` を自動適用 |

v4.1.5 では **単体での動作**（手動 `T.validate` 呼び出し + `fav build --schema`）を確立し、
各 rune への統合は後続バージョンで行う。

---

## 7. エラーコード

| コード | 状況 |
|--------|------|
| E0510 | リテラル値が `positive` / `non_negative` 制約に違反 |
| E0511 | リテラル値が `min` / `max` 制約に違反 |
| E0512 | リテラル文字列が `pattern` 制約に違反 |
| E0513 | リテラル文字列が `max_length` / `min_length` 制約に違反 |
| E0514 | `schemas/*.yaml` の型名に対応する型定義が見つからない（警告: W0514） |
| E0515 | `schemas/*.yaml` のフィールド名が型定義に存在しない（警告: W0515） |
| E0516 | `fav build --schema` で出力ファイルの書き込みに失敗 |

---

## 8. 新規 Cargo 依存

```toml
serde_yaml = "0.9"
regex      = "1"   # pattern 制約のコンパイル時検証
```

`serde` はすでに Cargo.toml に含まれているため追加不要。

---

## テスト目標

- `schemas/*.yaml` の読み込みテスト（正常・不正 YAML・ファイルなし）
- コンパイル時リテラル検査テスト（`positive`, `max_length`, `pattern` の各制約）
- `T.validate` の実行時テスト（OK / Err パス）
- `fav build --schema` の DDL 生成テスト（型 + 制約の組み合わせ）
- 制約なし型への影響がないことの確認（リグレッション防止）
- 全既存テストがパスすること
