# Plan: v51.0.0 — Developer Experience 3.0 宣言 ★クリーンアップ

## 作業ステップ

### Step 1: `MILESTONE.md` 更新

先頭（v50.0.0 エントリの直前）に v51.0.0 エントリを挿入する。

**追加内容:**

```markdown
## v51.0.0（2026-07-XX）— Developer Experience 3.0

> 「全エラーコードに修正提案が付き、JSON / LSP / CLI で一貫して届く。
>  エディタは型を表示し、trace はパイプラインの流れを可視化する。
>  Favnir の診断は開発者の思考を止めない。
>
>  これが Favnir v51.0 — Developer Experience 3.0 の姿である。」

**Developer Experience 3.0** の宣言バージョン。v50.1〜v50.9 の全機能統合を経て、
診断・エディタ統合・デバッグ体験の成熟を宣言する。

---
```

### Step 2: `README.md` 更新

`README.md` の適切な箇所（バージョン情報またはマイルストーンセクション）に DX 3.0 への言及を追加する。

**追加内容の例:**
```markdown
## マイルストーン

- **v51.0 — Developer Experience 3.0**（2026-07-XX）: 統一診断・LSP インレイヒント・trace/watch 完成
- **v50.0 — Language Maturity / Production 2.0**（2026-07-18）: 言語成熟宣言
```

テスト要件: `content.contains("DX 3.0") || content.contains("Developer Experience 3.0")` が真。

### Step 3: `driver.rs` — `v51000_tests` 追加・v509000_tests 削除

**対象**: `fav/src/driver.rs`（`v509000_tests` モジュールの直前）

**追加 (6 件):**
1. `cargo_toml_version_is_51_0_0`
2. `changelog_has_v51_0_0`
3. `milestone_has_dx3`
4. `readme_mentions_dx3`
5. `dx3_milestone_declared`
6. `code_freeze_v51_0_0`

**削除 (v509000_tests から 2 件):**
- `cargo_toml_version_is_50_9_0`（`"50.9.0"` assert → v51.0.0 では FAIL）
- `code_freeze_v50_9_0`（`"50.9.0"` assert → v51.0.0 では FAIL）

v509000_tests に残る 1 件（`dx3_overview_doc_exists`）は保持。

```rust
// -- v51000_tests (v51.0.0) -- Developer Experience 3.0 宣言 --
#[cfg(test)]
mod v51000_tests {
    #[test]
    fn cargo_toml_version_is_51_0_0() {
        let content = include_str!("../Cargo.toml");
        assert!(content.contains("version = \"51.0.0\""),
            "Cargo.toml version should be 51.0.0");
    }

    #[test]
    fn changelog_has_v51_0_0() {
        let content = include_str!("../../CHANGELOG.md");
        assert!(content.contains("v51.0.0"),
            "CHANGELOG.md must have v51.0.0 entry");
    }

    #[test]
    fn milestone_has_dx3() {
        let content = include_str!("../../MILESTONE.md");
        assert!(content.contains("Developer Experience 3.0"),
            "MILESTONE.md must mention Developer Experience 3.0");
    }

    #[test]
    fn readme_mentions_dx3() {
        let content = include_str!("../../README.md");
        assert!(
            content.contains("DX 3.0") || content.contains("Developer Experience 3.0"),
            "README.md must mention DX 3.0"
        );
    }

    #[test]
    fn dx3_milestone_declared() {
        let content = include_str!("../../MILESTONE.md");
        assert!(content.contains("v51.0"), "MILESTONE.md must have v51.0 entry");
        assert!(
            content.contains("診断は開発者の思考を止めない") || content.contains("Developer Experience 3.0"),
            "MILESTONE.md must contain DX 3.0 declaration"
        );
    }

    #[test]
    fn code_freeze_v51_0_0() {
        // v51.0.0 コードフリーズ宣言テスト（v509000_tests::code_freeze_v50_9_0 の後継）。
        // cargo_toml_version_is_51_0_0 と意図的に同じ assert を持つ。
        // 次バージョンアップ時は cargo_toml_version_is_X と本テストの両方を更新すること。
        let content = include_str!("../Cargo.toml");
        assert!(content.contains("version = \"51.0.0\""), "code freeze: version must be 51.0.0");
    }
}
```

### Step 4: `Cargo.toml` バージョン更新

`fav/Cargo.toml`: `version = "50.9.0"` → `version = "51.0.0"`

### Step 5: テスト・Lint 確認

```bash
cd /c/Users/yoshi/favnir/fav && cargo test -j 8 -- --test-threads=8 2>&1 | tail -5
cargo clippy -- -D warnings 2>&1 | tail -5
```

期待: 3113 tests passed, 0 failed、clippy 警告 0

### Step 6: ★クリーンアップ（`cargo clean`）

```bash
cd /c/Users/yoshi/favnir/fav && cargo clean
```

> **注意**: `cargo clean` 後に `fav/tmp/hello.fav` が消える場合がある。
> `bootstrap_c2_artifact_roundtrip` テスト用に `hello.fav` を復元する必要がある。
> 内容: `fn add(a: Int, b: Int) -> Int { a + b }` + `fn main() -> Bool { add(1, 2) == 3 }`
> `cargo clean` 後に `cargo test` を再実行して 3113 tests を確認すること。

### Step 7: CHANGELOG・current.md・roadmap 更新

- `CHANGELOG.md` に v51.0.0 エントリ追加
- `versions/current.md` を v51.0.0（3113 tests）に更新
- `versions/roadmap/roadmap-v50.1-v51.0.md` の v51.0.0 実績欄を更新
  - このロードマップファイルは v51.0.0 をもって完了（スプリント終了）。次スプリントは `roadmap-v51.1-v55.0.md`（または同等）を参照。

---

## ファイル変更一覧

| ファイル | 変更内容 |
|---|---|
| `MILESTONE.md` | v51.0.0 エントリ追加（先頭） |
| `README.md` | DX 3.0 マイルストーン言及追加 |
| `fav/src/driver.rs` | v51000_tests 追加（6件）+ v509000_tests から 2 件削除 |
| `fav/Cargo.toml` | version → `51.0.0` |
| `CHANGELOG.md` | v51.0.0 エントリ追加 |
| `versions/current.md` | v51.0.0 更新 |
| `versions/roadmap/roadmap-v50.1-v51.0.md` | v51.0.0 実績欄更新 |
| `versions/v50-v55/v51.0.0/tasks.md` | COMPLETE に更新 |

---

## リスク・注意点

- `cargo clean` 後に `fav/tmp/hello.fav` が消える可能性あり（過去 v30.0.0 等で発生）。`cargo clean` 後は `cargo test` で全通過を必ず再確認すること。
- `include_str!("../../CHANGELOG.md")` / `include_str!("../../MILESTONE.md")` / `include_str!("../../README.md")` は `fav/src/driver.rs` 起点で `../../` = `favnir/` ルート。各ファイルの位置を確認済み。
- `README.md` の変更は `readme_mentions_dx3` テストが要求する `"DX 3.0"` または `"Developer Experience 3.0"` のいずれかが含まれることを確認してから変更すること（テスト先行確認）。
- `dx3_milestone_declared` テストは MILESTONE.md の 2 つの assert を使用。どちらか一方が満たされれば OK（OR 条件）。
