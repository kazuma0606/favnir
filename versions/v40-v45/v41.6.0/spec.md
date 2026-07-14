# v41.6.0 仕様書 — Newtype 自動 impl

**フェーズ**: Type Precision（v41.x スプリント）
**前バージョン**: v41.5.0（Row polymorphism 強化、2862 tests）
**目標テスト数**: 2865（+3）

---

## 概要

`type Kg(Float)` のような Newtype（名目型ラッパー）宣言に対して、内側の型が持つ算術演算子（`+` / `-` / `*` / `/`）を自動委譲する。

v41.5.0 時点では `a: Kg` と `b: Kg` を `a + b` と書くと `checker.fav` が E0001 "arithmetic type mismatch: Kg vs Kg" を誤検出する。本バージョンでこれを解消する。

---

## 動機

```favnir
type Kg(Float)
type Meter(Float)

fn total_mass(a: Kg, b: Kg) -> Kg {
  a + b   // v41.6.0 以前: E0001 が発生
}
```

ドメイン型（`Kg`, `USD`, `Seconds` 等）を Newtype で定義している場合、同型間の算術が自然に書けることは必須要件。

---

## スコープ

### v41.6.0 に含む

- **同型間の算術**: `Kg + Kg → Kg`（`lty == rty` かつ両方とも同じ数値 Newtype の場合）
- 内側が `Int` の Newtype: `+`, `-`, `*`, `/` を自動委譲し結果型は Newtype 自身
- 内側が `Float` の Newtype: 同上
- 内側が `String` の Newtype: `+`（連結）のみ自動委譲（`-`/`*`/`/` は意味なしのため委譲しない）

### v41.6.0 スコープ外（v42.0+ へ）

- **異型間の算術**: `Kg * Float → Kg`（スカラー積）は v42.0+
- 演算子のカスタム定義（`impl` ベース）は v42.0+
- 比較演算子（`<`, `>`）の自動委譲は v42.0+

---

## 実装方針

### 前提確認

- checker.fav には `let x = expr;` 束縛構文は存在しない。`bind` のみが変数束縛に使われる
- `env_insert(env, key, val)` の戻り値は `List<KVPair>`（Result ではない）
- `collect_variant_constructors` の戻り値は `List<KVPair>`（Result ではない）
- `infer_op_with_newtypes` は `infer_op`（約 L317〜L355）の直後に配置する。`is_arith_op`（L287）は前方定義済みのため参照可能

### 1. checker.fav: `collect_variant_constructors` での newtype 内型登録

`collect_variant_constructors` の `IWrapper(wd)` ケースで、コンストラクタ関数スキームを env に追加するのに加えて、`"__newtype__" ++ wd.name → wd.inner` というエントリも追加する。ネスト `env_insert` 呼び出しで実現する：

```favnir
// 変更前（抜粋）
IWrapper(wd) => {
    bind ret_ty <- if wd.has_where {
        String.concat("Result<", String.concat(wd.inner, ", String>"))
    } else {
        wd.name
    };
    collect_variant_constructors(List.drop(items, 1),
        env_insert(env, wd.name, make_fn_scheme_str("", wd.inner, ret_ty)))
}

// 変更後（ネスト env_insert で __newtype__ エントリを追加）
IWrapper(wd) => {
    bind ret_ty <- if wd.has_where {
        String.concat("Result<", String.concat(wd.inner, ", String>"))
    } else {
        wd.name
    };
    collect_variant_constructors(List.drop(items, 1),
        env_insert(
            env_insert(env, wd.name, make_fn_scheme_str("", wd.inner, ret_ty)),
            String.concat("__newtype__", wd.name), wd.inner))
}
```

### 2. checker.fav: `infer_op_with_newtypes` 追加

`infer_op` の直後（L355 以降）に追加。`String` Newtype は `+`（OpAdd）のみ委譲する：

```favnir
fn infer_op_with_newtypes(op: Op, lty: String, rty: String, env: List<KVPair>) -> Result<String, String> {
    if is_arith_op(op) && (lty == rty) {
        match env_lookup(env, String.concat("__newtype__", lty)) {
            Some(inner) =>
                if (inner == "Float") || (inner == "Int") {
                    Result.ok(lty)
                } else {
                    if (inner == "String") && str_eq(op_to_str(op), "+") {
                        Result.ok(lty)
                    } else {
                        infer_op(op, lty, rty)
                    }
                }
            None => infer_op(op, lty, rty)
        }
    } else {
        infer_op(op, lty, rty)
    }
}
```

**注意**: `Op` を文字列に変換する `op_to_str` が存在するか確認すること。存在しない場合は `inner == "String"` のケースを省略し `infer_op(op, lty, rty)` に落とす（String Newtype の算術委譲なしで十分）：

```favnir
// op_to_str が存在しない場合の簡易版
fn infer_op_with_newtypes(op: Op, lty: String, rty: String, env: List<KVPair>) -> Result<String, String> {
    if is_arith_op(op) && (lty == rty) {
        match env_lookup(env, String.concat("__newtype__", lty)) {
            Some(inner) =>
                if (inner == "Float") || (inner == "Int") {
                    Result.ok(lty)
                } else {
                    infer_op(op, lty, rty)
                }
            None => infer_op(op, lty, rty)
        }
    } else {
        infer_op(op, lty, rty)
    }
}
```

### 3. checker.fav: `EBinOp` ハンドラー更新

`infer_expr` 内の `EBinOp` ケース（約 L1858）を変更：

```favnir
// 変更前
EBinOp({ _0: op, _1: left, _2: right }) =>
    Result.and_then(infer_expr(left, env), |lty|
        Result.and_then(infer_expr(right, env), |rty|
            infer_op(op, lty, rty)))

// 変更後
EBinOp({ _0: op, _1: left, _2: right }) =>
    Result.and_then(infer_expr(left, env), |lty|
        Result.and_then(infer_expr(right, env), |rty|
            infer_op_with_newtypes(op, lty, rty, env)))
```

---

## 既存コードへの影響

| ファイル | 変更 | 規模 |
|---|---|---|
| `fav/self/checker.fav` | ① `collect_variant_constructors` IWrapper ケースにネスト env_insert 追加<br>② `infer_op_with_newtypes` 新規追加（約 15 行）<br>③ `EBinOp` の `infer_op` → `infer_op_with_newtypes` | 小 |
| `fav/src/driver.rs` | v41600_tests（3 件）追加 | 小 |
| `fav/Cargo.toml` | version bump `41.5.0` → `41.6.0` | 1 行 |
| `CHANGELOG.md` | `[v41.6.0]` エントリ追加 | 数行 |

`ast_lower_checker.rs` は変更不要（`WrapperDef` の `inner` フィールドは既存）。

---

## テスト計画

### Rust テスト（driver.rs）

```rust
mod v41600_tests {
    #[test]
    fn cargo_toml_version_is_41_6_0() {
        // NOTE: この assert は次バージョン bump 時にスタブ化すること
        let cargo = include_str!("../Cargo.toml");
        assert!(cargo.contains("41.6.0"), "Cargo.toml must contain version 41.6.0");
    }

    #[test]
    fn changelog_has_v41_6_0() {
        let src = include_str!("../../CHANGELOG.md");
        assert!(src.contains("[v41.6.0]"), "CHANGELOG.md must contain [v41.6.0]");
    }

    #[test]
    fn checker_fav_has_newtype_arith() {
        let src = include_str!("../self/checker.fav");
        assert!(
            src.contains("infer_op_with_newtypes"),
            "checker.fav must implement infer_op_with_newtypes for newtype arithmetic delegation"
        );
    }
}
```

---

## 注意事項

- `"__newtype__"` プレフィックスはユーザー定義識別子と衝突しない（識別子は英字始まりのため）
- `env_lookup` が返す値は `env_insert` で格納した生の inner string（例: `"Float"`）
- ロードマップの「推定 2858 tests」は誤記（v41.5.0 が 2862 のため）。正しくは **2865**。実装完了後にロードマップも修正すること

---

## 完了条件

- `cargo test` 全通過（2865 tests passed, 0 failed）
- `v41600_tests` 3 件すべて pass
- `checker.fav` に `infer_op_with_newtypes` が存在する
- `type Kg(Float)` を定義し `a + b` と書いた場合に checker.fav が E0001 を発生させない（手動確認）
