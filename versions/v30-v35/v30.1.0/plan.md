# v30.1.0 — 実装計画

## 前提確認（T0）

実装開始前に以下のコマンドを実行して確認すること:

```bash
cd /c/Users/yoshi/favnir/fav

# バージョン確認
grep '^version' Cargo.toml
# → version = "30.0.0" が出力されること

# テスト数確認
cargo test --bin fav 2>&1 | grep "^test result"
# → 2372 passed, 0 failed が出力されること

# v301000_tests 未存在確認
grep -c 'v301000_tests' src/driver.rs || echo "not found"
# → not found または 0 が出力されること

# [profile.dev] 未存在確認
grep -c 'profile.dev' Cargo.toml || echo "not found"
# → not found または 0 が出力されること
```

- [ ] `fav/Cargo.toml` の version が `30.0.0` であること
- [ ] `cargo test --bin fav 2>&1 | grep "^test result"` が `2372 passed` を含むこと
- [ ] `driver.rs` に `mod v301000_tests` が存在しないこと
- [ ] `fav/Cargo.toml` に `[profile.dev]` セクションが存在しないこと
- [ ] v30.0.0 が COMPLETE であること

---

## 実装ステップ

### Step 1 — Cargo.toml バージョン更新

**対象ファイル:** `fav/Cargo.toml`

```toml
# 変更前
version = "30.0.0"

# 変更後
version = "30.1.0"
```

### Step 2 — `[profile.dev]` セクション追加

**対象ファイル:** `fav/Cargo.toml`（末尾に追記）

```toml
[profile.dev]
debug = 0
split-debuginfo = "off"
```

追記位置: `Cargo.toml` の末尾（既存の `[features]` セクションより後）。

**注意点:**
- `[profile.dev]` が既に存在しないか確認してから追加すること（重複定義はビルドエラーになる）
- `split-debuginfo` はプラットフォーム非依存の設定であり Windows でも有効

### Step 3 — ビルド確認

```bash
cd /c/Users/yoshi/favnir/fav
cargo build 2>&1 | tail -3
# → Finished `dev` profile ... が出力されること
```

ビルドが通ることを確認した後、テスト追加に進む。

### Step 4 — `v301000_tests` 追加

**対象ファイル:** `fav/src/driver.rs`（末尾に追加）

```rust
// ── v30.1.0 tests ────────────────────────────────────────────────────────────
#[cfg(test)]
mod v301000_tests {
    #[test]
    fn cargo_toml_version_is_30_1_0() {
        let src = include_str!("../Cargo.toml");
        assert!(
            src.contains("30.1.0"),
            "Cargo.toml must contain '30.1.0'"
        );
    }
    #[test]
    fn cargo_toml_has_profile_dev() {
        let src = include_str!("../Cargo.toml");
        assert!(
            src.contains("[profile.dev]"),
            "Cargo.toml must contain '[profile.dev]'"
        );
    }
    #[test]
    fn profile_dev_debug_is_zero() {
        let src = include_str!("../Cargo.toml");
        assert!(
            src.contains("debug = 0"),
            "Cargo.toml [profile.dev] must contain 'debug = 0'"
        );
    }
    #[test]
    fn profile_dev_split_debuginfo_off() {
        let src = include_str!("../Cargo.toml");
        assert!(
            src.contains("split-debuginfo = \"off\""),
            "Cargo.toml [profile.dev] must contain 'split-debuginfo = \"off\"'"
        );
    }
    #[test]
    fn changelog_has_v30_1_0() {
        let src = include_str!("../../CHANGELOG.md");
        assert!(
            src.contains("[v30.1.0]"),
            "CHANGELOG.md must contain '[v30.1.0]'"
        );
    }
    #[test]
    fn benchmark_v30_1_0_exists() {
        let src = include_str!("../../benchmarks/v30.1.0.json");
        assert!(
            src.contains("30.1.0"),
            "benchmarks/v30.1.0.json must contain '30.1.0'"
        );
    }
}
```

### Step 5 — CHANGELOG.md 更新

**対象ファイル:** `CHANGELOG.md`（先頭に追記）

```markdown
## [v30.1.0] — 2026-07-01

### Added
- `[profile.dev] debug = 0` — デバッグシンボル無効化によりビルド生成物を軽量化
- `[profile.dev] split-debuginfo = "off"` — デバッグ情報分割ファイルを無効化
```

### Step 6 — benchmarks/v30.1.0.json 作成

**対象ファイル:** `benchmarks/v30.1.0.json`（新規作成）

```json
{
  "version": "30.1.0",
  "date": "2026-07-01",
  "test_count": 2378,
  "notes": "ビルド軽量化: [profile.dev] debug=0 / split-debuginfo=off"
}
```

### Step 7 — テスト実行・確認

```bash
cd /c/Users/yoshi/favnir/fav

# v30.1.0 の 6 テストを確認
cargo test --bin fav v301000 2>&1 | tail -5

# 全件確認
cargo test 2>&1 | grep -E "test result|FAILED"
```

### Step 8 — tasks.md 更新

全チェックボックスを `[x]` にして COMPLETE にする。

---

## テスト実行

```bash
cd /c/Users/yoshi/favnir/fav && cargo test -j 8 -- --test-threads=8 2>&1 | tail -5
```

---

## コードレビューチェックリスト

- [ ] セキュリティ: `[profile.dev]` 設定はビルドのみに影響し、実行時の動作・セキュリティに変化なし
- [ ] 副作用: `cargo test` の動作に影響しないこと（デバッグシンボル不要）
- [ ] CI: GitHub Actions の CI ワークフローに影響しないこと
- [ ] 重複: `Cargo.toml` に既存の `[profile.dev]` セクションがないこと
