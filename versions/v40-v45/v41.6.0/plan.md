# v41.6.0 実装計画 — Newtype 自動 impl

## 前提（確認済み）

- checker.fav に `let x = expr;` 束縛は存在しない → `bind` または直接ネストのみ使用可
- `env_insert` 戻り値: `List<KVPair>`（Result ではない）
- `collect_variant_constructors` 戻り値: `List<KVPair>`（Result ではない）
- `infer_op` は約 L317〜L355、`is_arith_op` は約 L287 に定義済み

---

## 実装ステップ

### Step 1: `checker.fav` — `collect_variant_constructors` に newtype inner 登録追加

**変更対象**: `collect_variant_constructors` の `IWrapper(wd)` ケース（約 L2138）

**変更内容**（ネスト `env_insert` で `"__newtype__"` エントリを追加）:

```favnir
// 変更後
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

**確認ポイント**:
- 外側 `env_insert`: `wd.name` → コンストラクタ関数スキーム（既存）
- 内側（最初に評価）: まず既存の `env_insert(env, wd.name, ...)` を実行
- `String.concat("__newtype__", wd.name)` が key、`wd.inner`（例: `"Float"`）が value

---

### Step 2: `checker.fav` — `infer_op_with_newtypes` 追加

**変更対象**: `infer_op` 関数（約 L317〜L355）の直後に新規追加

**事前確認**: `op_to_str` 関数が checker.fav に存在するか確認する

**存在しない場合**（推奨・簡易版）:

```favnir
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

**配置**: `infer_op` の最終行（`}` の直後）に挿入。`is_arith_op`（L287）は前方定義済みのため参照可能。

---

### Step 3: `checker.fav` — `EBinOp` ハンドラー更新

**変更対象**: `infer_expr` 内の `EBinOp` ケース（約 L1858）

```favnir
// 変更前（1 行）
EBinOp({ _0: op, _1: left, _2: right }) => Result.and_then(infer_expr(left, env), |lty| Result.and_then(infer_expr(right, env), |rty| infer_op(op, lty, rty)))

// 変更後（infer_op → infer_op_with_newtypes）
EBinOp({ _0: op, _1: left, _2: right }) => Result.and_then(infer_expr(left, env), |lty| Result.and_then(infer_expr(right, env), |rty| infer_op_with_newtypes(op, lty, rty, env)))
```

---

### Step 4: `driver.rs` — v41500_tests スタブ化 + v41600_tests 追加

```rust
// v41500_tests 内の version アサーションをスタブ化
fn cargo_toml_version_is_41_5_0() {
    // Stubbed: version bumped to 41.6.0 -- assertion intentionally removed
}

// v41600_tests モジュールを末尾に追加
#[cfg(test)]
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

### Step 5: Cargo.toml バージョン bump

`version = "41.5.0"` → `"41.6.0"`

---

### Step 6: CHANGELOG.md 更新

```markdown
## [v41.6.0] — 2026-07-11

### Added
- Newtype 自動 impl: `type Kg(Float)` 宣言で `+`/`-`/`*`/`/` を Float/Int から自動委譲
- `checker.fav`: `infer_op_with_newtypes` — Newtype 算術の型推論ヘルパー
- `checker.fav`: `collect_variant_constructors` に `"__newtype__"` env エントリ追加
```

---

## リスク

| リスク | 影響 | 対策 |
|---|---|---|
| `"__newtype__"` プレフィックスが env_lookup で衝突 | 誤検出 | プレフィックスはアンダースコア始まりで識別子（英字始まり）と区別可 |
| `op_to_str` が存在しない | String 内側 Newtype の `+` 限定委譲が書けない | 簡易版（Float/Int のみ）で代替 |
| ネスト env_insert の順序 | 外側が key2、内側が key1 になることに注意 | `env_insert(env_insert(env, k1, v1), k2, v2)` で k1 が先に挿入される |

---

## 実装順序

1. `checker.fav` Step 1（IWrapper env 追加）→ ビルド確認
2. `checker.fav` Step 2（infer_op_with_newtypes 追加）→ ビルド確認
3. `checker.fav` Step 3（EBinOp 更新）→ ビルド確認
4. `Cargo.toml` Step 5
5. `CHANGELOG.md` Step 6
6. `driver.rs` Step 4（テスト追加）
7. `cargo test` 実行・確認
