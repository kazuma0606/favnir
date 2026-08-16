# v72.0.0 実装計画 — Type System 2.0 宣言 ★クリーンアップ

Date: 2026-08-11

---

## 依存関係

```
T0（事前確認）
  └→ T1（MILESTONE.md 追記）
       └→ T2（README.md 追記）
            └→ T3（CHANGELOG.md v72.0.0 エントリ追加）
                 └→ T4（v72000_tests 追加 + cargo_toml_version 更新）
                      └→ T5（Cargo.toml バージョン更新）
                           └→ T6（cargo clean）
                                └→ T7（cargo test v72000 確認）
                                     └→ T8（cargo test 全体確認）
                                          └→ T9（versions/current.md 更新）
                                               └→ T10（最終確認）
```

---

## 実装ステップ

### Step 0: 事前確認

- `fav/Cargo.toml` のバージョンが `71.9.0` であることを確認
- `cargo test` が 3608 tests pass であることを確認
- `MILESTONE.md` に「Type System 2.0」がまだ含まれていないことを確認
- `README.md` に「Type System 2.0」または「v72.0」がまだ含まれていないことを確認
- `driver.rs` に `v72000_tests` が未存在であることを確認

### Step 1: `MILESTONE.md` 追記

先頭に v72.0.0 エントリを追加:

```markdown
## v72.0.0（2026-08-11）— Type System 2.0

> 「依存型がベクトルの次元を守り、refined type がゼロ除算を型で止める。
>  Phantom type が ID の混用を防ぎ、定数がコンパイル時に評価される。
>  AOT バイナリが Docker 不要で動き、WASM がパイプラインをブラウザへ運ぶ。
>
>  これが Favnir v72.0 — Type System 2.0 の姿である。」

**Type System 2.0** の宣言バージョン。v71.1〜v71.9 で実装した
依存型・refined type・phantom type・const eval・generic constraints・AOT・WASM・型推論強化の統合を宣言した。

**v71.1〜v71.9 達成内容:**
- 依存型 `Vec<T>[N]`: 次元違いベクトルを型で防止（E0421）
- Refined Types: `type PositiveFloat = Float where self > 0.0`（E0425）
- Phantom Types: `type UserId = phantom String`（ID 混用防止）
- Const Eval: `const EMBED_DIM: Int = 1536`（コンパイル時定数）
- Generic Constraints: `<T: A & B>`、`<T: impl A>`
- AOT Native: `fav build --target native --arch arm64`
- WASM: `fav build --target wasm`（`\0asm` マジック確認）
- 型推論: `bind n <- fn()` 型注釈省略可

---
```

### Step 2: `README.md` 追記

`## v71.0 — Language Complete 1.0 宣言` の直前に追加:

```markdown
## v72.0 — Type System 2.0 宣言（2026-08-11）

Favnir v72.0 で「Type System 2.0」を宣言しました。
依存型・refined type・phantom type・const eval・generic constraints が揃い、
AOT バイナリと WASM により型安全なパイプラインがどこでも動きます。

---
```

### Step 3: `CHANGELOG.md` 追記

先頭に `## [v72.0.0]` エントリを追加。

### Step 4: `v72000_tests` 追加（`driver.rs`）

> **注意**: `driver.rs` に既存の `v71000_tests` モジュールは関数名・assert 文字列が実態と乖離している（例: `cargo_toml_version_is_71_2_0` が `"71.9.0"` を assert）。実装時の参照先はこの spec.md のコードスニペットのみ使用すること。

`v719000_tests` モジュールの直後に追加:

```rust
// ── v72.0.0: Type System 2.0 宣言 ────────────────────────────────────────────
#[cfg(test)]
mod v72000_tests {
    #[test]
    fn cargo_toml_version_is_72_0_0() {
        let src = include_str!("../Cargo.toml");
        assert!(src.contains("version = \"72.0.0\""), "Cargo.toml should declare version 72.0.0");
    }

    #[test]
    fn changelog_has_v72_0_0() {
        let src = include_str!("../../CHANGELOG.md");
        assert!(src.contains("[v72.0.0]"), "CHANGELOG.md should have v72.0.0 entry");
    }

    #[test]
    fn milestone_has_type_system_2() {
        let src = include_str!("../../MILESTONE.md");
        assert!(src.contains("Type System 2.0"), "MILESTONE.md should mention Type System 2.0");
    }

    #[test]
    fn readme_mentions_type_system_2() {
        let src = include_str!("../../README.md");
        assert!(
            src.contains("Type System 2.0") || src.contains("v72.0"),
            "README.md should mention Type System 2.0 or v72.0"
        );
    }
}
```

### Step 5: `Cargo.toml` バージョン更新 + `driver.rs` version アサーション更新

- `fav/Cargo.toml`: `71.9.0` → `72.0.0`
- `driver.rs` の `"71.9.0"` 文字列を `"72.0.0"` に replace_all

### Step 6: `cargo clean`

ビルドアーティファクトを削除（ディスク節約）。

### Step 7: `cargo test v72000` で 4 件 pass 確認

### Step 8: `cargo test` 全体で 3612 tests pass 確認

### Step 9: `versions/current.md` 更新

- 進行中: `v72.0.0`（Type System 2.0 宣言）
- 次: `v72.1.0`

---

## 注意事項

- `cargo clean` は Step 7 のテスト確認後に実施してもよいが、本バージョンでは Step 6 で早期実施
- `cargo clean` 後は `fav/tmp/hello.fav` が消える可能性 → `bootstrap_c2_artifact_roundtrip` テスト失敗リスク。`cargo clean` 後に `hello.fav` を確認・復元すること
- `v72000_tests` は `include_str!` を使用するため `use` は不要
- `cargo_toml_version_is_72_0_0` テストと `driver.rs` の既存 version アサーション（replace_all で更新）は別々のもの
