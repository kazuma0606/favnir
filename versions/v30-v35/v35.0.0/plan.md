# v35.0.0 — 実装プラン

## 方針

Production Ready マイルストーン宣言パターン。
`cargo clean` 必須（x.0.0 ルール）。

**前提**: v34.9.0 完了済み（2581 tests passed）。

---

## 実装ステップ

### Step 0: cargo clean（必須 — 最初に実施）

```bash
cd /c/Users/yoshi/favnir/fav
cargo clean
cargo build 2>&1 | tail -3
```

`cargo build` が通ることを確認してから次へ進む。

---

### Step 1: Cargo.toml バージョン更新

`fav/Cargo.toml` の version を `34.9.0` → `35.0.0` に変更。

---

### Step 2: driver.rs 更新

#### 2-1. `cargo_toml_version_is_34_9_0` スタブ化

```rust
fn cargo_toml_version_is_34_9_0() {
    // Stubbed: version bumped to 35.0.0 in v35.0.0.
}
```

#### 2-2. `v350000_tests` 挿入

v349000_tests 直後・`// ── v31.7.0 tests` の前に挿入。
`use super::*` なし（`include_str!` のみ使用）。

5 件のテスト:
- `cargo_toml_version_is_35_0_0` — `include_str!("../Cargo.toml")`
- `benchmark_v35_0_0_exists` — `include_str!("../../benchmarks/v35.0.0.json")`
- `milestone_production_ready_declared` — `include_str!("../../MILESTONE.md")`、`"Production Ready"` を assert
- `readme_mentions_v35` — `include_str!("../../README.md")`、`"v35"` を assert
- `real_world_etl_example_exists` — `include_str!("../../examples/real-world-etl/README.md")`、`"30 分"` を assert（固有内容で確認）

---

### Step 3: CHANGELOG.md 更新

先頭に追記:

```markdown
## [v35.0.0] — 2026-07-04

### Added
- **Production Ready マイルストーン宣言**（v34.1〜v34.9 完了）
- `MILESTONE.md` に `v35.0.0 — Production Ready` セクション追加
- `README.md` に v35.0 マイルストーン行追記
- `v350000_tests`: マイルストーン宣言確認テスト 5 件
- `cargo clean` 実施（最終クリーンアップ）

---
```

---

### Step 4: MILESTONE.md 更新

先頭に `v35.0.0 — Production Ready` セクションを追加（spec.md の MILESTONE.md 追加内容セクション参照）。

---

### Step 5: README.md 更新

v34.0 マイルストーン行の直後に v35.0 行を追記:

```markdown
| v35.0 — Production Ready | **完了** | v34.x 完了後（2026-07-04）|
```

README.md の「マイルストーン進捗」テーブルを確認し、v34.0 行の直後に挿入する。
README.md に「マイルストーン進捗」テーブルがない場合は適切な場所に v35 の言及を追加する。

---

### Step 6: benchmarks/v35.0.0.json 作成

```json
{
  "version": "35.0.0",
  "milestone": "Production Ready",
  "date": "2026-07-04",
  "tests_passed": 2586,
  "tests_failed": 0,
  "notes": "Production Ready マイルストーン宣言。cargo clean 実施。v350000_tests 5 件追加。"
}
```

`tests_passed` は実測後に確定値に更新する。

---

### Step 7: versions/current.md 更新

- `最新安定版`: `**v34.9.0**` → `**v35.0.0** — Production Ready 宣言`
- `cargo install` 行: `"34.9.0"` → `"35.0.0"`
- `進行中バージョン`: `なし（v34.9.0 完了直後）` → `なし（v35.0.0 完了 — Production Ready 宣言）`
- `次に切る版`: `**v35.0.0** — Production Ready 宣言` → `未定`
- マイルストーン進捗テーブルの `v35.0` 行: `planned` → `**完了**`

---

### Step 8: テスト実行

```bash
cd /c/Users/yoshi/favnir/fav
cargo test --bin fav v350000 2>&1 | tail -8   # 5/5 PASS を確認
cargo test 2>&1 | grep "test result"           # 2586 passed, 0 failures を確認
cargo clippy --locked -- -D warnings
./target/debug/fav lint --deny-warnings --allow W017 --allow W018 --allow W019 self/compiler.fav
./target/debug/fav lint --deny-warnings --allow W012 --allow W017 --allow W018 --allow W019 self/checker.fav
du -sh target/
```

---

### Step 9: 完了処理

- `benchmarks/v35.0.0.json` の `tests_passed` を実測値で確定更新
- 更新後に `cargo test 2>&1 | grep "test result"` を再実行して PASS を確認
- `tasks.md` を COMPLETE に更新

---

## 注意事項

- `cargo clean` を Step 0 で必ず実施すること（x.0.0 ルール）
- `real_world_etl_example_exists` テストは `examples/real-world-etl/README.md` が v34.1.0 で作成済みであることが前提
- MILESTONE.md には `v35.0.0` と `Production Ready` の両方が含まれるようにすること
