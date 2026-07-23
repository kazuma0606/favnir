# Plan: v49.1.0 — 全機能統合テスト + E2E デモ更新

## 作業順序

### Step 1: `examples/v50-demo/` ディレクトリ作成

`favnir/examples/v50-demo/` 以下の 3 ファイルを作成する。
**注記**: `#[test]` ブロックの内容は spec.md の記述に従う。ロードマップのコードスニペット（`assert_ok`/`assert_err` / 関数名 `test_validate`）はスケッチであり、実際の実装は spec.md の `test_validate_ok` / `test_validate_err_status`（`assert_eq` 使用）に準拠する。

1. `examples/v50-demo/fav.toml` — プロジェクト設定（`[runes] kafka = "2.1.0"`）
2. `examples/v50-demo/stages/validate.fav` — validate ステージ（`return` ガード節使用）
3. `examples/v50-demo/pipeline.fav` — メインパイプライン（新 import 構文 + `#[test]` ブロック）

### Step 2: `driver.rs` に `v491000_tests` 追加

**注記**: `include_str!("../../examples/v50-demo/pipeline.fav")` はコンパイル時にファイルを読み込む。Step 1 のデモ作成と同一作業セッションで追加すること（ファイルが存在しない状態での `cargo test` はビルドエラーになる）。

`v49000_tests` の直前に挿入（2テスト）:
- `e2e_demo_v50_structure`: `examples/v50-demo/` + 3 ファイルの存在確認
- `e2e_demo_uses_new_import`: `pipeline.fav` が `import kafka`（新構文）を使い `import rune` を使わないことを確認

### Step 3: `Cargo.toml` version 更新

`"49.0.0"` → `"49.1.0"`

### Step 4: 完了処理

- `cargo test` 3071 passed を確認
- `cargo clippy -- -D warnings` クリーン確認
- `CHANGELOG.md` に v49.1.0 エントリ追加
- `versions/current.md` 更新（v49.1.0・3071 tests・進行中 v49.2.0）
- `versions/roadmap/roadmap-v49.1-v50.0.md` の v49.1.0 実績を記入（推定値 3064 → 実績 3071 に更新）
- `tasks.md` を COMPLETE に更新

---

## 変更ファイル一覧

| ファイル | 変更種別 |
|---|---|
| `examples/v50-demo/fav.toml` | 新規作成 |
| `examples/v50-demo/stages/validate.fav` | 新規作成 |
| `examples/v50-demo/pipeline.fav` | 新規作成 |
| `fav/src/driver.rs` | `v491000_tests` 追加（2テスト）|
| `fav/Cargo.toml` | version 更新 |
| `CHANGELOG.md` | v49.1.0 エントリ |
| `versions/current.md` | バージョン更新 |
| `versions/roadmap/roadmap-v49.1-v50.0.md` | 実績記入 |
| `versions/v45-v50/v49.1.0/tasks.md` | COMPLETE 更新 |

## 変更しないファイル

| ファイル | 理由 |
|---|---|
| `fav/src/frontend/parser.rs` | デモ作成のみ、コンパイラ変更なし |
| `fav/src/middle/checker.rs` | デモ作成のみ、コンパイラ変更なし |
