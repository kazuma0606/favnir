# v79.4.0 実装計画 — Verifiable showcase パイプライン

Date: 2026-08-16

---

## 実装順序

### Step 1: contract.fav 更新

`infra/e2e-demo/favnir3-showcase/contract.fav` の末尾に以下を追加する:

```favnir
// --- Verifiable セクション（v77.x）---
contract Favnir3ShowcaseContract {
    input:     { rows: List<Row> }
    output:    { processed: List<Row> }
    invariant: output.row_count <= input.row_count
    invariant: SUM(output.amount) >= 0.0
    probabilistic_invariant score_dist:
        confidence: 0.95, sample_size: 1000,
        property: AVG(score) BETWEEN 40.0 AND 60.0
}
```

---

### Step 2: CHANGELOG.md 更新（テスト追加より先）

`CHANGELOG.md` の先頭に v79.4.0 エントリを追加。

---

### Step 3: driver.rs — v794000_tests モジュール追加

`fav/src/driver.rs` の末尾に以下を追加:

```rust
// --- v79.4.0: Verifiable showcase パイプライン ---
#[cfg(test)]
mod v794000_tests {
    const CONTRACT: &str = include_str!("../../infra/e2e-demo/favnir3-showcase/contract.fav");

    #[test]
    fn showcase_verifiable_invariants_declared() {
        assert!(CONTRACT.contains("Favnir3ShowcaseContract"), "contract.fav must define Favnir3ShowcaseContract");
        assert!(CONTRACT.contains("invariant"), "contract.fav must declare invariants");
        assert!(CONTRACT.contains("row_count"), "contract.fav must reference row_count invariant");
    }

    #[test]
    fn showcase_verifiable_probabilistic_contract() {
        assert!(CONTRACT.contains("probabilistic_invariant"), "contract.fav must declare probabilistic_invariant");
        assert!(CONTRACT.contains("confidence"), "contract.fav must specify confidence");
        assert!(CONTRACT.contains("sample_size"), "contract.fav must specify sample_size");
    }
}
```

注意: `use super::*` 不要。`const CONTRACT` パターンを採用。

---

### Step 4: Cargo.toml バージョン更新

`fav/Cargo.toml` の `version = "79.3.0"` → `"79.4.0"` に更新。

driver.rs 内の escaped `\"79.3.0\"` を `\"79.4.0\"` に一括更新（sed）。
エラーメッセージ文字列（unescaped）の `79.3.0` も `79.4.0` に更新。

更新後に `grep -c "79\.3\.0" /c/Users/yoshi/favnir/fav/src/driver.rs` → 出力が `1` であることを確認。
（残るのは `// --- v79.3.0: Provenance showcase パイプライン ---` コメント行の 1 件のみ）

---

### Step 5: versions/current.md 更新

- `## 進行中バージョン` → `**v79.4.0**（Verifiable showcase パイプライン）`
- `## 次に切る版` → `**v79.5.0**（Execution Effects showcase パイプライン）`

---

### Step 6: 最終確認

```bash
cargo test v794000 2>&1 | grep -E "test result|FAILED"
cargo test 2>&1 | grep "^test result"
```

3795 tests pass、v794000 2 件 pass を確認。

---

## 依存順序サマリ

```
contract.fav 更新（Step 1）
  → CHANGELOG 更新（Step 2）
  → driver.rs テスト追加（Step 3）← contract.fav が先に更新されていること
  → Cargo.toml + エラーメッセージ更新（Step 4）
  → current.md 更新（Step 5）
  → 最終確認（Step 6）
```
