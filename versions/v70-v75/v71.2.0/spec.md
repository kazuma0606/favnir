# v71.2.0 Spec — Refined Types（型レベル制約 `where self`）

Date: 2026-08-09
Status: 計画中

---

## Background

v71.1.0 で依存型（`Vec<T>[N]`）を追加した。v71.2.0 では Type System 2.0 フェーズの第 2 弾として、値域制約を型に組み込む **Refined Types** を実装する。

`type PositiveFloat = Float where self > 0.0` のように `where self <expr>` 節を使い、型に制約を付与する。制約違反はコンパイル時エラー（E0425）として検出される。

**パーサーは v41.1.0 で既に実装済み** — `type X = T where expr` の構文は `TypeDef.invariants: Vec<Expr>` として AST に格納される。今回追加するのはチェッカー側の制約評価ロジックのみ。

---

## Goals

1. **`refined_type_positive_float`** — `type PositiveFloat = Float where self > 0.0` の型定義と、それを引数に持つ関数が typecheck で通ることを確認
2. **`refined_type_violation_compile_error`** — 制約を違反するリテラルを渡した場合に E0425 が発生することを確認
3. テスト 2 件追加（3586 → 3588）

---

## Syntax / API Examples

```favnir
// 型レベル制約
type PositiveFloat = Float where self > 0.0
type NonEmptyStr   = String where String.length(self) > 0
type BatchSize     = Int where self >= 1 && self <= 10000

// 型違反はコンパイルエラー
fn safe_log(x: PositiveFloat) -> Float {
    1.0  // 本体は最小実装
}

fn bad() -> Float {
    safe_log(0.0)  // コンパイルエラー: 0.0 > 0.0 が false → E0425
}
```

---

## 実装スコープ

v71.2.0 は最小実装（2 テスト）にとどめる。

**実装内容:**

### 1. 既存 `type_invariants` の流用（checker.rs）

`register_item_signatures` ではすでに全 TypeDef（Alias 含む）の `invariants` を `type_invariants: HashMap<String, Vec<Expr>>` に登録している（line 2338–2339）。ただし alias 型は `continue` ステートメント（line 2324）で早期リターンしているため、その前に登録を行う必要がある。

`alias_type_invariants` 新フィールドは追加しない。`type_invariants` を直接参照する。

### 2. fn_alias_refinements レジストリ追加（checker.rs）

関数シグネチャ登録時、パラメータの TypeExpr が refined alias 型の場合に登録:

```rust
fn_alias_refinements: HashMap<String, Vec<(usize, Vec<Expr>)>>
// キー: 関数名（"safe_log"）
// 値: (パラメータインデックス, 制約式リスト) のリスト
```

### 3. 呼び出し時の制約チェック（Expr::Apply）

`unify` 成功後に `fn_alias_refinements` を照合:
- 該当パラメータにリテラルが渡されている場合
- `eval_static_expr(constraint, {"self": lit})` を評価
- false → E0425 を発行

### 4. check_type_def でエイリアス制約を型チェック

`TypeBody::Alias` の invariants を型チェック:
- `self` を `Float`（ターゲット型）として env に定義
- invariant 式が Bool であることを確認（E0245 を再利用）

---

## Error Codes

| コード | 内容 |
|---|---|
| E0425 | Refined type の制約違反（リテラルが `where` 制約を満たさない） |

注意: E0421 は v71.1.0 で依存型次元不一致に使用済み。E0422 はインターフェース境界違反、E0423 は HasField 制約違反、E0424 は RBAC アクセス拒否に使用済み。E0425 が空き（error_catalog.rs 確認済み）。

---

## テスト実装（概要）

```rust
#[test]
fn refined_type_positive_float() {
    let src = concat!(
        "type PositiveFloat = Float where self > 0.0\n",
        "fn safe_log(x: PositiveFloat) -> Float { 1.0 }\n",
        "public fn main() -> Bool { true }\n",
    );
    let prog = Parser::parse_str(src, "test.fav").expect("parse should succeed");
    let (errors, _) = Checker::check_program(&prog);
    assert!(
        errors.is_empty(),
        "refined type definition should typecheck cleanly; errors: {:?}",
        errors
    );
}

#[test]
fn refined_type_violation_compile_error() {
    let src = concat!(
        "type PositiveFloat = Float where self > 0.0\n",
        "fn safe_log(x: PositiveFloat) -> Float { 1.0 }\n",
        "fn bad() -> Float { safe_log(0.0) }\n",
        "public fn main() -> Bool { true }\n",
    );
    let prog = Parser::parse_str(src, "test.fav").expect("parse should succeed");
    let (errors, _) = Checker::check_program(&prog);
    assert!(
        errors.iter().any(|e| e.code == "E0425"),
        "constraint violation should produce E0425; errors: {:?}",
        errors
    );
}
```

---

## Success Criteria

- [ ] `refined_type_positive_float`: 型定義 + 関数定義が typecheck で通る（errors.is_empty()）
- [ ] `refined_type_violation_compile_error`: 制約違反リテラルで E0425 が発生する
- [ ] `cargo test v712000` で 2 件 pass
- [ ] `cargo test` 全体で 3588 tests pass（0 failures）

---

## Files to Modify

| ファイル | 変更内容 |
|---|---|
| `fav/src/middle/checker.rs` | `fn_alias_refinements` フィールド追加、`register_item_signatures`（alias 型の type_invariants 登録 + fn_alias_refinements 伝播）・`check_type_def`（Alias ブランチ追加）・`Expr::Apply`（E0425 チェック）更新 |
| `fav/src/error_catalog.rs` | E0425（Refined type 制約違反）エントリ追加 |
| `fav/src/driver.rs` | `v712000_tests` モジュール追加 + version 文字列更新 |
| `fav/Cargo.toml` | `version` を `"71.1.0"` → `"71.2.0"` |
| `CHANGELOG.md` | v71.2.0 エントリ追加 |
| `versions/current.md` | 進行中バージョンを v71.2.0 に更新 |

### 事前確認事項

- E0425 が未定義であることを確認（E0421/E0422/E0423 は使用済み）
- `type X = T where expr` が現行パーサーで通ることを確認（v41.1.0 実装済みのはず）
- `TypeDef.invariants` が alias type で正しく格納されることを確認
