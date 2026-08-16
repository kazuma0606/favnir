# v77.8.0 仕様書 — Probabilistic contracts

Date: 2026-08-16
Status: 計画中

---

## Background

確率的にしか検証できない不変条件（サンプリングベース）を表現する型・関数基盤を追加する。大規模データパイプラインでは全件検証が現実的でないため、サンプル平均が目標範囲内にあるかをサンプリングで検証する。`ProbabilisticContract` 構造体と `check_probabilistic_invariant` 関数を追加する。

---

## Goals

1. `ProbabilisticContract` 構造体（name: String, confidence: f64, sample_size: usize）を追加する
2. `check_probabilistic_invariant(samples: &[f64], target_min: f64, target_max: f64, contract: &ProbabilisticContract) -> Result<(), String>` を追加する
3. Rust テスト 2 件を追加し 3752 tests に到達する

---

## 型・関数仕様

### `ProbabilisticContract` 構造体

```rust
#[derive(Debug, Clone, PartialEq)]
pub struct ProbabilisticContract {
    pub name:        String,
    pub confidence:  f64,
    pub sample_size: usize,
}
```

| フィールド | 型 | 説明 |
|---|---|---|
| `name` | String | 契約名（違反メッセージに含まれる） |
| `confidence` | f64 | 信頼水準（0.0〜1.0）。v77.8.0 では統計的検定には使用せずメタデータとして保持。将来の t 検定対応（v78.x 以降）で利用する。 |
| `sample_size` | usize | 期待サンプルサイズ。同様に v77.8.0 ではメタデータとして保持。 |

> **設計注記**: `confidence: f64` を含むため `Eq` は derive しない。`PartialEq` は derive する（v77.1〜v77.5 の型と一貫した方針。NaN 等価性の問題はあるが、テスト用途では許容）。

---

### `check_probabilistic_invariant`

```rust
pub fn check_probabilistic_invariant(
    samples: &[f64],
    target_min: f64,
    target_max: f64,
    contract: &ProbabilisticContract,
) -> Result<(), String>
```

**動作:**
1. `samples.is_empty()` の場合 → `Err(format!("probabilistic invariant '{}': samples is empty", contract.name))`
2. サンプル平均を計算: `mean = samples.iter().sum::<f64>() / samples.len() as f64`
3. `mean >= target_min && mean <= target_max` の場合 → `Ok(())`
4. 範囲外の場合 → `Err(format!("probabilistic invariant '{}' violated: avg={:.4} not in [{:.4}, {:.4}] (confidence={:.2}, sample_size={})", contract.name, mean, target_min, target_max, contract.confidence, contract.sample_size))`

> **設計注記**: v77.8.0 では `confidence` を統計的検定（t 検定・信頼区間）には使用しない。サンプル平均が目標範囲内かを直接確認するシンプルな実装。将来の CLI 統合（v78.x 以降）で統計的検定を追加する。

---

## テスト仕様

### `probabilistic_contract_passes`

```rust
let contract = ProbabilisticContract {
    name:        "score_distribution".to_string(),
    confidence:  0.95,
    sample_size: 10_000,
};
// mean = (40.0 + 60.0 + 50.0) / 3 = 50.0 → [40.0, 60.0] 内
let samples = vec![40.0, 60.0, 50.0];
let result = check_probabilistic_invariant(&samples, 40.0, 60.0, &contract);
assert!(result.is_ok());
```

### `probabilistic_contract_low_confidence_fails`

```rust
let contract = ProbabilisticContract {
    name:        "score_distribution".to_string(),
    confidence:  0.95,
    sample_size: 10_000,
};
// mean = (10.0 + 20.0 + 15.0) / 3 = 15.0 → [40.0, 60.0] 外
let samples = vec![10.0, 20.0, 15.0];
let result = check_probabilistic_invariant(&samples, 40.0, 60.0, &contract);
assert!(result.is_err());
let msg = result.unwrap_err();
assert!(msg.contains("score_distribution"));
assert!(msg.contains("violated"));
```

---

## Success Criteria

- `ProbabilisticContract` 構造体が定義されている（Debug / Clone / PartialEq 付き、Eq は付与しない）
- `check_probabilistic_invariant` がサンプル平均を `[target_min, target_max]` と比較し、範囲内 → `Ok(())`、範囲外 → `Err(String)` を返す
- 空サンプルスライスに対して `Err("...samples is empty...")` を返す（実装上の動作保証。専用テストは追加しない — テスト数 3752 上限のため）
- `probabilistic_contract_passes` が pass
- `probabilistic_contract_low_confidence_fails` が pass
- `cargo test` が 3752 tests all pass
- `driver.rs` 内の `cargo_toml_version_is_X` 系テストの `77.7.0` バージョン文字列アサーションがすべて `77.8.0` に更新されている（セクションコメント `// --- v77.7.0: 反例自動生成 ---` は変更しない）
- `CHANGELOG.md` の先頭に v77.8.0 エントリが存在する

---

## 変更ファイル

実装順序は plan.md 参照。

- `fav/src/driver.rs` — `ProbabilisticContract`, `check_probabilistic_invariant`, `v778000_tests` を追加
- `CHANGELOG.md` — v77.8.0 エントリを追加
- `versions/current.md` — 進行中バージョンを更新
- `fav/Cargo.toml` — バージョンを `77.7.0` → `77.8.0` に更新
- `fav/Cargo.lock` — `Cargo.toml` バージョン更新に伴い自動更新（手動編集不要）

---

## 依存

- `InvariantViolation`（v77.1.0 定義済み）は v77.8.0 では使用しない。`check_probabilistic_invariant` は `Result<(), String>` を返す（エラー詳細は String で記述するため）
- `AggregateInvariant` / `check_aggregate_invariant`（v77.3.0）との関係: 同じ「集約値検証」だが独立した実装。`ProbabilisticContract` は全件ではなくサンプリングベース

---

## 対象外

- 統計的検定（t 検定・信頼区間計算）: 将来の v78.x 以降で実装
- `confidence` / `sample_size` フィールドの実際の統計的利用: v77.8.0 ではメタデータとして保持のみ
- `parser.rs` / `ast.rs` / `checker.rs` への変更は一切行わない
