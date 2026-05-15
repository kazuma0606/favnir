# Favnir v2.2.0 仕様書 — pipe match + pattern guard

作成日: 2026-05-13

> **テーマ**: Favnir のパイプライン×型安全の核心を完成させる。
> `|> match {}` でパイプ末尾の分岐を宣言的に書け、
> `where` ガードで条件付きアームを表現できるようにする。
>
> **前提**: v2.1.0 完了（556 テスト通過）

---

## 1. スコープ概要

| Phase | テーマ | Done definition |
|---|---|---|
| 0 | バージョン更新 | `v2.2.0` がビルドされ HELP テキストに反映される |
| 1 | variant 大文字小文字の正規化 | `Ok(v)` / `Err(e)` パターンが組み込み Result を正しく match する |
| 2 | pipe match テスト補完 | ロードマップ完了条件が全てテストで確認される |
| 3 | pattern guard テスト補完 | ガードのフォールスルー・複合ケースがテストで確認される |
| 4 | テスト・ドキュメント | 全テスト通過、langspec v2.2.0 |

---

## 2. 現状の実装状況

### 2-1. 既に実装済み

**`pipe match`（`|> match { ... }`）**:
- ✅ パーサー: `|> match { arms }` をその場で `Expr::Match` に脱糖（`parser.rs`）
- ✅ チェッカー: 通常の `Match` として型チェック
- ✅ codegen / VM: 通常の match として実行
- ✅ パーサーテスト: `test_parse_pipe_match`

**`pattern guard`（`where` 句）**:
- ✅ パーサー: `match arm` 末尾の `where <expr>` を `MatchArm.guard` に格納
- ✅ チェッカー: ガード式が `Bool` でなければ E027
- ✅ codegen: `JumpIfFalse` でガード不成立時に次アームへジャンプ
- ✅ VM テスト: `test_match_guard`
- ✅ パーサーテスト: `test_parse_match_guard`
- ✅ checker テスト: `test_guard_non_bool`

### 2-2. 未解決の問題

**variant 名の大文字小文字不一致（バグ）**:

組み込み Result / Option の VM タグは小文字（`"ok"`, `"err"`, `"some"`, `"none"`）。
一方、パターンで `Ok(v)` と書くと `IRPattern::Variant("Ok", ...)` となり、
タグ比較 `"Ok" == "ok"` が失敗して非網羅 match になる。

```favnir
// 現状: 実行時エラー "non-exhaustive match"
result |> match {
    Ok(v)  => v      // ← "Ok" が "ok" にマッチしない
    Err(_) => 0
}

// 現状: 動作する（小文字を使えば OK）
result |> match {
    ok(v)  => v
    err(_) => 0
}
```

ユーザー定義 ADT（`type Shape = | Circle(Int)`）は PascalCase タグで動作しており問題ない。
問題は組み込み Result / Option のタグが歴史的に小文字で作られた点のみ。

---

## 3. Phase 1 — variant 大文字小文字の正規化

### 3-1. 設計方針

コンパイラの `compile_pattern` でパターン名を正規化する。
既知の組み込み variant 名を正規化テーブルで小文字に変換する。

対象:
| パターン記述 | 正規化後 | VM タグ |
|---|---|---|
| `Ok(v)` → | `"ok"` | `"ok"` |
| `Err(e)` → | `"err"` | `"err"` |
| `Some(v)` → | `"some"` | `"some"` |
| `None` → | `"none"` | `"none"` |

ユーザー定義 ADT（`Circle`, `Square` 等）は変換しない。

### 3-2. 実装場所

`src/middle/compiler.rs` — `compile_pattern` 関数:

```rust
Pattern::Variant(name, inner, _) => {
    // Normalize built-in variant names to lowercase so that
    // `Ok(v)` / `Err(e)` / `Some(v)` / `None` match the runtime tags
    // produced by Result.ok / Result.err / Option.some / Option.none.
    let normalized = match name.as_str() {
        "Ok"   => "ok",
        "Err"  => "err",
        "Some" => "some",
        "None" => "none",
        other  => other,
    };
    IRPattern::Variant(
        normalized.to_string(),
        inner.as_ref().map(|p| Box::new(compile_pattern(p, ctx))),
    )
}
```

### 3-3. 影響範囲

- 既存テストへの影響なし（ユーザー定義 ADT は変換されない）
- `ok(v)` / `Ok(v)` が両方とも動作するようになる（後方互換）

---

## 4. Phase 2 — pipe match テスト補完

### 4-1. エンドツーエンドテスト

ロードマップの完了条件:

```favnir
// (A) Result |> match
public fn main() -> Int {
    bind result <- Result.ok(5)
    result |> match {
        Ok(v)  => v
        Err(_) => 0
    }
}
// 期待値: Int(5)
```

```favnir
// (B) Err のケース
public fn main() -> Int {
    bind result <- Result.err("oops")
    result |> match {
        Ok(v)  => v
        Err(_) => -1
    }
}
// 期待値: Int(-1)
```

```favnir
// (C) Option |> match
public fn main() -> Int {
    bind opt <- Option.some(42)
    opt |> match {
        Some(v) => v
        None    => 0
    }
}
// 期待値: Int(42)
```

```favnir
// (D) パイプライン末尾で使う
fn fetch(id: Int) -> Int { Result.ok(id * 10) }

public fn main() -> Int {
    fetch(3)
    |> match {
        Ok(v)  => v
        Err(_) => 0
    }
}
// 期待値: Int(30)
```

### 4-2. 型チェックテスト

```favnir
// (E) checker: 非 Option/Result でも match OK
fn f(x: Int) -> Int {
    x |> match {
        n => n + 1
    }
}
// → 型チェックを通る
```

---

## 5. Phase 3 — pattern guard テスト補完

### 5-1. 基本ガード（Int）

```favnir
// guard fallthrough: ガード不成立時に次アームへ
public fn main() -> String {
    match 15 {
        n where n > 20 => "big"
        n where n > 10 => "medium"
        _              => "small"
    }
}
// 期待値: String("medium")
```

### 5-2. レコード + ガード

ロードマップの完了条件:

```favnir
type User = { name: String  age: Int }

public fn main() -> String {
    bind u <- User { name: "Alice"  age: 20 }
    match u {
        { age } where age >= 18 => "adult"
        _                       => "minor"
    }
}
// 期待値: String("adult")
```

```favnir
// ガード不成立 → minor
type User = { name: String  age: Int }

public fn main() -> String {
    bind u <- User { name: "Bob"  age: 15 }
    match u {
        { age } where age >= 18 => "adult"
        _                       => "minor"
    }
}
// 期待値: String("minor")
```

### 5-3. 複合ガード（`&&` / `||`）

v2.1.0 で `&&` / `||` が追加されたため、ガード式でも使える:

```favnir
public fn main() -> String {
    match 25 {
        n where n >= 18 && n < 65 => "working-age"
        n where n >= 65           => "senior"
        _                         => "youth"
    }
}
// 期待値: String("working-age")
```

### 5-4. E027 — ガード式が Bool でない

```favnir
fn f(x: Int) -> Int {
    match x {
        n where n + 1 => n   // E027: guard must be Bool
        _ => 0
    }
}
// チェッカーエラー E027
```

---

## 6. エラーコード一覧（v2.2.0 追加分）

v2.2.0 で新規追加するエラーコードはない。
既存 E027 (pattern guard must be Bool) を引き続き使用。

---

## 7. 後方互換性

- v2.1.0 のコードはそのまま動く
- `ok(v)` / `err(e)` (小文字) は引き続き動作（変換前と同じ）
- `Ok(v)` / `Err(e)` (大文字) が新たに動作するようになる
- ユーザー定義 ADT のパターンマッチは変化なし

---

## 8. 完了条件

- `result |> match { Ok(v) => v  Err(_) => 0 }` が動く
- `result |> match { Ok(v) => v  Err(_) => -1 }` で Err のケースが動く
- `opt |> match { Some(v) => v  None => 0 }` が動く
- `match x { n where n > 10 => "big"  _ => "small" }` が動く
- `match u { { age } where age >= 18 => "adult"  _ => "minor" }` が動く
- ガード不成立時に次アームへフォールスルーする
- ガード式が Bool でない場合に E027 が出る
- 既存テストが全て通る
- `Cargo.toml` バージョンが `"2.2.0"`
- `versions/v2.2.0/langspec.md` 作成済み

---

## 9. 先送り一覧

| 機能 | 理由 | 対応予定 |
|---|---|---|
| match 網羅性チェック（Result/Option の全ケース要求） | 型システムの大幅拡張が必要 | v3.0.0 以降 |
| 短絡評価 `&&` / `||`（ガード内） | VM アーキテクチャの変更が必要 | v3.0.0 以降 |
| `pipe match` の専用 AST ノード | 脱糖で十分。専用ノード不要 | 対応予定なし |
