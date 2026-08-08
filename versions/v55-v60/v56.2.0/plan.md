# Plan — v56.2.0 — 境界付きジェネリクス Phase 2（複数 constraint・coherence 強化）

## ゴール

- E0423（coherence violation）を `error_catalog.rs` に正式登録
- `checker.rs` に coherence check を追加（built-in impl は対象外）
- 複数 `with` bound の動作確認テスト追加
- 3229 → 3231 tests

---

## 実装ステップ

### Phase 1: Cargo.toml バージョン更新

`56.0.0`（v56.1.0 で未反映）→ `56.2.0` に更新。

### Phase 2: error_catalog.rs に E0423 追加

E0422 エントリの直後に挿入する。

- `code: "E0423"`
- `title: "duplicate impl: coherence violation"`
- `category: "types"`
- `example` は正しい Favnir 構文を使う:
  - interface 宣言: `hello: Self -> String`（コロン形式）
  - impl メソッド: `hello = |s| "hello"`（等号形式）

### Phase 3: checker.rs 変更

1. `InterfaceRegistry` に `is_explicitly_implemented` メソッドを追加
   - `is_implemented` の変形: `entry.is_auto == false` の場合のみ `true`
   - これにより built-in（stdlib）impl を coherence check の対象外とする

2. `check_interface_impl_decl` 内、`register_impl` 呼び出しの直前に coherence check を挿入
   - 条件: `!id.is_auto && is_explicitly_implemented(interface, type)`
   - E0423 を emit し `continue` でスキップ（`// skip registration — duplicate impl rejected` コメント付き）

### Phase 4: driver.rs 変更

1. `v56000_tests` から `cargo_toml_version_is_56_0_0` を削除（Cargo.toml が 56.2.0 になるため）
2. `v56200_tests` モジュールを `v56100_tests` の直前に追加:
   - `cargo_toml_version_is_56_2_0`
   - `where_multiple_constraints`（複数 bound 正常系 — `errors.is_empty()`）
   - `impl_coherence_violation`（重複 impl → E0423 assert）

### Phase 5: ポスト処理

- `CHANGELOG.md` に v56.2.0 エントリを追加（`Cargo.toml version: 56.1.0 → 56.2.0`）
- `versions/current.md` を v56.2.0 / 3231 tests に更新
- 両ロードマップを COMPLETE に更新

---

## テスト戦略

| テスト | 検証内容 |
|--------|---------|
| `cargo_toml_version_is_56_2_0` | バージョン更新の確認 |
| `where_multiple_constraints` | `Int with Ord with Serialize` の両 bound を満たす → `errors.is_empty()` |
| `impl_coherence_violation` | Greet for Foo を 2 回 impl → E0423 確認 |
| 既存 3229 件全通過 | built-in impl が E0423 を発行しないことを確認 |

---

## リスク管理

| リスク | 対策 |
|--------|------|
| built-in impl が誤って E0423 を発行する | `is_explicitly_implemented`（`!is_auto`）で除外 |
| impl ブロック構文ミス（`: ` vs `=`） | 実際の parser を確認: impl は `method = expr`、interface は `method: Type` |
| Cargo.toml の段飛び（56.0.0 → 56.2.0） | v56.1.0 で未更新だったため直接 56.2.0 へ更新 |
| `continue` の意図が不明 | `// skip registration — duplicate impl rejected` コメントで明示 |
