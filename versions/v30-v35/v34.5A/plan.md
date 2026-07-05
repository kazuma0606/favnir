# v34.5A — 実装プラン

## 方針

`ast.rs` に `Effect::is_deprecated()` を追加し、`checker.rs` で FnDef チェック時に deprecation 警告を発行する。
`cargo clean` は x.1.0 のため不要。

---

## 実装ステップ

### Step 1: Cargo.toml バージョン更新

`fav/Cargo.toml` の version を `35.0.0` → `35.1.0` に変更。

---

### Step 2: ast.rs — Effect::is_deprecated() 追加

`fav/src/ast.rs` の `Effect` enum の定義後（`impl Effect` ブロックが存在する場合はその末尾、
存在しない場合は `enum Effect { ... }` の直後）に追加:

```rust
impl Effect {
    /// v35.1.0: !Effect アノテーションは非推奨。Pure 以外のすべての Effect が対象。
    pub fn is_deprecated(&self) -> bool {
        !matches!(self, Effect::Pure)
    }
}
```

挿入位置の確認:

```bash
grep -n "^impl Effect\|^pub enum Effect\|^}" fav/src/ast.rs | head -20
```

---

### Step 3: checker.rs — 非推奨 !Effect 診断追加

`fav/src/middle/checker.rs` の FnDef 処理部分を特定し、非推奨警告を追加する。

#### 3.1 挿入位置の特定

```bash
grep -n "check_fn_def\|FnDef\|fd\.effects\|fd\.name" fav/src/middle/checker.rs | head -30
```

#### 3.2 既存の警告発行 API の確認

```bash
grep -n "push_warning\|diag\|Warning\|warn" fav/src/middle/checker.rs | head -20
```

#### 3.3 警告ロジックの挿入

FnDef 処理の冒頭（`fd.effects` を参照する箇所の近く）に追加:

```rust
// v35.1.0: 非推奨 !Effect 警告
// checker.rs:1502 の既存 API: fn type_warning(&mut self, code: &'static str, msg: impl Into<String>, span: &Span)
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

**重要**: 既存の `!Effect` チェックロジックは**削除しない**（移行期間中は警告のみ）。
エラーにするのではなく `self.type_warning()` で警告として発行する。
`CheckWarning` 構造体は存在しない。`TypeWarning` を使う（`type_warning` メソッド経由）。

---

### Step 4: driver.rs 更新

#### 4.0 crate::ast::Effect の可視性確認

driver.rs テストから `crate::ast::Effect::Http` を参照するため、事前に確認:

```bash
grep -n "^pub mod ast\|^mod ast" fav/src/lib.rs
```

`pub mod ast` が存在すれば問題なし。存在しない場合は driver.rs と同一クレートルートのどこかで
`pub use ast::Effect;` 等が必要になるが、既存テスト（`crate::lint::check_w022_deprecated_effect_annotation`
経由で `Effect` を扱う v345000_tests）が通っているため通常は問題ない。

#### 4.1 cargo_toml_version_is_35_0_0 をスタブ化

```bash
grep -n "cargo_toml_version_is_35_0_0" fav/src/driver.rs
```

#### 4.2 v35100_tests の挿入位置確認

```bash
grep -n "v350000_tests\|// ── v31\.7\.0 tests" fav/src/driver.rs
```

`v350000_tests` の終端 `}` の直後に以下を挿入:

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
        let src = include_str!("../ast.rs");
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
        let src = include_str!("../middle/checker.rs");
        assert!(
            src.contains("is_deprecated"),
            "checker.rs must call Effect::is_deprecated() for deprecation check"
        );
    }
}
```

---

### Step 5: cargo build + テスト実行

```bash
cd /c/Users/yoshi/favnir/fav
cargo build 2>&1 | tail -5
cargo test --bin fav v35100 2>&1 | tail -8
cargo test 2>&1 | grep "test result"
cargo clippy --locked -- -D warnings 2>&1 | tail -5
```

---

### Step 6: CHANGELOG.md 更新

先頭に追記:

```markdown
## [v35.1.0] — 2026-07-04

### Added
- `fav/src/ast.rs` — `Effect::is_deprecated()` メソッド追加
- `fav/src/middle/checker.rs` — `!Effect` 使用時の deprecation 警告（W022 相当）を型チェック時に発行

### Notes
- v34.5A 補完実装: ロードマップが要求したコンパイラレベルの !Effect 非推奨化を実施
- lint.rs の W022（v34.5.0 で実装済み）と組み合わせることで、`fav lint` / `fav check` 両方で警告が出る
```

---

### Step 7: benchmarks/v35.1.0.json 作成

```json
{
  "version": "35.1.0",
  "milestone": "Production Ready",
  "date": "2026-07-04",
  "tests_passed": 2591,
  "tests_failed": 0,
  "notes": "v34.5A補完: ast.rs Effect::is_deprecated() + checker.rs 非推奨警告。v35100_tests 5 件追加。"
}
```

（`tests_passed` は `cargo test` 実測後に確定）

---

### Step 8: versions/current.md 更新

- `最新安定版` → `**v35.1.0** — !Effect 非推奨化（コンパイラレベル）`
- `次に切る版` → `**v35.2.0** — Rune ファイル ctx 移行（v34.6A）`

---

## テスト実行

```bash
cd /c/Users/yoshi/favnir/fav
cargo test --bin fav v35100 2>&1 | tail -8
cargo test 2>&1 | grep "test result"
```

---

## 完了処理

- `benchmarks/v35.1.0.json` の `tests_passed` を実測値で確定
- `tasks.md` を COMPLETE に更新
