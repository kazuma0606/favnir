# v68.2.0 タスクリスト

Status: COMPLETE
Version: 68.2.0
Note: MDX ドキュメントは v68.9.0 で一括作成のため本バージョンでは不要
Base tests: 3521
Target tests: 3523

---

## T0: 事前確認

- [x] `cargo test -j 8 -- --test-threads=8` でベース 3521 tests passed, 0 failed を確認
- [x] `fav/Cargo.toml` の version が `"68.0.0"` であることを確認（sub-version では変更しない）
- [x] `fav/src/checkpoint.rs` が存在しないことを確認（新規作成）
- [x] `driver.rs` に `v68100_tests` が存在することを確認（`v68200_tests` の挿入位置）
- [x] `driver.rs` に `v68200_tests` が存在しないことを確認（新規追加）
- [x] `cargo test --bin fav v68100_tests` で 2 件 PASS することを確認（前バージョンが正常）
  - 前バージョンのテスト関数名: `distributed_par_multi_node`, `distributed_work_rebalance`
- [x] `versions/current.md` の「進行中バージョン」が `v68.1.0` であることを確認

---

## T1: `fav/src/checkpoint.rs` 新規作成

- [x] `fav/src/checkpoint.rs` を新規作成
  - [x] `pub const CHECKPOINT_HELP: &str` を追加（`"--checkpoint"` / `"--resume"` / `"--checkpoint-ttl"` / `".ckpt"` を含む）
  - [x] `pub fn cmd_checkpoint_run(src: &str, checkpoint_dir: &str, resume_file: &str) -> String` を追加
    - [x] `resume_file` 空の場合: `"--checkpoint"` / `".ckpt"` / `"--resume"` を含む出力（`checkpoint_save_restore` テスト要件）
    - [x] `resume_file` 非空の場合: `"Resuming from step"` / `"--checkpoint-ttl"` を含む出力（`checkpoint_resume_mid_pipeline` テスト要件）
- [x] `cargo build` でエラーなし

---

## T2: `fav/src/main.rs` 変更

- [x] `mod checkpoint;` を mod 宣言部（`mod cluster;` の直後）に追加
- [x] `Some("run")` アームの先頭に `--checkpoint`/`--resume` ブランチを追加
  - [x] `--checkpoint <dir>` → `checkpoint_dir` 取得（省略時は `"./checkpoints/"`）
  - [x] `--resume <file>` → `resume_file` 取得（省略時は `""`）
  - [x] `src` 検出時に `checkpoint_dir` / `resume_file` の値を除外（誤検出防止）
  - [x] `src` 省略時デフォルト `"pipeline.fav"`
  - [x] `println!("{}", checkpoint::cmd_checkpoint_run(src, checkpoint_dir, resume_file))` + `return;`
- [x] `cargo build` でエラーなし

---

## T3: `driver.rs` — `v68200_tests` 追加

- [x] `// -- v68100_tests (v68.1.0)` の直前に `v68200_tests` を挿入
  - [x] `checkpoint_save_restore`: `cmd_checkpoint_run("pipeline.fav", "./checkpoints/", "")` の戻り値に `"--checkpoint"` / `".ckpt"` / `"--resume"` を含む
  - [x] `checkpoint_resume_mid_pipeline`: `cmd_checkpoint_run("pipeline.fav", "./checkpoints/", "step-2.ckpt")` の戻り値に `"Resuming from step"` / `"--checkpoint-ttl"` を含む
- [x] `use super::*` は不要（`crate::checkpoint::` で直接参照）
- [x] `cargo build` でエラーなし

---

## T4: ビルド・テスト

- [x] `cargo test --bin fav v68200_tests` で 2 件 PASS
  - [x] `checkpoint_save_restore` PASS
  - [x] `checkpoint_resume_mid_pipeline` PASS
- [x] `cargo test -j 8 -- --test-threads=8` で 3523 tests passed, 0 failed を確認

---

## T5: ドキュメント・ステータス更新

> T4 のテスト全通過（3523 tests passed）を確認してから実施すること。

- [x] `versions/roadmap/roadmap-v68.1-v69.0.md` の v68.2.0「状態」列を「未着手」→「完了」に変更
- [x] `versions/current.md` の「進行中バージョン」を v68.2.0 に更新
- [x] 本 `tasks.md` を COMPLETE に更新（T0 を含む全チェックボックスを `[x]` に）

> **sub-version ポリシー**: v68.x では Cargo.toml / CHANGELOG.md は変更しない。v69.0.0 宣言時に一括更新する。

---

## 設計上の意図的省略

- 実際の状態シリアライズ / ステージ出力の `.ckpt` バイナリ保存: 将来フェーズ
- `--checkpoint-ttl` の実際の古いファイル削除処理: 将来フェーズ
- チェックポイントの整合性検証（ハッシュチェック）: 将来フェーズ
- 部分的再開の実際の実行（完了済みステージのスキップ）: 将来フェーズ

## コードレビュー指摘と対応

| 深刻度 | 内容 | 対応 |
|---|---|---|
| [LOW] | `Some("run")` checkpoint branch で `checkpoint_dir` 値が `src` に誤検出されるリスク（フラグ先頭の場合） | `checkpoint_dir` / `resume_file` を除外するフィルターを追加 |

※ `Some("cluster")` の `src` 誤検出（[MED]）は v68.1.0 の実装に関する指摘のため v68.1.0/tasks.md に記録済み。
