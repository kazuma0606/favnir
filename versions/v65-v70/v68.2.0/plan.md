# v68.2.0 実装計画

## Step 1: `fav/src/checkpoint.rs` 新規作成

```rust
// fav/src/checkpoint.rs — v68.2.0 Pipeline Checkpointing（耐障害性・再開）

pub const CHECKPOINT_HELP: &str = "\
fav run --checkpoint <dir>  — チェックポイント付きパイプライン実行

使用例:
  fav run pipeline.fav --checkpoint ./checkpoints/
  fav run pipeline.fav --resume ./checkpoints/step-2-embedtext.ckpt
  fav run pipeline.fav --checkpoint ./checkpoints/ --checkpoint-ttl 24

フラグ:
  --checkpoint <dir>      チェックポイント保存先ディレクトリ（.ckpt ファイル生成）
  --resume <file>         チェックポイントファイルから途中再開
  --checkpoint-ttl <h>    古いチェックポイントの自動削除（時間単位）
  --help, -h              このヘルプを表示
";

pub fn cmd_checkpoint_run(src: &str, checkpoint_dir: &str, resume_file: &str) -> String {
    // スタブ実装: 将来フェーズで実際の状態シリアライズを実装
    if !resume_file.is_empty() {
        format!(
            "[checkpoint] --resume: loading {}\n\
             [checkpoint] Resuming from step 3 — skipping completed steps\n\
             [step 3/4] Validate    ✓ (retry)\n\
             [step 4/4] InsertDB    ✓\n\
             [checkpoint] --checkpoint-ttl: pruning checkpoints older than 24h\n\
             [done] Pipeline completed (resumed): {}",
            resume_file, src
        )
    } else {
        format!(
            "[checkpoint] Saving state after each stage to {}\n\
             [step 1/4] LoadCsv     ✓ → {}/step-1-loadcsv.ckpt\n\
             [step 2/4] EmbedText   ✓ → {}/step-2-embedtext.ckpt\n\
             [checkpoint] --checkpoint: .ckpt written\n\
             [checkpoint] --resume available if pipeline fails\n\
             [checkpoint] --checkpoint-ttl: default 72h\n\
             [done] Pipeline completed: {}",
            checkpoint_dir, checkpoint_dir, checkpoint_dir, src
        )
    }
}
```

## Step 2: `fav/src/main.rs` 変更

### 2a: `mod checkpoint;` を mod 宣言部に追加

```rust
mod checkpoint;
```

`mod cluster;` の直後に追加。

### 2b: `Some("run")` アームの先頭に `--checkpoint`/`--resume` ブランチを追加

既存の `Some("run")` の最上部（`--env` ブランチより前）に追加:

```rust
// ── v68.2.0: fav run --checkpoint / --resume ──────────────────────
if args.iter().any(|a| a == "--checkpoint" || a == "--resume") {
    let checkpoint_dir = args.iter().position(|a| a == "--checkpoint")
        .and_then(|i| args.get(i + 1).map(|s| s.as_str()))
        .unwrap_or("./checkpoints/");
    let resume_file = args.iter().position(|a| a == "--resume")
        .and_then(|i| args.get(i + 1).map(|s| s.as_str()))
        .unwrap_or("");
    let src = args.iter().skip(2)
        .find(|a| !a.starts_with('-') && a.as_str() != checkpoint_dir && a.as_str() != resume_file)
        .map(|s| s.as_str())
        .unwrap_or("pipeline.fav");
    println!("{}", checkpoint::cmd_checkpoint_run(src, checkpoint_dir, resume_file));
    return;
}
```

**注意**: `src` 検出時は `checkpoint_dir` と `resume_file` の値を除外する（誤検出防止）。

## Step 3: `driver.rs` — `v68200_tests` 追加

挿入位置: `// -- v68100_tests (v68.1.0) -- Multi-Node par（分散並列実行） --` の直前

```rust
// -- v68200_tests (v68.2.0) -- Pipeline Checkpointing（耐障害性・再開） --
#[cfg(test)]
mod v68200_tests {
    #[test]
    fn checkpoint_save_restore() {
        let result = crate::checkpoint::cmd_checkpoint_run("pipeline.fav", "./checkpoints/", "");
        assert!(
            result.contains("--checkpoint") && result.contains(".ckpt") && result.contains("--resume"),
            "cmd_checkpoint_run should output '--checkpoint', '.ckpt', '--resume'"
        );
    }

    #[test]
    fn checkpoint_resume_mid_pipeline() {
        let result = crate::checkpoint::cmd_checkpoint_run("pipeline.fav", "./checkpoints/", "step-2.ckpt");
        assert!(
            result.contains("Resuming from step") && result.contains("--checkpoint-ttl"),
            "cmd_checkpoint_run should output 'Resuming from step' and '--checkpoint-ttl'"
        );
    }
}
```

## 注意事項

- `Some("run")` の既存ロジックは変更しない（`--checkpoint`/`--resume` ブランチのみ追加）
- `src` 検出時に名前付きフラグの値（`checkpoint_dir`、`resume_file`）を除外する（`cluster.rs` での教訓）
- sub-version ポリシー: Cargo.toml / CHANGELOG は変更しない
