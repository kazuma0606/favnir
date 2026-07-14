# v41.4.0 Spec — ガード付き match

**バージョン**: v41.4.0
**テーマ**: `match n { m if m >= 90 => "A" ... }` の checker.fav 統合
**前バージョン**: v41.3.0（タプルパターン match）
**目標テスト数**: 2859（前バージョン 2856 + 3）

---

## 概要

`if` ガード付き match アームは v17.2.0 からパーサー・コードジェネレーター・
フォーマッターで既にサポートされている（実行もできる）。

v41.4.0 では **checker.fav への統合** のみを行う：

1. `ast_lower_checker.rs` でガード式を `EArmG` 形式として checker.fav に渡す
2. `checker.fav` に `EArmG` バリアントを追加し、全トラバーサル関数に対応する
3. 網羅性チェック（`collect_arm_ctors`）で「ガード付きワイルドカードは catch-all に
   カウントしない」ロジックを追加する

### 既存の動作（変更なし）

| 機能 | 実装場所 | 状態 |
|---|---|---|
| `if expr` ガード構文パース | `parser.rs` | v17.2.0 から実装済み |
| ガード評価（JumpIfFalse） | `codegen.rs` `emit_match` | 実装済み |
| `fav fmt` ガード出力 | `fmt.rs` | 実装済み |

### v41.4.0 スコープ

| 変更 | 内容 |
|---|---|
| `ast_lower_checker.rs` | `lower_arms` に `EArmG` 分岐を追加 |
| `checker.fav` | `EArmG` バリアント追加 + 全トラバーサル関数に EArmG ケース |
| `checker.fav` | `collect_arm_ctors` でガード付き `_`/`PVar` をスキップ |
| `driver.rs` | `v41300_tests::cargo_toml_version_is_41_3_0` スタブ化、`v41400_tests` 追加（3件） |
| `Cargo.toml` | `version = "41.4.0"` に bump |
| `CHANGELOG.md` | `[v41.4.0]` エントリ追加 |

---

## 変更ファイル一覧

| ファイル | 変更内容 |
|---|---|
| `fav/src/middle/ast_lower_checker.rs` | `v4` ヘルパー追加 + `lower_arms` に EArmG 分岐 |
| `fav/self/checker.fav` | `Expr` 型に `EArmG` 追加 + 各関数の EArmG ケース |
| `fav/src/driver.rs` | `v41300_tests` スタブ化 + `v41400_tests` 3 件追加 |
| `fav/Cargo.toml` | `version = "41.4.0"` |
| `CHANGELOG.md` | `[v41.4.0]` エントリ |

---

## 詳細仕様

### 1. ast_lower_checker.rs — v4 ヘルパーと lower_arms 変更

#### 1.1 `v4` ヘルパー追加

`v3` ヘルパーの直後に追加（`v2`/`v3` と同じ HashMap レコードパターン使用）:

```rust
/// 4-arg variant — payload is `{_0, _1, _2, _3}` record
#[inline]
fn v4(tag: &str, a: Value, b: Value, c: Value, d: Value) -> Value {
    let mut map = HashMap::new();
    map.insert("_0".to_string(), a);
    map.insert("_1".to_string(), b);
    map.insert("_2".to_string(), c);
    map.insert("_3".to_string(), d);
    Value::Variant(tag.to_string(), Some(Box::new(Value::Record(map))))
}
```

#### 1.2 `lower_arms` 変更

```rust
fn lower_arms(arms: &[ast::MatchArm]) -> Value {
    arms.iter()
        .rev()
        .fold(v0("EArmNil"), |acc, arm| {
            if let Some(guard) = &arm.guard {
                // v41.4.0: EArmG(pat, guard_expr, body, rest)
                v4("EArmG", lower_pat(&arm.pattern), lower_expr(guard), lower_expr(&arm.body), acc)
            } else {
                v3("EArm", lower_pat(&arm.pattern), lower_expr(&arm.body), acc)
            }
        })
}
```

---

### 2. checker.fav — EArmG バリアント追加

#### 2.1 `Expr` 型に `EArmG` 追加

`EArmNil` の直後に追加:

```favnir
| EArmNil
| EArmG(Pat, Expr, Expr, Expr)  // v41.4.0: (pat, guard_expr, body, rest)
```

#### 2.2 `infer_arms_effects` に EArmG ケース

```favnir
fn infer_arms_effects(arms: Expr) -> List<String> {
    match arms {
        EArmNil => List.empty()
        EArm({ _0: pat, _1: body, _2: rest }) => List.concat(infer_expr_effects(body), infer_arms_effects(rest))
        EArmG({ _0: pat, _1: guard, _2: body, _3: rest }) =>
            List.concat(infer_expr_effects(guard), List.concat(infer_expr_effects(body), infer_arms_effects(rest)))
        _ => List.empty()
    }
}
```

#### 2.3 `check_rebind` に EArmG ケース

`EArm` ケースの直後に追加（guard 式も再帰チェック対象に含める）:

```favnir
EArmG({ _0: pat, _1: guard, _2: body, _3: rest }) => {
    match check_rebind(guard, List.empty()) {
        Some(err) => Option.some(err)
        None => match check_rebind(body, List.empty()) {
            Some(err) => Option.some(err)
            None => check_rebind(rest, List.empty())
        }
    }
}
```

#### 2.4 `check_w006_arms` に EArmG ケース

guard 式の W006 も含める（`infer_arms_effects` のエフェクト収集と対称）:

```favnir
fn check_w006_arms(arms: Expr) -> List<String> {
    match arms {
        EArmNil => List.empty()
        EArm({ _0: pat, _1: body, _2: rest }) => List.concat(check_w006_expr(body), check_w006_arms(rest))
        EArmG({ _0: pat, _1: guard, _2: body, _3: rest }) =>
            List.concat(check_w006_expr(guard), List.concat(check_w006_expr(body), check_w006_arms(rest)))
        _ => List.empty()
    }
}
```

#### 2.5 `infer_arms` に EArmG ケース

`EArmNil` ケースの直後に追加。guard 式はパターンでバインドした変数を参照できるため `pat_env` で評価するが、guard 式自体の型チェック（Bool 確認）は v41.5.0 以降に延期する:

```favnir
EArmG({ _0: pat, _1: guard, _2: body, _3: rest }) => {
    bind pat_env <- env_from_pat(pat, env);
    Result.and_then(infer_expr(body, pat_env), |bty| match rest {
        EArmNil => Result.ok(bty)
        _ => Result.and_then(infer_arms(rest, env), |rty| if ((bty == rty) || (bty == "Unknown")) || (rty == "Unknown") {
            Result.ok(bty)
        } else {
            Result.ok("Unknown")
        })
    })
}
```

#### 2.6 `collect_arm_ctors` に EArmG ケース（網羅性チェックのコア）

```favnir
fn collect_arm_ctors(arms: Expr) -> List<String> {
    match arms {
        EArmNil => List.empty()
        EArm({ _0: pat, _1: body, _2: rest }) => {
            bind ctor <- pat_ctor_name(pat);
            List.push(collect_arm_ctors(rest), ctor)
        }
        EArmG({ _0: pat, _1: guard, _2: body, _3: rest }) => {
            // v41.4.0: ガード付きアーム — PVar/PWild は catch-all にカウントしない
            bind ctor <- pat_ctor_name(pat);
            if str_eq(ctor, "_") {
                // ガード付きワイルドカード: 網羅性に寄与しない
                collect_arm_ctors(rest)
            } else {
                // ガード付きコンストラクタ: ctor カバレッジには寄与
                List.push(collect_arm_ctors(rest), ctor)
            }
        }
        _ => List.empty()
    }
}
```

**設計注意**:
- `str_eq(ctor, "_")` は `PVar` も `PWild` も同じ `"_"` を返すため（`pat_ctor_name` の実装通り）、条件分岐として正しい
- `bind ctor <- pat_ctor_name(pat)` は `pat_ctor_name` が `String` を返すが、既存の `EArm` ケースも同じパターンを使用しており動作実績あり（Favnir は Identity monad として `bind` を `String` にも適用できる）

---

### 3. テスト設計（v41400_tests）

#### T1: `cargo_toml_version_is_41_4_0`（NOTE コメント付き）

```rust
#[test]
fn cargo_toml_version_is_41_4_0() {
    // NOTE: この assert は次バージョン bump 時にスタブ化すること
    let cargo = include_str!("../Cargo.toml");
    assert!(cargo.contains("41.4.0"), "Cargo.toml must contain version 41.4.0");
}
```

#### T2: `changelog_has_v41_4_0`

```rust
#[test]
fn changelog_has_v41_4_0() {
    let src = include_str!("../../CHANGELOG.md");
    assert!(src.contains("[v41.4.0]"), "CHANGELOG.md must contain [v41.4.0]");
}
```

#### T3: `guard_match_parseable`

```rust
#[test]
fn guard_match_parseable() {
    use crate::frontend::parser::Parser;
    let src = r#"fn f(n: Int) -> String { match n { m if m >= 90 => "A" m if m >= 70 => "B" _ => "C" } }"#;
    let result = Parser::parse_str(src, "test.fav");
    assert!(result.is_ok(), "Guard match should parse without error: {:?}", result.err());
}
```

---

## 完了条件

- `cargo test` が 2859 tests passed, 0 failed
- `v41400_tests` 3 件すべて pass
- `EArmG` が checker.fav の `Expr` 型に追加されている
- ガード付き match の網羅性チェックで、`n if n >= 90` のみの match が catch-all にならない（exhaustiveness チェック正確化）
- 既存の `EArm` を使うパスが壊れていない
- **既知の制約（スコープ外）**: guard 式自体の Bool 型チェックは行わない（v41.5.0 以降で精密化予定）。例: `m if "not_bool"` はエラーなしに通過する

---

## 設計ノート

- `EArmG(Pat, Expr, Expr, Expr)` = `(pat, guard_expr, body, rest)` ← フィールド順
- `lower_arms` は後ろから畳み込む（`rev().fold(EArmNil, ...)` の順序維持）
- ガード付き `PVariant` / `PVariantP` は引き続きコンストラクタ名を網羅性に寄与させる（`Some(x) if cond` → `Some` は counted）
- ガード付き `PWild` / `PVar` だけをスキップ（これが最重要の修正）
- `infer_arms` の EArmG ケースはガード式自体を型チェックしない（guard 式は `Bool` 相当と仮定 → v41.5.0 以降で精密化）
- `str_eq` はすでに checker.fav に定義済み
- WASM ビルドへの影響なし（`ast_lower_checker.rs` はネイティブパスのみ）
