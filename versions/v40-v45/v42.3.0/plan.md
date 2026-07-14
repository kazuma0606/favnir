# v42.3.0 実装計画 — CEP checker.fav 統合

## T0 — 事前確認

1. `cargo test` → 2880 passed, 0 failed を確認
2. `fav/Cargo.toml` version が `"42.2.0"` であることを確認
3. `error_catalog.rs` に `E0420` が存在しないことを確認（`grep "E0420" fav/src/error_catalog.rs`）
4. `checker.rs` の `CepPatternDef` Pass 2 スタブ行番号を記録（`grep -n "CepPatternDef" fav/src/middle/checker.rs`）
5. `v42200_tests::cargo_toml_version_is_42_2_0` が NOTE コメント付きライブアサーションであることを確認し行番号を記録
6. `v42200_tests` の閉じ `}` の行番号を確認し記録（`v42300_tests` の挿入位置のため）
7. `check_cep_pattern_def` 関数が存在しないことを確認

---

## T1 — `error_catalog.rs` 更新

E0406 エントリ（`refinement constraint type mismatch`）の直後、E05xx セクションコメントの直前に追加:

```rust
    // ── E042x: CEP パターン (v42.3.0) ─────────────────────────────────────────
    ErrorEntry {
        code: "E0420",
        title: "cep pattern within_secs must be positive",
        category: "types",
        description: "The `within` value in a `cep pattern` clause must be a positive integer (≥ 1). `within 0` is semantically invalid.",
        example: "cep pattern P { Login within 0 }  // E0420",
        fix: "Use `within N` where N ≥ 1 (e.g., `within 60`).",
    },
```

---

## T2 — `checker.rs` 更新

### 2-A: Pass 2 スタブ解除（T0 で確認した行番号）

```rust
// 変更前:
Item::CepPatternDef(_) => {} // v42.1.0: 型チェックは v42.3.0

// 変更後:
Item::CepPatternDef(cd) => self.check_cep_pattern_def(cd), // v42.3.0
```

### 2-B: `check_cep_pattern_def` メソッド追加

`check_abstract_trf_def` メソッドの直前（Pass 2 のメソッド群の先頭付近）に追加:

```rust
/// CEP パターンのセマンティクス検証 (v42.3.0)
/// - within_secs == Some(0) → E0420
fn check_cep_pattern_def(&mut self, cd: &CepPatternDef) {
    for clause in &cd.body {
        if clause.within_secs == Some(0) {
            self.errors.push(TypeError::new(
                "E0420",
                "`within 0` is not valid; use a positive integer (within N where N ≥ 1)",
                clause.span.clone(),
            ));
        }
    }
}
```

---

## T3 — `checker.fav` 設計コメント更新

v42.1.0 で追加した「v42.3.0 以降に実装予定」コメントを「E0420 実装済み（within_secs == 0 の検証）」に更新。

---

## T4 — `driver.rs` 更新

`v42200_tests::cargo_toml_version_is_42_2_0` をスタブ化（**先に行うこと**。旧テストが fail したままだと後続挿入後の cargo test が失敗する）。

`v42300_tests` を `v42200_tests` の直前に追加:

```rust
// -- v42300_tests (v42.3.0) -- CEP checker.fav 統合 --
#[cfg(test)]
mod v42300_tests {
    #[test]
    fn cargo_toml_version_is_42_3_0() {
        // NOTE: この assert は次バージョン bump 時にスタブ化すること
        let cargo = include_str!("../Cargo.toml");
        assert!(cargo.contains("42.3.0"), "Cargo.toml must contain version 42.3.0");
    }

    #[test]
    fn cep_e0420_within_zero() {
        // `within 0` は lexer が `Int(0)` を生成しパース成功する（v42.1.0 既存パターンと同形式）
        // checker で within_secs == Some(0) を検出し E0420 を返すことを確認
        use crate::frontend::parser::Parser;
        use crate::middle::checker::Checker;
        let src = r#"cep pattern P { Login within 0 }"#;
        let prog = Parser::parse_str(src, "test.fav").expect("parse ok");
        let (errors, _) = Checker::check_program(&prog);
        assert_eq!(errors.len(), 1, "expected 1 error, got: {:?}", errors);
        assert_eq!(errors[0].code, "E0420");
    }

    #[test]
    fn e0420_in_error_catalog() {
        use crate::error_catalog;
        assert!(
            error_catalog::lookup("E0420").is_some(),
            "E0420 must be registered in error_catalog"
        );
    }
}
```

---

## T5 — Cargo.toml バージョン bump

`version = "42.2.0"` → `"42.3.0"`

---

## T6 — CHANGELOG.md 更新

`[v42.3.0]` エントリを `[v42.2.0]` の直前に追加。

---

## T7 — `cargo test` 実行・確認

- 2883 passed, 0 failed を確認
- `v42300_tests` 3 件 pass を確認

---

## T8 — バージョン管理ドキュメント更新

- `versions/current.md` を v42.3.0（最新安定版）・v42.4.0（次に切る版）に更新
- `versions/roadmap/roadmap-v42.1-v43.0.md` の v42.3.0 を `✅ COMPLETE（2026-07-12）` にマーク
- `versions/v40-v45/v42.3.0/tasks.md` を COMPLETE ステータスに更新（全チェックボックス `[x]`）
