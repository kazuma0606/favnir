# v80.0.0 実装計画 — Favnir 3.0 宣言 ★クリーンアップ

Date: 2026-08-16

---

## 実装順序

### Step 1: cargo clean（最初に実施）

```bash
cd /c/Users/yoshi/favnir/fav && cargo clean
```

`fav/tmp/hello.fav` が消える場合があるため確認・復元:

```bash
ls /c/Users/yoshi/favnir/fav/tmp/hello.fav 2>/dev/null || echo "MISSING"
```

消えていた場合は以下の内容で復元:

```
fn add(a: Int, b: Int) -> Int { a + b }
fn main() -> Bool { add(1, 2) == 3 }
```

---

### Step 2: CHANGELOG.md 更新（テスト追加より先）

`CHANGELOG.md` の先頭に v80.0.0 エントリを追加:

```
## [v80.0.0] — 2026-08-16 — Favnir 3.0 宣言 ★クリーンアップ

### Declaration
- Favnir 3.0 宣言: 時間・来歴・正しさ・実行戦略がすべて型で語れる言語へ
- v75.1〜v79.9 の全スプリント（Temporal / Provenance / Verifiable / Execution Effects）完了

### Cleanup
- `cargo clean` 実施（ビルドキャッシュ完全クリア）
- MILESTONE.md に Favnir 3.0 宣言を追記
- README.md に v80.0 達成を追記

### Tests
- `cargo_toml_version_is_80_0_0`: バージョン 80.0.0 を確認
- `changelog_has_v80_0_0`: CHANGELOG エントリを確認
- `milestone_has_favnir_3`: MILESTONE.md の Favnir 3.0 宣言を確認
- `readme_mentions_favnir_3`: README.md の Favnir 3.0 記述を確認
```

---

### Step 3: MILESTONE.md 更新

`MILESTONE.md` に Favnir 3.0 宣言セクションを追記。
「Favnir 3.0」という文字列が含まれていること。

---

### Step 4: README.md 更新

`README.md` に v80.0.0 達成（Favnir 3.0）を追記。
「Favnir 3.0」という文字列が含まれていること。

---

### Step 5: driver.rs — v80000_tests モジュール追加

`fav/src/driver.rs` の末尾に以下を追加:

```rust
// --- v80.0.0: Favnir 3.0 宣言 ★クリーンアップ ---
#[cfg(test)]
mod v80000_tests {
    const CARGO_TOML: &str = include_str!("../Cargo.toml");
    const CHANGELOG:  &str = include_str!("../../CHANGELOG.md");
    const MILESTONE:  &str = include_str!("../../MILESTONE.md");
    const README:     &str = include_str!("../../README.md");

    #[test]
    fn cargo_toml_version_is_80_0_0() {
        assert!(CARGO_TOML.contains("version = \"80.0.0\""), "Cargo.toml must be bumped to 80.0.0");
    }

    #[test]
    fn changelog_has_v80_0_0() {
        assert!(CHANGELOG.contains("[v80.0.0]"), "CHANGELOG.md must have v80.0.0 entry");
    }

    #[test]
    fn milestone_has_favnir_3() {
        assert!(MILESTONE.contains("Favnir 3.0"), "MILESTONE.md must document Favnir 3.0 declaration");
    }

    #[test]
    fn readme_mentions_favnir_3() {
        assert!(README.contains("Favnir 3.0"), "README.md must mention Favnir 3.0");
    }
}
```

注意: `use super::*` 不要。`const CARGO_TOML` のパスは `../../Cargo.toml`。

---

### Step 6: Cargo.toml バージョン更新

```bash
sed -i 's/version = "79.9.0"/version = "80.0.0"/' /c/Users/yoshi/favnir/fav/Cargo.toml
```

driver.rs 内の escaped `\"79.9.0\"` を `\"80.0.0\"` に更新:

```bash
sed -i 's/\\"79\.9\.0\\"/\\"80.0.0\\"/g' /c/Users/yoshi/favnir/fav/src/driver.rs
```

driver.rs 内の unescaped エラーメッセージ `79.9.0` を `80.0.0` に更新:

```bash
sed -i 's/79\.9\.0 must be/80.0.0 must be/g' /c/Users/yoshi/favnir/fav/src/driver.rs
```

更新後確認（出力が `1` であること）:

```bash
grep -c "79\.9\.0" /c/Users/yoshi/favnir/fav/src/driver.rs
```

残るのは `// --- v79.9.0: 安定化・コードフリーズ ---` コメント行の 1 件のみ。
（前提: `v80000_tests` モジュール内には `79.9.0` を含む文字列を追加しないこと）

---

### Step 7: versions/current.md 更新

- `## 進行中バージョン` → `**v80.0.0**（Favnir 3.0 宣言 ★クリーンアップ）`
- `## 次に切る版` → 次フェーズ（v80.1〜）の記述に更新
- `## 最新安定版` → `**v80.0.0** — Favnir 3.0 宣言 ★クリーンアップ — 3809 tests` に更新する

---

### Step 8: roadmap-v79.1-v80.0.md 更新

v80.0.0 スプリントを「完了」に更新（コメント追記）。

---

### Step 9: 最終確認

```bash
cargo test v80000 2>&1 | grep -E "test result|FAILED"
cargo test 2>&1 | grep "^test result"
```

3809 tests pass、v80000 4 件 pass を確認。

---

## 依存順序サマリ

```
cargo clean（Step 1）
  → CHANGELOG 更新（Step 2）
  → MILESTONE.md 更新（Step 3）
  → README.md 更新（Step 4）
  → driver.rs テスト追加（Step 5）
  → Cargo.toml バージョン更新（Step 6）
  → current.md 更新（Step 7）
  → ロードマップ更新（Step 8）
  → 最終確認（Step 9）
```
