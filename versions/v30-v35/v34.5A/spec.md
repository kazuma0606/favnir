# v34.5A — Spec

## 概要

**テーマ**: コンパイラレベルの `!Effect` 非推奨化（checker.rs + ast.rs）

**バージョン番号**: Cargo.toml `35.1.0`（プロジェクト追跡名 v34.5A）

**背景**: v34.5.0 では lint レベル（W022）での警告のみ実装した。
ロードマップが要求するコンパイラレベルの変更（`ast.rs` + `checker.rs`）が未実装のまま v35.0.0 に到達した。
本バージョンはその補完実装（差分）である。

---

## 実装されていなかった要件（ロードマップ原文）

`versions/roadmap/roadmap-v33.1-v34.0.md` の「移行対象（破壊的変更）」より:

> 1. **コンパイラ・型チェッカー**
>    - `fav/src/ast.rs` — `Effect` enum を deprecated 化（最終的に削除）
>    - `fav/src/middle/checker.rs` — `!Effect` チェックロジックを ctx 検査に置換

v34.5.0 では `lint.rs` の W022 のみが実装され、上記 2 点は未実装だった。

---

## 実装スコープ

### 変更ファイル

1. `fav/Cargo.toml` — version `35.0.0` → `35.1.0`
2. `fav/src/ast.rs` — `Effect` enum に `is_deprecated()` メソッドを追加
3. `fav/src/middle/checker.rs` — FnDef チェック時に `!Effect` 使用を検出し deprecation 診断を発行
4. `fav/src/driver.rs` — `cargo_toml_version_is_35_0_0` をスタブ化、`v35100_tests` 5 件追加
5. `benchmarks/v35.1.0.json` — 新規作成
6. `CHANGELOG.md` — `[v35.1.0]` セクション先頭追記
7. `versions/current.md` — 最新安定版を v35.1.0 に更新

---

## ast.rs — Effect::is_deprecated() 仕様

`Effect` enum に以下のメソッドを追加する:

```rust
impl Effect {
    /// v35.1.0: !Effect アノテーションは非推奨。Pure 以外のすべての Effect が対象。
    pub fn is_deprecated(&self) -> bool {
        !matches!(self, Effect::Pure)
    }
}
```

**要件**:
- `Effect::Pure` → `false`
- それ以外すべて → `true`（実装は `!matches!(self, Effect::Pure)` で包括的に判定する）
  対象 variant: Io, Db, DbRead, DbWrite, DbAdmin, Network, Http, Llm, Snowflake, Gcp,
  Stream, Postgres, Redis, MySQL, MongoDB, DynamoDB, Elasticsearch,
  AzureDb, AzureStorage, Rpc, File, Checkpoint, Trace, PipelineState,
  Emit(_), EmitUnion(_), Unknown(_)

---

## checker.rs — 非推奨 !Effect 診断仕様

FnDef の型チェック時（`check_fn_def` 相当の処理）に以下を実行する:

```rust
// 非推奨 !Effect 警告（v35.1.0）
for eff in &fd.effects {
    if eff.is_deprecated() {
        self.type_warning(
            "W022",
            format!(
                "function `{}` uses deprecated `!Effect` annotation \
                 — migrate to Capability Context using `fav migrate --from-effects`",
                fd.name
            ),
            &fd.span,
        );
        break; // 関数ごとに最大 1 警告
    }
}
```

**注意**:
- `self.type_warning(code, msg, span)` を使用（`fav/src/middle/checker.rs:1502` の既存 API）
- `TypeWarning::new(code, msg, span)` を内部で使う。`CheckWarning` という構造体は存在しない
- エラーではなく **警告**（型チェック失敗にはしない）
- 同一関数への重複警告を避けるため `break` を使用
- W022 は lint.rs と同一コードを使う

---

## テスト仕様（v35100_tests）

```rust
// ── v35.1.0 tests (v34.5A supplement: ast.rs is_deprecated + checker.rs W022) ──
#[cfg(test)]
mod v35100_tests {
    #[test]
    fn cargo_toml_version_is_35_1_0() {
        let src = include_str!("../Cargo.toml");
        assert!(src.contains("35.1.0"), "Cargo.toml must contain '35.1.0'");
    }

    #[test]
    fn ast_effect_is_deprecated_exists() {
        let src = include_str!("ast.rs");
        assert!(
            src.contains("is_deprecated"),
            "ast.rs must define Effect::is_deprecated()"
        );
    }

    #[test]
    fn effect_pure_is_not_deprecated() {
        assert!(
            !crate::ast::Effect::Pure.is_deprecated(),
            "Effect::Pure must not be deprecated"
        );
    }

    #[test]
    fn effect_http_is_deprecated() {
        assert!(
            crate::ast::Effect::Http.is_deprecated(),
            "Effect::Http must be deprecated"
        );
    }

    #[test]
    fn checker_has_effect_deprecation_check() {
        let src = include_str!("middle/checker.rs");
        assert!(
            src.contains("is_deprecated"),
            "checker.rs must call Effect::is_deprecated() for deprecation check"
        );
    }
}
```

### 設計注記

- `v35100_tests` は `v350000_tests` 直後・`// ── v31.7.0 tests` の前に挿入
- `use super::*` なし（`crate::ast::Effect` を絶対パスで参照）
- WASM ゲートなし

---

## 完了条件

- [ ] `Cargo.toml` version = `"35.1.0"`
- [ ] `cargo_toml_version_is_35_0_0` が空スタブになっていること
- [ ] `ast.rs` に `Effect::is_deprecated()` が定義されていること
- [ ] `checker.rs` に `is_deprecated()` を呼び出す deprecation 警告ロジックが存在すること
- [ ] `cargo test --bin fav v35100` — 5/5 PASS
- [ ] `cargo test` — 全件 PASS（2591 件想定 = 2586 + 5、0 failures）
- [ ] `cargo clippy --locked -- -D warnings` — PASS
- [ ] `CHANGELOG.md` に `[v35.1.0]` セクション
- [ ] `benchmarks/v35.1.0.json` 存在かつ `tests_passed` が実測値
- [ ] `versions/current.md` が v35.1.0 に更新されていること
- [ ] `tasks.md` が COMPLETE
