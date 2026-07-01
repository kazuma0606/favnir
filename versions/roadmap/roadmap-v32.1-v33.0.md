# Roadmap v32.1.0 〜 v33.0.0 — Language Power

Date: 2026-07-01
Status: 骨格確定（詳細は v32.0 完了後に更新）

---

## 目標

v32.0「Language Polish」で「書きやすく・デバッグしやすい」を実現した。
次のフェーズは **「型で設計できる」** だ。

現在の型システムには以下の空白がある:

```
✗ 境界付きジェネリクス T with Ord
    → fn max<T with Ord>(a: T, b: T) -> T が書けない
    → AST には TypeParam.unbounded のみ。制約の型チェックが未実装。

✗ 行多相 Row Polymorphism
    → fn stamp<R with { id: Int }>(r: R) -> { ...R, ts: String } が書けない
    → 汎用的なレコード変換関数を型安全に書けない

✗ where 制約（関数引数）
    → fn divide(a: Int, b: Int where { b != 0 }) -> Int が書けない
    → 前提条件をコンパイル時・実行時に保証できない

✗ スキーマ型
    → type User = schema "postgres:users" が書けない
    → DB スキーマと型定義の二重管理が必要

✗ 型駆動コード生成
    → fav generate type --from postgres users が動かない
```

> **Language Power の定義（本プロジェクト固有）**
> 「Favnir の型システムを使って、DB スキーマから型を自動生成し、
>  汎用的なレコード変換関数を型安全に書き、
>  コンパイル時に前提条件を保証できること」

---

## ⚠️ 重要：v32.0 完了後に更新が必要

このファイルは **骨格のみ** である。

v31.1〜v31.9 のドッグフード・v32.0 のマイルストーン宣言完了後に、
以下の判断を加えて各節を具体化する:

1. ドッグフードで「実際に欲しくなった型システム機能」は何か
2. 境界付きジェネリクス・行多相のどちらを先にやるべきか
3. スキーマ型の実装コストと優先度（`fav infer` との統合で既に一部実現している）
4. v32.5〜v32.9 に何を入れるか

**更新担当**: v32.0 リリース時

---

## 設計決定事項（暫定）

| 項目 | 暫定決定 | 確定時期 |
|---|---|---|
| ジェネリクス制約の構文 | `fn f<T with Ord>(...)` | v32.0 完了後 |
| 組み込み Interface | Ord / Eq / Display / Hash（最小セット） | v32.0 完了後 |
| 行多相の構文 | `<R with { id: Int }>` | v32.0 完了後 |
| スキーマ型のソース | postgres / bigquery / json-schema | v32.0 完了後 |
| 型生成コマンド | `fav generate type --from <source> <table>` | v32.0 完了後 |
| 破壊的変更 | なし | 固定 |

---

## バージョン計画（骨格）

### v32.1 — 境界付きジェネリクス T with Ord

**テーマ**: 型パラメータに制約を付けて汎用関数を型安全に書ける。

```favnir
fn max<T with Ord>(a: T, b: T) -> T {
    if a > b { a } else { b }
}

fn sort<T with Ord>(list: List<T>) -> List<T> {
    List.sort_by(list, |x| x)
}
```

組み込み Interface（最小セット）:

| Interface | 意味 | 自動実装 |
|---|---|---|
| `Ord` | 順序比較（`<` `>` `<=` `>=`）| Int / Float / String |
| `Eq` | 等値比較（`==` `!=`）| 全プリミティブ型 |
| `Display` | 文字列表現（f-string 補間）| String / Int / Float / Bool |
| `Hash` | ハッシュ値計算 | Int / String |

**実装内容**（骨格）:
- `TypeParam` に `bounds: Vec<String>` フィールドを追加
- `parse_type_param` が `with Ord` / `with Eq` を解析
- 型チェッカーで制約を検証
- 組み込み 4 Interface を `checker.rs` に登録

---

### v32.2 — 行多相 Row Polymorphism

**テーマ**: 「このフィールドを持つ任意のレコード型」を受け取れる。

```favnir
// { id: Int, ...rest } を持つ任意のレコードを受け取れる
fn add_timestamp<R with { id: Int }>(row: R) -> { ...R, timestamp: String } {
    { ...row, timestamp: DateTime.format_iso(DateTime.now()) }
}

// 使用例
let user_with_ts  = add_timestamp(User { id: 1, name: "Alice" })
let order_with_ts = add_timestamp(Order { id: 42, amount: 100.0 })
```

---

### v32.3 — where 制約（関数引数）

**テーマ**: 関数の引数レベルで事前条件を保証する。

```favnir
fn divide(a: Int, b: Int where { b != 0 }) -> Int {
    a / b
}

fn process(rows: List<Row> where { List.length(rows) > 0 }) -> Result<Summary, String> {
    ...
}

// コンパイル時チェック（リテラルの場合）
divide(10, 0)   // E0xxx: 制約違反
divide(10, 2)   // OK

// 実行時アサーション（変数の場合）
divide(a, b)    // 実行時に b != 0 を検証
```

---

### v32.4 — スキーマ型

**テーマ**: DB / API スキーマから型を自動生成する。

```favnir
// Postgres テーブルから型を生成（コンパイル時 or fav infer で事前生成）
type UserRow = schema "postgres:users"
// → { id: Int, name: String, email: String, created_at: String }

// fav infer --from postgres --table users --emit-type
// → src/types/user_row.fav に型定義を出力
```

`fav infer` との統合:
```bash
fav generate type --from postgres users
# → src/types/users.fav を生成
```

---

### v32.5〜v32.9 — ドッグフード結果で決定

v32.0 完了後にドッグフード結果を見て以下から選択:

- エフェクト推論の強化（現状の自動推論の精度向上）
- 線形型の実用化（`-o` のコンパイラ強制）
- 型エラーメッセージのさらなる改善
- ジェネリクスの `impl` 対応
- 型駆動 API 生成（`fav generate api --format openapi`）

---

## v33.0 — Language Power マイルストーン宣言

**暫定完了条件（v32.0 完了後に確定）:**

| コンポーネント | 暫定完了基準 |
|---|---|
| 境界付きジェネリクス | `fn f<T with Ord>(...)` が型チェックを通る |
| 行多相 | `fn stamp<R with { id: Int }>(r: R) -> {...R, ts: String}` が動作 |
| where 制約（引数）| `fn f(x: Int where { x > 0 })` のコンパイル時・実行時チェック |
| スキーマ型 | `fav generate type --from postgres <table>` が型定義を生成 |

**★ クリーンアップ実施（v33.0 リリース時）:**

```bash
cd /c/Users/yoshi/favnir/fav
cargo clean
cargo build
cargo test 2>&1 | grep "test result"
du -sh target/
```

---

## 参考リンク

- マスタースケジュール: `versions/roadmap/roadmap-v30.1-v35.0.md`
- 前フェーズ: `versions/roadmap/roadmap-v31.1-v32.0.md`
- 次フェーズ: `versions/roadmap/roadmap-v33.1-v34.0.md`
