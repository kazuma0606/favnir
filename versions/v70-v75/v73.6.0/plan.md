# v73.6.0 実装計画 — Rune 品質パス（スタブ実装 → VM primitive 接続）

Date: 2026-08-13
Status: 計画中

---

## 前提確認

- `fav/Cargo.toml` version = "73.5.0"
- `cargo test` 3657 tests pass（0 failures）
- `driver.rs` に `v735000_tests` が存在する

---

## 実装ステップ

### Step 1: `RuneLinalgMatrix` 構造体 + `rune_linalg_matmul` 追加

`driver.rs` の v73.5.0 実装コードの直後（`v735000_tests` より前）に追加:

```rust
// --- v73.6.0: Rune Quality Pass (VM primitive connection) ---

pub struct RuneLinalgMatrix {
    pub rows: usize,
    pub cols: usize,
    pub data: Vec<f64>,
}

pub fn rune_linalg_matmul(a: &RuneLinalgMatrix, b: &RuneLinalgMatrix) -> Result<RuneLinalgMatrix, String> {
    if a.cols != b.rows {
        return Err(format!(
            "dimension mismatch: a.cols={} != b.rows={}",
            a.cols, b.rows
        ));
    }
    let mut result = vec![0.0f64; a.rows * b.cols];
    for i in 0..a.rows {
        for j in 0..b.cols {
            let mut sum = 0.0;
            for k in 0..a.cols {
                sum += a.data[i * a.cols + k] * b.data[k * b.cols + j];
            }
            result[i * b.cols + j] = sum;
        }
    }
    Ok(RuneLinalgMatrix { rows: a.rows, cols: b.cols, data: result })
}
```

### Step 2: `RuneStatsResult` 構造体 + `rune_stats_mean_std` 追加

Step 1 の直後に追加:

```rust
pub struct RuneStatsResult {
    pub mean: f64,
    pub std: f64,
    pub count: usize,
}

pub fn rune_stats_mean_std(values: &[f64]) -> Result<RuneStatsResult, String> {
    if values.is_empty() {
        return Err("empty input: cannot compute mean/std".to_string());
    }
    let count = values.len();
    let mean = values.iter().sum::<f64>() / count as f64;
    let variance = values.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / count as f64;
    Ok(RuneStatsResult { mean, std: variance.sqrt(), count })
}
```

### Step 3: `cargo build` 確認

### Step 4: `v736000_tests` モジュール追加

`v735000_tests` の直後に追加:

```rust
#[cfg(test)]
mod v736000_tests {
    use super::{RuneLinalgMatrix, rune_linalg_matmul, rune_stats_mean_std};

    #[test]
    fn rune_linalg_matmul_runs() {
        // 2x2 * 2x2 の積
        let a = RuneLinalgMatrix { rows: 2, cols: 2, data: vec![1.0, 2.0, 3.0, 4.0] };
        let b = RuneLinalgMatrix { rows: 2, cols: 2, data: vec![5.0, 6.0, 7.0, 8.0] };
        let c = rune_linalg_matmul(&a, &b).expect("2x2 matmul should succeed");
        assert_eq!(c.rows, 2);
        assert_eq!(c.cols, 2);
        // [[1*5+2*7, 1*6+2*8], [3*5+4*7, 3*6+4*8]] = [[19, 22], [43, 50]]
        assert!((c.data[0] - 19.0).abs() < 1e-9);
        assert!((c.data[1] - 22.0).abs() < 1e-9);
        assert!((c.data[2] - 43.0).abs() < 1e-9);
        assert!((c.data[3] - 50.0).abs() < 1e-9);

        // 次元不一致 → Err
        let bad_b = RuneLinalgMatrix { rows: 3, cols: 2, data: vec![0.0; 6] };
        let err = rune_linalg_matmul(&a, &bad_b);
        assert!(err.is_err());
        assert!(err.unwrap_err().contains("dimension mismatch"));
    }

    #[test]
    fn rune_stats_mean_std_runs() {
        // [1.0, 2.0, 3.0, 4.0, 5.0]
        let result = rune_stats_mean_std(&[1.0, 2.0, 3.0, 4.0, 5.0]).expect("should compute stats");
        assert_eq!(result.count, 5);
        assert!((result.mean - 3.0).abs() < 1e-9);
        // 母標準偏差: sqrt(((1-3)^2 + (2-3)^2 + (3-3)^2 + (4-3)^2 + (5-3)^2) / 5)
        //           = sqrt((4+1+0+1+4)/5) = sqrt(2.0) ≈ 1.41421356...
        assert!((result.std - 2.0f64.sqrt()).abs() < 1e-9);

        // 1 要素 → std = 0.0
        let single = rune_stats_mean_std(&[42.0]).expect("single element should work");
        assert!((single.mean - 42.0).abs() < 1e-9);
        assert!((single.std - 0.0).abs() < 1e-9);

        // 空リスト → Err
        let empty = rune_stats_mean_std(&[]);
        assert!(empty.is_err());
        assert!(empty.unwrap_err().contains("empty input"));
    }
}
```

### Step 5: `cargo test v736000` で 2 件 pass 確認

### Step 6: バージョン更新

- `fav/Cargo.toml`: version = "73.5.0" → "73.6.0"
- `driver.rs`: `"73.5.0"` → `"73.6.0"`（replace_all）
  ※ バージョン検証テスト（`cargo_toml_version_is_*`）内の文字列リテラルも対象
  ※ ただし `// --- v73.5.0:` 等のコメントヘッダーおよびセクション見出しは書き換えないこと（replace_all は文字列リテラル `"73.5.0"` のみ対象のため問題なし）

### Step 7: `cargo build` 確認

### Step 8: `cargo test` 全体確認（3659 tests pass）

### Step 9: `CHANGELOG.md` 更新

### Step 10: `versions/current.md` 更新

- 「最終更新」を `2026-08-13 (v73.6.0)` に変更
- 「進行中バージョン」を `v73.6.0` に変更
- 「次に切る版」を `v73.7.0` に変更
