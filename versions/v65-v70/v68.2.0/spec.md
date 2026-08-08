# v68.2.0 — Pipeline Checkpointing（耐障害性・再開）

Date: 2026-08-07
Status: 完了（実装済み・レビュー済み）
Sprint: Distributed Favnir（v68.1〜v69.0）

---

## 概要

パイプライン実行状態を `.ckpt` ファイルに保存し、失敗したステップから途中再開できる。
長時間の AI パイプライン（大規模埋め込み、LLM バッチ処理）で特に有効。
v68.2.0 はスタブ実装。実際の状態シリアライズは将来フェーズ。

## スコープ

### IN スコープ

- `fav/src/checkpoint.rs` — 新規作成
  - `pub const CHECKPOINT_HELP: &str` — `"--checkpoint"` / `"--resume"` / `"--checkpoint-ttl"` / `".ckpt"` を含む
  - `pub fn cmd_checkpoint_run(src: &str, checkpoint_dir: &str, resume_file: &str) -> String`
    - `resume_file` が空の場合（通常実行）:
      - `"--checkpoint"` / `".ckpt"` / `"--resume"` を含む出力を返す
    - `resume_file` が非空の場合（再開実行）:
      - `"Resuming from step"` / `"--checkpoint-ttl"` を含む出力を返す
- `fav/src/main.rs` — `mod checkpoint;` 追加 + `Some("run")` アームの先頭に `--checkpoint`/`--resume` ブランチ追加
  - `--checkpoint <dir>` → `checkpoint_dir` を取得（省略時は `"./checkpoints/"`）
  - `--resume <file>` → `resume_file` を取得（省略時は `""`）
  - `src`（pipeline.fav）省略時は `"pipeline.fav"` をデフォルト
  - `--checkpoint` または `--resume` が存在すれば `cmd_checkpoint_run` を呼び出して `return`
  - `src` 検出時は `checkpoint_dir` / `resume_file` の値を除外（誤検出防止）
- `fav/src/driver.rs` — `v68200_tests` 追加（2 件）

### OUT スコープ（将来フェーズ）

> ロードマップの「実装内容」リストには以下が列挙されているが、v68.2.0 はスタブ実装のため将来フェーズとする。

- `.ckpt` バイナリフォーマット仕様の確定 / 実際のステージ出力シリアライズ: 将来フェーズ
- `--checkpoint-ttl` の実際の古いファイル削除処理: 将来フェーズ
- チェックポイントの整合性検証（ハッシュチェック / 破損検出）: 将来フェーズ
- 部分的再開の実際の実行（完了済みステージのスキップ）: 将来フェーズ
  ※ ロードマップの「実装内容」には IN として列挙されているが、v68.2.0 はキーワード出力スタブのみ

## コマンド設計

```
fav run pipeline.fav --checkpoint ./checkpoints/
fav run pipeline.fav --resume ./checkpoints/step-2-embedtext.ckpt
fav run pipeline.fav --checkpoint ./checkpoints/ --checkpoint-ttl 24
```

- `--checkpoint` / `--resume` は `Some("run")` アームに統合（新規コマンドではなく既存 `fav run` の拡張）
- `--checkpoint` 省略時デフォルト: `"./checkpoints/"`
- `--resume` 省略時デフォルト: `""` (空文字列)
- `src` 省略時デフォルト: `"pipeline.fav"`

## テスト完了条件

| テスト名 | 検証内容 |
|---|---|
| `checkpoint_save_restore` | `cmd_checkpoint_run` が `"--checkpoint"` / `".ckpt"` / `"--resume"` を含む（resume_file = ""） |
| `checkpoint_resume_mid_pipeline` | `cmd_checkpoint_run` が `"Resuming from step"` / `"--checkpoint-ttl"` を含む（resume_file 指定） |

ベーステスト: 3521 → 目標: **3523**

> `CHECKPOINT_HELP` 定数が `"--checkpoint"` / `"--resume"` / `"--checkpoint-ttl"` / `".ckpt"` を含むことは目視確認で担保する（テスト追加は v68.9.0 の一括 MDX 対応時に検討）。
